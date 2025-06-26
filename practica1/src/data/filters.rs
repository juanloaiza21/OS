use super::trip_struct::Trip;
use super::disk_hash::{DiskHashTable, build_hash_table_from_csv};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::Mutex;
use std::sync::LazyLock;

// Constantes para el directorio de hash
const HASH_DIR: &str = "tmp/hash_index";
static HASH_INIT: Once = Once::new();
static HASH_TABLE: LazyLock<Mutex<Option<DiskHashTable>>> = LazyLock::new(|| Mutex::new(None));

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

// Nueva función para inicializar o recuperar la tabla hash
fn get_or_initialize_hash_table<P: AsRef<Path>>(csv_path: P) -> Result<&'static Mutex<Option<DiskHashTable>>, Box<dyn Error>> {
    HASH_INIT.call_once(|| {
        println!("Inicializando tabla hash en disco...");
        let hash_path = PathBuf::from(HASH_DIR);
        
        // Verificar si ya existe la tabla hash
        let mut needs_build = true;
        if hash_path.exists() {
            // Si existe al menos un archivo de bucket, asumimos que la tabla está construida
            for i in 0..256 { // NUM_BUCKETS from disk_hash.rs
                let bucket_path = hash_path.join(format!("bucket_{}.json", i));
                if bucket_path.exists() && fs::metadata(&bucket_path).map(|m| m.len() > 0).unwrap_or(false) {
                    needs_build = false;
                    break;
                }
            }
        }
        
        // Inicializar la tabla hash
        match DiskHashTable::new(&hash_path) {
            Ok(hash_table) => {
                let mut table_ref = HASH_TABLE.lock().unwrap();
                *table_ref = Some(hash_table);
                
                // Si necesitamos construir la tabla, hacerlo ahora
                if needs_build {
                    println!("Construyendo índice hash desde CSV...");
                    match build_hash_table_from_csv(&csv_path, &hash_path) {
                        Ok(count) => println!("Índice hash construido con {} registros", count),
                        Err(e) => eprintln!("Error al construir índice hash: {}", e),
                    }
                } else {
                    println!("Usando índice hash existente");
                }
            },
            Err(e) => {
                eprintln!("Error al inicializar tabla hash: {}", e);
            }
        }
    });
    
    Ok(&HASH_TABLE)
}

/// Determina si un filtro puede beneficiarse del uso de hash table
fn can_use_hash_index(filter: &TripFilter) -> Option<String> {
    match filter {
        // Si es un filtro directo por índice, podemos usar hash
        TripFilter::Index(idx) => Some(idx.clone()),
        
        // Si es un AND con un filtro de índice, podemos usarlo para la primera búsqueda
        TripFilter::And(filters) => {
            // Buscamos un filtro de índice en la lista
            for f in filters {
                if let TripFilter::Index(idx) = f {
                    return Some(idx.clone());
                }
            }
            None
        },
        
        // Para otros tipos de filtros, no podemos usar hash directamente
        _ => None
    }
}

/// Filtrar trips y guardar resultados en un archivo
pub fn filter_to_file<P: AsRef<Path>>(
    csv_path: P, 
    output_file: P,
    filter: TripFilter, 
    max_results: Option<usize>
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
    
    // Verificar si podemos usar un índice hash para este filtro
    if let Some(index) = can_use_hash_index(&filter) {
        println!("Usando índice hash para búsqueda rápida por índice: {}", index);
        
        // Obtener la tabla hash
        let hash_table_ref = get_or_initialize_hash_table(&csv_path)?;
        if let Some(hash_table) = hash_table_ref.lock().unwrap().as_ref() {
            // Buscar directamente por índice
            if let Ok(Some(trip)) = hash_table.get(&index) {
                // Verificar si el trip completo cumple con todos los criterios del filtro
                if filter.matches(&trip) {
                    // Escribir el trip en el archivo de salida
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
                    count = 1;
                }
            }
            writer.flush()?;
            return Ok(count);
        }
    }
    
    // Si no podemos usar hash o la búsqueda hash falló, caemos al método tradicional
    println!("Usando escaneo secuencial de CSV para filtrado");
    
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
    
    // Verificar si podemos usar un índice hash para este filtro
    if let Some(index) = can_use_hash_index(&filter) {
        println!("Usando índice hash para estadísticas por índice: {}", index);
        
        // Obtener la tabla hash
        let hash_table_ref = get_or_initialize_hash_table(&csv_path)?;
        if let Some(hash_table) = hash_table_ref.lock().unwrap().as_ref() {
            // Buscar directamente por índice
            if let Ok(Some(trip)) = hash_table.get(&index) {
                // Verificar si el trip completo cumple con todos los criterios del filtro
                if filter.matches(&trip) {
                    count = 1;
                    total_distance = trip.trip_distance.parse::<f64>().unwrap_or(0.0);
                    total_amount = trip.total_amount.parse::<f64>().unwrap_or(0.0);
                    total_passengers = trip.passenger_count.parse::<i32>().unwrap_or(0);
                    
                    // Calcular estadísticas
                    stats.insert("count".to_string(), count as f64);
                    stats.insert("avg_distance".to_string(), total_distance);
                    stats.insert("avg_amount".to_string(), total_amount);
                    stats.insert("avg_passengers".to_string(), total_passengers as f64);
                    stats.insert("total_amount".to_string(), total_amount);
                    
                    return Ok(stats);
                }
            }
        }
    }
    
    // Si no podemos usar hash o la búsqueda hash falló, caemos al método tradicional
    println!("Usando escaneo secuencial de CSV para estadísticas");
    
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
    // Para destinos populares, necesitamos procesar todos los registros
    // así que no hay ventaja en usar la hash table aquí
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

/// Nueva función: Inicializar manualmente el índice hash
pub fn initialize_hash_index<P: AsRef<Path>>(csv_path: P) -> Result<usize, Box<dyn Error>> {
    println!("Inicializando índice hash manualmente...");
    let hash_path = PathBuf::from(HASH_DIR);
    
    // Limpiar índice existente si existe
    if hash_path.exists() {
        println!("Eliminando índice hash existente...");
        fs::remove_dir_all(&hash_path)?;
    }
    
    // Construir nuevo índice
    println!("Construyendo nuevo índice hash...");
    let count = build_hash_table_from_csv(csv_path, &hash_path)?;
    
    // Reinicializar la referencia estática
    let hash_table = DiskHashTable::new(&hash_path)?;
    let mut table_ref = HASH_TABLE.lock().unwrap();
    *table_ref = Some(hash_table);
    
    println!("Índice hash inicializado con {} registros", count);
    
    Ok(count)
}
