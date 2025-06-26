use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use csv::StringRecord;

#[derive(Debug)]
pub struct Trip {
    pub index: String,
    pub vendor_id: String,
    pub pickup_datetime: String,
    pub dropoff_datetime: String,
    pub passenger_count: u32,
    pub trip_distance: f64,
    pub rate_code: String,
    pub store_and_fwd_flag: String,
    pub pu_location_id: String,
    pub do_location_id: String,
    pub payment_type: String,
    pub fare_amount: f64,
    pub extra: f64,
    pub mta_tax: f64,
    pub tip_amount: f64,
    pub tolls_amount: f64,
    pub improvement_surcharge: f64,
    pub total_amount: f64,
}

pub fn stream_process_csv<F>(path: &str, mut callback: F) -> Result<(), Box<dyn Error>>
where
    F: FnMut(Trip),
{
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(BufReader::new(file));
    for (index, result) in reader.records().enumerate() {
        let record = result?;
        let trip = Trip {
            index: index.to_string(), // 🛠️ corregido aquí
            vendor_id: record[0].to_string(),
            pickup_datetime: record[1].to_string(),
            dropoff_datetime: record[2].to_string(),
            passenger_count: record[3].parse().unwrap_or(0),
            trip_distance: record[4].parse().unwrap_or(0.0),
            rate_code: record[5].to_string(),
            store_and_fwd_flag: record[6].to_string(),
            pu_location_id: record[7].to_string(),
            do_location_id: record[8].to_string(),
            payment_type: record[9].to_string(),
            fare_amount: record[10].parse().unwrap_or(0.0),
            extra: record[11].parse().unwrap_or(0.0),
            mta_tax: record[12].parse().unwrap_or(0.0),
            tip_amount: record[13].parse().unwrap_or(0.0),
            tolls_amount: record[14].parse().unwrap_or(0.0),
            improvement_surcharge: record[15].parse().unwrap_or(0.0),
            total_amount: record[16].parse().unwrap_or(0.0),
        };
        callback(trip);
    }

    Ok(())
}
