#[cfg(test)]
mod tests {
    use flutter_pub::pubcache::{PubCache, PubCacheError};
    use flutter_pub::pubspeclock::{PackageDescription, PackageName, PackageVersion, Sha256};
    use std::fs;
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
            .get_package_path(
                PackageName::new("test_package"),
                PackageVersion::new("1.0.0"),
                &desc,
            )
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
            .create_package_dir(
                PackageName::new("test_package"),
                PackageVersion::new("1.0.0"),
                &desc,
            )
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

        let result = cache.get_package_path(
            PackageName::new("test_package"),
            PackageVersion::new("1.0.0"),
            &desc,
        );
        assert!(matches!(result, Err(PubCacheError::UnsupportedSource)));
    }

    #[test]
    fn test_package_hash_operations() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PubCache::new(temp_dir.path()).unwrap();

        let host = "pub.dev";
        let package_name = "test_package";
        let version = "1.0.0";
        let hash = "abcdef1234567890";

        // Initially, there should be no hash
        assert_eq!(
            cache
                .read_package_hash(host, package_name, version)
                .unwrap(),
            None
        );

        // Write the hash
        cache
            .write_package_hash(host, package_name, version, hash)
            .unwrap();

        // Read it back
        assert_eq!(
            cache
                .read_package_hash(host, package_name, version)
                .unwrap(),
            Some(hash.to_string())
        );

        // Verify the hash
        assert!(
            cache
                .verify_package_hash(host, package_name, version, hash)
                .unwrap()
        );
        assert!(
            !cache
                .verify_package_hash(host, package_name, version, "wrong_hash")
                .unwrap()
        );

        // Check the file structure
        let hash_file = temp_dir
            .path()
            .join("hosted-hashes")
            .join(host)
            .join(format!("{}.{}.sha256", package_name, version));
        assert!(hash_file.exists());

        // Verify file content
        let content = fs::read_to_string(hash_file).unwrap();
        assert_eq!(content, hash);
    }
}
