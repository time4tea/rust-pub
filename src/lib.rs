use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

pub mod downloader;
pub mod pubspec;
pub mod pubspeclock;
pub mod scanner;

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

impl Package {
    /// Create a new package with required fields
    pub fn new(name: String, package_uri: String) -> Self {
        Package {
            name,
            root_uri: None,
            root_path: None,
            package_uri,
            language_version: None,
            supported: HashMap::new(),
        }
    }

    /// Set the root URI for the package
    pub fn with_root_uri(mut self, uri: String) -> Self {
        self.root_uri = Some(uri);
        self
    }

    /// Set the root path for the package
    pub fn with_root_path(mut self, path: String) -> Self {
        self.root_path = Some(path);
        self
    }

    /// Set the language version for the package
    pub fn with_language_version(mut self, version: String) -> Self {
        self.language_version = Some(version);
        self
    }

    /// Add platform support information
    pub fn add_platform_support(&mut self, platform: String, supported: bool) {
        self.supported.insert(platform, supported);
    }
}

impl PackageConfig {
    /// Load package configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: PackageConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Create a new PackageConfig with default values
    pub fn new() -> Self {
        PackageConfig {
            config_version: 2, // Current standard version
            packages: Vec::new(),
            generator: Some("flutter-pub".to_string()),
            generator_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    pub fn add_package(&mut self, package: Package) {
        self.packages.push(package);
    }

    /// Write the package configuration to a file
    pub fn write_to_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}
