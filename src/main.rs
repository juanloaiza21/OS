use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
#[allow(dead_code)]
struct Trip {
    vendor_id: String,
    tpep_pickup_datetime: String,
    tpep_dropoff_datetime: String,
    passenger_count: String,
    trip_distance: String,
    ratecode_id: String,
    store_and_fwd_flag: String,
    pu_location_id: String,
    do_location_id: String,
    payment_type: String,
    fare_amount: String,
    extra: String,
    mta_tax: String,
    tip_amount: String,
    tolls_amount: String,
    improvement_surcharge: String,
    total_amount: String,
    congestion_surcharge: String,
    index: String,
}

fn read_csv<P: AsRef<Path>>(filename: P) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut array = Vec::new();
    for result in rdr.records() {
        let record = result?;
        // meter en un array
        for field in record.iter() {
            array.push(field.to_string());
        }
    }
    return Ok(array);
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = "/home/jorge/Documents/juan/uni/os/taller1/src/yellow_tripdata_2020-06.csv";
    let array = read_csv(filename)?;
    // print array
    println!("Number of celds: {}", array.len());
    let mut i = 0;
    let mut trips_map: HashMap<String, Trip> = HashMap::new();
    while i < array.len() {
        let trip: Trip = Trip {
            vendor_id: array[i].clone(),
            tpep_pickup_datetime: array[i + 1].clone(),
            tpep_dropoff_datetime: array[i + 2].clone(),
            passenger_count: array[i + 3].clone(),
            trip_distance: array[i + 4].clone(),
            ratecode_id: array[i + 5].clone(),
            store_and_fwd_flag: array[i + 6].clone(),
            pu_location_id: array[i + 7].clone(),
            do_location_id: array[i + 8].clone(),
            payment_type: array[i + 9].clone(),
            fare_amount: array[i + 10].clone(),
            extra: array[i + 11].clone(),
            mta_tax: array[i + 12].clone(),
            tip_amount: array[i + 13].clone(),
            tolls_amount: array[i + 14].clone(),
            improvement_surcharge: array[i + 15].clone(),
            total_amount: array[i + 16].clone(),
            congestion_surcharge: array[i + 17].clone(),
            index: array[i + 18].clone(),
        };
        trips_map.insert(array[i + 18].clone().to_string(), trip);
        i += 19;
        if i == array.len() {
            break;
        }
    }
    //TODO input del index por consola
    let index_to_find = "1";
    // print hash
    if let Some(found_trip) = trips_map.get(index_to_find) {
        println!(
            "Found trip with vendor_id: {} and pickup time: {}",
            found_trip.vendor_id, found_trip.tpep_pickup_datetime
        );
    } else {
        println!("Trip with index {} not found", index_to_find);
    }
    Ok(())
}
