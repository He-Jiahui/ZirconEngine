use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::UiSize,
    surface::{UiRenderCommandKind, UiRichTextFormat, UiTextAlign},
};

const REACTBITS_KANBAN_SCHEDULING_ZUI: &str =
    include_str!("fixtures/ui/reactbits_kanban_scheduling_components.zui");

#[test]
fn reactbits_kanban_scheduling_keeps_reference_and_content_contracts() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_KANBAN_SCHEDULING_ZUI)
        .expect("ReactBits kanban/scheduling fixture should satisfy the .zui v2 source profile");

    assert_eq!(
        document.asset.id,
        "res://ui/tests/reactbits_kanban_scheduling_components.zui"
    );
    assert_eq!(document.root_node_id(), Some("root"));
    assert!(document.nodes.len() >= 90);

    let root = document.nodes.get("root").expect("kanban/scheduling root");
    assert_eq!(
        root.props
            .get("fixture_data_only")
            .and_then(toml::Value::as_bool),
        Some(true),
        "board and agenda samples must remain test-only fixture data"
    );
    assert_eq!(
        root.props
            .get("reference_primary")
            .and_then(toml::Value::as_str),
        Some("https://pro.reactbits.dev/docs/app-ui/kanban/kanban-5")
    );

    for (node_id, component) in [
        ("body", "ScrollableBox"),
        ("panels", "FlowBox"),
        ("columns", "FlowBox"),
        ("column_todo", "VerticalBox"),
        ("calendar", "FlowBox"),
        ("agenda", "ScrollableBox"),
        ("booking_summary", "Paper"),
    ] {
        assert_eq!(
            document
                .nodes
                .get(node_id)
                .expect("kanban/scheduling semantic node")
                .component,
            component
        );
    }

    for (node_id, owner) in [
        ("body", "kanban-scheduling-body"),
        ("agenda", "deployment-agenda"),
    ] {
        assert_eq!(
            document
                .nodes
                .get(node_id)
                .expect("scroll owner")
                .props
                .get("scroll_owner")
                .and_then(toml::Value::as_str),
            Some(owner)
        );
    }

    for node_id in [
        "column_todo",
        "column_building",
        "column_review",
        "calendar",
        "agenda",
    ] {
        let node = document.nodes.get(node_id).expect("windowed collection");
        assert_eq!(
            node.props
                .get("collection_virtualization")
                .and_then(toml::Value::as_str),
            Some("windowed"),
            "large board/calendar collections must opt into retained windowing: {node_id}"
        );
        assert!(node
            .props
            .get("visible_limit")
            .and_then(toml::Value::as_integer)
            .is_some_and(|limit| limit > 0));
        assert_eq!(
            node.props
                .get("collection_test_cases")
                .and_then(toml::Value::as_array)
                .map(|values| values.len()),
            Some(6),
            "collection matrix should cover empty/one/normal/overflow/narrow/long-locale: {node_id}"
        );
        assert!(node
            .props
            .get("long_locale_fixture_label")
            .and_then(toml::Value::as_str)
            .is_some_and(|value| value.len() > 40));
    }

    let mut keyed_text_count = 0;
    for (node_id, node) in &document.nodes {
        if node.props.get("text").is_some() {
            keyed_text_count += 1;
            assert!(
                node.props
                    .get("text_key")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|key| key.starts_with("fixture.")),
                "visible board/schedule text must carry an i18n key: {node_id}"
            );
        }
        if node.props.get("value_text").is_some() {
            assert!(
                node.props
                    .get("value_key")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|key| key.starts_with("fixture.")),
                "agenda values must carry a binding key: {node_id}"
            );
        }
    }
    assert!(keyed_text_count >= 70);

    for node_id in ["review_card_1_title", "booking_detail"] {
        let node = document.nodes.get(node_id).expect("rich-text node");
        assert!(
            node.props
                .get("text")
                .and_then(toml::Value::as_str)
                .is_some_and(|text| text.contains("**")),
            "inline emphasis must stay in one localized text source: {node_id}"
        );
        assert_eq!(
            node.props
                .get("rich_text_format")
                .and_then(toml::Value::as_str),
            Some("markdown_inline_v1")
        );
        assert!(node.props.get("text_key").is_some());
    }

    for node_id in [
        "points_value",
        "capacity_value",
        "burn_down_value",
        "schedule_timezone",
        "booking_detail",
    ] {
        assert_eq!(
            document
                .nodes
                .get(node_id)
                .expect("end-aligned metric/value")
                .props
                .get("text_align")
                .and_then(toml::Value::as_str),
            Some("right"),
            "numeric/status values should align to the end: {node_id}"
        );
    }
    for node_id in ["day_mon", "day_wed", "date_18"] {
        assert_eq!(
            document
                .nodes
                .get(node_id)
                .expect("centered date control")
                .props
                .get("text_align")
                .and_then(toml::Value::as_str),
            Some("center")
        );
    }
}

#[test]
fn reactbits_kanban_scheduling_compiles_at_wide_and_narrow_flow_sizes() {
    let document = UiZuiAssetLoader::load_zui_str(REACTBITS_KANBAN_SCHEDULING_ZUI)
        .expect("kanban/scheduling fixture should parse");
    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("kanban/scheduling fixture should compile into the retained UI arena");
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("reactbits-kanban-scheduling-contract"),
        &document,
        &compiled,
    )
    .expect("kanban/scheduling fixture should build a retained UI surface");

    for size in [UiSize::new(1160.0, 820.0), UiSize::new(640.0, 520.0)] {
        surface
            .compute_layout(size)
            .expect("kanban/scheduling fixture should produce a render extract");
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
            .expect("retained kanban/scheduling node");
        surface
            .render_extract
            .list
            .commands
            .iter()
            .filter(|command| command.node_id == node_id)
            .collect::<Vec<_>>()
    };

    let rich = find_commands("ReactBitsKanbanReviewCardOneTitle");
    assert!(rich.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.style.rich_text_format == UiRichTextFormat::MarkdownInlineV1
    }));
    let booking = find_commands("ReactBitsSchedulingBookingDetail");
    assert!(booking.iter().any(|command| {
        command.kind == UiRenderCommandKind::Text
            && command.style.rich_text_format == UiRichTextFormat::MarkdownInlineV1
            && command.style.text_align == UiTextAlign::Right
    }));
    let date = find_commands("ReactBitsSchedulingDate18");
    assert!(date
        .iter()
        .any(|command| command.style.text_align == UiTextAlign::Center));

    for control_id in [
        "ReactBitsKanbanReviewCardOneTitle",
        "ReactBitsSchedulingBookingDetail",
        "ReactBitsSchedulingDate18",
    ] {
        assert!(
            find_commands(control_id)
                .iter()
                .all(|command| command.image.is_none()),
            "{control_id} should use semantic text rather than image fallbacks"
        );
    }
}
