use clap::Parser;
use flutter_pub::downloader::{DownloadEvent, PackageDownloader};
use flutter_pub::extensions::FilterNotIterator;
use flutter_pub::pubcache::PubCache;
use flutter_pub::pubspeclock::{HostedPackage, PackageDescription, PackageName, PackageVersion};
use flutter_pub::scanner::{PubspecInfo, Scanner, ScannerError};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use threadpool::ThreadPool;

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

    let user_home = dirs::home_dir().expect("Could not find home directory");

    let pub_cache = PubCache::new(&user_home.join(".pub-cache-2"))?;
    let downloader = PackageDownloader::new(pub_cache.download_path())?;

    let pub_specs = Scanner::new(cli.dirs).scan();
    
    let had_errors = pub_specs
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

    let hosted_packages = hosted_packages_from(pub_specs);
    let missing_packages = packages_missing_in_cache(&pub_cache, &hosted_packages);

    if missing_packages.is_empty() {
        println!("All packages are cached");
    } else {
        let threadpool = ThreadPool::new(8);

        println!("Downloading {} packages...", missing_packages.len());

        let things = missing_packages
            .iter()
            .map(|hp| (hp.name.clone(), hp.version.clone()))
            .collect::<Vec<_>>();

        let (tx, rx) = mpsc::channel();

        let count = things.len() as u64;

        thread::spawn(move || {
            display_progress_ind(count, rx);
        });

        downloader.download_packages_with_pool(&things, &threadpool, &tx);
    }

    Ok(())
}

fn hosted_packages_from(
    results: Vec<Result<PubspecInfo, ScannerError>>,
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
                .inspect(|p| println!("{}", p.display()))
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

fn display_progress_ind(expected: u64, rx: Receiver<DownloadEvent>) {
    let multi = MultiProgress::new();
    let main_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:20.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
        .unwrap()
        .progress_chars("#>-");

    let download_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:20.cyan/blue}] {bytes}/{total_bytes} ({percent}%) {msg}")
        .unwrap()
        .progress_chars("=>-");

    let overall = multi.add(ProgressBar::new(0));
    overall.set_style(main_style);
    overall.set_message("Starting downloads...");

    let mut active_downloads = HashMap::new();

    for event in rx {
        match event {
            DownloadEvent::Started { package } => {
                let pb = multi.add(ProgressBar::new(0));
                pb.set_style(download_style.clone());
                pb.set_message(format!("Downloading {}", package));
                active_downloads.insert(package, pb);
                overall.set_length(expected);
            }

            DownloadEvent::Progress {
                package,
                bytes,
                total_size,
            } => {
                if let Some(pb) = active_downloads.get(&package) {
                    pb.set_length(total_size);
                    pb.set_position(bytes);
                }
            }

            DownloadEvent::Completed { package } => {
                if let Some(pb) = active_downloads.get(&package) {
                    pb.finish_with_message(format!("{} downloaded successfully", package));
                    overall.inc(1);
                    multi.remove(&pb);
                    active_downloads.remove(&package);
                }
            }

            DownloadEvent::Failed { package, error } => {
                if let Some(pb) = active_downloads.get(&package) {
                    pb.abandon_with_message(format!("{} failed: {}", package, error));
                    overall.inc(1);
                }
            }

            DownloadEvent::AllCompleted => {
                overall.finish_with_message("All downloads completed!");
                break;
            }
        }
    }
}
