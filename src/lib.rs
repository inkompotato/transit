mod gtfs_reader;

use gtfs_reader::{parse_gtfs, H3Stop};
use pyo3::prelude::*;
use std::collections::HashMap;

/// A Python module implemented in Rust.
#[pymodule]
fn rs_transit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gtfs_reader, m)?)?;

    Ok(())
}

// #[pyfunction]
// pub fn gtfs_reader(file: String) -> PyResult<HashMap<String, H3Stop>> {
//     parse_gtfs(file)
// }
