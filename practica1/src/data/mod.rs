use std::collections::HashMap;
use std::path::Path;

pub mod hash_builder_csv;
pub mod hash_table_disk;
mod stream_process_csv;
pub mod trip_entry;
pub mod trip_struct;

pub fn parsecsv<P: AsRef<Path>>(filename: P) {
    println!("Implementation in progress")
}
