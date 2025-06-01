use flutter_pub::pubspec::Pubspec;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_pubspec_file() -> NamedTempFile {
    let yaml_content = r#"
name: my_flutter_app
description: A new Flutter project
version: 1.0.0+1

environment:
  sdk: ">=2.12.0 <3.0.0"
  flutter: ">=2.0.0"

dependencies:
  flutter:
    sdk: flutter
  cupertino_icons: ^1.0.2
  http: ^0.13.0
  path_provider:
    git:
      url: https://github.com/flutter/plugins.git
      path: packages/path_provider
  local_package:
    path: ../local_package

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^1.0.0

flutter:
  uses-material-design: true
  assets:
    - assets/images/
    - assets/icons/
  fonts:
    - family: Schyler
      fonts:
        - asset: fonts/Schyler-Regular.ttf
        - asset: fonts/Schyler-Italic.ttf
          style: italic
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temporary file");
    temp_file.flush().expect("Failed to flush temporary file");
    temp_file
}

#[test]
fn test_load_pubspec() {
    let temp_file = create_test_pubspec_file();
    let pubspec = Pubspec::from_file(temp_file.path()).expect("Failed to load pubspec");

    // Test basic fields
    assert_eq!(pubspec.name, "my_flutter_app");
    assert_eq!(pubspec.version.as_deref(), Some("1.0.0+1"));
    
    // Test environment
    let env = pubspec.environment.unwrap();
    assert_eq!(env.sdk, ">=2.12.0 <3.0.0");
    assert_eq!(env.flutter.as_deref(), Some(">=2.0.0"));

    // Test dependencies
    assert!(pubspec.dependencies.contains_key("flutter"));
    assert!(pubspec.dependencies.contains_key("http"));
    assert!(pubspec.dev_dependencies.contains_key("flutter_test"));

    // Test Flutter config
    let flutter_config = pubspec.flutter.unwrap();
    assert_eq!(flutter_config.uses_material_design, Some(true));
    
    // Test assets
    let assets = flutter_config.assets.unwrap();
    assert!(assets.contains(&"assets/images/".to_string()));
    assert!(assets.contains(&"assets/icons/".to_string()));

    // Test fonts
    let fonts = flutter_config.fonts.unwrap();
    assert_eq!(fonts[0].family, "Schyler");
    assert_eq!(fonts[0].fonts.len(), 2);
}

#[test]
fn test_load_invalid_yaml() {
    let invalid_yaml = r#"invalid: - yaml: content"#;
    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    temp_file.write_all(invalid_yaml.as_bytes()).expect("Failed to write to temporary file");
    temp_file.flush().expect("Failed to flush temporary file");

    let result = Pubspec::from_file(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_load_nonexistent_file() {
    let result = Pubspec::from_file("nonexistent_pubspec.yaml");
    assert!(result.is_err());
}