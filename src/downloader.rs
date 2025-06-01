
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use thiserror::Error;

use std::sync::mpsc;
use threadpool::ThreadPool;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] ureq::Error),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Package not found: {name} {version}")]
    PackageNotFound { name: String, version: String },
    #[error("Invalid package archive")]
    InvalidArchive,
}

pub struct PackageDownloader {
    cache_dir: PathBuf,
}

impl PackageDownloader {
    pub fn new(cache_dir: impl AsRef<Path>) -> io::Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }


    /// Downloads a package and returns the path to the downloaded archive
    pub fn download_package(
        &self,
        name: &str,
        version: &str,
    ) -> Result<PathBuf, DownloadError> {
        let archive_path = self.cache_dir.join(format!("{}-{}.tar.gz", name, version));

        // Check if we already have this package cached
        if archive_path.exists() {
            return Ok(archive_path);
        }

        // Construct the download URL
        let url = format!(
            "https://pub.dev/packages/{}/versions/{}.tar.gz",
            name, version
        );

        // Start a thread for the download
        let download_result = thread::spawn(move || -> Result<Vec<u8>, ureq::Error> {
            let response = ureq::get(&url).call()?;
            let mut bytes = Vec::new();
            response.into_reader().read_to_end(&mut bytes)?;
            Ok(bytes)
        })
            .join()
            .map_err(|_| DownloadError::InvalidArchive)??;

        // Write the downloaded content to a temporary file first
        let temp_path = archive_path.with_extension("tmp");
        let mut temp_file = File::create(&temp_path)?;
        temp_file.write_all(&download_result)?;
        temp_file.flush()?;

        // Verify the archive is valid
        self.verify_archive(&temp_path)?;

        // Move the temporary file to the final location
        fs::rename(temp_path, &archive_path)?;

        Ok(archive_path)
    }

    pub fn download_packages_with_pool(
        &self,
        packages: &[(&str, &str)],
        pool: ThreadPool,
    ) -> Vec<Result<PathBuf, DownloadError>> {
        let (tx, rx) = mpsc::channel();
        let total_packages = packages.len();

        for &(name, version) in packages {
            let tx = tx.clone();
            let name = name.to_string();
            let version = version.to_string();
            let cache_dir = self.cache_dir.clone();

            pool.execute(move || {
                let downloader = match PackageDownloader::new(cache_dir) {
                    Ok(d) => d,
                    Err(e) => {
                        tx.send(Err(DownloadError::IoError(e))).unwrap();
                        return;
                    }
                };

                let result = downloader.download_package(&name, &version);
                tx.send(result).unwrap();
            });
        }

        // Drop the original sender so rx.iter() will stop after all jobs complete
        drop(tx);

        // Collect results in the order they complete
        rx.iter().take(total_packages).collect()
    }

    /// Downloads multiple packages concurrently
    pub fn download_packages(
        &self,
        packages: &[(&str, &str)],
    ) -> Vec<Result<PathBuf, DownloadError>> {
        let cache_dir = self.cache_dir.clone();

        let handles: Vec<_> = packages
            .iter()
            .map(|&(name, version)| {
                let name = name.to_string();
                let version = version.to_string();
                let cache_dir = cache_dir.clone();

                thread::spawn(move || {
                    let downloader = PackageDownloader::new(cache_dir)
                        .map_err(DownloadError::IoError)?;
                    downloader.download_package(&name, &version)
                })
            })
            .collect();

        handles
            .into_iter()
            .map(|handle| handle.join().unwrap_or(Err(DownloadError::InvalidArchive)))
            .collect()
    }

    /// Verifies that the downloaded file is a valid tar.gz archive
    fn verify_archive<P: AsRef<Path>>(&self, path: P) -> Result<(), DownloadError> {
        let file = File::open(path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        // Try to iterate through the archive entries
        archive.entries()?;

        Ok(())
    }

    /// Extracts a package archive to a specified directory
    pub fn extract_package<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        archive_path: P,
        extract_path: Q,
    ) -> Result<(), DownloadError> {
        let file = File::open(archive_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        fs::create_dir_all(&extract_path)?;
        archive.unpack(&extract_path)?;

        Ok(())
    }
}
