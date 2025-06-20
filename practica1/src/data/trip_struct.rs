use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Trip {
    pub vendor_id: String,
    pub tpep_pickup_datetime: String,
    pub tpep_dropoff_datetime: String,
    pub passenger_count: String,
    pub trip_distance: String,
    pub ratecode_id: String,
    pub store_and_fwd_flag: String,
    pub pu_location_id: String,
    pub do_location_id: String,
    pub payment_type: String,
    pub fare_amount: String,
    pub extra: String,
    pub mta_tax: String,
    pub tip_amount: String,
    pub tolls_amount: String,
    pub improvement_surcharge: String,
    pub total_amount: String,
    pub congestion_surcharge: String,
    pub index: String,
}
