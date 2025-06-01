#[cfg(test)]
mod tests {
    use flutter_pub::scanner::Scanner;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_scanner() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test directory structure
        let project1_dir = temp_dir.path().join("project1");
        let project2_dir = temp_dir.path().join("project2");
        let nested_dir = project1_dir.join("nested");

        fs::create_dir_all(&project1_dir).unwrap();
        fs::create_dir_all(&project2_dir).unwrap();
        fs::create_dir_all(&nested_dir).unwrap();

        // Create test pubspec files
        let pubspec1 = r#"
name: project1
version: 1.0.0
"#;
        let pubspec2 = r#"
name: project2
version: 2.0.0
"#;

        File::create(project1_dir.join("pubspec.yaml"))
            .unwrap()
            .write_all(pubspec1.as_bytes())
            .unwrap();

        File::create(project2_dir.join("pubspec.yaml"))
            .unwrap()
            .write_all(pubspec2.as_bytes())
            .unwrap();

        // Create a scanner and scan
        let scanner = Scanner::new(vec![temp_dir.path().to_path_buf()]);
        let results = scanner.scan();

        // Verify results
        assert_eq!(results.len(), 2);

        for result in results {
            let info = result.unwrap();
            match info.pubspec.name.as_str() {
                "project1" => assert_eq!(info.pubspec.version.as_deref(), Some("1.0.0")),
                "project2" => assert_eq!(info.pubspec.version.as_deref(), Some("2.0.0")),
                _ => panic!("Unexpected project name"),
            }
        }
    }
}
