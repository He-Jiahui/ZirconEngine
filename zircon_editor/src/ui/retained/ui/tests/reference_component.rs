fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

fn template_node_contract_source() -> String {
    [
        "src/ui/retained/contract/data/nodes.rs",
        "src/ui/retained/contract/data/nodes/actions.rs",
        "src/ui/retained/contract/data/nodes/collection.rs",
        "src/ui/retained/contract/data/nodes/menu.rs",
        "src/ui/retained/contract/data/nodes/node.rs",
        "src/ui/retained/contract/data/nodes/options.rs",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n")
}

fn component_showcase_reference_contract_source() -> String {
    [
        "assets/ui/editor/component_showcase.zui",
        "assets/ui/editor/components/showcase/selection_section.zui",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n")
}

#[test]
fn component_showcase_reference_wells_are_projected_into_rust_template_nodes() {
    let template_nodes = template_node_contract_source();
    let showcase_asset = component_showcase_reference_contract_source();

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
