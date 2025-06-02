#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use flutter_pub::pubpackage::PubPackageVersion;
    use flutter_pub::pubspeclock::{PackageVersion, Sha256};

    #[test]
    fn test_parse_package_version() {
        let json = r#"{
            "version": "0.13.6",
            "pubspec": {
                "name": "http",
                "version": "0.13.6",
                "description": "A composable, multi-platform, Future-based API for HTTP requests.",
                "repository": "https://github.com/dart-lang/http/tree/master/pkgs/http",
                "environment": {
                    "sdk": ">=2.19.0 <3.0.0"
                },
                "dependencies": {
                    "async": "^2.5.0",
                    "http_parser": "^4.0.0",
                    "meta": "^1.3.0"
                }
            },
            "archive_url": "https://pub.dev/api/archives/http-0.13.6.tar.gz",
            "archive_sha256": "5895291c13fa8a3bd82e76d5627f69e0d85ca6a30dcac95c4ea19a5d555879c2",
            "published": "2023-05-01T17:54:17.086948Z"
        }"#;

        let package = PubPackageVersion::from_json(json).unwrap();
        assert_eq!(package.version, PackageVersion::new("0.13.6"));
        assert_eq!(
            package.archive_sha256,
            Sha256::new("5895291c13fa8a3bd82e76d5627f69e0d85ca6a30dcac95c4ea19a5d555879c2")
        );
        assert_eq!(
            package.published,
            DateTime::parse_from_rfc3339("2023-05-01T17:54:17.086948Z").unwrap()
        );
    }
}
