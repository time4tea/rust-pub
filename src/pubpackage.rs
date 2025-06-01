use crate::pubspeclock::{PackageVersion, Sha256};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PubPackageVersion {
    pub version: PackageVersion,
    pub archive_url: String,
    pub archive_sha256: Sha256,
    pub published: DateTime<Utc>,
}

impl PubPackageVersion {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
