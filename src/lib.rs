use std::collections::HashMap;

use geo_types::Point;
use gtfs_structures::*;
use h3ron::{H3Cell, Index};
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn rs_transit(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RGtfs>()?;
    m.add_class::<TransitStop>()?;

    Ok(())
}

#[pyclass]
pub struct RGtfs {
    #[pyo3(get)]
    pub name: String,
    gtfs: Gtfs,
}

#[pymethods]
impl RGtfs {
    #[new]
    pub fn new(filename: String) -> Self {
        let gtfs = Gtfs::new(&filename).unwrap();

        Self {
            name: filename,
            gtfs,
        }
    }

    /// Get a list of all stops
    pub fn get_stops(&self) -> PyResult<Vec<TransitStop>> {
        let res = self
            .gtfs
            .stops
            .iter()
            .filter_map(|(id, data)| {
                let lat = data.latitude?;
                let lon = data.longitude?;
                let point : Point<f64> = (lon, lat).into();

                let res = H3Cell::from_point(&point, 10).map(|h3| {
                    TransitStop {
                        id: id.to_string(),
                        name: (&data.name).to_string(),
                        location: (lat, lon),
                        h3,
                    }
                });
                res.ok()
            })
            .collect::<Vec<TransitStop>>();

        Ok(res)
    }

    /// Get a list of all lines
    pub fn get_routes(&self) -> PyResult<HashMap<String, Vec<String>>> {
        Ok(HashMap::new())
    }

    pub fn __repr__(&self) -> String {
        format!("Rust Gtfs object for {}", self.name)
    }
}

#[derive(Debug)]
#[pyclass]
pub struct TransitStop {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub location: (f64, f64),
    h3: H3Cell,
}

#[pymethods]
impl TransitStop {
    pub fn __repr__(&self) -> String {
        format!("{} (ID:{}) @ {:?}", self.name, self.id, self.location)
    }

    pub fn get_h3_id(&self) -> u64 {
        self.h3.h3index() as u64
    }
}
