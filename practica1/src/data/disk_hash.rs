use std::{error::Error, fs::File, path::Path};
use csv::ReaderBuilder;
use crate::data::trip_struct::Trip;

pub fn buscar_registros<P: AsRef<Path>>(
    path: P,
    index: Option<usize>,
    pickup_datetime: Option<&str>,
    total_amount_filter: Option<(String, f64)>,
) -> Result<Vec<Trip>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut results = Vec::new();

    for result in rdr.records() {
        let record = result?;

        let trip = Trip {
            index: record.get(17).unwrap_or("").trim().to_string(),
            pickup_datetime: record.get(1).unwrap_or("").trim().to_string(),
            dropoff_datetime: record.get(2).unwrap_or("").trim().to_string(),
            total_amount: record.get(15).unwrap_or("").trim().to_string(),
            passengers: record.get(3).unwrap_or("").trim().to_string(),
            distance: record.get(4).unwrap_or("").trim().to_string(),
        };

        if let Some(idx) = index {
            if trip.index != idx.to_string() {
                continue;
            }
        }

        if let Some(date) = pickup_datetime {
            if !trip.pickup_datetime.contains(date) {
                continue;
            }
        }

        if let Some((op, val)) = &total_amount_filter {
            if let Ok(amount) = trip.total_amount.parse::<f64>() {
                let matched = match op.as_str() {
                    ">" => amount > *val,
                    ">=" => amount >= *val,
                    "<" => amount < *val,
                    "<=" => amount <= *val,
                    "=" => (amount - *val).abs() < f64::EPSILON,
                    _ => false,
                };
                if !matched {
                    continue;
                }
            } else {
                continue;
            }
        }

        results.push(trip);
    }

    Ok(results)
}
