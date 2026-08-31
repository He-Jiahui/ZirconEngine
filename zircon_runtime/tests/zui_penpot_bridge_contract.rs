use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiZuiAssetLoader};

const PENPOT_ROUNDTRIP_ZUI: &str = include_str!("fixtures/ui/penpot_roundtrip.zui");

#[test]
fn penpot_roundtrip_output_loads_and_compiles_as_a_zui_view() {
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
}
