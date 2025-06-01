use flutter_pub::{PackageConfig, Package};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_config_file() -> NamedTempFile {
    let json_content = r#"{
        "configVersion": 2,
        "packages": [
            {
                "name": "package_name",
                "rootUri": "../path/to/package",
                "packageUri": "lib/",
                "languageVersion": "2.12"
            }
        ],
        "generator": "pub",
        "generatorVersion": "2.15.1"
    }"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    temp_file.write_all(json_content.as_bytes()).expect("Failed to write to temporary file");
    temp_file.flush().expect("Failed to flush temporary file");
    temp_file
}

#[test]
fn test_load_package_config() {
    let temp_file = create_test_config_file();
    let config = PackageConfig::from_file(temp_file.path()).expect("Failed to load config");

    // Test root level fields
    assert_eq!(config.config_version, 2);
    assert_eq!(config.generator.unwrap(), "pub");
    assert_eq!(config.generator_version.unwrap(), "2.15.1");

    // Test packages array
    assert_eq!(config.packages.len(), 1);
    
    // Test first package
    let package = &config.packages[0];
    assert_eq!(package.name, "package_name");
    assert_eq!(package.root_uri.as_ref().unwrap(), "../path/to/package");
    assert_eq!(package.package_uri, "lib/");
    assert_eq!(package.language_version.as_ref().unwrap(), "2.12");
}

#[test]
fn test_load_invalid_json() {
    let invalid_json = r#"{ invalid json content }"#;
    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    temp_file.write_all(invalid_json.as_bytes()).expect("Failed to write to temporary file");
    temp_file.flush().expect("Failed to flush temporary file");

    let result = PackageConfig::from_file(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_load_nonexistent_file() {
    let result = PackageConfig::from_file("nonexistent_file.json");
    assert!(result.is_err());
}