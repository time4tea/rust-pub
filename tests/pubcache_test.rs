#[cfg(test)]
mod tests {
    use flutter_pub::pubcache::{PubCache, PubCacheError};
    use flutter_pub::pubspeclock::{PackageDescription, PackageName, Sha256};
    use tempfile::TempDir;
    use url::Url;

    #[test]
    fn test_create_pub_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PubCache::new(temp_dir.path()).unwrap();
        assert!(cache.root_path().exists());
    }

    #[test]
    fn test_hosted_package_path() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PubCache::new(temp_dir.path()).unwrap();

        let desc = PackageDescription::Hosted {
            name: PackageName::new("test_package"),
            url: Url::parse("https://pub.dev").unwrap(),
            sha256: Sha256::new("abc123"),
        };

        let path = cache
            .get_package_path("test_package", "1.0.0", &desc)
            .unwrap();
        assert_eq!(
            path,
            cache
                .root_path()
                .join("hosted")
                .join("pub.dev")
                .join("test_package-1.0.0")
        );
    }

    #[test]
    fn test_create_hosted_package_dir() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PubCache::new(temp_dir.path()).unwrap();

        let desc = PackageDescription::Hosted {
            name: PackageName::new("test_package"),
            url: Url::parse("https://pub.dev").unwrap(),
            sha256: Sha256::new("abc123"),
        };

        let path = cache
            .create_package_dir("test_package", "1.0.0", &desc)
            .unwrap();
        assert!(path.exists());
        assert!(path.is_dir());
        assert_eq!(
            path,
            cache
                .root_path()
                .join("hosted")
                .join("pub.dev")
                .join("test_package-1.0.0")
        );
    }

    #[test]
    fn test_unsupported_package_source() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PubCache::new(temp_dir.path()).unwrap();

        let desc = PackageDescription::Path {
            path: "/some/path".to_string(),
        };

        let result = cache.get_package_path("test_package", "1.0.0", &desc);
        assert!(matches!(result, Err(PubCacheError::UnsupportedSource)));
    }
}
