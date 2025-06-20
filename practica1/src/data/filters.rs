use super::trip_struct::Trip;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

pub enum TripFilter {
    Price { min: Option<f64>, max: Option<f64> },
    Index(String),
    Destination(String),
    And(Vec<TripFilter>),
    Or(Vec<TripFilter>),
}

impl TripFilter {
    pub fn matches(&self, trip: &Trip) -> bool {
        match self {
            TripFilter::Price { min, max } => {
                // Convertir el precio a f64, usar 0.0 si hay error
                let price = trip.total_amount.parse::<f64>().unwrap_or(0.0);

                // Verificar límites mínimo y máximo si existen
                let min_check = min.map_or(true, |min_val| price >= min_val);
                let max_check = max.map_or(true, |max_val| price <= max_val);

                min_check && max_check
            }
            TripFilter::Index(target_index) => trip.index == *target_index,
            TripFilter::Destination(target_dest) => trip.do_location_id == *target_dest,
            TripFilter::And(filters) => {
                // Todos los filtros deben cumplirse (AND lógico)
                filters.iter().all(|filter| filter.matches(trip))
            }
            TripFilter::Or(filters) => {
                // Al menos un filtro debe cumplirse (OR lógico)
                filters.iter().any(|filter| filter.matches(trip))
            }
        }
    }
}

/// Filtrar trips y guardar resultados en un archivo
pub fn filter_to_file<P: AsRef<Path>>(
    csv_path: P,
    output_file: P,
    filter: TripFilter,
    max_results: Option<usize>,
) -> Result<usize, Box<dyn Error>> {
    let output_file = output_file.as_ref();

    // Crear directorio padre si no existe
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(output_file)?;
    let mut writer = BufWriter::new(file);

    // Escribir encabezado CSV
    writeln!(
        writer,
        "vendor_id,tpep_pickup_datetime,tpep_dropoff_datetime,passenger_count,trip_distance,ratecode_id,store_and_fwd_flag,pu_location_id,do_location_id,payment_type,fare_amount,extra,mta_tax,tip_amount,tolls_amount,improvement_surcharge,total_amount,congestion_surcharge,index"
    )?;

    let mut count = 0;

    // Procesar CSV en streaming y aplicar filtros
    super::data_lector::stream_process_csv(csv_path, |trip| {
        if filter.matches(trip) {
            // Escribir el viaje filtrado al archivo de salida
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                trip.vendor_id,
                trip.tpep_pickup_datetime,
                trip.tpep_dropoff_datetime,
                trip.passenger_count,
                trip.trip_distance,
                trip.ratecode_id,
                trip.store_and_fwd_flag,
                trip.pu_location_id,
                trip.do_location_id,
                trip.payment_type,
                trip.fare_amount,
                trip.extra,
                trip.mta_tax,
                trip.tip_amount,
                trip.tolls_amount,
                trip.improvement_surcharge,
                trip.total_amount,
                trip.congestion_surcharge,
                trip.index
            )?;

            count += 1;

            // Verificar si hemos alcanzado el máximo de resultados
            if let Some(max) = max_results {
                if count >= max {
                    return Err("Límite de resultados alcanzado".into());
                }
            }
        }

        Ok(())
    })
    .or_else(|e| {
        // Ignorar el error específico de límite alcanzado
        if e.to_string() == "Límite de resultados alcanzado" {
            Ok(())
        } else {
            Err(e)
        }
    })?;

    writer.flush()?;

    Ok(count)
}

/// Obtiene estadísticas de los trips que cumplen con un filtro
pub fn get_filter_stats<P: AsRef<Path>>(
    csv_path: P,
    filter: TripFilter,
) -> Result<HashMap<String, f64>, Box<dyn Error>> {
    let mut stats = HashMap::new();
    let mut count = 0;
    let mut total_distance = 0.0;
    let mut total_amount = 0.0;
    let mut total_passengers = 0;

    // Procesar CSV en streaming y acumular estadísticas
    super::data_lector::stream_process_csv(csv_path, |trip| {
        if filter.matches(trip) {
            count += 1;
            total_distance += trip.trip_distance.parse::<f64>().unwrap_or(0.0);
            total_amount += trip.total_amount.parse::<f64>().unwrap_or(0.0);
            total_passengers += trip.passenger_count.parse::<i32>().unwrap_or(0);
        }

        Ok(())
    })?;

    // Calcular promedios y almacenar estadísticas
    stats.insert("count".to_string(), count as f64);

    if count > 0 {
        stats.insert("avg_distance".to_string(), total_distance / count as f64);
        stats.insert("avg_amount".to_string(), total_amount / count as f64);
        stats.insert(
            "avg_passengers".to_string(),
            total_passengers as f64 / count as f64,
        );
        stats.insert("total_amount".to_string(), total_amount);
    }

    Ok(stats)
}

/// Obtiene una lista de los destinos más populares
pub fn get_popular_destinations<P: AsRef<Path>>(
    csv_path: P,
    limit: usize,
) -> Result<Vec<(String, usize)>, Box<dyn Error>> {
    let mut dest_counts: HashMap<String, usize> = HashMap::new();

    // Contar ocurrencias de cada destino
    super::data_lector::stream_process_csv(csv_path, |trip| {
        let dest = &trip.do_location_id;
        *dest_counts.entry(dest.clone()).or_insert(0) += 1;

        Ok(())
    })?;

    // Convertir a vector para ordenar
    let mut dest_vec: Vec<(String, usize)> = dest_counts.into_iter().collect();
    dest_vec.sort_by(|a, b| b.1.cmp(&a.1)); // Ordenar por frecuencia descendente

    // Limitar resultados
    let result = dest_vec.into_iter().take(limit).collect();

    Ok(result)
}
