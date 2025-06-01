use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};


pub mod pubspec;

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Config file format version
    #[serde(rename = "configVersion")]
    pub config_version: u32,
    /// List of all packages in the project
    pub packages: Vec<Package>,
    /// Generator information
    pub generator: Option<String>,
    /// Generator version
    #[serde(rename = "generatorVersion")]
    pub generator_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub name: String,
    /// Root URI of the package
    #[serde(rename = "rootUri")]
    pub root_uri: Option<String>,
    /// Root path of the package
    #[serde(rename = "rootPath")]
    pub root_path: Option<String>,
    /// Path to the package's lib directory
    #[serde(rename = "packageUri")]
    pub package_uri: String,
    /// Language version for this package
    #[serde(rename = "languageVersion")]
    pub language_version: Option<String>,
    /// Map of supported platforms to their configuration
    #[serde(default)]
    pub supported: HashMap<String, bool>,
}

impl PackageConfig {
    /// Load package configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: PackageConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }
}