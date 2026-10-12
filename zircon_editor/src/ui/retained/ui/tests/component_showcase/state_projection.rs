use super::*;

#[test]
fn component_showcase_template_nodes_preserve_scroll_clip_frames() {
    let nodes = default_component_showcase_nodes_for_size(1080.0, 360.0);
    let scroll = template_node(&nodes, "ComponentShowcaseScroll");
    let collections = template_node(&nodes, "ContextActionMenuDemo");

    assert!(
        collections.has_clip_frame,
        "scrolled child should expose a retained clip frame for painter clipping"
    );
    assert!(
        collections.clip_frame.width > 0.0 && collections.clip_frame.height > 0.0,
        "scrolled child should carry a retained clip frame for painter clipping"
    );
    assert!(
        collections.clip_frame.y + collections.clip_frame.height
            <= scroll.frame.y + scroll.frame.height + 0.5,
        "scrolled child clip should be bounded by the ScrollableBox viewport: child clip bottom={} scroll bottom={}",
        collections.clip_frame.y + collections.clip_frame.height,
        scroll.frame.y + scroll.frame.height
    );
    assert!(
        collections.clip_frame.y >= scroll.frame.y - 0.5,
        "scrolled child clip should start inside the ScrollableBox viewport: child clip top={} scroll top={}",
        collections.clip_frame.y,
        scroll.frame.y
    );
}

#[test]
fn component_showcase_pane_uses_supplied_runtime_demo_state() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let (_fixture, chrome, _model, _ui_asset_panes, _animation_panes) = root_shell_fixture();
    let body_spec = PaneBodySpec::new(
        "res://ui/editor/component_showcase.zui",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    );
    let body = host_window::build_pane_body_presentation(
        &body_spec,
        &host_window::PanePayloadBuildContext::new(&chrome),
    );
    let mut pane = host_pane("component-showcase", "UI Component Showcase");
    pane.kind = "UiComponentShowcase".into();
    pane.pane_presentation = Some(host_window::PanePresentation::new(
        host_window::PaneShellPresentation::new(
            "UI Component Showcase",
            "ui-components",
            "Runtime components",
            "",
            None,
            false,
            blank_viewport_chrome(),
        ),
        body,
    ));

    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .load_builtin_host_templates()
        .expect("built-in host templates should load");
    let binding = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .expect("showcase projection should compile")
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/NumberFieldDragUpdate")
        .expect("showcase should expose NumberField drag binding")
        .binding;
    runtime
        .apply_showcase_demo_binding(&binding, UiComponentShowcaseDemoEventInput::DragDelta(5.0))
        .expect("showcase runtime should accept NumberField drag input");
    let combo_open_binding = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .expect("showcase projection should compile")
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/ComboBoxOpenPopup")
        .expect("showcase should expose ComboBox open binding")
        .binding;
    runtime
        .apply_showcase_demo_binding(&combo_open_binding, UiComponentShowcaseDemoEventInput::None)
        .expect("showcase runtime should accept ComboBox popup input");
    let asset_drop_binding = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .expect("showcase projection should compile")
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == "UiComponentShowcase/AssetFieldDropped")
        .expect("showcase should expose AssetField drop binding")
        .binding;
    runtime
        .apply_showcase_demo_binding(
            &asset_drop_binding,
            UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.albedo.png",
                )
                .with_source(UiDragSourceMetadata::asset(
                    "browser",
                    "AssetBrowserContentPanel",
                    "asset-uuid-1",
                    "res://textures/grid.albedo.png",
                    "Grid Albedo",
                    "Texture",
                    "png",
                )),
            },
        )
        .expect("showcase runtime should accept AssetField drop input");

    let host_contract_pane =
        super::pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
            &pane,
            host_window::PaneContentSize::new(1080.0, 720.0),
            &runtime,
        );

    let nodes = (0..host_contract_pane.nodes.row_count())
        .filter_map(|row| host_contract_pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let number = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "NumberFieldDemo")
        .expect("component showcase pane should expose NumberFieldDemo");
    assert_eq!(number.value_text.as_str(), "47");

    let combo_box = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComboBoxDemo")
        .expect("component showcase pane should expose ComboBoxDemo");
    assert!(combo_box.popup_open);
    assert_eq!(
        combo_box.action_id.as_str(),
        "ui_component_showcase.combo_box_changed"
    );

    let asset = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "AssetFieldDemo")
        .expect("component showcase pane should expose AssetFieldDemo");
    assert_eq!(asset.value_text.as_str(), "res://textures/grid.albedo.png");
    assert_eq!(asset.drop_source_summary.as_str(), "Texture: Grid Albedo");

    let event_log = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "ComponentShowcaseEventLog")
        .expect("component showcase pane should expose event log node");
    assert!(
        event_log
            .text
            .contains("NumberFieldDemo -> DragDelta.NumberField = 47"),
        "event log should reflect the supplied runtime state: {}",
        event_log.text
    );
}
