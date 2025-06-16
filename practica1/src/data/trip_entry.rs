use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TripEntry {
    pub key: String,
    pub trip: super::trip_struct::Trip,
}
