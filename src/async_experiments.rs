use std::{num::ParseIntError, future::Future};
use h3ron::{FromH3Index, H3Cell, Index};
use itertools;
use serde::{Deserialize, Serialize};
use tokio::task;
use tokio_postgres::{Error, NoTls, Statement};

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

const DB_CONN: &str = "postgresql://postgres:byS*<7AxwYC#U24s@srv-captain--postgres-db-db/postgres";

// in rust, we need to treat h3 cells as u64s and convert to and from string for querying
#[derive(Debug, Serialize, Deserialize)]
struct Task {
    origin_h3: u64,
    current_h3: u64,
    scores: Vec<f32>,
    distance: i32,
}

impl Task {
    pub async fn complete_task() -> Result<(), AppError> {
        Ok(())
    }
}

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> Result<(), AppError> {
    let (client, connection) = tokio_postgres::connect(DB_CONN, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let stations: Vec<Task> = client
        .query(
            "
            select * from transit 
            where type = 0
            and h3_4 = '841f22bffffffff'
        ",
            &[],
        )
        .await?
        .iter()
        .filter_map(|row| {
            Some(Task {
                origin_h3: hex_to_u64(row.get(0))?,
                current_h3: hex_to_u64(row.get(0))?,
                scores: row.get(3),
                distance: 0,
            })
        })
        .collect::<Vec<Task>>();

    let query_statement = 
        client.prepare("
        select h3 from transit
            where h3 = any($1::ARRAY)
            and $2::TEXT != all(visitors)
            and cardinality(visitors) < 5
        ");

    // let update_statement = client.prepare("

    // ");

    println!("got {} stations from DB", stations.len());
    stations.into_iter().for_each(|task| {
        task::spawn(async move {
            
            let join = perform_task(&task, query_statement.await?.clone());
            if let Err(e) = join.await {
                eprintln!("error: {:?}", e)
            }
        });
    });

    Ok(())
}

async fn perform_task(task: &Task, query_statement: Statement) -> Result<(), AppError> {
    let (client, connection) = tokio_postgres::connect(DB_CONN, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // get relevant data from DB
    // get neighbors as list of strings
    let neighbors = H3Cell::from_h3index(task.current_h3)
        .hex_ring(1)?
        .first()
        .iter()
        .map(|cell| u64_to_hex(cell.h3index()))
        .collect::<Vec<String>>();

    println!("{:?}", neighbors);
    let db_neighbors = client
        .query(
            "
        select h3 from transit
            where h3 = any($1::ARRAY)
            and $2::TEXT != all(visitors)
            and cardinality(visitors) < 5
        ",
            &[&neighbors, &u64_to_hex(task.current_h3)],
        )
        .await?
        .iter()
        .map(|row| row.get(0))
        .collect::<Vec<String>>();

    db_neighbors.iter().for_each(|neighbor| {
        // let new_score = &task.scores;
        println!("calculating {}", neighbor)
    });

    // calculate scores for surounding cells

    // write data to DB

    Ok(())
}

fn hex_to_u64(input: String) -> Option<u64> {
    u64::from_str_radix(&input, 16).ok()
}

fn u64_to_hex(input: u64) -> String {
    format!("{:X}", input)
}
