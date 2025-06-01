use clap::Parser;
use flutter_pub::pubcache::{PubCache, PubCacheError};
use flutter_pub::scanner::Scanner;
use std::path::PathBuf;
use flutter_pub::pubspeclock::PackageDescription;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Directories to scan for pubspec files
    #[arg(short, long = "dir", required = true, num_args = 1.., value_name = "DIRECTORY")]
    dirs: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let scanner = Scanner::new(cli.dirs);
    let results = scanner.scan();

    let cache = PubCache::new(
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".pub-cache"),
    )
    .unwrap();

    for result in results {
        match result {
            Ok(info) => {
                println!(
                    "Found project '{}' at {}",
                    info.pubspec.name,
                    info.path.display()
                );

                if let Some(lockfile) = info.lock_file {
                    println!("  Dependencies:");
                    for (name, spec) in lockfile.packages {
                        match (spec.description) {
                            Some(desc) => {
                                let path = cache.get_package_path(name.clone(), spec.version.clone(), &desc);
                                match (path) {
                                    Ok(p) => {
                                        println!("Have {}@{} located at: {} ", name, spec.version, p.display());
                                    }
                                    Err(_) => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error scanning pubspec: {}", e),
        }
    }

    Ok(())
}
