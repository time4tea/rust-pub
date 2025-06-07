#[cfg(test)]
mod tests {
    use flutter_pub::downloader::{DownloadEvent, PackageDownloader};
    use flutter_pub::pubspeclock::{PackageName, PackageVersion};
    use std::sync::mpsc;
    use tempfile::TempDir;
    use threadpool::ThreadPool;

    #[test]
    fn test_download_single_package() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = PackageDownloader::new(temp_dir.path()).unwrap();

        let (tx, _rx) = mpsc::channel();

        let result = downloader.download_package(
            &PackageName::new("path"),
            &PackageVersion::new("1.8.3"),
            &tx,
        );
        assert!(result.is_ok());

        let archive_path = result.unwrap();
        assert!(archive_path.exists());
    }

    #[test]
    fn test_download_multiple_packages_with_pool() {
        let temp_dir = TempDir::new().unwrap();
        let pool = ThreadPool::new(4);
        let downloader = PackageDownloader::new(temp_dir.path()).unwrap();

        let packages = vec![
            (PackageName::new("path"), PackageVersion::new("1.8.3")),
            (PackageName::new("http"), PackageVersion::new("0.13.6")),
        ];
        let (tx, _rx) = mpsc::channel();

        let results = downloader.download_packages_with_pool(&packages, &pool, &tx);
        assert_eq!(results.len(), 2);

        let expected_files: std::collections::HashSet<String> = packages
            .iter()
            .map(|(name, version)| format!("{}-{}.tar.gz", name, version))
            .collect();

        for result in results {
            let path = result.expect("Download should succeed");
            assert!(path.exists(), "File should exist on disk");

            let file_name = path
                .file_name()
                .expect("Path should have a file name")
                .to_str()
                .expect("File name should be valid UTF-8")
                .to_string();

            assert!(
                expected_files.contains(&file_name),
                "File name '{}' should be one of the expected files",
                file_name
            );

            // Verify it's actually a file and has content
            let metadata = std::fs::metadata(&path).expect("Should be able to get metadata");
            assert!(metadata.is_file(), "Should be a regular file");
            assert!(metadata.len() > 0, "File should not be empty");
        }
    }

    #[test]
    fn test_download_nonexistent_package() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = PackageDownloader::new(temp_dir.path()).unwrap();
        let (tx, _rx) = mpsc::channel();

        let result = downloader.download_package(
            &PackageName::new("this_package_does_not_exist_12345"),
            &PackageVersion::new("1.0.0"),
            &tx,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_download_events_order() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = PackageDownloader::new(temp_dir.path()).unwrap();

        let (tx, rx) = mpsc::channel();
        let name = PackageName::new("path");
        let version = PackageVersion::new("1.8.3");

        // Spawn download in separate thread since it's blocking
        let handle = std::thread::spawn(move || downloader.download_package(&name, &version, &tx));

        // Collect all events
        let mut events = Vec::new();
        while let Ok(event) = rx.recv() {
            events.push(event);
        }

        // Wait for download to complete
        handle.join().unwrap().unwrap();

        // Verify event sequence
        assert!(matches!(
            &events[0],
            DownloadEvent::Started { package } if package == "path-1.8.3"
        ));

        // Verify all middle events are Progress
        for event in &events[1..events.len() - 1] {
            assert!(matches!(
                event,
                DownloadEvent::Progress { package, bytes, total_size } if package == "path-1.8.3" && *bytes > 0 && *total_size > 0
            ));
        }

        // Verify last event is Completed
        assert!(matches!(
            events.last().unwrap(),
            DownloadEvent::Completed { package } if package == "path-1.8.3"
        ));

        // Verify progress is increasing
        let progress_values: Vec<u64> = events
            .iter()
            .filter_map(|event| match event {
                DownloadEvent::Progress { bytes, .. } => Some(*bytes),
                _ => None,
            })
            .collect();

        // Check progress values are strictly increasing
        for i in 1..progress_values.len() {
            assert!(progress_values[i] > progress_values[i - 1]);
        }
    }

    #[test]
    fn test_download_error_events() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = PackageDownloader::new(temp_dir.path()).unwrap();

        let (tx, rx) = mpsc::channel();
        let name = PackageName::new("nonexistent-package");
        let version = PackageVersion::new("0.0.1");

        let handle = std::thread::spawn(move || downloader.download_package(&name, &version, &tx));

        let mut events = Vec::new();
        while let Ok(event) = rx.recv() {
            events.push(event);
        }

        assert!(handle.join().unwrap().is_err());

        // Should only get Started and Failed events
        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            DownloadEvent::Started { package, .. } if package == "nonexistent-package-0.0.1"
        ));
        assert!(matches!(
            &events[1],
            DownloadEvent::Failed { package, error: _ } if package == "nonexistent-package-0.0.1"
        ));
    }
}
