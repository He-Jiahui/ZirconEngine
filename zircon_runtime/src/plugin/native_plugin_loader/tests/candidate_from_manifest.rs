use std::path::PathBuf;

use crate::plugin::{PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest};

use super::*;

#[test]
fn native_library_path_projection_preallocates_module_kind_bound() {
    let package_manifest = PluginPackageManifest::new("solari", "Solari")
        .with_runtime_module(PluginModuleManifest::runtime(
            "solari.runtime",
            "zircon_plugin_solari_runtime",
        ))
        .with_native_module(PluginModuleManifest::native(
            "solari.dist",
            "zircon_plugin_solari_dist",
        ))
        .with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            dist_crate: "zircon_plugin_solari_dist".to_string(),
            ..PluginDistributionManifest::default()
        });
    let candidate = NativePluginCandidate {
        plugin_id: "solari".to_string(),
        package_manifest,
        manifest_path: PathBuf::from("plugins/solari/plugin.toml"),
        library_path: PathBuf::new(),
    };

    let module_kinds = [PluginModuleKind::Runtime, PluginModuleKind::Editor];
    let paths = native_library_paths_for_candidate(&candidate, &module_kinds);

    assert_eq!(paths.len(), 1);
    assert_eq!(paths.capacity(), module_kinds.len());
    assert_eq!(paths[0].1, module_kinds);
    assert_eq!(
        paths[0].0,
        PathBuf::from("plugins/solari/native")
            .join(dynamic_library_file_name("zircon_plugin_solari_dist"))
    );
}

#[test]
fn bounded_manifest_read_error_preserves_file_context() {
    let missing_manifest = std::env::temp_dir().join(format!(
        "zircon-missing-native-plugin-manifest-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    let error = read_error(
        "native plugin manifest",
        &missing_manifest,
        io::Error::new(io::ErrorKind::NotFound, "manifest unavailable"),
    );

    assert!(matches!(
        error,
        NativePluginDiscoveryRefreshError::Collector { ref message }
            if message.contains("native plugin manifest")
                && message.contains(&missing_manifest.display().to_string())
                && message.contains("manifest unavailable")
    ));
}

#[test]
fn production_bounded_read_stability_check_rejects_short_or_changed_handle_reads() {
    let manifest_path = PathBuf::from("plugins/weather/plugin.toml");

    assert!(ensure_bounded_read_is_stable(
        "native plugin manifest",
        &manifest_path,
        12,
        12,
        12,
        12
    )
    .is_ok());
    for (filled, current_bytes) in [(11, 12), (12, 13), (11, 13)] {
        let error = ensure_bounded_read_is_stable(
            "native plugin manifest",
            &manifest_path,
            12,
            12,
            filled,
            current_bytes,
        )
        .expect_err("changed handle length must be rejected");
        assert!(error
            .to_string()
            .contains("native plugin manifest changed while it was read"));
    }
}

#[test]
fn manifest_candidate_typed_error_preserves_missing_module_message() {
    let error = NativePluginManifestCandidateError::MissingRuntimeOrEditorModule {
        plugin_id: "plugin.without.module".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "native plugin plugin.without.module has no runtime or editor module crate declared"
    );
    assert!(
        std::error::Error::source(&error).is_none(),
        "missing-module error should not invent an IO or TOML source"
    );
}
