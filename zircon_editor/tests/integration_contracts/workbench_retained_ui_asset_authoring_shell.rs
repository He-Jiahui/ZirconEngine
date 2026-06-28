use std::fs;
use std::path::Path;

fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

fn sources(relatives: &[&str]) -> String {
    relatives
        .iter()
        .map(|relative| source(relative))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn ui_asset_authoring_preview_and_binding_contracts_are_rust_owned() {
    let ui_asset = sources(&[
        "src/ui/retained_host/host_contract/data/ui_asset.rs",
        "src/ui/retained_host/host_contract/data/ui_asset/preview.rs",
        "src/ui/retained_host/host_contract/data/ui_asset/inspector/binding.rs",
    ]);
    let asset = source("assets/ui/editor/ui_asset_editor.zui");

    for required in [
        "pub(crate) struct UiAssetPreviewMockData",
        "pub subject_collection: UiAssetStringSelectionData",
        "pub subject_node_id: SharedString",
        "pub expression_result: SharedString",
        "pub schema_items: ModelRc<SharedString>",
        "pub(crate) struct UiAssetInspectorBindingData",
        "pub route_suggestion_collection: UiAssetStringSelectionData",
        "pub action_suggestion_collection: UiAssetStringSelectionData",
    ] {
        assert!(
            ui_asset.contains(required),
            "ui asset DTO missing `{required}`"
        );
    }

    for required in [
        "MockWorkspacePanel",
        "MockSubjectsPanel",
        "MockEditorPanel",
        "MockStateGraphPanel",
        "InspectorBindingSection",
    ] {
        assert!(
            asset.contains(required),
            "ui asset TOML missing `{required}`"
        );
    }
}
