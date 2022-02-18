use pyo3::prelude::*;
use transitfeed::{Agency, GTFSIterator, Stop};

/// A Python module implemented in Rust.
#[pymodule]
fn rs_transit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_agencies, m)?)?;
    m.add_function(wrap_pyfunction!(get_stops, m)?)?;

    m.add_class::<TransitStop>()?;

    Ok(())
}

/// Extract Names of Transit agencies from agency.txt file
#[pyfunction]
fn get_agencies(filename: String) -> PyResult<Vec<String>> {
    let iterator: GTFSIterator<_, Agency> = GTFSIterator::from_path(&filename).unwrap();
    Ok(iterator
        .filter_map(|agency| {
            if let Ok(agency) = agency {
                Some(agency.agency_name)
            } else {
                None
            }
        })
        .collect::<Vec<String>>())
}

/// Extract Names of Transit agencies from agency.txt file
#[pyfunction]
fn get_stops(filename: String) -> PyResult<Vec<TransitStop>> {
    let iterator: GTFSIterator<_, Stop> = GTFSIterator::from_path(&filename).unwrap();
    Ok(iterator
        .filter_map(|stop| {
            if let Ok(stop) = stop {
               Some(TransitStop::new(stop.stop_id, stop.stop_name, (stop.stop_lat, stop.stop_lon)))
            } else {
                None
            }
        })
        .collect::<Vec<TransitStop>>())
}

#[derive(Debug)]
#[pyclass]
pub struct TransitStop {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub location: (f64, f64)
}

#[pymethods]
impl TransitStop {
    /// Construct new Transit Stop
    #[new]
    pub fn new(id: String, name: String, location: (f64, f64)) -> TransitStop {
        TransitStop {id, name, location}
    }

    pub fn to_string(&self) -> String {
        format!("{} (ID:{}) @ {:?}", self.name, self.id, self.location)
    }
}
