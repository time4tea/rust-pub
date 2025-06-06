use clap::Parser;
use flutter_pub::pubcache::PubCache;
use flutter_pub::pubspeclock::PackageDescription;
use flutter_pub::scanner::Scanner;
use std::collections::HashMap;
use std::path::PathBuf;

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

    let _cache = PubCache::new(
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".pub-cache"),
    )
    .unwrap();

    let had_errors = results
        .iter()
        .filter_map(|r| r.as_ref().err())
        .inspect(|e| {
            eprintln!("Error: {}", e);
        })
        .next()
        .is_some();

    if had_errors {
        panic!("Problems with pubspecs...");
    }

    let hosted_packages: HashMap<_, _> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|info| info.lock_file.as_ref())
        .flat_map(|lockfile| &lockfile.packages)
        .filter_map(|(name, spec)| {
            spec.description.as_ref().and_then(|desc| match desc {
                PackageDescription::Hosted(hosted) => Some((name.clone(), hosted)),
                _ => None,
            })
        })
        .collect();

    
    
    
    hosted_packages.iter().for_each(|(name, hosted)| {
        println!("{}: {}  sha:{}", name, hosted.url, hosted.sha256);
    });

    Ok(())
}
