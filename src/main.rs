use std::error::Error;
use std::fs::File;
use std::path::Path;

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
    let mut array = Vec::new();
    array = read_csv(filename)?;
    // print array
    println!("Number of celds: {}", array.len());
    let mut carlos = Vec::new();
    for i in 0..carlos.len() {}
    // print array
    println!("Data: {}", carlos[0]);
    Ok(())
}
