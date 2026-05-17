use std::fs;

use zircon_runtime::ui::v2::UiV2AssetLoader;

use super::support::*;

#[test]
fn material_component_lab_shell_keeps_material_lab_layout_regions() {
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.v2.ui.toml");
    let lab = UiV2AssetLoader::load_toml_file(&lab_path).unwrap_or_else(|error| {
        panic!(
            "Material Component Lab should load as runtime UI v2 from {}: {error}",
            lab_path.display()
        )
    });

    assert_eq!(
        child_nodes(&lab, "material_lab_root"),
        vec!["appbar", "body"],
        "Material Lab root should keep the top AppBar plus body shell"
    );
    assert_eq!(
        child_nodes(&lab, "appbar"),
        vec![
            "appbar_title",
            "appbar_scope",
            "appbar_count",
            "appbar_status",
            "appbar_capture",
        ],
        "Material Lab AppBar should keep title plus Material status chips"
    );
    assert_eq!(
        child_nodes(&lab, "body"),
        vec!["drawer", "content", "side_panel"],
        "Material Lab body should keep Drawer, content grid, and feedback panel"
    );
    assert_eq!(
        child_nodes(&lab, "drawer"),
        vec![
            "drawer_title",
            "drawer_inputs",
            "drawer_data_display",
            "drawer_feedback",
            "drawer_surfaces",
            "drawer_navigation",
            "drawer_layout",
            "drawer_mui_x",
        ],
        "Material Lab Drawer should keep structured Material navigation items"
    );
    assert_eq!(
        child_nodes(&lab, "content"),
        vec![
            "section_data_display",
            "section_feedback",
            "section_inputs",
            "section_layout",
            "section_mui_x",
            "section_navigation",
            "section_surfaces",
            "section_utils_lab",
        ],
        "Material Lab content should keep the official component-family sections"
    );
    assert_eq!(
        child_nodes(&lab, "section_data_display"),
        vec![
            "data_display_title",
            "prototype_avatars",
            "prototype_badges",
            "prototype_chips",
            "prototype_dividers",
            "prototype_icons",
            "prototype_image_list",
            "prototype_lists",
            "prototype_material_icons",
            "prototype_table",
            "prototype_timeline",
            "prototype_typography",
        ],
        "Material Lab Data Display section should keep the planned component order"
    );
    assert_eq!(
        child_nodes(&lab, "section_feedback"),
        vec![
            "feedback_title",
            "prototype_alert",
            "prototype_backdrop",
            "prototype_dialogs",
            "prototype_modal",
            "prototype_popover",
            "prototype_popper",
            "prototype_progress",
            "prototype_skeleton",
            "prototype_snackbars",
            "prototype_speed_dial",
            "prototype_tooltips",
        ],
        "Material Lab Feedback section should keep the planned component order"
    );
    assert_eq!(
        child_nodes(&lab, "section_inputs"),
        vec![
            "inputs_title",
            "prototype_autocomplete",
            "prototype_button_group",
            "prototype_buttons",
            "prototype_checkboxes",
            "prototype_floating_action_button",
            "prototype_number_field",
            "prototype_radio_buttons",
            "prototype_rating",
            "prototype_selects",
            "prototype_slider",
            "prototype_switches",
            "prototype_text_fields",
            "prototype_textarea_autosize",
            "prototype_toggle_button",
        ],
        "Material Lab Inputs section should keep the planned component order"
    );
    assert_eq!(
        child_nodes(&lab, "section_layout"),
        vec![
            "layout_title",
            "prototype_box",
            "prototype_container",
            "prototype_grid",
            "prototype_masonry",
            "prototype_stack",
        ],
        "Material Lab Layout section should keep the planned component order"
    );
    assert_eq!(
        child_nodes(&lab, "section_mui_x"),
        vec![
            "mui_x_title",
            "prototype_mui_x_tree_view",
            "prototype_mui_x_data_grid",
            "prototype_mui_x_date_time_pickers",
            "prototype_mui_x_charts",
            "prototype_mui_x_line_chart",
            "prototype_mui_x_bar_chart",
            "prototype_mui_x_pie_chart",
            "prototype_mui_x_sparkline",
            "prototype_mui_x_gauge",
            "prototype_mui_x_agent_chat",
            "prototype_mui_x_chat_composer",
        ],
        "Material Lab MUI X section should follow the planned Tree/Data/Pickers/Charts/Chat order"
    );
    assert_eq!(
        child_nodes(&lab, "section_navigation"),
        vec![
            "navigation_title",
            "prototype_bottom_navigation",
            "prototype_breadcrumbs",
            "prototype_links",
            "prototype_menubar",
            "prototype_menus",
            "prototype_pagination",
            "prototype_steppers",
            "prototype_tabs",
            "prototype_transfer_list",
        ],
        "Material Lab Navigation section should keep the planned component order"
    );
    assert_eq!(
        child_nodes(&lab, "section_surfaces"),
        vec![
            "surfaces_title",
            "prototype_accordion",
            "prototype_app_bar",
            "prototype_cards",
            "prototype_drawers",
            "prototype_paper",
        ],
        "Material Lab Surfaces section should keep the planned component order"
    );
    assert_eq!(
        child_nodes(&lab, "section_utils_lab"),
        vec![
            "utils_lab_title",
            "prototype_about_the_lab",
            "prototype_click_away_listener",
            "prototype_css_baseline",
            "prototype_init_color_scheme_script",
            "prototype_no_ssr",
            "prototype_portal",
            "prototype_transitions",
            "prototype_use_media_query",
        ],
        "Material Lab Utils/Lab section should keep the planned component order"
    );

    assert_component(&lab, "appbar", "HorizontalBox");
    assert_component(&lab, "drawer", "VerticalBox");
    assert_component(&lab, "body", "HorizontalBox");
    assert_component(&lab, "content", "ScrollableBox");
    assert_node_class(&lab, "content", "material-lab-content-surface");
    assert_component(&lab, "side_panel", "VerticalBox");
    assert_node_class(&lab, "side_panel", "material-lab-side-panel");
    assert_eq!(
        child_nodes(&lab, "side_panel"),
        vec![
            "side_panel_title",
            "side_panel_variant_caption",
            "side_panel_variant_row_a",
            "side_panel_variant_row_b",
            "side_panel_feedback_caption",
            "side_panel_feedback_row_a",
            "side_panel_feedback_row_b",
            "side_panel_evidence_caption",
            "side_panel_evidence_row",
        ],
        "Material Lab side panel should keep structured guidance rows"
    );

    assert_component(&lab, "appbar_title", "Label");
    assert_node_class(&lab, "appbar_title", "material-lab-appbar-title");
    assert_eq!(
        lab.nodes
            .get("appbar_title")
            .and_then(|node| node.props.get("text"))
            .and_then(|value| value.as_str()),
        Some("Material Component Lab"),
        "AppBar title should keep the lab name"
    );
    for (node_id, expected_text) in [
        ("appbar_scope", "MUI + MUI X"),
        ("appbar_count", "74 prototypes"),
        ("appbar_status", "Static contracts"),
        ("appbar_capture", "Capture ready"),
    ] {
        assert_component(&lab, node_id, "Label");
        assert_node_class(&lab, node_id, "material-lab-appbar-chip");
        let chip = lab
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("AppBar chip `{node_id}` should exist"));
        assert_eq!(
            chip.props.get("text").and_then(|value| value.as_str()),
            Some(expected_text),
            "AppBar chip `{node_id}` should keep text `{expected_text}`"
        );
        for prop in ["corner_radius", "border_width"] {
            assert!(
                chip.props.get(prop).is_some(),
                "AppBar chip `{node_id}` should expose Material prop `{prop}`"
            );
        }
    }
    assert_node_class(&lab, "appbar_scope", "material-lab-appbar-primary");
    assert_eq!(
        lab.nodes
            .get("appbar_scope")
            .and_then(|node| node.props.get("selected"))
            .and_then(|value| value.as_bool()),
        Some(true),
        "AppBar scope chip should show selected coverage"
    );
    assert_node_class(&lab, "appbar_status", "material-lab-appbar-status");
    assert_eq!(
        lab.nodes
            .get("appbar_status")
            .and_then(|node| node.props.get("validation_level"))
            .and_then(|value| value.as_str()),
        Some("success"),
        "AppBar status chip should keep success validation tone"
    );

    for (
        header_id,
        label_id,
        count_id,
        status_id,
        expected_label,
        expected_count,
        expected_status,
    ) in [
        (
            "data_display_title",
            "data_display_title_label",
            "data_display_title_count",
            "data_display_title_status",
            "Data Display",
            "11 components",
            "display",
        ),
        (
            "feedback_title",
            "feedback_title_label",
            "feedback_title_count",
            "feedback_title_status",
            "Feedback",
            "11 components",
            "overlay",
        ),
        (
            "inputs_title",
            "inputs_title_label",
            "inputs_title_count",
            "inputs_title_status",
            "Inputs",
            "14 components",
            "interactive",
        ),
        (
            "layout_title",
            "layout_title_label",
            "layout_title_count",
            "layout_title_status",
            "Layout",
            "5 components",
            "structure",
        ),
        (
            "mui_x_title",
            "mui_x_title_label",
            "mui_x_title_count",
            "mui_x_title_status",
            "MUI X",
            "11 prototypes",
            "mockups",
        ),
        (
            "navigation_title",
            "navigation_title_label",
            "navigation_title_count",
            "navigation_title_status",
            "Navigation",
            "9 components",
            "routing",
        ),
        (
            "surfaces_title",
            "surfaces_title_label",
            "surfaces_title_count",
            "surfaces_title_status",
            "Surfaces",
            "5 components",
            "chrome",
        ),
        (
            "utils_lab_title",
            "utils_lab_title_label",
            "utils_lab_title_count",
            "utils_lab_title_status",
            "Utils/Lab",
            "8 utilities",
            "utility",
        ),
    ] {
        assert_component(&lab, header_id, "HorizontalBox");
        assert_node_class(&lab, header_id, "material-lab-section-header");
        assert_eq!(
            child_nodes(&lab, header_id),
            vec![label_id, count_id, status_id],
            "Material Lab section header `{header_id}` should keep title/count/status chips"
        );

        for (node_id, expected_text, expected_class) in [
            (label_id, expected_label, "material-lab-section-title"),
            (count_id, expected_count, "material-lab-section-chip"),
            (status_id, expected_status, "material-lab-section-status"),
        ] {
            assert_component(&lab, node_id, "Label");
            assert_node_class(&lab, node_id, expected_class);
            assert_eq!(
                lab.nodes
                    .get(node_id)
                    .and_then(|node| node.props.get("text"))
                    .and_then(|value| value.as_str()),
                Some(expected_text),
                "Material Lab section header node `{node_id}` should keep text `{expected_text}`"
            );
        }
    }

    assert_component(&lab, "drawer_title", "Label");
    assert_node_class(&lab, "drawer_title", "material-lab-drawer-title");
    for (node_id, label_id, count_id, expected_label, expected_count) in [
        (
            "drawer_inputs",
            "drawer_inputs_label",
            "drawer_inputs_count",
            "Inputs",
            "14",
        ),
        (
            "drawer_data_display",
            "drawer_data_display_label",
            "drawer_data_display_count",
            "Data Display",
            "11",
        ),
        (
            "drawer_feedback",
            "drawer_feedback_label",
            "drawer_feedback_count",
            "Feedback",
            "11",
        ),
        (
            "drawer_surfaces",
            "drawer_surfaces_label",
            "drawer_surfaces_count",
            "Surfaces",
            "5",
        ),
        (
            "drawer_navigation",
            "drawer_navigation_label",
            "drawer_navigation_count",
            "Navigation",
            "9",
        ),
        (
            "drawer_layout",
            "drawer_layout_label",
            "drawer_layout_count",
            "Layout / Utils",
            "13",
        ),
        (
            "drawer_mui_x",
            "drawer_mui_x_label",
            "drawer_mui_x_count",
            "MUI X",
            "11",
        ),
    ] {
        assert_component(&lab, node_id, "HorizontalBox");
        assert_node_class(&lab, node_id, "material-lab-nav-item");
        assert_eq!(
            child_nodes(&lab, node_id),
            vec![label_id, count_id],
            "Drawer nav row `{node_id}` should keep label plus count chip"
        );
        let nav = lab
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("Drawer node `{node_id}` should exist"));
        let container_kind = nav
            .layout
            .as_ref()
            .and_then(|layout| layout.get("container"))
            .and_then(|container| container.as_table())
            .and_then(|container| container.get("kind"))
            .and_then(|kind| kind.as_str());
        assert_eq!(
            container_kind,
            Some("HorizontalBox"),
            "Drawer nav row `{node_id}` should keep horizontal label/count layout"
        );
        for prop in ["selected", "hovered", "focused", "disabled"] {
            assert!(
                nav.props
                    .get(prop)
                    .is_some_and(|value| value.as_bool().is_some()),
                "Drawer node `{node_id}` should expose boolean Material prop `{prop}`"
            );
        }
        assert_component(&lab, label_id, "Label");
        assert_node_class(&lab, label_id, "material-lab-nav-label");
        assert_eq!(
            lab.nodes
                .get(label_id)
                .and_then(|node| node.props.get("text"))
                .and_then(|value| value.as_str()),
            Some(expected_label),
            "Drawer label `{label_id}` should keep text `{expected_label}`"
        );
        assert_component(&lab, count_id, "Label");
        assert_node_class(&lab, count_id, "material-lab-nav-count");
        let count = lab
            .nodes
            .get(count_id)
            .unwrap_or_else(|| panic!("Drawer count chip `{count_id}` should exist"));
        assert_eq!(
            count.props.get("text").and_then(|value| value.as_str()),
            Some(expected_count),
            "Drawer count chip `{count_id}` should keep count `{expected_count}`"
        );
        for prop in ["corner_radius", "border_width"] {
            assert!(
                count.props.get(prop).is_some(),
                "Drawer count chip `{count_id}` should expose Material prop `{prop}`"
            );
        }
    }
    assert_node_class(&lab, "drawer_inputs", "material-lab-nav-active");
    assert_node_class(&lab, "drawer_inputs_count", "material-lab-nav-count-active");
    assert_eq!(
        lab.nodes
            .get("drawer_inputs")
            .and_then(|node| node.props.get("selected"))
            .and_then(|value| value.as_bool()),
        Some(true),
        "Drawer should show Inputs as the active selected family"
    );
    assert_node_class(&lab, "drawer_data_display", "material-lab-nav-hover");
    assert_node_class(
        &lab,
        "drawer_data_display_count",
        "material-lab-nav-count-hover",
    );
    assert_eq!(
        lab.nodes
            .get("drawer_data_display")
            .and_then(|node| node.props.get("hovered"))
            .and_then(|value| value.as_bool()),
        Some(true),
        "Drawer should keep one hover-state navigation example"
    );

    let content = lab
        .nodes
        .get("content")
        .expect("content node should exist in Material Lab");
    let scroll_axis = content
        .layout
        .as_ref()
        .and_then(|layout| layout.get("container"))
        .and_then(|container| container.as_table())
        .and_then(|container| container.get("axis"))
        .and_then(|axis| axis.as_str());
    assert_eq!(
        scroll_axis,
        Some("Vertical"),
        "Material Lab content should scroll vertically across component families"
    );

    for (section_id, title_id, expected_rows) in [
        ("section_data_display", "data_display_title", 7usize),
        ("section_feedback", "feedback_title", 7),
        ("section_inputs", "inputs_title", 8),
        ("section_layout", "layout_title", 4),
        ("section_mui_x", "mui_x_title", 7),
        ("section_navigation", "navigation_title", 6),
        ("section_surfaces", "surfaces_title", 4),
        ("section_utils_lab", "utils_lab_title", 5),
    ] {
        assert_component(&lab, section_id, "GridBox");
        let section = lab
            .nodes
            .get(section_id)
            .unwrap_or_else(|| panic!("Material Lab section `{section_id}` should exist"));
        let container = section
            .layout
            .as_ref()
            .and_then(|layout| layout.get("container"))
            .and_then(|container| container.as_table())
            .unwrap_or_else(|| panic!("Material Lab section `{section_id}` should define grid"));
        assert_eq!(
            container.get("kind").and_then(|kind| kind.as_str()),
            Some("GridBox"),
            "Material Lab section `{section_id}` should use a GridBox container"
        );
        assert_eq!(
            container
                .get("columns")
                .and_then(|columns| columns.as_integer()),
            Some(2),
            "Material Lab section `{section_id}` should keep a two-column prototype grid"
        );
        assert_eq!(
            container.get("rows").and_then(|rows| rows.as_integer()),
            Some(expected_rows as i64),
            "Material Lab section `{section_id}` should keep the expected grid row count"
        );
        for gap in ["column_gap", "row_gap"] {
            assert_eq!(
                container.get(gap).and_then(|value| value.as_float()),
                Some(10.0),
                "Material Lab section `{section_id}` should keep 10px `{gap}`"
            );
        }

        let title_mount = section
            .children
            .first()
            .unwrap_or_else(|| panic!("Material Lab section `{section_id}` should have a title"));
        assert_eq!(
            title_mount.node.as_str(),
            title_id,
            "Material Lab section `{section_id}` should keep its title first"
        );
        let title_slot = title_mount
            .slot
            .get("layout")
            .and_then(|layout| layout.as_table())
            .unwrap_or_else(|| {
                panic!("Material Lab section `{section_id}` title should span grid")
            });
        assert_eq!(
            title_slot
                .get("column")
                .and_then(|value| value.as_integer()),
            Some(0)
        );
        assert_eq!(
            title_slot.get("row").and_then(|value| value.as_integer()),
            Some(0)
        );
        assert_eq!(
            title_slot
                .get("column_span")
                .and_then(|value| value.as_integer()),
            Some(2),
            "Material Lab section `{section_id}` title should span both grid columns"
        );

        for (prototype_index, child) in section.children.iter().skip(1).enumerate() {
            let slot = child
                .slot
                .get("layout")
                .and_then(|layout| layout.as_table())
                .unwrap_or_else(|| {
                    panic!(
                        "Material Lab section `{section_id}` child `{}` should keep grid slot",
                        child.node
                    )
                });
            assert_eq!(
                slot.get("column").and_then(|value| value.as_integer()),
                Some((prototype_index % 2) as i64),
                "Material Lab section `{section_id}` child `{}` should keep grid column",
                child.node
            );
            assert_eq!(
                slot.get("row").and_then(|value| value.as_integer()),
                Some((1 + prototype_index / 2) as i64),
                "Material Lab section `{section_id}` child `{}` should keep grid row",
                child.node
            );
        }
    }

    for (node_id, expected_text, expected_class) in [
        (
            "side_panel_title",
            "Interaction Contract",
            "material-lab-side-title",
        ),
        (
            "side_panel_variant_caption",
            "Variant Chips",
            "material-lab-side-caption",
        ),
        (
            "side_panel_feedback_caption",
            "Interaction Feedback",
            "material-lab-side-caption",
        ),
        (
            "side_panel_evidence_caption",
            "Capture Evidence",
            "material-lab-side-caption",
        ),
    ] {
        assert_component(&lab, node_id, "Label");
        assert_node_class(&lab, node_id, expected_class);
        assert_eq!(
            lab.nodes
                .get(node_id)
                .and_then(|node| node.props.get("text"))
                .and_then(|value| value.as_str()),
            Some(expected_text),
            "Material Lab side panel label `{node_id}` should keep text `{expected_text}`"
        );
    }

    for (row_id, expected_children) in [
        (
            "side_panel_variant_row_a",
            vec!["side_panel_family_chip", "side_panel_response_chip"],
        ),
        (
            "side_panel_variant_row_b",
            vec!["side_panel_appearance_chip", "side_panel_layout_chip"],
        ),
        (
            "side_panel_feedback_row_a",
            vec![
                "side_panel_hover_chip",
                "side_panel_pressed_chip",
                "side_panel_focus_chip",
            ],
        ),
        (
            "side_panel_feedback_row_b",
            vec![
                "side_panel_selected_chip",
                "side_panel_disabled_chip",
                "side_panel_error_chip",
            ],
        ),
        (
            "side_panel_evidence_row",
            vec![
                "side_panel_startup_chip",
                "side_panel_hover_evidence_chip",
                "side_panel_click_evidence_chip",
            ],
        ),
    ] {
        assert_component(&lab, row_id, "HorizontalBox");
        assert_node_class(&lab, row_id, "material-lab-side-row");
        assert_eq!(
            child_nodes(&lab, row_id),
            expected_children,
            "Material Lab side panel row `{row_id}` should keep its chip order"
        );
    }

    for (node_id, expected_text) in [
        ("side_panel_family_chip", "family"),
        ("side_panel_response_chip", "response"),
        ("side_panel_appearance_chip", "appearance"),
        ("side_panel_layout_chip", "layout"),
        ("side_panel_hover_chip", "Hover"),
        ("side_panel_pressed_chip", "Pressed"),
        ("side_panel_focus_chip", "Focus"),
        ("side_panel_selected_chip", "Selected"),
        ("side_panel_disabled_chip", "Disabled"),
        ("side_panel_error_chip", "Error"),
        ("side_panel_startup_chip", "startup"),
        ("side_panel_hover_evidence_chip", "hover"),
        ("side_panel_click_evidence_chip", "click"),
    ] {
        assert_component(&lab, node_id, "Label");
        assert_node_class(&lab, node_id, "material-lab-side-chip");
        let chip = lab
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("side panel chip `{node_id}` should exist"));
        assert_eq!(
            chip.props.get("text").and_then(|value| value.as_str()),
            Some(expected_text),
            "Material Lab side panel chip `{node_id}` should keep text `{expected_text}`"
        );
        assert!(
            chip.props.get("corner_radius").is_some() && chip.props.get("border_width").is_some(),
            "Material Lab side panel chip `{node_id}` should expose radius and border props"
        );
    }

    assert_node_class(&lab, "side_panel_error_chip", "material-lab-side-error");
    assert_eq!(
        lab.nodes
            .get("side_panel_error_chip")
            .and_then(|node| node.props.get("validation_level"))
            .and_then(|value| value.as_str()),
        Some("error"),
        "Material Lab side panel should keep the error feedback tone visible"
    );
}

