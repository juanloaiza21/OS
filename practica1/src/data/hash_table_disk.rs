use super::trip_entry::TripEntry;
use super::trip_struct::Trip;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::hash_set;
use std::error::Error;
use std::fs::{File, OpenOptions, create_dir_all};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const NUM_BUCKETS: usize = 256;

fn calculate_hash(key: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

fn get_bucket_index(key: &str) -> usize {
    (calculate_hash(key) % NUM_BUCKETS as u64) as usize
}

pub struct DiskHashTable {
    bucket_dir: PathBuf,
    //TODO add a cache mem manipulation?
}

impl DiskHashTable {
    //CreacionHashTable
    pub fn new<P: AsRef<Path>>(dir_path: P) -> Result<Self, Box<dyn Error>> {
        let bucket_dir = dir_path.as_ref().to_path_buf();
        create_dir_all(&bucket_dir)?;

        //Inicializar Buckets vacios.
        for i in 0..NUM_BUCKETS {
            let bucket_path = bucket_dir.join(format!("bucket_{}.json", i));
            if !bucket_path.exists() {
                File::create(bucket_path)?;
            }
        }
        Ok(Self { bucket_dir })
    }

    //Insertar

    pub fn insert(&self, key: String, trip: Trip) -> Result<(), Box<dyn Error>> {
        let bucket_idx = get_bucket_index(&key);
        let bucket_path = self.bucket_dir.join(format!("bucket_{}.json", bucket_idx));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&bucket_path)?;
        let mut contents = String::new();
        let mut reader = BufReader::new(&file);

        reader.read_to_string(&mut contents)?;

        //Cargue de entradas existentes
        let mut entries: Vec<super::trip_entry::TripEntry> = if contents.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&contents)?
        };

        let entry_idx = entries.iter().position(|e| e.key == key);
        let entry = super::trip_entry::TripEntry { key, trip };

        if let Some(idx) = entry_idx {
            entries[idx] = entry;
        } else {
            entries.push(entry);
        }

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&bucket_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &entries)?;
        writer.flush()?;

        Ok(())
    }
    //Get
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

        let entries: Vec<super::trip_entry::TripEntry> = serde_json::from_str(&contents)?;
        // Buscar la clave
        for entry in entries {
            if entry.key == key {
                return Ok(Some(entry.trip));
            }
        }
        Ok(None)
    }
    //Delete
    pub fn remove(&self, key: &str) -> Result<bool, Box<dyn Error>> {
        let bucket_idx = get_bucket_index(key);
        let bucket_path = self.bucket_dir.join(format!("bucket_{}.json", bucket_idx));

        if !bucket_path.exists() {
            return Ok(false);
        }

        let file = File::open(&bucket_path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        if contents.is_empty() {
            return Ok(false);
        }

        let mut entries: Vec<super::trip_entry::TripEntry> = serde_json::from_str(&contents)?;
        let initial_len = entries.len();

        // Filtrar la entrada a eliminar
        entries.retain(|e| e.key != key);

        if entries.len() == initial_len {
            return Ok(false);
        }

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&bucket_path)?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &entries)?;
        writer.flush()?;

        Ok(true)
    }
}
