use h3ron::{FromH3Index, H3Cell, Index};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::{
    collections::{HashMap, VecDeque},
    fs::{self, File},
    io::{self, BufReader},
    num::ParseIntError,
};
use tokio::time::Instant;
use tokio_postgres::Error;

// const DB_CONN: &str = "postgresql://postgres:byS*<7AxwYC#U24s@srv-captain--postgres-db-db/postgres";

struct Config {
    reasonable_distances: Vec<f32>,
    rual_scale_factor: f32,
}

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

// ,h3,h3_group,h3_group_agg,freq,urban,type
type Record = (i64, String, String, String, Vec<f32>, i32, i32);

#[derive(Debug, Serialize, Deserialize)]
struct JsonCell {
    h3: String,
    h3_group: String,
    h3_group_agg: String,
    freq: Vec<f32>,
    urban: i32,
    #[serde(rename(deserialize = "type"))]
    transit_type: i32,
}

#[derive(Debug)]
struct Cell {
    h3: u64,
    h3_4: u64,
    h3_10: u64,
    freq: Vec<f32>,
    urban: bool,
    transit_type: i32,
    scores: Vec<f32>,
    visitors: Vec<u64>,
}

impl Cell {
    pub fn from_record(record: Record) -> Option<Self> {
        Some(Cell {
            h3: hex_to_u64(record.1)?,
            h3_4: hex_to_u64(record.2)?,
            h3_10: hex_to_u64(record.3)?,
            freq: record.4,
            urban: if record.5 == 1 { true } else { false },
            transit_type: record.6,
            scores: vec![0.0; 24 * 7],
            visitors: Vec::new(),
        })
    }

    pub fn from_json_cell(cell: &JsonCell) -> Option<Self> {
        Some(Cell {
            h3: hex_to_u64(cell.h3.clone())?,
            h3_4: hex_to_u64(cell.h3_group.clone())?,
            h3_10: hex_to_u64(cell.h3_group_agg.clone())?,
            freq: cell.freq.clone(),
            urban: if cell.urban == 1 { true } else { false },
            transit_type: cell.transit_type,
            scores: vec![0.0; 24 * 7],
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
            .sorted_by_key(|list| -*list.get(31).unwrap_or(&0.0) as i32)
            .fold(self.freq.clone(), |a, b| {
                // reduce function, initial score + half of the next score, repeat for all
                a.iter()
                    .zip(b.iter())
                    .map(|(xa, xb)| xa + (0.5 * xb))
                    .collect()
            })
    }
}

fn read_csv() -> Result<Vec<Cell>, AppError> {
    println!("reading file");
    let file = fs::read_to_string("./resources/dataframe.json").expect("msg");
    println!("parsing file");
    let data: Vec<JsonCell> = serde_json::from_str(&file).expect("error");
    let data = data
        .iter()
        .filter_map(|json_cell| Cell::from_json_cell(json_cell))
        .collect::<Vec<Cell>>();

    Ok(data)
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let config = Config {
        // bus, tram, metro, train
        reasonable_distances: vec![8.0, 8.0, 12.0, 15.0],
        rual_scale_factor: 1.5,
    };

    let start_time = Instant::now();
    println!("started transit score calculation");
    let mut data: Vec<Cell> = read_csv()?;

    // this is created to enable (near) constant time lookups in the data array
    let index: HashMap<u64, usize> = data
        .iter()
        .enumerate()
        .map(|(i, c)| (c.h3, i))
        .collect::<HashMap<u64, usize>>();

    println!(
        "[INFO DATA] got {} cells from file in {:?}",
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

        let (current_h3, origin_h3, origin_freq, origin_type) = (
            data[current_index].h3,
            data[origin_index].h3,
            data[origin_index].freq.clone(),
            data[origin_index].transit_type,
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
                    let factor = if neighbor_cell.urban {
                        1.0
                    } else {
                        config.rual_scale_factor
                    };

                    if !neighbor_cell.visitors.contains(&origin_h3) {
                        // cell is part of our network
                        // update its scores and add it back to the queue
                        let distance = distance + 1;
                        let mut max_value = 0.0;
                        let new_score = origin_freq
                            .iter()
                            .map(|value| {
                                // calculate score
                                // let new_value = value / (1.0 + 5.0 * f32::exp(0.5 * (f32::from(distance) - 1.5 * value)));
                                let new_value = value
                                    / (1.0
                                        + f32::exp(
                                            (1.0 / factor) * f32::from(distance)
                                                - config.reasonable_distances[origin_type as usize],
                                        ));
                                if new_value > max_value {
                                    max_value = new_value
                                }
                                new_value
                            })
                            .collect::<Vec<f32>>();

                        if max_value >= 0.2 {
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
    let start_time = Instant::now();
    println!("[INFO EXPORT] starting data export");
    // get all h3-4 groups
    data.iter()
        .filter(|cell| cell.transit_type != -1 || cell.scores.len() > 0)
        .into_group_map_by(|cell| cell.h3_4)
        .into_iter()
        .for_each(|(h3_4group, cells)| {
            // aggregate to h3-10
            print!(" .. aggregating {} \r", &h3_4group);
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
                            let new_value = f32::powf(value / divisor as f32, 1.0 / 1.4);
                            if new_value < 0.1 {
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
            let file = File::create(path).expect("could not create file");
            serde_json::to_writer(file, &vis_cells).expect("could not export json");
        });

    println!(
        "[INFO EXPORT] export finished in {:?}",
        start_time.elapsed()
    );
    Ok(())
}

fn hex_to_u64(input: String) -> Option<u64> {
    u64::from_str_radix(&input, 16).ok()
}

fn u64_to_hex(input: u64) -> String {
    format!("{:x}", input)
}
