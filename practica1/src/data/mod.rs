use std::collections::HashMap;
use std::path::Path;

mod data_lector;
pub mod trip_struct;

pub fn parsecsv<P: AsRef<Path>>(filename: P) -> HashMap<String, trip_struct::Trip> {
    data_lector::parsecsv(filename)
}
