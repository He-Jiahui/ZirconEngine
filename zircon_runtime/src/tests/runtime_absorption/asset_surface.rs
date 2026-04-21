#[test]
fn asset_module_registration_is_absorbed_into_runtime_asset_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_entry = runtime_root.join("src/asset.rs");
    let asset_mod = runtime_root.join("src/asset/mod.rs");
    let legacy_asset_lib = runtime_root.join("../zircon_asset/src/lib.rs");

    let asset_entry_source = std::fs::read_to_string(&asset_entry).unwrap_or_default();
    let asset_mod_source = std::fs::read_to_string(&asset_mod).unwrap_or_default();

    assert!(
        asset_mod.exists(),
        "expected zircon_runtime/src/asset/mod.rs to own the absorbed asset module registration surface"
    );
    assert!(
        asset_mod_source.contains("AssetModule"),
        "zircon_runtime::asset should define AssetModule after asset module absorption"
    );
    assert!(
        !asset_entry_source.contains("pub use zircon_asset::*"),
        "zircon_runtime/src/asset.rs should stop re-exporting the entire zircon_asset crate after absorption"
    );
    assert!(
        !asset_mod_source.contains("pub use zircon_asset::*"),
        "zircon_runtime/src/asset/mod.rs should stop wildcard-re-exporting zircon_asset"
    );
    assert!(
        !legacy_asset_lib.exists(),
        "standalone zircon_asset crate should be removed after merging into zircon_runtime::asset"
    );
}

#[test]
fn runtime_asset_surface_keeps_project_and_watch_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_mod_source =
        std::fs::read_to_string(runtime_root.join("src/asset/mod.rs")).unwrap_or_default();
    let legacy_asset_editor_surface = runtime_root.join("src/asset/editor");

    for required in [
        "pub mod artifact;",
        "pub mod assets;",
        "pub mod importer;",
        "pub mod pipeline;",
        "pub mod project;",
        "pub mod watch;",
    ] {
        assert!(
            asset_mod_source.contains(required),
            "zircon_runtime::asset should expose namespace surface `{required}`"
        );
    }

    for forbidden in [
        "pub use zircon_asset::ArtifactStore;",
        "pub use zircon_asset::MaterialAsset;",
        "pub use zircon_asset::ProjectAssetManager;",
        "pub use zircon_asset::EditorAssetManager;",
        "pub use zircon_asset::AssetWorkerPool;",
        "pub use zircon_asset::AssetId;",
        "pub use zircon_asset::AssetKind;",
        "pub use zircon_asset::AssetReference;",
        "pub use zircon_asset::AssetUri;",
        "pub use zircon_asset::AssetUuid;",
        "pub use zircon_asset::project::{",
        "pub use zircon_asset::watch::{AssetChange, AssetChangeKind, AssetWatchEvent, AssetWatcher};",
        "pub use zircon_asset::{",
    ] {
        assert!(
            !asset_mod_source.contains(forbidden),
            "zircon_runtime::asset should stop flattening namespace-owned surface `{forbidden}`"
        );
    }

    assert!(
        !asset_mod_source.contains("pub mod editor;"),
        "zircon_runtime::asset should not keep the absorbed editor asset surface at the runtime root"
    );
    assert!(
        !legacy_asset_editor_surface.exists(),
        "runtime asset namespace should delete the legacy editor surface after zircon_editor absorbs it"
    );
}
