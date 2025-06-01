use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PubspecLock {
    /// Format version of the lockfile
    #[serde(rename = "sdks")]
    pub sdks: Option<Sdks>,
    /// All packages (direct and transitive dependencies)
    pub packages: HashMap<String, PackageSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sdks {
    pub dart: Option<String>,
    pub flutter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageSpec {
    pub version: String,
    pub source: String,
    pub dependency: String,
    pub description: Option<PackageDescription>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageDescription {
    Hosted {
        name: String,
        url: String,
        sha256: String,
    },
    Path {
        path: String,
    },
    Git {
        url: String,
        #[serde(rename = "ref")]
        ref_: Option<String>,
        path: Option<String>,
    },
    Sdk(String),

}

impl PubspecLock {
    /// Load pubspec.lock from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Loading pubspec.lock from {}", path.as_ref().display());
        let contents = fs::read_to_string(path)?;
        let lock_file = serde_yaml::from_str(&contents)?;
        Ok(lock_file)
    }

    /// Get all packages with their versions
    pub fn get_package_versions(&self) -> Vec<(&str, &str)> {
        self.packages
            .iter()
            .map(|(name, spec)| (name.as_str(), spec.version.as_str()))
            .collect()
    }
}
