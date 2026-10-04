const REACTBITS_FIXTURES: &[&str] = &[
    include_str!("fixtures/ui/reactbits_agent_native_components.zui"),
    include_str!("fixtures/ui/reactbits_agent_workflow_components.zui"),
    include_str!("fixtures/ui/reactbits_agent_workspace.zui"),
    include_str!("fixtures/ui/reactbits_data_surface_components.zui"),
    include_str!("fixtures/ui/reactbits_auth_onboarding_components.zui"),
    include_str!("fixtures/ui/reactbits_kanban_scheduling_components.zui"),
    include_str!("fixtures/ui/reactbits_settings_form_components.zui"),
    include_str!("fixtures/ui/reactbits_workbench_interaction_surfaces.zui"),
];

const WINDOWED_COLLECTION_FIXTURES: &[&str] = &[
    include_str!("fixtures/ui/reactbits_agent_workflow_components.zui"),
    include_str!("fixtures/ui/reactbits_data_surface_components.zui"),
    include_str!("fixtures/ui/reactbits_auth_onboarding_components.zui"),
    include_str!("fixtures/ui/reactbits_kanban_scheduling_components.zui"),
    include_str!("fixtures/ui/reactbits_settings_form_components.zui"),
];

const SEMANTIC_PAINTERS: &[&str] = &[
    include_str!("../src/ui/surface/render/semantic_components/agent_workflow.rs"),
    include_str!("../src/ui/surface/render/semantic_components/data_surfaces.rs"),
];

const CONTENT_OWNERS: &[&str] = &[
    include_str!("../src/ui/surface/render/agent_chat.rs"),
    include_str!("../src/ui/surface/render/command_palette.rs"),
    include_str!("../src/ui/surface/render/notification_center.rs"),
];

fn has_numeric_icon_size(source: &str) -> bool {
    source.match_indices("icon_size =").any(|(index, token)| {
        source[index + token.len()..]
            .trim_start()
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit() || character == '-')
    })
}

#[test]
fn icon_size_contract_detects_inline_numeric_values() {
    assert!(has_numeric_icon_size("props = { icon_size = 18 }"));
    assert!(has_numeric_icon_size("layout_icon_size = -1"));
    assert!(!has_numeric_icon_size("props = { icon_size = \"m\" }"));
}

#[test]
fn reactbits_fixture_layouts_use_flow_containers_and_local_spacing_only() {
    for source in REACTBITS_FIXTURES {
        assert!(
            source.contains("container = { kind = \"VerticalBox\"")
                || source.contains("container = { kind = \"HorizontalBox\"")
                || source.contains("component = \"HorizontalBox\"")
                || source.contains("component = \"VerticalBox\""),
            "fixture should declare a flow container"
        );
        for forbidden in [
            "layout_offset_x",
            "layout_offset_y",
            "CanvasBox",
            "position =",
            "absolute =",
        ] {
            assert!(
                !source.contains(forbidden),
                "ordinary ReactBits fixture layout must not use coordinate-style authoring: {forbidden}"
            );
        }
        assert!(
            !has_numeric_icon_size(source),
            "ReactBits fixture icons must use a named s/m/l/xl tier rather than a numeric size"
        );
    }
}

#[test]
fn reactbits_fixture_collections_are_explicit_test_data() {
    for source in REACTBITS_FIXTURES {
        assert!(source.contains("fixture_data_only = true"));
    }
    for source in WINDOWED_COLLECTION_FIXTURES {
        assert!(source.contains("collection_virtualization = \"windowed\""));
        assert!(source.contains("visible_limit ="));
    }
}

#[test]
fn semantic_painters_keep_copy_caller_owned_and_layout_flow_based() {
    for source in SEMANTIC_PAINTERS {
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "Agent plan",
            "Tool calls",
            "Approval required",
            "AI usage",
            "No folders",
            "No assets",
            "layout_offset_x",
            "layout_offset_y",
            "CanvasBox",
            "absolute =",
        ] {
            assert!(
                !production.contains(forbidden),
                "semantic painter must not own visible fallback copy or coordinate authoring: {forbidden}"
            );
        }
        assert!(production.contains("flow_content_frame"));
        assert!(production.contains("collection_window"));
    }
}

#[test]
fn content_owners_never_invent_visible_default_copy() {
    for source in CONTENT_OWNERS {
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "Search commands",
            "No commands found",
            "Notifications",
            "No notifications",
            "more\")",
        ] {
            assert!(
                !production.contains(forbidden),
                "production UI copy must stay caller- or localization-owned: {forbidden}"
            );
        }
    }
    let agent_chat = CONTENT_OWNERS[0].split("#[cfg(test)]").next().unwrap();
    let command_palette = CONTENT_OWNERS[1].split("#[cfg(test)]").next().unwrap();
    let notification_center = CONTENT_OWNERS[2].split("#[cfg(test)]").next().unwrap();
    assert!(agent_chat.contains("overflow_text"));
    assert!(command_palette.contains("PLACEHOLDER"));
    assert!(command_palette.contains("empty_text"));
    assert!(notification_center.contains("EMPTY_TEXT"));
}
