use super::trip_struct::Trip;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn stream_process_csv<P, F>(filename: P, mut process_trip: F) -> Result<(), Box<dyn Error>> where 
    P: AsRef<Path>,
    F: FnMut(&Trip) -> Result<(), Box<dyn Error>>, {
    let file = File::open(filename)?;
    //Manejar memoria me va a volver loco
    let buf_reader = BufReader::with_capacity(64 * 1024, file); // Buffer de 64KB
    let mut csv_reader = csv::ReaderBuilder::new()
        .buffer_capacity(128 * 1024) // Buffer de 128KB para el parser CSV
        .has_headers(true)
        .from_reader(buf_reader);
    //TODO finish this
    for result in csv_reader
}
