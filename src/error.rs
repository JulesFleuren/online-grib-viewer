use grib::GribError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use wasm_bindgen::prelude::*;


#[derive(Debug)]
pub enum GribViewerError {
    Grib(grib::GribError),
    MessageNotFound(String),
    InvalidKey(String),
    Other(String),
}

impl Error for GribViewerError {
    fn description(&self) -> &str {
        "grib viewer error"
    }
}

impl From<GribError> for GribViewerError {
    fn from(err: GribError) -> Self {
        GribViewerError::Grib(err)
    }
}

impl From<GribViewerError> for JsValue {
    fn from(err: GribViewerError) -> JsValue {
        JsValue::from_str(&err.to_string())
    }
}

impl Display for GribViewerError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            GribViewerError::Grib(e) => write!(f, "Grib error: {}", e),
            GribViewerError::MessageNotFound(e) => write!(f, "Message not found: {}", e),
            GribViewerError::InvalidKey(e) => write!(f, "Invalid key: {}", e),
            GribViewerError::Other(msg) => write!(f, "{}", msg),
        }
    }
}
