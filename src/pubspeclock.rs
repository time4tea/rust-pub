use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

use crate::stringy;

#[derive(Debug, Serialize, Deserialize)]
pub struct PubspecLock {
    /// Format version of the lockfile
    #[serde(rename = "sdks")]
    pub sdks: Option<Sdks>,
    /// All packages (direct and transitive dependencies)
    pub packages: HashMap<PackageName, PackageSpec>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Sdks {
    pub dart: Option<String>,
    pub flutter: Option<String>,
}

stringy!(PackageName);
stringy!(PackageVersion);
stringy!(Sha256);

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageSpec {
    pub version: PackageVersion,
    pub source: String,
    pub dependency: String,
    pub description: Option<PackageDescription>,
}

#[derive(Debug, Display, Clone, Serialize, Deserialize)]
#[display("{} {} {}", name, url, sha256)]
pub struct HostedPackage {
    pub name: PackageName,
    #[serde(with = "url_serde")]
    pub url: Url,
    pub sha256: Sha256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitPackage {
    #[serde(with = "url_serde")]
    pub url: Url,
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathPackage {
    pub path: String,
    pub relative: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageDescription {
    Hosted(HostedPackage),
    Path(PathPackage),
    Git(GitPackage),
    Sdk(String),
}

// Helper module for URL serialization/deserialization
mod url_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use url::Url;

    pub fn serialize<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(url.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url_str = String::deserialize(deserializer)?;
        Url::parse(&url_str).map_err(serde::de::Error::custom)
    }
}

impl PubspecLock {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Loading pubspec.lock from {}", path.as_ref().display());
        let contents = fs::read_to_string(path)?;
        let lock_file = serde_yaml::from_str(&contents)?;
        Ok(lock_file)
    }
}
