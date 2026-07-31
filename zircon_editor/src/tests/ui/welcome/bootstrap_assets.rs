use crate::ui::layouts::views::welcome_pane_nodes;
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;
use zircon_runtime_interface::ui::layout::UiSize;

const WELCOME_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/welcome.zui"
));
const WELCOME_MAIN_COLUMN_RS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/main_column.rs"
));
const WELCOME_FORM_MODULE_RS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/main_column/form.rs"
));

fn numeric_prop(value: Option<&toml::Value>) -> Option<f64> {
    value.and_then(|value| {
        value
            .as_float()
            .or_else(|| value.as_integer().map(|integer| integer as f64))
    })
}

#[test]
fn welcome_bootstrap_layout_self_hosts_shell_sections() {
    let layout = UiV2AssetLoader::load_toml_str(WELCOME_LAYOUT_TOML).expect("welcome layout");

    for required_node in [
        "welcome_root",
        "outer_panel",
        "recent_panel",
        "recent_header_panel",
        "recent_list_panel",
        "main_panel",
        "hero_panel",
        "status_panel",
        "new_project_header_panel",
        "project_name_label",
        "project_name_field",
        "location_label",
        "location_field",
        "preview_panel",
        "validation_panel",
        "actions_row",
    ] {
        assert!(
            layout.nodes.contains_key(required_node),
            "welcome bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn welcome_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = welcome_pane_nodes(UiSize::new(1280.0, 720.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    for label in [
        "WelcomeOuterPanel",
        "WelcomeRecentPanel",
        "WelcomeRecentHeaderPanel",
        "WelcomeRecentListPanel",
        "WelcomeMainPanel",
        "WelcomeHeroPanel",
        "WelcomeStatusPanel",
        "WelcomeNewProjectHeaderPanel",
        "WelcomeProjectNameField",
        "WelcomeLocationField",
        "WelcomePreviewPanel",
        "WelcomeValidationPanel",
        "WelcomeActionsRow",
    ] {
        let frame = nodes
            .iter()
            .find(|node| node.control_id == label)
            .expect("welcome mount node")
            .frame
            .clone();
        assert!(
            frame.width > 0.0 && frame.height > 0.0,
            "expected `{label}` frame to be laid out by the bootstrap asset"
        );
    }

    let outer = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeOuterPanel")
        .expect("outer panel node");
    let recent = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeRecentPanel")
        .expect("recent panel node");
    let recent_header = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeRecentHeaderPanel")
        .expect("recent header node");
    let recent_list = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeRecentListPanel")
        .expect("recent list node");
    let main = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeMainPanel")
        .expect("main panel node");
    let hero = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeHeroPanel")
        .expect("hero node");
    let status = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeStatusPanel")
        .expect("status node");
    let new_project_header = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeNewProjectHeaderPanel")
        .expect("new project header node");
    let project_name = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeProjectNameField")
        .expect("project name node");
    let location = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeLocationField")
        .expect("location node");
    let preview = nodes
        .iter()
        .find(|node| node.control_id == "WelcomePreviewPanel")
        .expect("preview node");
    let validation = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeValidationPanel")
        .expect("validation node");
    let actions = nodes
        .iter()
        .find(|node| node.control_id == "WelcomeActionsRow")
        .expect("actions node");

    assert_eq!(outer.role.to_string(), "Mount");
    assert!(recent.frame.x >= outer.frame.x);
    assert!(main.frame.x >= recent.frame.x + recent.frame.width);
    assert!(recent_header.frame.y >= recent.frame.y);
    assert!(recent_list.frame.y >= recent_header.frame.y);
    assert!(new_project_header.frame.y >= main.frame.y);
    assert!(project_name.frame.y >= new_project_header.frame.y + new_project_header.frame.height);
    assert!(location.frame.y >= project_name.frame.y + project_name.frame.height);
    assert!(validation.frame.y >= location.frame.y + location.frame.height);
    assert!(actions.frame.y >= validation.frame.y + validation.frame.height);
    assert!(hero.frame.y >= actions.frame.y + actions.frame.height);
    assert!(status.frame.y >= hero.frame.y + hero.frame.height);
    assert!(preview.frame.y >= status.frame.y + status.frame.height);
}

#[test]
fn welcome_mvp_project_actions_remain_inside_short_viewports_before_optional_content() {
    const EPSILON: f32 = 0.01;

    for viewport in [
        UiSize::new(560.0, 384.0),
        UiSize::new(640.0, 420.0),
        UiSize::new(900.0, 520.0),
    ] {
        let pane = welcome_pane_nodes(viewport);
        let nodes = (0..pane.row_count())
            .filter_map(|row| pane.row_data(row))
            .collect::<Vec<_>>();
        let node = |control_id: &str| {
            nodes
                .iter()
                .find(|node| node.control_id == control_id)
                .unwrap_or_else(|| panic!("welcome projection should contain `{control_id}`"))
        };

        let main = node("WelcomeMainPanel");
        let header = node("WelcomeNewProjectHeaderPanel");
        let project_name = node("WelcomeProjectNameField");
        let location = node("WelcomeLocationField");
        let validation = node("WelcomeValidationPanel");
        let actions = node("WelcomeActionsRow");
        let open = node("WelcomeOpenExistingButton");
        let create = node("WelcomeCreateProjectButton");

        assert!(
            actions.frame.y + actions.frame.height <= main.frame.y + main.frame.height + EPSILON,
            "project actions must stay reachable inside a {viewport:?} welcome main panel"
        );
        assert!(project_name.frame.y >= header.frame.y + header.frame.height - EPSILON);
        assert!(location.frame.y >= project_name.frame.y + project_name.frame.height - EPSILON);
        assert!(validation.frame.y >= location.frame.y + location.frame.height - EPSILON);
        assert!(actions.frame.y >= validation.frame.y + validation.frame.height - EPSILON);

        for action in [open, create] {
            assert!(action.frame.x >= actions.frame.x - EPSILON);
            assert!(action.frame.y >= actions.frame.y - EPSILON);
            assert!(
                action.frame.x + action.frame.width
                    <= actions.frame.x + actions.frame.width + EPSILON,
                "action `{}` must not overflow the responsive action row at {viewport:?}",
                action.control_id
            );
            assert!(
                action.frame.y + action.frame.height
                    <= actions.frame.y + actions.frame.height + EPSILON
            );
        }
        assert!(open.frame.x + open.frame.width <= create.frame.x + EPSILON);

        for optional_control in [
            "WelcomeHeroPanel",
            "WelcomeStatusPanel",
            "WelcomePreviewPanel",
            "WelcomeStartupChooserRow",
        ] {
            let optional = node(optional_control);
            assert!(
                optional.frame.y >= actions.frame.y + actions.frame.height - EPSILON,
                "optional `{optional_control}` content must follow project actions at {viewport:?}"
            );
        }
    }
}

#[test]
fn welcome_mvp_columns_prioritize_project_actions_before_recent_history_width() {
    const EPSILON: f32 = 0.01;

    for viewport in [
        UiSize::new(560.0, 384.0),
        UiSize::new(640.0, 420.0),
        UiSize::new(900.0, 520.0),
        UiSize::new(1260.0, 780.0),
    ] {
        let pane = welcome_pane_nodes(viewport);
        let nodes = (0..pane.row_count())
            .filter_map(|row| pane.row_data(row))
            .collect::<Vec<_>>();
        let node = |control_id: &str| {
            nodes
                .iter()
                .find(|node| node.control_id == control_id)
                .unwrap_or_else(|| panic!("welcome projection should contain `{control_id}`"))
        };
        let outer = node("WelcomeOuterPanel");
        let recent = node("WelcomeRecentPanel");
        let main = node("WelcomeMainPanel");

        assert!(recent.frame.width >= 220.0 - EPSILON);
        assert!(recent.frame.width <= 320.0 + EPSILON);
        assert!(
            main.frame.width >= 280.0 - EPSILON,
            "project actions need at least 280px at {viewport:?}"
        );
        assert!(
            recent.frame.width + main.frame.width <= outer.frame.width + EPSILON,
            "responsive Welcome columns must stay inside the outer panel at {viewport:?}"
        );
    }
}

#[test]
fn welcome_mvp_recent_section_uses_starship_vertical_rhythm() {
    const EPSILON: f32 = 0.01;
    let pane = welcome_pane_nodes(UiSize::new(560.0, 384.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("welcome projection should contain `{control_id}`"))
    };
    let recent = node("WelcomeRecentPanel");
    let header = node("WelcomeRecentHeaderPanel");
    let list = node("WelcomeRecentListPanel");

    assert!((header.frame.y - recent.frame.y - 18.0).abs() <= EPSILON);
    assert!((header.frame.height - 46.0).abs() <= EPSILON);
    assert!((list.frame.y - header.frame.y - header.frame.height - 8.0).abs() <= EPSILON);
}

#[test]
fn welcome_mvp_fields_and_actions_use_starship_control_density() {
    let layout = UiV2AssetLoader::load_toml_str(WELCOME_LAYOUT_TOML).expect("welcome layout");

    for field_id in ["project_name_field", "location_field"] {
        let field = layout
            .nodes
            .get(field_id)
            .unwrap_or_else(|| panic!("welcome layout should contain `{field_id}`"));
        assert_eq!(numeric_prop(field.props.get("corner_radius")), Some(4.0));
        assert_eq!(numeric_prop(field.props.get("height")), Some(44.0));
        let height = field
            .layout
            .as_ref()
            .and_then(|layout| layout.get("height"))
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("`{field_id}` should define a height layout constraint"));
        assert_eq!(numeric_prop(height.get("preferred")), Some(44.0));
    }

    for button_id in ["open_existing_button", "create_project_button"] {
        let button = layout
            .nodes
            .get(button_id)
            .unwrap_or_else(|| panic!("welcome layout should contain `{button_id}`"));
        assert_eq!(numeric_prop(button.props.get("corner_radius")), Some(4.0));
        assert_eq!(numeric_prop(button.props.get("height")), Some(32.0));
    }
}

