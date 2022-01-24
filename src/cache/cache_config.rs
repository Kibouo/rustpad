use serde::{Deserialize, Serialize};

use crate::{
    calibrator::calibration_response::{CalibrationResponse, SerializableCalibrationResponse},
    oracle::oracle_location::{OracleLocation, SerializableOracleLocation},
};

/// State which defines the validity of a cache entry.
/// In other words, all of the properties between the current and the cache's must match to allow loading of the associated values.
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub(crate) struct CacheConfig {
    oracle_location: SerializableOracleLocation,
    calibration_response: Option<SerializableCalibrationResponse>,
}

impl CacheConfig {
    pub(crate) fn new(
        oracle_location: OracleLocation,
        calibration_response: Option<CalibrationResponse>,
    ) -> Self {
        Self {
            oracle_location: oracle_location.into(),
            calibration_response: calibration_response.map(SerializableCalibrationResponse::from),
        }
    }
}
