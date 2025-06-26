use super::trip_struct::Trip;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::fs::{File, OpenOptions, create_dir_all};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

// Número de buckets para la hash table
const NUM_BUCKETS: usize = 256;

// Estructura para entradas de hash table
#[derive(Serialize, Deserialize)]
struct HashEntry {
    key: String,
    trip: Trip,
}

// Calcula el hash para una clave
fn calculate_hash(key: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

// Determina el bucket para una clave
fn get_bucket_index(key: &str) -> usize {
    (calculate_hash(key) % NUM_BUCKETS as u64) as usize
}

// Implementación de hash table basada en disco
pub struct DiskHashTable {
    bucket_dir: PathBuf,
}

impl DiskHashTable {
    // Crear nueva hash table
    pub fn new<P: AsRef<Path>>(dir_path: P) -> Result<Self, Box<dyn Error>> {
        let bucket_dir = dir_path.as_ref().to_path_buf();
        create_dir_all(&bucket_dir)?;

        // Inicializa buckets vacíos
        for i in 0..NUM_BUCKETS {
            let bucket_path = bucket_dir.join(format!("bucket_{}.json", i));
            if !bucket_path.exists() {
                File::create(bucket_path)?;
            }
        }

        Ok(Self { bucket_dir })
    }

    // Insertar en la hash table
    pub fn insert(&self, key: String, trip: Trip) -> Result<(), Box<dyn Error>> {
        let bucket_idx = get_bucket_index(&key);
        let bucket_path = self.bucket_dir.join(format!("bucket_{}.json", bucket_idx));

        // Crear archivo si no existe
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&bucket_path)?;

        let mut contents = String::new();
        let mut reader = BufReader::new(&file);
        reader.read_to_string(&mut contents)?;

        // Cargar entradas existentes
        let mut entries: Vec<HashEntry> = if contents.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&contents)?
        };

        // Buscar si la clave ya existe
        let entry_idx = entries.iter().position(|e| e.key == key);

        // Crear nueva entrada o actualizar existente
        let entry = HashEntry { key, trip };

        if let Some(idx) = entry_idx {
            entries[idx] = entry;
        } else {
            entries.push(entry);
        }

        // Guardar actualizaciones
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&bucket_path)?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &entries)?;
        writer.flush()?;

        Ok(())
    }

    // Obtener de la hash table
    pub fn get(&self, key: &str) -> Result<Option<Trip>, Box<dyn Error>> {
        let bucket_idx = get_bucket_index(key);
        let bucket_path = self.bucket_dir.join(format!("bucket_{}.json", bucket_idx));

        if !bucket_path.exists() {
            return Ok(None);
        }

        let file = File::open(&bucket_path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        if contents.is_empty() {
            return Ok(None);
        }

        let entries: Vec<HashEntry> = serde_json::from_str(&contents)?;

        for entry in entries {
            if entry.key == key {
                return Ok(Some(entry.trip));
            }
        }

        Ok(None)
    }

    // Contar el número total de entradas
    pub fn count_entries(&self) -> Result<usize, Box<dyn Error>> {
        let mut total_entries = 0;

        for i in 0..NUM_BUCKETS {
            let bucket_path = self.bucket_dir.join(format!("bucket_{}.json", i));

            if !bucket_path.exists() {
                continue;
            }

            let file = File::open(&bucket_path)?;
            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            reader.read_to_string(&mut contents)?;

            if contents.is_empty() {
                continue;
            }

            let entries: Vec<HashEntry> = serde_json::from_str(&contents)?;
            total_entries += entries.len();
        }

        Ok(total_entries)
    }
}

// Construir la hash table desde CSV
pub fn build_hash_table_from_csv<P: AsRef<Path>>(
    csv_path: P,
    hash_dir: P,
) -> Result<usize, Box<dyn Error>> {
    let hash_table = DiskHashTable::new(&hash_dir)?;
    let mut count = 0;

    super::data_lector::stream_process_csv(csv_path, |trip| {
        let key = trip.index.clone();
        hash_table.insert(key, trip.clone())?;
        count += 1;

        if count % 10000 == 0 {
            println!("Procesados {} registros...", count);
        }

        Ok(())
    })?;

    Ok(count)
}
