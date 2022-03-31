use chrono::{NaiveDate, NaiveDateTime};
use geo_types::Point;
use h3ron::{H3Cell, Index};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

pub fn parse_gtfs(path: String) -> PyResult<HashMap<String, H3Stop>> {
    let file = File::open(&path)?;
    let mut zip =
        zip::ZipArchive::new(file).or(Err(PyValueError::new_err("error acessing zip archive")))?;

    let stop_times = zip
        .by_name("stop_times.txt")
        .or(Err(PyValueError::new_err("stop_times.txt not present")))?;
    let mut rdr = csv::Reader::from_reader(stop_times);
    for result in rdr.deserialize() {
        let record: GtfsStopTime =
            result.or(Err(PyValueError::new_err("error deserializing stop time")))?;
        println!("{:?}", record);
    }

    Ok(HashMap::new())
}

#[pyclass]
pub struct H3Stop {
    coordinates: (f64, f64),
    name: String,
    h3_cell: String,
    lines: Vec<Line>,
}

#[pyclass]
pub struct Line {
    route_id: String,
    route_name: String,
    frequencies: [f32; 7 * 24],
    stop_seq: i32,
}

fn convert_time(time: u32, day: u32) -> i64 {
    let hour = time / 3600;
    let minute = time % 3600 / 60;
    let seconds = time % 60;

    let date: NaiveDateTime = NaiveDate::from_ymd(2022, 1, day).and_hms(hour, minute, seconds);
    date.timestamp()
}

fn coord_to_h3(lat: f64, lon: f64, res: Option<u8>) -> Result<String, h3ron::Error> {
    let point: Point<f64> = (lon, lat).into();
    let h3 = H3Cell::from_point(&point, res.unwrap_or(10))?.h3index();
    Ok(format!("{:X}", h3))
}

// GTFS Structs

#[derive(Debug, Deserialize)]
struct GtfsStopTime {
    trip_id: String,
    arrival_time: String,
    departure_time: String,
    stop_id: String,
    stop_sequence: u32,
}
