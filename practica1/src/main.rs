mod data;
mod visual;

fn main() {
    let data =
        data::parsecsv("/home/juloaizar/Documents/university/OS/practica1/src/data/data.csv");
    let i = "1";
    if let Some(found_trip) = data.get(i) {
        println!(
            "Found trip with vendor_id: {} and pickup time: {}",
            found_trip.vendor_id, found_trip.tpep_pickup_datetime
        );
    } else {
        println!("Trip {} not found", i)
    }
}
