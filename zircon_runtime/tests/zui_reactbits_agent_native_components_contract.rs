use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

const REACTBITS_AGENT_NATIVE_ZUI: &str =
    include_str!("fixtures/ui/reactbits_agent_native_components.zui");

#[test]
fn reactbits_agent_native_components_retains_full_thread_semantics() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_AGENT_NATIVE_ZUI)
        .expect("ReactBits native component fixture should parse");

    let chat = document
        .nodes
        .get("agent_chat")
        .expect("native AgentChat node");
    assert_eq!(
        chat.props
            .get("messages")
            .and_then(toml::Value::as_array)
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        chat.props
            .get("messages")
            .and_then(toml::Value::as_array)
            .and_then(|messages| messages.get(2))
            .and_then(toml::Value::as_str),
        Some("user|Can the tool result and cited file stay in the same thread?")
    );

    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("native AgentChat fixture should compile into retained UI");
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reactbits-agent-native-components-contract"),
        &document,
        &compiled,
    )
    .expect("native AgentChat fixture should build a retained surface");

    assert_eq!(surface.tree.roots.len(), 1);
    assert_eq!(surface.tree.nodes.len(), document.nodes.len());
    let root = surface
        .tree
        .node(surface.tree.roots[0])
        .expect("native root");
    assert_eq!(
        root.layout_padding,
        zircon_runtime_interface::ui::layout::UiMargin::new(18.0, 18.0, 18.0, 18.0)
    );
    let retained_chat = surface
        .tree
        .nodes
        .values()
        .find(|node| node.node_path.0 == "v2/ReactBitsNativeAgentChat")
        .expect("full-thread AgentChat should be retained");
    assert_eq!(
        retained_chat
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.attributes.get("streaming"))
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let chat_commands = surface
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| command.node_id == retained_chat.node_id)
        .collect::<Vec<_>>();
    assert!(
        chat_commands
            .iter()
            .filter_map(|command| command.text.as_deref())
            .all(|text| !text.contains("Native AgentChat primitives")),
        "AgentChat semantic text must not duplicate a generic owner label"
    );
}
