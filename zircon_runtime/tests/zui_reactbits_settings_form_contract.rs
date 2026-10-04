use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiRichTextFormat, UiTextAlign},
};

const REACTBITS_SETTINGS_FORM_ZUI: &str =
    include_str!("fixtures/ui/reactbits_settings_form_components.zui");

#[test]
fn reactbits_settings_form_keeps_semantic_regions_and_content_contracts() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_SETTINGS_FORM_ZUI)
        .expect("ReactBits settings fixture should satisfy the .zui v2 source profile");

    assert_eq!(
        document.asset.id,
        "res://ui/tests/reactbits_settings_form_components.zui"
    );
    assert_eq!(document.root_node_id(), Some("root"));
    assert!(document.nodes.len() >= 40);

    let root = document.nodes.get("root").expect("settings root");
    assert_eq!(
        root.props
            .get("fixture_data_only")
            .and_then(toml::Value::as_bool),
        Some(true),
        "literal settings values must remain test-only fixture data"
    );
    assert_eq!(
        root.props
            .get("reference_primary")
            .and_then(toml::Value::as_str),
        Some("https://pro.reactbits.dev/docs/app-ui/settings-form/settings-form-1")
    );

    for (node_id, component) in [
        ("nav", "VerticalBox"),
        ("form", "ScrollableBox"),
        ("api_keys", "DataGrid"),
        ("save_bar", "HorizontalBox"),
        ("notifications", "NotificationCenter"),
        ("confirm_dialog", "ConfirmDialog"),
    ] {
        assert_eq!(
            document
                .nodes
                .get(node_id)
                .expect("settings semantic node")
                .component,
            component
        );
    }

    let form = document
        .nodes
        .get("form")
        .expect("settings form scroll owner");
    assert_eq!(
        form.props.get("scroll_owner").and_then(toml::Value::as_str),
        Some("settings-form")
    );
    let api_keys = document.nodes.get("api_keys").expect("API key collection");
    assert_eq!(
        api_keys
            .props
            .get("collection_virtualization")
            .and_then(toml::Value::as_str),
        Some("windowed")
    );
    assert_eq!(
        api_keys
            .props
            .get("visible_limit")
            .and_then(toml::Value::as_integer),
        Some(3)
    );
    assert_eq!(
        api_keys
            .props
            .get("collection_items")
            .and_then(toml::Value::as_array)
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        api_keys
            .props
            .get("overflow_fixture_label")
            .and_then(toml::Value::as_str),
        Some("A deliberately long localized integration label for overflow coverage")
    );
    assert_eq!(
        api_keys
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
    assert!(api_keys
        .props
        .get("long_locale_fixture_label")
        .and_then(toml::Value::as_str)
        .is_some_and(|value| value.len() > 40));
    assert_eq!(
        api_keys
            .props
            .get("column_alignments")
            .and_then(toml::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .collect::<Vec<_>>()
            }),
        Some(vec!["left", "right", "center"])
    );

    for node_id in [
        "header_title",
        "header_subtitle",
        "nav_profile",
        "display_name_label_text",
        "display_name_help",
        "display_name_field",
        "locale_value",
        "preferences_heading",
        "notification_label_text",
        "quiet_hours_value",
        "integrations_heading",
        "save_text",
        "save_reset",
        "save_button",
    ] {
        assert!(
            document
                .nodes
                .get(node_id)
                .and_then(|node| node.props.get("text_key"))
                .and_then(toml::Value::as_str)
                .is_some_and(|key| key.starts_with("fixture.settings.")),
            "visible settings content must carry an i18n key: {node_id}"
        );
    }

    let save_text = document.nodes.get("save_text").expect("unsaved copy");
    assert_eq!(
        save_text
            .props
            .get("rich_text_format")
            .and_then(toml::Value::as_str),
        Some("markdown_inline_v1")
    );
    let confirm = document
        .nodes
        .get("confirm_dialog")
        .expect("destructive confirmation");
    assert_eq!(
        confirm
            .props
            .get("requires_explicit_action")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        confirm
            .props
            .get("destructive")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
}

#[test]
fn reactbits_settings_form_builds_a_flow_surface_and_aligns_values_by_meaning() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_SETTINGS_FORM_ZUI)
        .expect("settings fixture should parse");
    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("settings fixture should compile into the retained UI arena");
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reactbits-settings-form-contract"),
        &document,
        &compiled,
    )
    .expect("settings fixture should build a retained UI surface");
    surface
        .compute_layout(UiSize::new(1120.0, 760.0))
        .expect("settings fixture should produce a render extract");

    assert_eq!(surface.tree.roots.len(), 1);
    assert_eq!(surface.tree.nodes.len(), document.nodes.len());

    let find_commands = |control_id: &str| {
        let node_id = surface
            .tree
            .nodes
            .values()
            .find(|node| node.node_path.0 == format!("v2/{control_id}"))
            .map(|node| node.node_id)
            .expect("retained settings node");
        surface
            .render_extract
            .list
            .commands
            .iter()
            .filter(|command| command.node_id == node_id)
            .collect::<Vec<_>>()
    };

    let save_commands = find_commands("ReactBitsSettingsSaveText");
    assert!(save_commands.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Changes are **not saved** until you apply them.")
            && command.style.rich_text_format == UiRichTextFormat::MarkdownInlineV1
    }));

    let status_commands = find_commands("ReactBitsSettingsStatus");
    assert!(status_commands
        .iter()
        .any(|command| command.style.text_align == UiTextAlign::Right));

    let quiet_hours_commands = find_commands("ReactBitsSettingsQuietHoursValue");
    assert!(quiet_hours_commands
        .iter()
        .any(|command| command.style.text_align == UiTextAlign::Right));

    for control_id in [
        "ReactBitsSettingsSaveText",
        "ReactBitsSettingsStatus",
        "ReactBitsSettingsQuietHoursValue",
    ] {
        assert!(
            find_commands(control_id)
                .iter()
                .all(|command| command.image.is_none()),
            "{control_id} should use semantic text/quads rather than image fallbacks"
        );
    }
}
