use crate::pubspeclock::PackageDescription;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PubCacheError {
    #[error("Failed to create directory: {0}")]
    CreateDirError(#[from] std::io::Error),
    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Unsupported package source")]
    UnsupportedSource,
}

pub struct PubCache {
    root: PathBuf,
}

impl PubCache {
    /// Create a new PubCache at the specified path
    pub fn new(path: impl AsRef<Path>) -> Result<Self, PubCacheError> {
        let root = path.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(PubCache { root })
    }

    /// Get the path for a hosted package
    pub fn get_package_path(
        &self,
        name: &str,
        version: &str,
        desc: &PackageDescription,
    ) -> Result<PathBuf, PubCacheError> {
        match desc {
            PackageDescription::Hosted { url, .. } => {
                let host = url.host_str().unwrap_or("pub.dev");

                Ok(self
                    .root
                    .join("hosted")
                    .join(host)
                    .join(format!("{}-{}", name, version)))
            }
            _ => Err(PubCacheError::UnsupportedSource),
        }
    }

    pub fn create_package_dir(
        &self,
        name: &str,
        version: &str,
        desc: &PackageDescription,
    ) -> Result<PathBuf, PubCacheError> {
        let path = self.get_package_path(name, version, desc)?;
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }
}
