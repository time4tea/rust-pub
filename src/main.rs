use clap::Parser;
use flutter_pub::extensions::FilterNotIterator;
use flutter_pub::pubcache::PubCache;
use flutter_pub::pubspeclock::{HostedPackage, PackageDescription, PackageName, PackageVersion};
use flutter_pub::scanner::{PubspecInfo, Scanner};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long = "dir", required = true, num_args = 1.., value_name = "DIRECTORY")]
    dirs: Vec<PathBuf>,
}

struct HostedDependency {
    name: PackageName,
    version: PackageVersion,
    hosted: HostedPackage,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let scanner = Scanner::new(cli.dirs);
    let results = scanner.scan();

    let cache = PubCache::new(
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".pub-cache-2"),
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

    let hosted_packages = hosted_packages_from(results);

    let missing_packages = packages_missing_in_cache(&cache, &hosted_packages);

    if missing_packages.is_empty() {
        println!("All packages are cached");
    } else {
        missing_packages.iter().for_each(|p| {
            println!("{}: {}  sha:{}", p.name, p.version, p.hosted);
        });
    }

    Ok(())
}

fn hosted_packages_from(
    results: Vec<Result<PubspecInfo, Box<dyn Error>>>,
) -> Vec<HostedDependency> {
    let hosted_packages: Vec<HostedDependency> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|info| info.lock_file.as_ref())
        .flat_map(|lockfile| &lockfile.packages)
        .filter_map(|(name, spec)| {
            spec.description.as_ref().and_then(|desc| match desc {
                PackageDescription::Hosted(hosted) => Some(HostedDependency {
                    name: name.clone(),
                    version: spec.version.clone(),
                    hosted: hosted.clone(),
                }),
                _ => None,
            })
        })
        .collect();
    hosted_packages
}

fn packages_missing_in_cache<'a>(
    cache: &PubCache,
    hosted_packages: &'a Vec<HostedDependency>,
) -> Vec<&'a HostedDependency> {
    let missing_packages: Vec<_> = hosted_packages
        .iter()
        .filter_not(|d| {
            cache
                .get_package_path(&d.name, &d.version, &d.hosted)
                .map(|path| path.exists())
                .unwrap_or(false)
        })
        .fold(BTreeMap::new(), |mut map, package| {
            map.entry(&package.name).or_insert(package);
            map
        })
        .into_values()
        .collect();
    missing_packages
}
