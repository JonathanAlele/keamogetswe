use serde::Serialize;
use crate::path::GeodesicPath;

/// A complete output file containing one or more paths for the viewer.
#[derive(Debug, Serialize)]
pub struct GeodesicOutput {
    pub metadata: Metadata,
    pub paths: Vec<PathEntry>,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub name: String,
    pub metric: String,
    pub resolution: usize,
}

#[derive(Debug, Serialize)]
pub struct PathEntry {
    pub label: String,
    pub points: Vec<[f64; 3]>,
}

impl GeodesicOutput {
    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
