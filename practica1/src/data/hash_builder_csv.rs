use super::hash_table_disk::{self, DiskHashTable};
use super::trip_struct::Trip;
use std::error::Error;
use std::path::Path;

pub fn build_hash_table_from_csv<P: AsRef<Path>>(
    csv_path: P,
    hash_dir: P,
) -> Result<usize, Box<dyn Error>> {
    let hash_table = DiskHashTable::new(hash_dir)?;
    let mut count = 0;
}
