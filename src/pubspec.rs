
use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Pubspec {
    /// Package name
    pub name: String,
    /// Package description
    pub description: Option<String>,
    /// Package version
    pub version: Option<String>,
    /// Package homepage
    pub homepage: Option<String>,
    /// Package repository URL
    pub repository: Option<String>,
    /// Package documentation URL
    pub documentation: Option<String>,
    /// Package environment constraints
    pub environment: Option<Environment>,
    /// Package dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    /// Development dependencies
    #[serde(default)]
    pub dev_dependencies: HashMap<String, DependencySpec>,
    /// Dependencies that are not included in the app
    #[serde(default)]
    pub dependency_overrides: HashMap<String, DependencySpec>,
    /// Flutter-specific configuration
    pub flutter: Option<FlutterConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    pub sdk: String,
    pub flutter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Simple(String),
    Detailed(DetailedDependency),
    Git(GitDependency),
    Path(PathDependency),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedDependency {
    pub version: Option<String>,
    pub hosted: Option<HostedDependency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitDependency {
    pub git: GitRepo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitRepo {
    pub url: String,
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathDependency {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HostedDependency {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlutterConfig {
    /// Flutter SDK version constraint
    pub sdk: Option<String>,
    /// Assets to include in the app
    pub assets: Option<Vec<String>>,
    /// Fonts to include in the app
    pub fonts: Option<Vec<FontFamily>>,
    /// Uses-material-design flag
    #[serde(rename = "uses-material-design")]
    pub uses_material_design: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontFamily {
    pub family: String,
    pub fonts: Vec<FontFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontFile {
    pub asset: String,
    pub weight: Option<i32>,
    pub style: Option<String>,
}

impl Pubspec {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let pubspec = serde_yaml::from_str(&contents)?;
        Ok(pubspec)
    }
}