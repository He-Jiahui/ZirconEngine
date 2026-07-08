use super::support::{read_runtime_file, runtime_root};

#[test]
fn asset_module_registration_is_absorbed_into_runtime_asset_surface() {
    let runtime_root = runtime_root();
    let asset_mod = runtime_root.join("src/asset/mod.rs");
    let legacy_asset_lib = runtime_root.join("../zircon_asset/src/lib.rs");

    let asset_entry_source = read_runtime_file("src/asset.rs");
    let asset_mod_source = read_runtime_file("src/asset/mod.rs");

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
