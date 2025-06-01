use clap::Parser;
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

    for result in results {
        match result {
            Ok(info) => {
                println!(
                    "Found project '{}' at {}",
                    info.pubspec.name,
                    info.path.display()
                );

                if let Some(lock) = info.lock_file {
                    println!("  Dependencies:");
                    for (name, spec) in lock.packages {
                        println!("    {} {} {}", name, match &spec.description {
                            None => "".to_string(),
                            Some(PackageDescription::Hosted{name,url,sha256}) => {
                                format!("Name: {}, URL: {}, SHA256: {}", &name, &url, &sha256)
                            },
                            Some(_) => "".to_string()
                        }, spec.version);
                    }
                }
            }
            Err(e) => eprintln!("Error scanning pubspec: {}", e),
        }
    }

    Ok(())
}
