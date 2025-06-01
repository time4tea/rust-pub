use flutter_pub::packageconfig::{Package, PackageConfig};
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
    temp_file
        .write_all(json_content.as_bytes())
        .expect("Failed to write to temporary file");
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
    temp_file
        .write_all(invalid_json.as_bytes())
        .expect("Failed to write to temporary file");
    temp_file.flush().expect("Failed to flush temporary file");

    let result = PackageConfig::from_file(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_load_nonexistent_file() {
    let result = PackageConfig::from_file("nonexistent_file.json");
    assert!(result.is_err());
}

#[test]
fn test_package_config_roundtrip() {
    let mut original_config = PackageConfig::new();

    // Add some test data
    let package = Package::new("test_package".to_string(), "lib/".to_string())
        .with_root_uri("file:///test/path".to_string())
        .with_root_path("/test/path".to_string())
        .with_language_version("2.12".to_string());

    original_config.add_package(package);
    original_config.generator = Some("test_generator".to_string());
    original_config.generator_version = Some("1.0.0".to_string());

    // Create a temporary file
    let temp_file = NamedTempFile::new().expect("Failed to create temporary file");

    // Write the config to the file
    original_config
        .write_to_file(temp_file.path())
        .expect("Failed to write config");

    // Read it back
    let loaded_config =
        PackageConfig::from_file(temp_file.path()).expect("Failed to load written config");

    // Verify the contents match
    assert_eq!(loaded_config.config_version, original_config.config_version);
    assert_eq!(loaded_config.generator, original_config.generator);
    assert_eq!(
        loaded_config.generator_version,
        original_config.generator_version
    );
    assert_eq!(loaded_config.packages.len(), original_config.packages.len());

    // Check the package contents
    let original_package = &original_config.packages[0];
    let loaded_package = &loaded_config.packages[0];

    assert_eq!(loaded_package.name, original_package.name);
    assert_eq!(loaded_package.root_uri, original_package.root_uri);
    assert_eq!(loaded_package.root_path, original_package.root_path);
    assert_eq!(loaded_package.package_uri, original_package.package_uri);
    assert_eq!(
        loaded_package.language_version,
        original_package.language_version
    );
    assert_eq!(loaded_package.supported, original_package.supported);
}
