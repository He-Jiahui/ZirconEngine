use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::{StretchMode, UiContainerKind},
};

const PENPOT_ROUNDTRIP_ZUI: &str = include_str!("fixtures/ui/penpot_roundtrip.zui");

#[test]
fn penpot_roundtrip_output_builds_a_retained_zui_surface() {
    let document = UiZuiAssetLoader::load_zui_str(PENPOT_ROUNDTRIP_ZUI)
        .expect("Penpot bridge output should satisfy the .zui v2 source profile");

    assert_eq!(document.asset.id, "res://ui/tests/penpot_roundtrip.zui");
    assert_eq!(document.root_node_id(), Some("root"));
    assert_eq!(
        document.nodes["virtual_rows"]
            .repeat
            .as_ref()
            .map(|repeat| repeat.prototype.as_str()),
        Some("row_template")
    );

    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("Penpot bridge output should compile into Zircon's retained UI arena");
    assert_eq!(compiled.asset_id, document.asset.id);
    assert!(compiled.node_handles.contains_key("root"));
    assert!(compiled.node_handles.contains_key("row_template"));
    assert!(compiled.node_handles.contains_key("detached_template"));

    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("penpot-roundtrip-contract"),
        &document,
        &compiled,
    )
    .expect("Penpot bridge output should build Zircon's retained UI surface");
    assert_eq!(surface.tree.roots.len(), 1);
    assert_eq!(surface.tree.nodes.len(), 7);
    assert_eq!(surface.tree.layout_slots().len(), 6);
    assert!(!surface
        .tree
        .nodes
        .values()
        .any(|node| node.node_path.0.contains("detached_template")));

    let root = surface
        .tree
        .node(surface.tree.roots[0])
        .expect("retained surface root");
    assert_eq!(root.constraints.width.preferred, 420.0);
    assert_eq!(root.constraints.width.stretch_mode, StretchMode::Fixed);
    assert_eq!(root.constraints.height.preferred, 320.0);
    assert_eq!(root.constraints.height.stretch_mode, StretchMode::Fixed);
    assert_eq!(root.position.x, 40.0);
    assert_eq!(root.position.y, 60.0);
    assert_eq!(
        root.layout_padding,
        zircon_runtime_interface::ui::layout::UiMargin::new(20.0, 16.0, 20.0, 16.0)
    );
    assert!(root.clip_to_bounds);
    let UiContainerKind::VerticalBox(root_box) = root.container else {
        panic!("Penpot root should remain a vertical box");
    };
    assert_eq!(root_box.gap, 12.0);

    let root_layout = root
        .template_metadata
        .as_ref()
        .and_then(|metadata| metadata.attributes.get("layout"))
        .and_then(toml::Value::as_table)
        .expect("root layout metadata");
    let root_padding = root_layout
        .get("padding")
        .and_then(toml::Value::as_table)
        .expect("root padding metadata");
    assert_eq!(
        root_padding.get("left").and_then(toml::Value::as_integer),
        Some(20)
    );
    assert_eq!(
        root_padding.get("top").and_then(toml::Value::as_integer),
        Some(16)
    );

    let actions = surface
        .tree
        .nodes
        .values()
        .find(|node| node.node_path.0.starts_with("v2/actions["))
        .expect("actions retained node");
    assert_eq!(actions.constraints.width.stretch_mode, StretchMode::Stretch);
    assert_eq!(actions.constraints.height.preferred, 40.0);
    let mount_width = actions
        .template_metadata
        .as_ref()
        .and_then(|metadata| metadata.slot_attributes.get("layout"))
        .and_then(toml::Value::as_table)
        .and_then(|layout| layout.get("width"))
        .and_then(toml::Value::as_table)
        .and_then(|width| width.get("stretch"))
        .and_then(toml::Value::as_str);
    assert_eq!(mount_width, Some("Stretch"));
}
