#[cfg(test)]
mod tests {
    use flutter_pub::pubspeclock::{PackageDescription, PubspecLock};
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_pubspec_lock() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("pubspec.lock");

        // Create a sample pubspec.lock
        let sample_content = r#"
sdks:
  dart: ">=2.12.0 <3.0.0"
  flutter: ">=2.0.0"
packages:
  adaptive_number:
    dependency: transitive
    description:
      name: adaptive_number
      sha256: "3a567544e9b5c9c803006f51140ad544aedc79604fd4f3f2c1380003f97c1d77"
      url: "https://pub.dev"
    source: hosted
    version: "1.0.0"
  path:
    version: "1.8.3"
    dependency: "direct main"
    source: "hosted"
    description:
      name: "path"
      sha256: "1234"
      url: "https://pub.dev"
  http:
    dependency: transitive
    version: "0.13.6"
    source: "hosted"
    description:
      name: "http"
      sha256: "4567"
      url: "https://pub.dev"
"#;

        File::create(&lock_path)
            .unwrap()
            .write_all(sample_content.as_bytes())
            .unwrap();

        // Test loading
        let lock_file = PubspecLock::from_file(lock_path).unwrap();

        // Verify contents
        assert!(lock_file.sdks.is_some());
        assert_eq!(lock_file.packages.len(), 3);

        let adaptive_pkg = lock_file.packages.get("adaptive_number").unwrap();
        assert_eq!(adaptive_pkg.version, "1.0.0");
        match &adaptive_pkg.description.as_ref().unwrap() {
            PackageDescription::Hosted {
                name,
                url,
                sha256,
            } => {
                assert_eq!(name, "adaptive_number");
                assert_eq!(url, "https://pub.dev");
                assert_eq!(sha256, "3a567544e9b5c9c803006f51140ad544aedc79604fd4f3f2c1380003f97c1d77");
            }
            _ => panic!("Expected Hosted variant"),
        }

        let path_pkg = lock_file.packages.get("path").unwrap();
        assert_eq!(path_pkg.version, "1.8.3");

        let http_pkg = lock_file.packages.get("http").unwrap();
        assert_eq!(http_pkg.version, "0.13.6");
    }
}
