use super::trip_struct::Trip;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// Función para procesar CSV en streaming con bajo consumo de memoria
pub fn stream_process_csv<P, F>(filename: P, mut process_trip: F) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
    F: FnMut(&Trip) -> Result<(), Box<dyn Error>>,
{
    let file = File::open(filename)?;
    let buf_reader = BufReader::with_capacity(64 * 1024, file);
    let mut csv_reader = csv::ReaderBuilder::new()
        .buffer_capacity(128 * 1024)
        .has_headers(true)
        .from_reader(buf_reader);

    for result in csv_reader.records() {
        match result {
            Ok(record) => {
                if record.len() < 19 {
                    eprintln!("Registro con formato incorrecto: {:?}", record);
                    continue;
                }

                let trip = Trip {
                    vendor_id: record[0].to_string(),
                    tpep_pickup_datetime: record[1].to_string(),
                    tpep_dropoff_datetime: record[2].to_string(),
                    passenger_count: record[3].to_string(),
                    trip_distance: record[4].to_string(),
                    ratecode_id: record[5].to_string(),
                    store_and_fwd_flag: record[6].to_string(),
                    pu_location_id: record[7].to_string(),
                    do_location_id: record[8].to_string(),
                    payment_type: record[9].to_string(),
                    fare_amount: record[10].to_string(),
                    extra: record[11].to_string(),
                    mta_tax: record[12].to_string(),
                    tip_amount: record[13].to_string(),
                    tolls_amount: record[14].to_string(),
                    improvement_surcharge: record[15].to_string(),
                    total_amount: record[16].to_string(),
                    congestion_surcharge: record[17].to_string(),
                    index: record[18].to_string(),
                };

                process_trip(&trip)?;
            }
            Err(e) => {
                eprintln!("Error al leer registro: {}", e);
            }
        }
    }

    Ok(())
}
