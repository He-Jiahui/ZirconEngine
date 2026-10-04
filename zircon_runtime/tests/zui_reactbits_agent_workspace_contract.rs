use zircon_runtime::ui::{
    surface::UiSurface,
    v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader},
};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::{StretchMode, UiContainerKind, UiLinearBoxConfig, UiSize},
    tree::UiVisibility,
    widget::UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE,
};

const REACTBITS_AGENT_WORKSPACE_ZUI: &str =
    include_str!("fixtures/ui/reactbits_agent_workspace.zui");

fn retained_node_id(
    surface: &UiSurface,
    control_id: &str,
) -> zircon_runtime_interface::ui::event_ui::UiNodeId {
    surface
        .tree
        .nodes
        .values()
        .find(|node| node.node_path.0 == format!("v2/{control_id}"))
        .map(|node| node.node_id)
        .expect("responsive shell control should be retained")
}

#[test]
fn reactbits_agent_workspace_builds_a_responsive_flow_surface() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_AGENT_WORKSPACE_ZUI)
        .expect("ReactBits reference slice should satisfy the .zui v2 source profile");

    assert_eq!(
        document.asset.id,
        "res://ui/tests/reactbits_agent_workspace.zui"
    );
    assert_eq!(document.root_node_id(), Some("root"));
    assert!(
        document.nodes.len() >= 90,
        "the reference slice should keep the full shell, message, state, and context topology"
    );
    assert!(document.nodes.contains_key("approval_card"));
    assert!(document.nodes.contains_key("call_failure"));
    assert!(document.nodes.contains_key("reasoning"));

    let sidebar = document
        .nodes
        .get("sidebar")
        .expect("responsive sidebar node");
    assert_eq!(
        sidebar
            .props
            .get("responsive_min_tier")
            .and_then(toml::Value::as_str),
        Some("regular")
    );
    let context = document
        .nodes
        .get("context")
        .expect("responsive context node");
    assert_eq!(
        context
            .props
            .get("responsive_min_tier")
            .and_then(toml::Value::as_str),
        Some("wide")
    );
    let context_summary = document
        .nodes
        .get("context_summary")
        .expect("compact context summary node");
    assert_eq!(
        context_summary
            .props
            .get("responsive_min_tier")
            .and_then(toml::Value::as_str),
        Some("regular")
    );
    assert_eq!(
        document
            .nodes
            .get("root")
            .expect("responsive shell root")
            .component,
        "Stack"
    );
    assert_eq!(
        document
            .nodes
            .get("root")
            .expect("responsive shell root")
            .props
            .get("direction")
            .and_then(toml::Value::as_table)
            .and_then(|directions| directions.get("xs"))
            .and_then(toml::Value::as_str),
        Some("column")
    );
    assert_eq!(
        document
            .nodes
            .get("root")
            .expect("responsive shell root")
            .props
            .get("direction")
            .and_then(toml::Value::as_table)
            .and_then(|directions| directions.get("md"))
            .and_then(toml::Value::as_str),
        Some("row")
    );
    for (node_id, display) in [
        ("sidebar", &["none", "flex"][..]),
        ("context", &["none", "none", "flex"][..]),
        ("context_summary", &["none", "flex", "none"][..]),
    ] {
        let values = document
            .nodes
            .get(node_id)
            .expect("responsive shell child")
            .props
            .get("display")
            .and_then(toml::Value::as_table)
            .expect("responsive display table");
        let keys = ["xs", "md", "lg"];
        for (key, expected) in keys.into_iter().zip(display.iter().copied()) {
            assert_eq!(
                values.get(key).and_then(toml::Value::as_str),
                Some(expected),
                "{node_id} display at {key}"
            );
        }
    }
    for (row_id, time_id) in [
        ("activity_prepare", "activity_prepare_time"),
        ("activity_nested", "activity_nested_time"),
        ("activity_nested_child", "activity_nested_child_time"),
        ("activity_verify", "activity_verify_time"),
    ] {
        let row = document.nodes.get(row_id).expect("activity row");
        assert_eq!(row.component, "HorizontalBox");
        assert_eq!(row.children.len(), 2, "activity row keeps label/value flow");
        let time = document.nodes.get(time_id).expect("activity value slot");
        assert_eq!(
            time.props.get("text_align").and_then(toml::Value::as_str),
            Some("right"),
            "activity duration/status values use an end-aligned slot"
        );
    }
    for line in REACTBITS_AGENT_WORKSPACE_ZUI.lines() {
        if !line.contains("props =") || !line.contains("text = \"") {
            continue;
        }
        let Some(text_start) = line.find("text = \"") else {
            continue;
        };
        let value = &line[text_start + "text = \"".len()..];
        let value = value.split('"').next().unwrap_or_default();
        assert!(
            !value.contains("   "),
            "activity and comparable values must not use manual spaces for alignment: {value:?}"
        );
    }
    assert_eq!(
        context_summary
            .props
            .get("responsive_max_tier")
            .and_then(toml::Value::as_str),
        Some("regular")
    );
    assert!(
        document
            .nodes
            .get("context_summary_open")
            .expect("compact context open control")
            .events
            .iter()
            .any(|event| event.route.as_deref() == Some("agent.context.open")),
        "compact context control must expose the context-open route"
    );

    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("ReactBits reference slice should compile into the retained UI arena");
    assert_eq!(compiled.asset_id, document.asset.id);
    assert_eq!(compiled.arena.node_count(), document.nodes.len());

    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reactbits-agent-workspace-contract"),
        &document,
        &compiled,
    )
    .expect("ReactBits reference slice should build a retained UI surface");

    assert_eq!(surface.tree.roots.len(), 1);
    assert_eq!(surface.tree.nodes.len(), document.nodes.len());
    assert_eq!(surface.tree.layout_slots().len(), document.nodes.len() - 1);

    let root = surface
        .tree
        .node(surface.tree.roots[0])
        .expect("retained ReactBits root");
    assert_eq!(root.constraints.width.preferred, 1280.0);
    assert_eq!(root.constraints.height.preferred, 800.0);
    assert_eq!(root.constraints.width.stretch_mode, StretchMode::Stretch);
    assert_eq!(root.constraints.height.stretch_mode, StretchMode::Stretch);
    let UiContainerKind::VerticalBox(root_box) = root.container else {
        panic!("ReactBits shell must start as a vertical responsive flow container");
    };
    // MUI Stack spacing is expressed in theme units.  The native adapter
    // resolves the authored 1.5 unit value with the default 8 px spacing
    // unit, so the retained base layout exposes the reference's 12 px gap.
    assert_eq!(root_box.gap, 12.0);

    let root_id = retained_node_id(&surface, "ReactBitsAgentWorkspaceRoot");
    let sidebar_id = retained_node_id(&surface, "ReactBitsAgentSidebar");
    let main_id = retained_node_id(&surface, "ReactBitsAgentMain");
    let context_id = retained_node_id(&surface, "ReactBitsAgentContextPanel");
    let summary_id = retained_node_id(&surface, "ReactBitsAgentContextSummary");

    surface
        .compute_layout(UiSize::new(640.0, 520.0))
        .expect("narrow responsive workspace should lay out");
    assert_eq!(
        surface.tree.node(root_id).unwrap().container,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 12.0 })
    );
    assert_eq!(
        surface.tree.node(sidebar_id).unwrap().visibility,
        UiVisibility::Collapsed
    );
    assert_eq!(
        surface.tree.node(context_id).unwrap().visibility,
        UiVisibility::Collapsed
    );
    assert_eq!(
        surface.tree.node(summary_id).unwrap().visibility,
        UiVisibility::Collapsed
    );
    assert_eq!(
        surface.tree.node(main_id).unwrap().visibility,
        UiVisibility::Visible
    );

    surface
        .compute_layout(UiSize::new(900.0, 620.0))
        .expect("regular responsive workspace should lay out");
    assert_eq!(
        surface.tree.node(root_id).unwrap().container,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 12.0 })
    );
    assert_eq!(
        surface.tree.node(sidebar_id).unwrap().visibility,
        UiVisibility::Visible
    );
    assert_eq!(
        surface.tree.node(context_id).unwrap().visibility,
        UiVisibility::Collapsed
    );
    assert_eq!(
        surface.tree.node(summary_id).unwrap().visibility,
        UiVisibility::Visible
    );

    surface
        .compute_layout(UiSize::new(1280.0, 800.0))
        .expect("wide responsive workspace should lay out");
    assert_eq!(
        surface.tree.node(root_id).unwrap().container,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 12.0 })
    );
    assert_eq!(
        surface.tree.node(context_id).unwrap().visibility,
        UiVisibility::Visible
    );
    assert_eq!(
        surface.tree.node(summary_id).unwrap().visibility,
        UiVisibility::Collapsed
    );

    for (node_id, control_id) in [
        ("sidebar", "ReactBitsAgentSidebar"),
        ("main", "ReactBitsAgentMain"),
        ("context", "ReactBitsAgentContextPanel"),
    ] {
        assert!(
            surface
                .tree
                .nodes
                .values()
                .any(|retained| retained.node_path.0 == format!("v2/{control_id}")),
            "retained shell is missing {node_id}"
        );
    }

    let agent_chat = surface
        .tree
        .nodes
        .values()
        .find(|node| node.node_path.0 == "v2/ReactBitsAgentAssistantMessage")
        .expect("assistant message retained node");
    let composer = surface
        .tree
        .nodes
        .values()
        .find(|node| node.node_path.0 == "v2/ReactBitsAgentComposerInput")
        .expect("chat composer retained node");
    assert_eq!(
        agent_chat
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.attributes.get("background_color"))
            .and_then(toml::Value::as_str),
        Some("#171b24")
    );
    assert_eq!(
        composer
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.attributes.get(UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE))
            .and_then(toml::Value::as_str),
        Some("mui-x-chat-composer")
    );
    assert_eq!(
        composer
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.attributes.get("streaming"))
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let approval = surface
        .tree
        .nodes
        .values()
        .find(|node| node.node_path.0 == "v2/ReactBitsAgentApprovalCard")
        .expect("approval card retained node");
    assert_eq!(
        approval
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.attributes.get("approval_state"))
            .and_then(toml::Value::as_str),
        Some("pending")
    );
    assert_eq!(
        approval
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.attributes.get("destructive"))
            .and_then(toml::Value::as_bool),
        Some(true)
    );
}
