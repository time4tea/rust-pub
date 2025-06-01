use crate::pubspec::Pubspec;
use crate::pubspeclock::PubspecLock;
use std::error::Error;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct PubspecInfo {
    pub path: PathBuf,
    pub pubspec: Pubspec,
    pub lock_file: Option<PubspecLock>,
}

pub struct Scanner {
    root_dirs: Vec<PathBuf>,
}

impl Scanner {
    pub fn new(dirs: Vec<PathBuf>) -> Self {
        Self { root_dirs: dirs }
    }

    fn is_pubspec_yaml(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s == "pubspec.yaml")
            .unwrap_or(false)
    }

    fn load_pubspec_files(pubspec_path: &Path) -> Result<PubspecInfo, Box<dyn Error>> {
        println!("Loading pubspec file: {}", pubspec_path.display());
        let pubspec = Pubspec::from_file(pubspec_path)?;

        // Try to load the lock file if it exists
        let lock_path = pubspec_path.with_file_name("pubspec.lock");
        let lock_file = if lock_path.exists() {
            Some(PubspecLock::from_file(&lock_path)?)
        } else {
            None
        };

        Ok(PubspecInfo {
            path: pubspec_path.to_path_buf(),
            pubspec,
            lock_file,
        })
    }

    pub fn scan(&self) -> Vec<Result<PubspecInfo, Box<dyn Error>>> {
        let mut results = Vec::new();

        for root_dir in &self.root_dirs {
            let mut walker = WalkDir::new(root_dir).follow_links(true).into_iter();

            loop {
                match walker.next() {
                    None => {
                        break;
                    }
                    Some(Ok(entry)) => {
                        if Self::is_pubspec_yaml(&entry) {
                            // Found a pubspec.yaml file
                            let result = Self::load_pubspec_files(entry.path());
                            results.push(result);
                            walker.skip_current_dir();
                        }
                    }
                    Some(Err(err)) => {
                        panic!("Error walking directory: {}", err);
                    }
                };
            }
        }

        results
    }
}
