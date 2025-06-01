use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

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

macro_rules! stringy {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }
        }
        
        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageDescription {
    Hosted {
        name: PackageName,
        #[serde(with = "url_serde")]
        url: Url,
        sha256: Sha256,
    },
    Path {
        path: String,
    },
    Git {
        #[serde(with = "url_serde")]
        url: Url,
        #[serde(rename = "ref")]
        ref_: Option<String>,
        path: Option<String>,
    },
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
