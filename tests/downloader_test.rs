#[cfg(test)]
mod tests {
    use flutter_pub::downloader::PackageDownloader;
    use flutter_pub::pubspeclock::{PackageName, PackageVersion};
    use tempfile::TempDir;
    use threadpool::ThreadPool;

    #[test]
    fn test_download_single_package() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = PackageDownloader::new(temp_dir.path()).unwrap();

        let result =
            downloader.download_package(&PackageName::new("path"), &PackageVersion::new("1.8.3"));
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

        let results = downloader.download_packages_with_pool(&packages, &pool);
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

        let result = downloader.download_package(
            &PackageName::new("this_package_does_not_exist_12345"),
            &PackageVersion::new("1.0.0"),
        );

        assert!(result.is_err());
    }
}
