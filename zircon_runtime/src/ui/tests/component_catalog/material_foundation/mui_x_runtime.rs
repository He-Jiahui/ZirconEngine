use super::*;

#[test]
fn material_editor_foundation_catalog_covers_mui_x_runtime_visibility_contracts() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let expected_len = registry.len();
    for component_id in ["Collapse", "Fade", "Grow", "Slide", "Zoom"] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing transition descriptor `{component_id}`"));
        for prop in [
            "transition_kind",
            "in",
            "transition_status",
            "transition_progress",
            "timeout_ms",
            "transition_duration_ms",
            "easing",
            "transition_easing",
            "mount_on_enter",
            "unmount_on_exit",
        ] {
            assert_has_prop(descriptor, prop);
        }
    }
    assert_has_prop(
        registry
            .descriptor("Collapse")
            .expect("Collapse descriptor"),
        "orientation",
    );
    assert_has_prop(
        registry
            .descriptor("Collapse")
            .expect("Collapse descriptor"),
        "collapsed_size",
    );
    assert_has_prop(
        registry.descriptor("Slide").expect("Slide descriptor"),
        "direction",
    );

    let material_tree = registry
        .descriptor("MaterialTreeView")
        .expect("MUI X Tree View descriptor");
    assert_has_prop(material_tree, "editable");
    assert_has_event(material_tree, UiComponentEventKind::ToggleExpanded);

    let data_grid = registry
        .descriptor("DataGrid")
        .expect("DataGrid descriptor");
    assert_eq!(data_grid.layout_role, UiComponentLayoutRole::VirtualList);
    assert_has_prop(data_grid, "columns");
    assert_has_prop(data_grid, "rows");
    assert_has_event(data_grid, UiComponentEventKind::SetVisibleRange);

    let date_time_pickers = registry
        .descriptor("DateTimePickers")
        .expect("DateTimePickers descriptor");
    assert_has_prop(date_time_pickers, "date_value");
    assert_has_prop(date_time_pickers, "time_value");
    assert_has_prop(date_time_pickers, "picker_mode");
    assert_has_event(date_time_pickers, UiComponentEventKind::OpenPopup);
    assert_has_event(date_time_pickers, UiComponentEventKind::ClosePopup);
    assert_has_event(date_time_pickers, UiComponentEventKind::Commit);

    for component_id in [
        "Charts",
        "LineChart",
        "BarChart",
        "PieChart",
        "SparkLineChart",
        "Gauge",
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing chart descriptor `{component_id}`"));
        assert_has_prop(descriptor, "series");
        assert_has_event(descriptor, UiComponentEventKind::Hover);
    }

    let agent_chat = registry
        .descriptor("AgentChat")
        .expect("AgentChat descriptor");
    assert_has_prop(agent_chat, "messages");
    assert_has_prop(agent_chat, "composer_text");
    assert_has_prop(agent_chat, "streaming");
    assert_has_event(agent_chat, UiComponentEventKind::Commit);

    let editor_visible = registry.descriptors_for_host(&UiHostCapabilitySet::editor_authoring());
    assert_eq!(editor_visible.len(), expected_len);
    let runtime_visible_ids = registry
        .descriptors_for_host(&UiHostCapabilitySet::runtime_basic())
        .into_iter()
        .map(|descriptor| descriptor.id.as_str())
        .collect::<BTreeSet<_>>();
    assert!(!runtime_visible_ids.contains("DockHost"));
    assert!(!runtime_visible_ids.contains("WorkbenchShell"));
}
