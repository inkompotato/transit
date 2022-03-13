use std::collections::HashMap;

use geo_types::Point;
use gtfs_structures::*;
use h3ron::{H3Cell, Index};
use itertools::Itertools;
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

    /// Get a list of all routes
    pub fn get_routes(&self) -> PyResult<Vec<(String, String)>> {
        let res = self.gtfs.routes.iter().map(|(_, route)| {
            (route.id.to_string(), route.short_name.to_string())
        }).collect::<Vec<_>>();

        Ok(res)
    }

    /// Get a list of all lines
    /// route_ID -> Vec<(stop_id, departure_time)>
    pub fn get_route_departures(&self) -> PyResult<HashMap<String, Vec<(String, u32)>>> {
        let res: HashMap<String, Vec<(String, u32)>> = self
            .gtfs
            .trips
            .iter()
            .map(|(_, trip)| {
                let route = &trip.route_id;
                let stop_sequence = trip.stop_times.iter().map(|stop_time| {
                    let departure = stop_time.departure_time.unwrap_or_default();
                    let stop_id = stop_time.stop.id.to_string();
                    (stop_id, departure)
                }).collect::<Vec<_>>();
                (route, stop_sequence)
            }).into_group_map_by(|tuple| tuple.0 ).into_iter().map(|(key, value)| {
                //key: route ID
                //value: all trips on that route as Vec<StopID, departure time>
                //unify into routeID -> Vec<stop_id<Vec<departure times>>
                let stop_departure_times = value.into_iter().map(|(_, stop_sequence)| stop_sequence ).flatten().collect::<Vec<_>>();

                (key.to_string(), stop_departure_times)
            }).collect();
        
        Ok(res)
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
