use flutter_pub::PackageConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PackageConfig::from_file("package_config.json")?;

    println!("Config version: {}", config.config_version);
    println!("\nPackages:");
    for package in config.packages {
        println!("\nPackage: {}", package.name);
        if let Some(root_uri) = package.root_uri {
            println!("Root URI: {}", root_uri);
        }
        println!("Package URI: {}", package.package_uri);
        if let Some(lang_ver) = package.language_version {
            println!("Language version: {}", lang_ver);
        }
    }

    Ok(())
}