use h3ron::{FromH3Index, H3Cell, Index};
use itertools::Itertools;
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    num::ParseIntError,
};
use tokio::time::Instant;
use tokio_postgres::{Error, NoTls, Row};

const DB_CONN: &str = "postgresql://postgres:byS*<7AxwYC#U24s@srv-captain--postgres-db-db/postgres";

#[derive(Debug)]
struct AppError {
    message: String,
}

impl From<Error> for AppError {
    fn from(error: Error) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<ParseIntError> for AppError {
    fn from(error: ParseIntError) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<h3ron::Error> for AppError {
    fn from(error: h3ron::Error) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct VisCell {
    h3: String,
    #[serde(rename(serialize = "type"))]
    transit_type: i32,
    freq: Vec<f32>,
}

#[derive(Debug)]
struct Cell {
    h3: u64,
    h3_4: u64,
    h3_10: u64,
    freq: Vec<f32>,
    transit_type: i32,
    scores: Vec<f32>,
    visitors: Vec<u64>,
}

impl Cell {
    pub fn from_row(row: &Row) -> Option<Self> {
        Some(Cell {
            h3: hex_to_u64(row.get(0))?,
            h3_4: hex_to_u64(row.get(1))?,
            h3_10: hex_to_u64(row.get(2))?,
            freq: row.get(3),
            transit_type: row.get(4),
            scores: row.get(5),
            visitors: Vec::new(),
        })
    }

    pub fn append_scores(&mut self, mut scores: Vec<f32>, origin: u64) {
        (&mut self.scores).append(&mut scores);
        (&mut self.visitors).push(origin);
    }

    pub fn aggregate_scores(&mut self) {
        self.scores = self
            .scores
            // create chunks the size of a week
            .chunks(24 * 7 as usize)
            .map(|x| x.to_vec())
            // sort by highest value (in this case, just by the score for monday morning to speed things up)
            .sorted_by_key(|list| *list.get(31).unwrap_or(&0.0) as i32)
            .reduce(|a, b| {
                // reduce function, initial score + half of the next score, repeat for all
                a.iter()
                    .zip(b.iter())
                    .map(|(xa, xb)| xa + (0.2 * xb))
                    .collect()
            })
            .unwrap_or_default();
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let (client, connection) = tokio_postgres::connect(DB_CONN, NoTls).await?;
    let start_time = Instant::now();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("[INFO DB] connected to DB at {}", DB_CONN);

    // load all data from the DB into memory
    let mut data: Vec<Cell> = client
        .query(
            "
            select * from transit 
        ",
            &[],
        )
        .await?
        .iter()
        .filter_map(|row| Cell::from_row(row))
        .collect();

    // this is created to enable (near) constant time lookups in the data array
    let index: HashMap<u64, usize> = data
        .iter()
        .enumerate()
        .map(|(i, c)| (c.h3, i))
        .collect::<HashMap<u64, usize>>();

    println!(
        "[INFO DB] got {} cells from DB in {:?}",
        data.len(),
        start_time.elapsed()
    );
    println!("[INFO MEM] size: {} bytes", std::mem::size_of_val(&*data));

    // current h3, origin (station) h3, distance
    let mut queue: VecDeque<(usize, usize, i16)> = VecDeque::new();
    let mut counter: i64 = 0;

    // add stations to the queue
    data.iter()
        .enumerate()
        .filter(|(_, queue_cell)| queue_cell.transit_type >= 0)
        .for_each(|(origin_cell_index, _)| {
            // add initial stations to queue
            queue.push_back((origin_cell_index, origin_cell_index, 0))
        });
    println!("[INFO MAIN] added {} stations to queue", queue.len());

    //main loop
    let start_time = Instant::now();
    while !queue.is_empty() {
        let (current_index, origin_index, distance) = queue.pop_front().unwrap();

        let (current_h3, origin_h3, origin_freq) = (
            data[current_index].h3,
            data[origin_index].h3,
            data[origin_index].freq.clone(),
        );

        print!("   ... calculating cell {} ({}) \r", current_h3, counter);
        counter += 1;

        // get neighbors
        H3Cell::from_h3index(current_h3)
            .hex_ring(1)
            .unwrap_or_default()
            .into_iter()
            .for_each(|neighbor_h3_cell| {
                if let Some(neighbor_cell_index) = index.get(&neighbor_h3_cell.h3index()) {
                    let neighbor_cell = &mut data[*neighbor_cell_index];

                    if !neighbor_cell.visitors.contains(&origin_h3) {
                        // cell is part of our network
                        // update its scores and add it back to the queue
                        let distance = distance + 1;
                        let mut max_value = 0.0;
                        let new_score = origin_freq
                            .iter()
                            .map(|value| {
                                // calculate score
                                let new_value = value
                                    / (1.0
                                        + 5.0
                                            * f32::exp(0.5 * (f32::from(distance) - 1.5 * value)));
                                if new_value > max_value {
                                    max_value = new_value
                                }
                                new_value
                            })
                            .collect::<Vec<f32>>();

                        if max_value >= 0.5 {
                            // write back score
                            neighbor_cell.append_scores(new_score, origin_h3);
                            // append to queue
                            queue.push_back((*neighbor_cell_index, origin_index, distance));
                        }
                    }
                }
            });
    }
    println!();
    println!(
        "[INFO MAIN] processed {} cells in {:?}",
        counter,
        start_time.elapsed()
    );

    // aggregate scores
    let start_time = Instant::now();
    println!("[INFO AGG-1] aggregating visitor scores");
    for cell in &mut data {
        cell.aggregate_scores()
    }
    println!(
        "[INFO AGG-1] visitor score aggregation finished in {:?}",
        start_time.elapsed()
    );

    // get all h3-4 groups
    data.iter()
        .filter(|cell| cell.transit_type != -1 || cell.scores.len() > 0)
        .into_group_map_by(|cell| cell.h3_4)
        .into_iter()
        .for_each(|(h3_4group, cells)| {
            // aggregate to h3-10
            let vis_cells = cells
                .into_iter()
                .into_group_map_by(|cell| cell.h3_10)
                .iter()
                .map(|(h3_10_group, h3_10_cells)| {
                    // aggregate the scores from each cell
                    let transit_type = h3_10_cells
                        .iter()
                        .map(|cell| cell.transit_type)
                        .max()
                        .unwrap_or(-1);
                    let score = h3_10_cells
                        .iter()
                        .map(|cell| vec![cell.scores.clone(), cell.freq.clone()])
                        .flatten()
                        .reduce(|a, b| {
                            a.iter()
                                .zip(b.iter())
                                .map(|(xa, xb)| xa + xb)
                                .collect::<Vec<f32>>()
                        })
                        .unwrap_or(vec![0.0; 24 * 7])
                        .iter()
                        .map(|value| {
                            let divisor: usize = h3_10_cells
                                .iter()
                                .map(|cell| if cell.transit_type >= 0 { 2 } else { 1 })
                                .sum();
                            let new_value = f32::powf(2.0, f32::exp(value / divisor as f32));
                            if new_value < 0.01 {
                                -1.0
                            } else {
                                new_value
                            }
                        })
                        .collect::<Vec<f32>>();
                    VisCell {
                        h3: u64_to_hex(*h3_10_group),
                        transit_type,
                        freq: score,
                    }
                })
                .collect::<Vec<VisCell>>();

            // write result for h3-4 group to json
            let path = format!("docs/h3/{}.json", u64_to_hex(h3_4group));
            let file = File::create(path).expect("could not create file :(");
            serde_json::to_writer(file, &vis_cells).expect("could not export json :(");
        });
    Ok(())
}

fn hex_to_u64(input: String) -> Option<u64> {
    u64::from_str_radix(&input, 16).ok()
}

fn u64_to_hex(input: u64) -> String {
    format!("{:x}", input)
}
