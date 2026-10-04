use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiTextAlign},
};

const REACTBITS_DATA_SURFACE_ZUI: &str =
    include_str!("fixtures/ui/reactbits_data_surface_components.zui");

#[test]
fn reactbits_data_surface_keeps_filter_tree_table_and_empty_state_semantics() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_DATA_SURFACE_ZUI)
        .expect("ReactBits data surface fixture should satisfy the .zui v2 source profile");

    assert_eq!(
        document.asset.id,
        "res://ui/tests/reactbits_data_surface_components.zui"
    );
    assert_eq!(document.root_node_id(), Some("root"));
    assert_eq!(document.nodes.len(), 22);

    let root = document.nodes.get("root").expect("data surface root");
    assert_eq!(
        root.props
            .get("fixture_data_only")
            .and_then(toml::Value::as_bool),
        Some(true),
        "literal collection examples belong only to this test fixture"
    );
    assert_eq!(
        root.props
            .get("reference_primary")
            .and_then(toml::Value::as_str),
        Some("https://pro.reactbits.dev/docs/app-ui/data-table")
    );

    for (node_id, component, role) in [
        ("tree", "TreeView", "mui-x-tree-view"),
        ("grid", "DataGrid", "mui-x-data-grid"),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .expect("data surface native node");
        assert_eq!(node.component, component);
        assert_eq!(
            node.props
                .get("component_role")
                .and_then(toml::Value::as_str),
            Some(role)
        );
        assert!(node.props.contains_key("collection_items"));
        assert!(node.props.contains_key("empty_text"));
        assert_eq!(
            node.props
                .get("collection_virtualization")
                .and_then(toml::Value::as_str),
            Some("windowed")
        );
        assert_eq!(
            node.props
                .get("visible_limit")
                .and_then(toml::Value::as_integer),
            Some(3)
        );
    }
    assert_eq!(
        document
            .nodes
            .get("grid")
            .and_then(|node| node.props.get("column_alignments"))
            .and_then(toml::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .collect::<Vec<_>>()
            }),
        Some(vec!["left", "left", "right", "center"])
    );

    let query = document
        .nodes
        .get("filter_query")
        .expect("filter query node");
    assert_eq!(
        query
            .props
            .get("component_role")
            .and_then(toml::Value::as_str),
        Some("search-field")
    );
    assert_eq!(
        query.props.get("value_text").and_then(toml::Value::as_str),
        Some("mesh")
    );

    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("data surface fixture should compile into the retained UI arena");
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reactbits-data-surface-contract"),
        &document,
        &compiled,
    )
    .expect("data surface fixture should build a retained UI surface");
    surface
        .compute_layout(UiSize::new(1160.0, 760.0))
        .expect("data surface fixture should produce a render extract");

    assert_eq!(surface.tree.roots.len(), 1);
    assert_eq!(surface.tree.nodes.len(), document.nodes.len());
    for (node_id, control_id, retained_role) in [
        ("tree", "ReactBitsDataSurfaceTree", "tree-view"),
        ("grid", "ReactBitsDataSurfaceGrid", "mui-x-data-grid"),
    ] {
        let retained = surface
            .tree
            .nodes
            .values()
            .find(|node| node.node_path.0 == format!("v2/{control_id}"))
            .expect("retained data surface native node");
        assert_eq!(
            retained
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.attributes.get("component_role"))
                .and_then(toml::Value::as_str),
            Some(retained_role)
        );
    }

    for (control_id, expected_text) in [
        ("ReactBitsDataSurfaceTree", "Meshes"),
        ("ReactBitsDataSurfaceGrid", "SM_Tree_Oak_01"),
    ] {
        let node_id = surface
            .tree
            .nodes
            .values()
            .find(|node| node.node_path.0 == format!("v2/{control_id}"))
            .map(|node| node.node_id)
            .expect("data surface node id");
        let commands = surface
            .render_extract
            .list
            .commands
            .iter()
            .filter(|command| command.node_id == node_id)
            .collect::<Vec<_>>();
        assert!(
            !commands.is_empty(),
            "{control_id} should emit native commands"
        );
        assert!(
            commands.iter().any(|command| {
                command.kind == UiRenderCommandKind::Text
                    && command.text.as_deref() == Some(expected_text)
            }),
            "{control_id} should preserve semantic text {expected_text:?}"
        );
        if control_id == "ReactBitsDataSurfaceGrid" {
            let size = commands
                .iter()
                .find(|command| command.text.as_deref() == Some("Size"))
                .expect("numeric Size column header");
            let status = commands
                .iter()
                .find(|command| command.text.as_deref() == Some("Status"))
                .expect("status column header");
            assert_eq!(size.style.text_align, UiTextAlign::Right);
            assert_eq!(status.style.text_align, UiTextAlign::Center);
        }
        assert!(
            commands.iter().all(|command| command.image.is_none()),
            "{control_id} should use text/quads rather than image fallback commands"
        );
    }
}
