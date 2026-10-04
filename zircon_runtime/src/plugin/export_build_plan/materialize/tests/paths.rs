use super::validated_materialized_relative_path;

#[test]
fn streaming_normalization_preserves_portable_paths() {
    assert_eq!(
        validated_materialized_relative_path("plugins/rendering/plugin.toml")
            .expect("portable path should remain valid"),
        "plugins/rendering/plugin.toml"
    );
    assert_eq!(
        validated_materialized_relative_path("plugins//rendering///plugin.toml")
            .expect("repeated separators should normalize"),
        "plugins/rendering/plugin.toml"
    );
}

#[test]
fn streaming_normalization_preserves_path_rejections() {
    for path in [
        "",
        "./plugin.toml",
        "../plugin.toml",
        "plugins\\plugin.toml",
        "plugins/",
    ] {
        assert!(
            validated_materialized_relative_path(path).is_err(),
            "path {path:?} should remain invalid"
        );
    }
}
