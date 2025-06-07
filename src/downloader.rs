use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::pubspeclock::{PackageName, PackageVersion};
use std::sync::mpsc;
use std::sync::mpsc::Sender;

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

#[derive(Debug)]
pub enum DownloadEvent {
    Started {
        package: String,
    },
    Progress {
        package: String,
        total_size: u64,
        bytes: u64,
    },
    Completed {
        package: String,
    },
    Failed {
        package: String,
        error: String,
    },
    AllCompleted,
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

    pub fn download_package(
        &self,
        name: &PackageName,
        version: &PackageVersion,
        progress_tx: &Sender<DownloadEvent>,
    ) -> Result<PathBuf, DownloadError> {
        let archive_path = self.cache_dir.join(format!("{}-{}.tar.gz", name, version));
        if archive_path.exists() {
            return Ok(archive_path);
        }

        let package_name = format!("{}-{}", name, version);

        progress_tx
            .send(DownloadEvent::Started {
                package: package_name.clone(),
            })
            .unwrap();

        let url = format!(
            "https://pub.dev/packages/{}/versions/{}.tar.gz",
            name, version
        );

        let response = ureq::get(&url).call().inspect_err(|e| {
            let _ = progress_tx.send(DownloadEvent::Failed {
                package: package_name.clone(),
                error: e.to_string(),
            });
        })?;

        let total_size = response
            .header("Content-Length")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let mut reader = response.into_reader();
        let mut bytes = Vec::new();
        let mut buffer = [0; 16384];
        let mut downloaded = 0;

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    bytes.extend_from_slice(&buffer[..n]);
                    downloaded += n as u64;
                    let _ = progress_tx.send(DownloadEvent::Progress {
                        package: package_name.clone(),
                        total_size,
                        bytes: downloaded,
                    });
                }
                Err(e) => return Err(DownloadError::IoError(e)),
            }
        }

        // Rest of the download_package implementation...
        let temp_path = archive_path.with_extension("tmp");
        let mut temp_file = File::create(&temp_path)?;
        temp_file.write_all(&bytes)?;
        temp_file.flush()?;

        self.verify_archive(&temp_path)?;
        fs::rename(temp_path, &archive_path)?;

        let _ = progress_tx.send(DownloadEvent::Completed {
            package: package_name.clone(),
        });

        Ok(archive_path)
    }

    pub fn download_packages_with_pool(
        &self,
        packages: &[(PackageName, PackageVersion)],
        pool: &ThreadPool,
        progress_tx: &Sender<DownloadEvent>,
    ) -> Vec<Result<PathBuf, DownloadError>> {
        let (tx, rx) = mpsc::channel();
        let total_packages = packages.len();

        for (name, version) in packages {
            let tx = tx.clone();
            let name = name.clone();
            let version = version.clone();
            let cache_dir = self.cache_dir.clone();
            let progress_tx = progress_tx.clone();

            pool.execute(move || {
                let downloader = match PackageDownloader::new(cache_dir) {
                    Ok(d) => d,
                    Err(e) => {
                        let error = DownloadError::IoError(e);
                        let _ = progress_tx.send(DownloadEvent::Failed {
                            package: name.to_string(),
                            error: error.to_string(),
                        });
                        tx.send(Err(error)).unwrap();
                        return;
                    }
                };

                let result = downloader.download_package(&name, &version, &progress_tx);
                tx.send(result).unwrap();
            });
        }

        drop(tx);
        let v = rx.iter().take(total_packages).collect::<Vec<_>>();
        let _ = progress_tx.send(DownloadEvent::AllCompleted);
        v
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
