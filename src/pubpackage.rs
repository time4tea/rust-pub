use crate::pubspeclock::{PackageVersion, Sha256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PubPackageVersion {
    pub version: PackageVersion,
    // We're ignoring the pubspec field as requested
    #[serde(skip)]
    pub pubspec: serde::de::IgnoredAny,
    pub archive_url: String,
    pub archive_sha256: Sha256,
    pub published: String, // or use chrono::DateTime<Utc> if you want parsed dates
}

impl PubPackageVersion {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
