use anyhow::{Context, Result};
use getset::Getters;
use reqwest::{
    blocking::Response,
    header::{self, HeaderValue},
    StatusCode,
};
use serde::{Deserialize, Serialize};

/// Contains the parts of web response which are relevant to deciding whether the web oracle decided the padding was correct or not.
#[derive(Hash, Eq, PartialEq, Debug, Clone, Getters)]
pub(crate) struct CalibrationResponse {
    #[getset(get = "pub(super)")]
    status: StatusCode,
    #[getset(get = "pub(super)")]
    location: Option<HeaderValue>,
    #[getset(get)] // private
    content: Option<String>,
    #[getset(get = "pub(super)")]
    content_length: Option<u64>,
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub(crate) struct SerializableCalibrationResponse {
    status: u16,
    location: Option<Vec<u8>>,
    content: Option<String>,
    content_length: Option<u64>,
}

impl CalibrationResponse {
    pub(crate) fn from_response(response: Response, consider_body: bool) -> Result<Self> {
        let status = response.status();
        let location = response.headers().get(header::LOCATION).cloned();
        let content_length = if consider_body {
            response.content_length()
        } else {
            None
        };
        let content = if consider_body {
            Some(response.text()?)
        } else {
            None
        };

        Ok(CalibrationResponse {
            status,
            location,
            content,
            content_length,
        })
    }
}

impl From<CalibrationResponse> for SerializableCalibrationResponse {
    fn from(response: CalibrationResponse) -> Self {
        Self {
            status: response.status().as_u16(),
            location: response
                .location()
                .as_ref()
                .map(|v| Vec::from(v.as_bytes())),
            content: response.content().clone(),
            content_length: *response.content_length(),
        }
    }
}

impl From<SerializableCalibrationResponse> for CalibrationResponse {
    fn from(response: SerializableCalibrationResponse) -> Self {
        Self {
            status: StatusCode::from_u16(response.status).context("Status code stored in cache is invalid").expect("Data stored in the cache was verified when it was created. As such, the only possible reason for this must be a corrupted cache file."),
            location: response
                .location
                .map(|v| HeaderValue::from_bytes(&v[..]).context("Header value stored in cache is invalid").expect("Data stored in the cache was verified when it was created. As such, the only possible reason for this must be a corrupted cache file.")),
            content: response.content,
            content_length: response.content_length,
        }
    }
}
