use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use super::data_lector::Trip;

/// Filtra viajes y los guarda en un archivo si cumplen con el filtro dado.
pub fn filter_to_file<P: AsRef<Path>>(
    csv_path: P,
    output_path: P,
    filter: fn(&Trip) -> bool,
) -> Result<(), Box<dyn Error>> {
    let input_str = csv_path.as_ref().to_str().ok_or("Invalid input path")?;
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);

    super::data_lector::stream_process_csv(input_str, |trip| {
        if filter(&trip) {
            // Escribe los datos del trip como string (ajustar según formato deseado)
            writeln!(writer, "{:?}", trip).unwrap();
        }
    })?;

    Ok(())
}

/// Devuelve la cantidad de viajes que cumplen un filtro dado.
pub fn get_filter_stats<P: AsRef<Path>>(
    csv_path: P,
    filter: fn(&Trip) -> bool,
) -> Result<usize, Box<dyn Error>> {
    let input_str = csv_path.as_ref().to_str().ok_or("Invalid input path")?;
    let mut count = 0;

    super::data_lector::stream_process_csv(input_str, |trip| {
        if filter(&trip) {
            count += 1;
        }
    })?;

    Ok(count)
}

/// Retorna las ubicaciones de destino más populares.
pub fn get_popular_destinations<P: AsRef<Path>>(
    csv_path: P,
) -> Result<HashMap<String, usize>, Box<dyn Error>> {
    let input_str = csv_path.as_ref().to_str().ok_or("Invalid input path")?;
    let mut dest_counts: HashMap<String, usize> = HashMap::new();

    super::data_lector::stream_process_csv(input_str, |trip| {
        let count = dest_counts.entry(trip.do_location_id.clone()).or_insert(0);
        *count += 1;
    })?;

    Ok(dest_counts)
}
