use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiRichTextFormat},
};

const REACTBITS_AGENT_WORKFLOW_ZUI: &str =
    include_str!("fixtures/ui/reactbits_agent_workflow_components.zui");

#[test]
fn reactbits_agent_workflow_keeps_native_component_roles_and_semantic_fields() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_AGENT_WORKFLOW_ZUI)
        .expect("ReactBits workflow fixture should satisfy the .zui v2 source profile");

    assert_eq!(
        document.asset.id,
        "res://ui/tests/reactbits_agent_workflow_components.zui"
    );
    assert_eq!(document.root_node_id(), Some("root"));
    assert_eq!(document.nodes.len(), 9);

    let root = document.nodes.get("root").expect("workflow root");
    assert_eq!(
        root.props
            .get("fixture_data_only")
            .and_then(toml::Value::as_bool),
        Some(true),
        "literal workflow examples must remain test-only fixture data"
    );
    assert_eq!(
        root.props
            .get("reference_primary")
            .and_then(toml::Value::as_str),
        Some("https://pro.reactbits.dev/docs/app-ui/agent-plan")
    );

    for (node_id, component, role) in [
        ("plan", "AgentPlan", "mui-x-agent-plan"),
        ("tool_calls", "ToolCalls", "mui-x-tool-calls"),
        ("approval", "AgentApproval", "mui-x-agent-approval"),
        ("usage", "AIUsage", "mui-x-ai-usage"),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .expect("workflow component node");
        assert_eq!(node.component, component);
        assert_eq!(
            node.props
                .get("component_role")
                .and_then(toml::Value::as_str),
            Some(role)
        );
    }

    let plan = document.nodes.get("plan").expect("agent plan");
    assert_eq!(
        plan.props
            .get("collection_virtualization")
            .and_then(toml::Value::as_str),
        Some("windowed")
    );
    assert_eq!(
        plan.props
            .get("collection_items")
            .and_then(toml::Value::as_array)
            .map(Vec::len),
        Some(3)
    );
    let tool_calls = document.nodes.get("tool_calls").expect("tool calls");
    assert_eq!(
        tool_calls
            .props
            .get("collection_virtualization")
            .and_then(toml::Value::as_str),
        Some("windowed")
    );
    let approval = document.nodes.get("approval").expect("agent approval");
    assert_eq!(
        approval
            .props
            .get("approval_state")
            .and_then(toml::Value::as_str),
        Some("pending")
    );
    assert_eq!(
        approval
            .props
            .get("destructive")
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("workflow fixture should compile with unknown source-owned painter components");
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reactbits-agent-workflow-contract"),
        &document,
        &compiled,
    )
    .expect("workflow fixture should build a retained UI surface");
    surface
        .compute_layout(UiSize::new(1080.0, 720.0))
        .expect("workflow fixture should produce a render extract");

    assert_eq!(surface.tree.roots.len(), 1);
    assert_eq!(surface.tree.nodes.len(), document.nodes.len());
    for (_node_id, control_id, role) in [
        ("plan", "ReactBitsAgentPlan", "mui-x-agent-plan"),
        ("tool_calls", "ReactBitsToolCalls", "mui-x-tool-calls"),
        ("approval", "ReactBitsAgentApproval", "mui-x-agent-approval"),
        ("usage", "ReactBitsAIUsage", "mui-x-ai-usage"),
    ] {
        let retained = surface
            .tree
            .nodes
            .values()
            .find(|node| node.node_path.0 == format!("v2/{control_id}"))
            .expect("retained workflow node");
        assert_eq!(
            retained
                .template_metadata
                .as_ref()
                .and_then(|metadata| metadata.attributes.get("component_role"))
                .and_then(toml::Value::as_str),
            Some(role)
        );
    }

    for (control_id, expected_text) in [
        ("ReactBitsAgentPlan", "Build retained ZUI surface"),
        ("ReactBitsToolCalls", "**search_web**  |  ReactBits"),
        ("ReactBitsAgentApproval", "Allow write"),
        ("ReactBitsAIUsage", "12.4k used · 19.6k remaining · $0.08"),
    ] {
        let node_id = surface
            .tree
            .nodes
            .values()
            .find(|node| node.node_path.0 == format!("v2/{control_id}"))
            .map(|node| node.node_id)
            .expect("workflow node id");
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
        if control_id == "ReactBitsToolCalls" {
            let matching = commands
                .iter()
                .filter(|command| command.text.as_deref() == Some(expected_text))
                .collect::<Vec<_>>();
            assert_eq!(
                matching.len(),
                1,
                "tool name and detail must be one continuous rich-text flow"
            );
            assert_eq!(
                matching[0].style.rich_text_format,
                UiRichTextFormat::MarkdownInlineV1,
                "inline emphasis must use the retained rich-text pipeline"
            );
        }
        assert!(
            commands.iter().all(|command| command.image.is_none()),
            "{control_id} should use text/quads rather than image fallback commands"
        );
    }
}
