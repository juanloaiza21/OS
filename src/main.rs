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
    let array = read_csv(filename)?;
    // print array
    println!("Number of celds: {}", array.len());
    let mut carlos: Vec<Vec<String>> = Vec::new();
    let mut j = 0;
    let mut i = 0;
    let mut auxArray: Vec<String> = Vec::new();
    while i < array.len() && j <= 18 {
        auxArray.push(array[i].to_string());
        j += 1;
        i += 1;
        if j == 18 {
            carlos.push(auxArray);
            j = 0;
            auxArray = Vec::new();
        }
    }
    //TODO input del index por consola
    // print array
    println!("Data: {:?}", carlos[1]);
    Ok(())
}
