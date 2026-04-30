use std::fs;
use std::path::Path;

fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn ui_asset_theme_shell_contract_is_rust_owned_and_toml_projected() {
    let ui_asset = source("src/ui/slint_host/host_contract/data/ui_asset.rs");
    let callbacks = source("src/ui/slint_host/host_contract/globals.rs");
    let asset = source("assets/ui/editor/ui_asset_editor.ui.toml");

    for required in [
        "pub(crate) struct UiAssetThemeSourceData",
        "pub can_promote_local: bool",
        "pub cascade_layer_items: ModelRc<SharedString>",
        "pub cascade_token_items: ModelRc<SharedString>",
        "pub cascade_rule_items: ModelRc<SharedString>",
        "pub compare_items: ModelRc<SharedString>",
        "pub merge_preview_items: ModelRc<SharedString>",
        "pub rule_helper_items: ModelRc<SharedString>",
        "pub refactor_items: ModelRc<SharedString>",
        "pub promote_asset_id: SharedString",
        "pub can_prune_duplicate_local_overrides: bool",
    ] {
        assert!(ui_asset.contains(required), "theme DTO missing `{required}`");
    }
    assert!(callbacks.contains("on_ui_asset_action"));
    assert!(asset.contains("StylesheetThemeSection"));
}
