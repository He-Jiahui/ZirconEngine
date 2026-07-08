use super::support::{
    assert_contains_all, assert_not_contains_all, read_runtime_file, runtime_root,
};

#[test]
fn runtime_asset_surface_keeps_project_and_watch_under_namespaces() {
    let runtime_root = runtime_root();
    let asset_mod_source = read_runtime_file("src/asset/mod.rs");
    let legacy_asset_editor_surface = runtime_root.join("src/asset/editor");

    assert_contains_all(
        "zircon_runtime::asset namespace surface",
        &asset_mod_source,
        &[
            "pub mod artifact;",
            "pub mod assets;",
            "pub mod importer;",
            "pub mod pipeline;",
            "pub mod project;",
            "pub mod watch;",
        ],
    );

    assert_not_contains_all(
        "zircon_runtime::asset namespace surface",
        &asset_mod_source,
        &[
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
            "pub mod editor;",
        ],
    );

    assert!(
        !legacy_asset_editor_surface.exists(),
        "runtime asset namespace should delete the legacy editor surface after zircon_editor absorbs it"
    );
}