#[test]
fn welcome_mvp_field_labels_use_shared_runtime_text_label_nodes() {
    let layout = UiV2AssetLoader::load_toml_str(WELCOME_LAYOUT_TOML).expect("welcome layout");
    let pane = welcome_pane_nodes(UiSize::new(640.0, 420.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    for (node_id, control_id, field_control_id, text) in [
        (
            "project_name_label",
            "WelcomeProjectNameLabel",
            "WelcomeProjectNameField",
            "Project name",
        ),
        (
            "location_label",
            "WelcomeLocationLabel",
            "WelcomeLocationField",
            "Location",
        ),
    ] {
        let label = layout
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("welcome layout should contain `{node_id}`"));
        assert_eq!(label.component, "Label");
        assert_eq!(
            label.props.get("text").and_then(toml::Value::as_str),
            Some(text)
        );
        assert_eq!(
            label.props.get("text_tone").and_then(toml::Value::as_str),
            Some("muted")
        );
        assert_eq!(
            numeric_prop(label.props.get("font_size")),
            Some(EditorTypographyTokens::WORKBENCH_CAPTION_SIZE as f64)
        );

        let projected = nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("welcome projection should contain `{control_id}`"));
        assert!(projected.frame.width > 0.0 && projected.frame.height > 0.0);
        let field = nodes
            .iter()
            .find(|node| node.control_id == field_control_id)
            .unwrap_or_else(|| panic!("welcome projection should contain `{field_control_id}`"));
        assert!(projected.frame.y + projected.frame.height <= field.frame.y + 0.01);
    }
}

#[test]
fn welcome_standard_controls_are_owned_by_template_painter_not_native_overlays() {
    let layout = UiV2AssetLoader::load_toml_str(WELCOME_LAYOUT_TOML).expect("welcome layout");

    for (node_id, component) in [
        ("project_name_field", "TextField"),
        ("location_field", "TextField"),
        ("open_existing_button", "Button"),
        ("create_project_button", "Button"),
    ] {
        assert_eq!(
            layout
                .nodes
                .get(node_id)
                .unwrap_or_else(|| panic!("welcome layout should contain `{node_id}`"))
                .component,
            component,
            "Welcome standard control `{node_id}` must remain on the shared template painter"
        );
    }

    for forbidden in ["mod actions;", "draw_welcome_actions", "draw_welcome_field"] {
        assert!(
            !WELCOME_MAIN_COLUMN_RS.contains(forbidden),
            "Welcome native main column must not overlay shared controls through `{forbidden}`"
        );
    }
    assert!(
        !WELCOME_FORM_MODULE_RS.contains("mod field;"),
        "Welcome native form module must not retain a duplicate TextField painter"
    );
}