#[test]
fn material_component_lab_shell_keeps_material_style_contract() {
    let lab_path = editor_asset("assets/ui/editor/material_component_lab.v2.ui.toml");
    let lab = UiV2AssetLoader::load_toml_file(&lab_path).unwrap_or_else(|error| {
        panic!(
            "Material Component Lab should load as runtime UI v2 from {}: {error}",
            lab_path.display()
        )
    });
    let source = fs::read_to_string(&lab_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", lab_path.display()));

    assert!(
        lab.imports
            .styles
            .iter()
            .any(|style| style == "res://ui/theme/editor_material.v2.ui.toml"),
        "Material Lab should import the shared dark Material v2 theme"
    );
    assert_node_class(&lab, "material_lab_root", "material-lab-shell");
    for node_id in [
        "appbar",
        "drawer",
        "side_panel",
        "data_display_title",
        "feedback_title",
        "inputs_title",
        "layout_title",
        "mui_x_title",
        "navigation_title",
        "surfaces_title",
        "utils_lab_title",
    ] {
        assert_node_class(&lab, node_id, "material-lab-card");
    }

    for token in [
        "selector = \".material-lab-shell\"",
        "selector = \".material-lab-card\"",
        "selector = \".material-lab-appbar-title\"",
        "selector = \".material-lab-appbar-chip\"",
        "selector = \".material-lab-appbar-primary\"",
        "selector = \".material-lab-appbar-status\"",
        "selector = \".material-lab-section-header\"",
        "selector = \".material-lab-section-title\"",
        "selector = \".material-lab-section-chip\"",
        "selector = \".material-lab-section-status\"",
        "selector = \".material-lab-side-panel\"",
        "selector = \".material-lab-side-title\"",
        "selector = \".material-lab-side-caption\"",
        "selector = \".material-lab-side-row\"",
        "selector = \".material-lab-side-chip\"",
        "selector = \".material-lab-side-info\"",
        "selector = \".material-lab-side-success\"",
        "selector = \".material-lab-side-error\"",
        "selector = \".material-lab-content-surface\"",
        "selector = \".material-lab-drawer-title\"",
        "selector = \".material-lab-nav-item\"",
        "selector = \".material-lab-nav-label\"",
        "selector = \".material-lab-nav-count\"",
        "selector = \".material-lab-nav-count-active\"",
        "selector = \".material-lab-nav-count-hover\"",
        "selector = \".material-lab-nav-active\"",
        "selector = \".material-lab-nav-hover\"",
        "selector = \".material-lab-meta-strip\"",
        "selector = \".material-lab-meta-chip\"",
        "selector = \".material-lab-meta-response\"",
        "selector = \".material-lab-meta-variant\"",
        "selector = \".material-lab-meta-layout\"",
        "selector = \".material-lab-state-strip\"",
        "selector = \".material-lab-state-pill\"",
        "selector = \".material-lab-state-hover\"",
        "selector = \".material-lab-state-pressed\"",
        "selector = \".material-lab-state-focus\"",
        "selector = \".material-lab-state-disabled\"",
        "selector = \".material-lab-state-selected\"",
        "selector = \".material-lab-state-open\"",
        "selector = \".material-lab-state-error\"",
        "background_color = \"#101418\"",
        "background_color = \"#202830\"",
        "background_color = \"#0b4050\"",
        "background_color = \"#1e3540\"",
        "background_color = \"#141b20\"",
        "background_color = \"#182128\"",
        "background_color = \"#173126\"",
        "background_color = \"#301b1e\"",
        "background_color = \"#151c22\"",
        "background_color = \"#121b20\"",
        "background_color = \"#142229\"",
        "foreground_color = \"#8fd7e8\"",
        "foreground_color = \"#e6f1f4\"",
        "foreground_color = \"#c8d7dd\"",
        "foreground_color = \"#d8f7fb\"",
        "foreground_color = \"#b9eaf2\"",
        "foreground_color = \"#ffb4ab\"",
        "foreground_color = \"#9de7be\"",
        "foreground_color = \"#ffe08a\"",
        "border_color = \"#4b626d\"",
        "border_color = \"#2aaac0\"",
        "border_color = \"#3f7080\"",
        "border_color = \"#344954\"",
        "border_color = \"#33764f\"",
        "border_color = \"#2c6575\"",
        "border_color = \"#7f6820\"",
        "border_color = \"#2d5b68\"",
        "border_color = \"#36c7d9\"",
        "border_color = \"#e56767\"",
        "radius = 12.0",
        "radius = 10.0",
        "radius = 999.0",
    ] {
        assert!(
            source.contains(token),
            "Material Lab style contract should keep `{token}`"
        );
    }
}

#[test]
fn material_component_lab_profile_capture_scenarios_open_the_lab_window() {
    let script_path = workspace_root().join("tools/ui-profile-capture.ps1");
    let source = fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", script_path.display()));

    for scenario in [
        "material_lab_startup",
        "material_lab_hover",
        "material_lab_click",
    ] {
        assert!(
            source.contains(scenario),
            "profile capture script should define `{scenario}`"
        );
    }
    assert!(source.contains("--builtin-view"));
    assert!(source.contains("editor.material_component_lab"));
    assert!(source.contains("Expand-CaptureScenarioNames"));
    assert!(source.contains("$name -split \",\""));
    assert!(source.contains("Resolve-InteractionScenarioName"));
    assert!(source.contains("UI scenario evidence ($evidenceScenario):"));
    assert!(source.contains("has no hover redraw batch"));
    assert!(source.contains("dependency-bound"));
    assert!(
        source.contains("$templateControlsOnly = $normalizedScenario -eq \"material_lab_click\"")
    );
    assert!(source.contains("-TemplateControlsOnly:$templateControlsOnly"));
}
