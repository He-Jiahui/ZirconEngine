use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiRichTextFormat, UiTextAlign},
};

const REACTBITS_AUTH_ONBOARDING_ZUI: &str =
    include_str!("fixtures/ui/reactbits_auth_onboarding_components.zui");

#[test]
fn reactbits_auth_onboarding_keeps_reference_and_content_contracts() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_AUTH_ONBOARDING_ZUI)
        .expect("ReactBits auth/onboarding fixture should satisfy the .zui v2 source profile");

    assert_eq!(
        document.asset.id,
        "res://ui/tests/reactbits_auth_onboarding_components.zui"
    );
    assert_eq!(document.root_node_id(), Some("root"));
    assert!(document.nodes.len() >= 70);

    let root = document.nodes.get("root").expect("auth/onboarding root");
    assert_eq!(
        root.props
            .get("fixture_data_only")
            .and_then(toml::Value::as_bool),
        Some(true),
        "literal auth and onboarding values must remain test-only fixture data"
    );
    assert_eq!(
        root.props
            .get("reference_primary")
            .and_then(toml::Value::as_str),
        Some("https://pro.reactbits.dev/docs/app-ui/authentication/authentication-1")
    );

    for (node_id, component) in [
        ("body", "ScrollableBox"),
        ("main", "Stack"),
        ("auth_card", "VerticalBox"),
        ("provider_row", "HorizontalBox"),
        ("verification", "VerticalBox"),
        ("stepper", "HorizontalBox"),
        ("workspace_picker", "ScrollableBox"),
    ] {
        assert_eq!(
            document
                .nodes
                .get(node_id)
                .expect("auth/onboarding semantic node")
                .component,
            component
        );
    }

    let body = document
        .nodes
        .get("body")
        .expect("single body scroll owner");
    assert_eq!(
        body.props.get("scroll_owner").and_then(toml::Value::as_str),
        Some("auth-onboarding-body")
    );
    let workspace_picker = document
        .nodes
        .get("workspace_picker")
        .expect("workspace list");
    assert_eq!(
        workspace_picker
            .props
            .get("scroll_owner")
            .and_then(toml::Value::as_str),
        Some("workspace-picker")
    );
    assert_eq!(
        workspace_picker
            .props
            .get("collection_virtualization")
            .and_then(toml::Value::as_str),
        Some("windowed")
    );
    assert_eq!(
        workspace_picker
            .props
            .get("visible_limit")
            .and_then(toml::Value::as_integer),
        Some(3)
    );
    assert_eq!(
        workspace_picker
            .props
            .get("collection_items")
            .and_then(toml::Value::as_array)
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        workspace_picker
            .props
            .get("collection_test_cases")
            .and_then(toml::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .collect::<Vec<_>>()
            }),
        Some(vec![
            "empty",
            "one",
            "normal",
            "overflow",
            "narrow",
            "long-locale",
        ])
    );
    assert!(workspace_picker
        .props
        .get("long_locale_fixture_label")
        .and_then(toml::Value::as_str)
        .is_some_and(|value| value.len() > 40));

    let mut keyed_text_count = 0;
    for (node_id, node) in &document.nodes {
        if node.props.get("text").is_some() {
            keyed_text_count += 1;
            assert!(
                node.props
                    .get("text_key")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|key| key.starts_with("fixture.")),
                "visible auth/onboarding text must carry an i18n key: {node_id}"
            );
        }
        if node.props.get("value_text").is_some() {
            assert!(
                node.props
                    .get("value_key")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|key| key.starts_with("fixture.")),
                "editable auth/onboarding values must carry a binding key: {node_id}"
            );
        }
        if node.props.get("placeholder").is_some() {
            assert!(
                node.props
                    .get("placeholder_key")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|key| key.starts_with("fixture.")),
                "editable auth/onboarding placeholders must carry an i18n key: {node_id}"
            );
        }
    }
    assert!(keyed_text_count >= 30);

    for node_id in ["terms", "verification_note"] {
        let node = document.nodes.get(node_id).expect("rich-text node");
        assert_eq!(
            node.props
                .get("rich_text_format")
                .and_then(toml::Value::as_str),
            Some("markdown_inline_v1")
        );
        assert!(node.props.get("text_key").is_some());
    }

    for node_id in ["locale", "onboarding_progress", "slug_value", "forgot"] {
        let node = document.nodes.get(node_id).expect("end-aligned value");
        assert_eq!(
            node.props.get("text_align").and_then(toml::Value::as_str),
            Some("right"),
            "meaningful status/value content should align to the end: {node_id}"
        );
    }
    for node_id in ["auth_title", "auth_subtitle", "auth_mark"] {
        let node = document.nodes.get(node_id).expect("centered auth content");
        assert_eq!(
            node.props.get("text_align").and_then(toml::Value::as_str),
            Some("center")
        );
    }
}

#[test]
fn reactbits_auth_onboarding_compiles_at_wide_and_narrow_flow_sizes() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_AUTH_ONBOARDING_ZUI)
        .expect("auth/onboarding fixture should parse");
    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("auth/onboarding fixture should compile into the retained UI arena");
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reactbits-auth-onboarding-contract"),
        &document,
        &compiled,
    )
    .expect("auth/onboarding fixture should build a retained UI surface");

    for size in [UiSize::new(1120.0, 820.0), UiSize::new(640.0, 520.0)] {
        surface
            .compute_layout(size)
            .expect("auth/onboarding fixture should produce a render extract");
        assert_eq!(surface.tree.roots.len(), 1);
        assert_eq!(surface.tree.nodes.len(), document.nodes.len());
    }

    let find_commands = |control_id: &str| {
        let node_id = surface
            .tree
            .nodes
            .values()
            .find(|node| node.node_path.0 == format!("v2/{control_id}"))
            .map(|node| node.node_id)
            .expect("retained auth/onboarding node");
        surface
            .render_extract
            .list
            .commands
            .iter()
            .filter(|command| command.node_id == node_id)
            .collect::<Vec<_>>()
    };

    let terms_commands = find_commands("ReactBitsAuthTerms");
    assert!(terms_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.style.rich_text_format == UiRichTextFormat::MarkdownInlineV1
    }));

    let verification_commands = find_commands("ReactBitsAuthVerificationNote");
    assert!(verification_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.style.rich_text_format == UiRichTextFormat::MarkdownInlineV1
    }));

    let centered = find_commands("ReactBitsAuthTitle");
    assert!(centered
        .iter()
        .any(|command| command.style.text_align == UiTextAlign::Center));
    let end_aligned = find_commands("ReactBitsOnboardingSlugValue");
    assert!(end_aligned
        .iter()
        .any(|command| command.style.text_align == UiTextAlign::Right));

    for control_id in [
        "ReactBitsAuthTerms",
        "ReactBitsAuthVerificationNote",
        "ReactBitsAuthTitle",
        "ReactBitsOnboardingSlugValue",
    ] {
        assert!(
            find_commands(control_id)
                .iter()
                .all(|command| command.image.is_none()),
            "{control_id} should use semantic text rather than image fallbacks"
        );
    }
}
