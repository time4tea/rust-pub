use crate::pubspeclock::{PackageDescription, PackageName, PackageVersion, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PubCacheError {
    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Unsupported package source")]
    UnsupportedSource,
    #[error("IO error: {0}")]
    IoError(io::Error),
}
impl From<io::Error> for PubCacheError {
    fn from(error: io::Error) -> Self {
        PubCacheError::IoError(error)
    }
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
        name: PackageName,
        version: PackageVersion,
        desc: &PackageDescription,
    ) -> Result<PathBuf, PubCacheError> {
        match desc {
            PackageDescription::Hosted { url, .. } => url
                .host_str()
                .ok_or(PubCacheError::UnsupportedSource)
                .map(|host| {
                    self.root
                        .join("hosted")
                        .join(host)
                        .join(format!("{}-{}", name, version))
                }),
            _ => Err(PubCacheError::UnsupportedSource),
        }
    }

    pub fn create_package_dir(
        &self,
        name: PackageName,
        version: PackageVersion,
        desc: &PackageDescription,
    ) -> Result<PathBuf, PubCacheError> {
        let path = self.get_package_path(name, version, desc)?;
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn get_hash_file_path(
        &self,
        host: &str,
        package_name: &PackageName,
        version: &PackageVersion,
    ) -> PathBuf {
        self.root
            .join("hosted-hashes")
            .join(host)
            .join(format!("{}.{}.sha256", package_name, version))
    }

    pub fn read_package_hash(
        &self,
        host: &str,
        package_name: &PackageName,
        version: &PackageVersion,
    ) -> Result<Option<Sha256>, PubCacheError> {
        let hash_path = self.get_hash_file_path(host, package_name, version);

        if !hash_path.exists() {
            return Ok(None);
        }

        let mut content = String::new();
        fs::File::open(&hash_path)
            .and_then(|mut file| file.read_to_string(&mut content))
            .map_err(PubCacheError::IoError)?;

        // Remove any whitespace and newlines
        Ok(Some(Sha256::new(content.trim().to_string())))
    }

    /// Writes the SHA256 hash for a package to the cache
    pub fn write_package_hash(
        &self,
        host: &str,
        package_name: &PackageName,
        version: &PackageVersion,
        hash: &Sha256,
    ) -> Result<(), PubCacheError> {
        let hash_path = self.get_hash_file_path(host, package_name, version);

        // Create parent directories if they don't exist
        if let Some(parent) = hash_path.parent() {
            fs::create_dir_all(parent).map_err(PubCacheError::IoError)?;
        }

        fs::File::create(&hash_path)
            .and_then(|mut file| file.write_all(hash.as_ref().as_bytes()))
            .map_err(PubCacheError::IoError)?;

        Ok(())
    }

    /// Verifies if a package's hash matches the cached hash
    pub fn verify_package_hash(
        &self,
        host: &str,
        package_name: &PackageName,
        version: &PackageVersion,
        expected_hash: &Sha256,
    ) -> Result<bool, PubCacheError> {
        match self.read_package_hash(host, package_name, version)? {
            Some(cached_hash) => Ok(&cached_hash == expected_hash),
            None => Ok(false),
        }
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }
}
