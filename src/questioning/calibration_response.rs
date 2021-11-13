use anyhow::Result;
use reqwest::{
    blocking::Response,
    header::{self, HeaderValue},
    StatusCode,
};

/// Contains the parts of web response which are relevant to deciding whether the web server decided the padding was correct or not.
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct CalibrationResponse {
    status: StatusCode,
    location: Option<HeaderValue>,
    content: Option<String>,
    content_length: Option<u64>,
}

impl CalibrationResponse {
    pub fn from_response(response: Response, consider_body: bool) -> Result<Self> {
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
