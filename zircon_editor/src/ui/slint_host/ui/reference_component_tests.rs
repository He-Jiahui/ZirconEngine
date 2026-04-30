fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn component_showcase_reference_wells_are_projected_into_rust_template_nodes() {
    let template_nodes = source("src/ui/slint_host/host_contract/data/template_nodes.rs");
    let showcase_asset = source("assets/ui/editor/component_showcase.ui.toml");

    for required in [
        "pub accepted_drag_payloads: SharedString",
        "pub drop_source_summary: SharedString",
        "pub validation_message: SharedString",
        "pub drop_hovered: bool",
        "pub active_drag_target: bool",
        "pub actions: ModelRc<TemplatePaneActionData>",
    ] {
        assert!(
            template_nodes.contains(required),
            "template node DTO missing `{required}`"
        );
    }
    for required in [
        "AssetFieldDemo",
        "InstanceFieldDemo",
        "ObjectFieldDemo",
        "UiComponentShowcase/AssetFieldDropped",
    ] {
        assert!(
            showcase_asset.contains(required),
            "component showcase asset missing `{required}`"
        );
    }
}
