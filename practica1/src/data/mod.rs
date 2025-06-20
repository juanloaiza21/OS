// Declara los submódulos públicos
pub mod data_lector;
pub mod disk_hash;
pub mod filters;
pub mod trip_struct; // Nuevo módulo de filtros

// Re-exporta los componentes principales
pub use data_lector::stream_process_csv;
pub use disk_hash::DiskHashTable;
pub use filters::{TripFilter, filter_to_file, get_filter_stats, get_popular_destinations};
pub use trip_struct::Trip;

