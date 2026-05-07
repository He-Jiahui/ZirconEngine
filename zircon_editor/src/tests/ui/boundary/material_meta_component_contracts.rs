use std::path::PathBuf;

use toml::Value;

const REQUIRED_M2_ROLE_TOKENS: &[&str] = &[
    "material_surface",
    "material_surface_inset",
    "material_surface_hover",
    "material_surface_pressed",
    "material_surface_selected",
    "material_surface_disabled",
    "material_accent",
    "material_accent_soft",
    "material_border",
    "material_text",
    "material_text_muted",
    "material_text_disabled",
    "material_warning",
    "material_warning_container",
    "material_error",
    "material_error_container",
    "material_popup",
    "material_track",
    "material_focus_ring",
    "material_color_primary",
    "material_color_on_primary",
    "material_color_surface",
    "material_color_surface_container",
    "material_color_surface_container_low",
    "material_color_surface_container_high",
    "material_color_on_surface",
    "material_color_on_surface_variant",
    "material_color_outline",
    "material_color_outline_variant",
    "material_color_error",
    "material_color_on_error",
    "material_color_shadow",
    "material_color_scrim",
    "material_state_layer_opacity_hover",
    "material_state_layer_opacity_focus",
    "material_state_layer_opacity_press",
    "material_state_layer_opacity_disabled",
    "material_state_layer_opacity_drag",
    "material_disabled_opacity",
    "material_focus_ring_width",
    "material_focus_ring_offset",
    "material_density_compact_height",
    "material_density_default_height",
    "material_density_prominent_height",
    "material_density_compact_control_height",
    "material_density_default_control_height",
    "material_density_field_control_height",
    "material_density_list_item_height",
    "material_spacing_button_content_x",
    "material_spacing_button_content_y",
    "material_spacing_button_gap",
    "material_spacing_field_content_x",
    "material_spacing_field_content_y",
    "material_spacing_list_item_content_x",
    "material_spacing_list_item_gap",
    "material_icon_size_button",
    "material_icon_size_standard",
    "material_shadow",
    "material_shadow_soft",
    "material_elevation_level0",
    "material_elevation_level1",
    "material_elevation_level2",
    "material_radius_control",
    "material_radius_small",
    "material_radius_medium",
    "material_radius_large",
    "material_radius_pill",
    "material_border_width",
    "material_border_width_none",
    "material_font_size_title",
    "material_font_size_body",
    "material_font_size_meta",
];
const REQUIRED_STATEFUL_COMPONENTS: &[(&str, &str, &[&str])] = &[
    (
        "MaterialButton",
        "material-button",
        &["hovered", "pressed", "focused", "disabled"],
    ),
    (
        "MaterialIconButton",
        "material-icon-button",
        &[
            "selected",
            "hovered",
            "pressed",
            "focused",
            "disabled",
            "validation_level",
            "text_tone",
        ],
    ),
    (
        "MaterialToggleButton",
        "material-toggle-button",
        &[
            "checked", "selected", "hovered", "pressed", "focused", "disabled",
        ],
    ),
    (
        "MaterialCheckboxRow",
        "material-checkbox-row",
        &[
            "checked", "selected", "hovered", "pressed", "focused", "disabled",
        ],
    ),
    (
        "MaterialCheckBox",
        "material-check-box",
        &[
            "checked", "selected", "hovered", "pressed", "focused", "disabled",
        ],
    ),
    (
        "MaterialOutlinedField",
        "material-outlined-field",
        &["hovered", "focused", "disabled"],
    ),
    (
        "MaterialLineEdit",
        "material-line-edit",
        &["hovered", "focused", "disabled", "validation_level"],
    ),
    (
        "MaterialComboBox",
        "material-combo-box",
        &[
            "popup_open",
            "selection_state",
            "hovered",
            "pressed",
            "focused",
            "disabled",
        ],
    ),
    (
        "MaterialSliderField",
        "material-slider-field",
        &["hovered", "pressed", "focused", "disabled"],
    ),
    (
        "MaterialSlider",
        "material-slider",
        &["hovered", "pressed", "focused", "disabled"],
    ),
    (
        "MaterialSpinBox",
        "material-spin-box",
        &[
            "hovered",
            "pressed",
            "focused",
            "disabled",
            "validation_level",
        ],
    ),
    (
        "MaterialSwitch",
        "material-switch",
        &[
            "checked", "selected", "hovered", "pressed", "focused", "disabled",
        ],
    ),
    (
        "MaterialListItem",
        "material-list-item",
        &["selected", "hovered", "pressed", "focused", "disabled"],
    ),
    (
        "MaterialTableRow",
        "material-table-row",
        &["selected", "hovered", "pressed", "focused", "disabled"],
    ),
    (
        "MaterialMenuItem",
        "material-menu-item",
        &[
            "checked", "selected", "hovered", "pressed", "focused", "disabled",
        ],
    ),
    (
        "MaterialTabImpl",
        "material-tab-impl",
        &["selected", "hovered", "pressed", "focused", "disabled"],
    ),
    (
        "MaterialTextEdit",
        "material-text-edit",
        &["hovered", "focused", "disabled"],
    ),
];
const REQUIRED_INPUT_CAPABILITY_COMPONENTS: &[&str] = &[
    "MaterialButton",
    "MaterialIconButton",
    "MaterialToggleButton",
    "MaterialCheckboxRow",
    "MaterialOutlinedField",
    "MaterialSliderField",
    "MaterialListItem",
    "MaterialTableRow",
    "MaterialTextButton",
    "MaterialCheckBox",
    "MaterialComboBox",
    "MaterialDatePickerPopup",
    "MaterialGroupBox",
    "MaterialLineEdit",
    "MaterialMenuBarItem",
    "MaterialMenuFrame",
    "MaterialMenuItem",
    "MaterialScrollView",
    "MaterialSlider",
    "MaterialSpinBox",
    "MaterialSwitch",
    "MaterialStandardTableView",
    "MaterialTabImpl",
    "MaterialTextEdit",
    "MaterialTimePickerPopup",
];
const REQUIRED_INPUT_CAPABILITY_PROPS: &[&str] = &[
    "input_interactive",
    "input_clickable",
    "input_hoverable",
    "input_focusable",
];
const REQUIRED_POPUP_ANCHOR_COMPONENTS: &[&str] = &[
    "MaterialComboBox",
    "MaterialDatePickerPopup",
    "MaterialMenuFrame",
    "MaterialTimePickerPopup",
];
const REQUIRED_POPUP_ANCHOR_PROPS: &[&str] = &["popup_open", "popup_anchor_x", "popup_anchor_y"];
const REQUIRED_ROOT_FORWARDING_COMPONENTS: &[(&str, &[&str])] = &[
    (
        "MaterialButton",
        &[
            "binding_id",
            "action_id",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialIconButton",
        &[
            "binding_id",
            "action_id",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialOutlinedField",
        &[
            "binding_id",
            "edit_action_id",
            "commit_action_id",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialComboBox",
        &[
            "binding_id",
            "action_id",
            "popup_open",
            "popup_anchor_x",
            "popup_anchor_y",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialListItem",
        &[
            "binding_id",
            "action_id",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialMenuFrame",
        &[
            "action_id",
            "popup_open",
            "popup_anchor_x",
            "popup_anchor_y",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialMenuItem",
        &[
            "binding_id",
            "action_id",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialDatePickerPopup",
        &[
            "action_id",
            "popup_open",
            "popup_anchor_x",
            "popup_anchor_y",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialTimePickerPopup",
        &[
            "action_id",
            "popup_open",
            "popup_anchor_x",
            "popup_anchor_y",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialTabImpl",
        &[
            "binding_id",
            "action_id",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialMenuBarItem",
        &[
            "binding_id",
            "action_id",
            "input_focusable",
            "capability",
            "accessibility_label",
        ],
    ),
    (
        "MaterialGroupBox",
        &["input_focusable", "capability", "accessibility_label"],
    ),
    (
        "MaterialScrollView",
        &["input_focusable", "capability", "accessibility_label"],
    ),
];
const REQUIRED_VISUAL_DENSITY_TOKENS: &[(&str, f64)] = &[
    ("material_density_compact_height", 28.0),
    ("material_density_default_height", 32.0),
    ("material_density_prominent_height", 40.0),
    ("material_spacing_button_content_x", 12.0),
    ("material_spacing_button_content_y", 6.0),
    ("material_spacing_button_gap", 6.0),
    ("material_spacing_field_content_x", 10.0),
    ("material_spacing_list_item_content_x", 10.0),
    ("material_icon_size_button", 16.0),
    ("material_icon_size_standard", 18.0),
];
const REQUIRED_SHARED_STYLE_ROOTS: &[(&str, &[&str])] = &[
    (
        "MaterialButtonBase",
        &[
            "surface_variant",
            "button_variant",
            "text_tone",
            "disabled",
            "focused",
            "hovered",
            "pressed",
            "border_width",
            "corner_radius",
            "font_size",
            "layout_min_height",
        ],
    ),
    (
        "MaterialTextButton",
        &[
            "surface_variant",
            "button_variant",
            "text_tone",
            "disabled",
            "focused",
            "hovered",
            "pressed",
            "border_width",
            "corner_radius",
            "font_size",
            "layout_min_height",
        ],
    ),
    (
        "MaterialMenuBarItem",
        &[
            "surface_variant",
            "button_variant",
            "disabled",
            "focused",
            "hovered",
            "pressed",
            "border_width",
            "corner_radius",
            "font_size",
            "layout_min_height",
        ],
    ),
    (
        "MaterialGroupBox",
        &[
            "surface_variant",
            "disabled",
            "focused",
            "hovered",
            "pressed",
            "border_width",
            "corner_radius",
            "font_size",
            "layout_min_height",
        ],
    ),
];
const REQUIRED_VISUAL_DENSITY_TOKEN_ALIASES: &[(&str, &str)] = &[
    (
        "material_density_field_control_height",
        "$material_density_default_height",
    ),
    (
        "material_icon_button_size",
        "$material_density_default_control_height",
    ),
];

#[test]
fn material_theme_declares_m2_role_tokens_and_styles_material_classes() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let material_theme = load_document(&repo.join("assets/ui/theme/editor_material.ui.toml"));
    let material_meta =
        load_document(&repo.join("assets/ui/editor/material_meta_components.ui.toml"));
    let tokens = material_theme
        .get("tokens")
        .and_then(Value::as_table)
        .expect("Material theme declares [tokens]");
    let missing_tokens = REQUIRED_M2_ROLE_TOKENS
        .iter()
        .copied()
        .filter(|token| !tokens.contains_key(*token))
        .collect::<Vec<_>>();
    assert!(
        missing_tokens.is_empty(),
        "editor_material.ui.toml missing M2 Material role tokens: {}",
        missing_tokens.join(", ")
    );

    let selectors = stylesheet_selectors(&material_theme);
    let mut missing_rules = Vec::new();
    for class in material_classes(&material_meta) {
        if !selectors
            .iter()
            .any(|selector| selector_has_class(selector, &class))
        {
            missing_rules.push(class);
        }
    }
    assert!(
        missing_rules.is_empty(),
        "editor_material.ui.toml must style every material_meta_components class: {}",
        missing_rules.join(", ")
    );
}

#[test]
fn material_meta_components_emit_stable_state_metadata() {
    let document = material_meta_document();
    let components = material_components(&document);
    let mut failures = Vec::new();

    for (component, stable_class, required_props) in REQUIRED_STATEFUL_COMPONENTS {
        let Some(root) = components
            .get(*component)
            .and_then(|component| component.get("root"))
        else {
            failures.push(format!("missing component `{component}`"));
            continue;
        };
        if !node_has_class(root, stable_class) {
            failures.push(format!(
                "{component} root must emit stable `{stable_class}` class"
            ));
        }
        for prop in *required_props {
            if !node_props(root).is_some_and(|props| props.contains_key(*prop)) {
                failures.push(format!(
                    "{component} root must project `{prop}` state metadata"
                ));
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn material_meta_components_project_input_and_popup_contracts() {
    let document = material_meta_document();
    let components = material_components(&document);
    let mut failures = Vec::new();

    for component in REQUIRED_INPUT_CAPABILITY_COMPONENTS {
        let Some(root) = components
            .get(*component)
            .and_then(|component| component.get("root"))
        else {
            failures.push(format!("missing component `{component}`"));
            continue;
        };
        for prop in REQUIRED_INPUT_CAPABILITY_PROPS {
            if !node_props(root).is_some_and(|props| props.contains_key(*prop)) {
                failures.push(format!(
                    "{component} root must project `{prop}` input capability metadata"
                ));
            }
        }
    }

    for component in REQUIRED_POPUP_ANCHOR_COMPONENTS {
        let Some(root) = components
            .get(*component)
            .and_then(|component| component.get("root"))
        else {
            failures.push(format!("missing component `{component}`"));
            continue;
        };
        for prop in REQUIRED_POPUP_ANCHOR_PROPS {
            if !node_props(root).is_some_and(|props| props.contains_key(*prop)) {
                failures.push(format!(
                    "{component} root must project `{prop}` popup anchor metadata"
                ));
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn material_meta_component_roots_forward_interaction_accessibility_and_capability_params() {
    let document = material_meta_document();
    let components = material_components(&document);
    let mut failures = Vec::new();

    for (component_name, forwarded_props) in REQUIRED_ROOT_FORWARDING_COMPONENTS {
        let Some(component) = components.get(*component_name) else {
            failures.push(format!("missing component `{component_name}`"));
            continue;
        };
        let Some(params) = component.get("params").and_then(Value::as_table) else {
            failures.push(format!("{component_name} must declare [params]"));
            continue;
        };
        let Some(root_props) = component.get("root").and_then(node_props) else {
            failures.push(format!("{component_name} root must declare props"));
            continue;
        };

        for prop in *forwarded_props {
            if !params.contains_key(*prop) {
                failures.push(format!("{component_name} must declare `{prop}` param"));
                continue;
            }
            match root_props.get(*prop).and_then(Value::as_str) {
                Some(value) if value == format!("$param.{prop}") => {}
                Some(value) => failures.push(format!(
                    "{component_name} root `{prop}` must forward `$param.{prop}`, got `{value}`"
                )),
                None => failures.push(format!("{component_name} root must project `{prop}`")),
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn material_meta_components_carry_shared_style_defaults_on_root_nodes() {
    let document = material_meta_document();
    let components = material_components(&document);
    let mut failures = Vec::new();

    for (component_name, required_props) in REQUIRED_SHARED_STYLE_ROOTS {
        let Some(root_props) = components
            .get(*component_name)
            .and_then(|component| component.get("root"))
            .and_then(node_props)
        else {
            failures.push(format!("{component_name} root must declare props"));
            continue;
        };
        for prop in *required_props {
            if !root_props.contains_key(*prop) {
                failures.push(format!(
                    "{component_name} root must carry shared Material style prop `{prop}`"
                ));
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn editor_visual_density_contracts_keep_icons_and_chrome_professional_scale() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let material_meta =
        load_document(&repo.join("assets/ui/editor/material_meta_components.ui.toml"));
    let activity_rail =
        load_document(&repo.join("assets/ui/editor/workbench_activity_rail.ui.toml"));
    let workbench_shell =
        load_document(&repo.join("assets/ui/editor/host/workbench_shell.ui.toml"));
    let inspector_controls =
        load_document(&repo.join("assets/ui/editor/host/inspector_surface_controls.ui.toml"));
    let tokens = material_meta
        .get("tokens")
        .and_then(Value::as_table)
        .expect("material meta components declare [tokens]");
    let mut failures = Vec::new();

    for (token, expected) in REQUIRED_VISUAL_DENSITY_TOKENS {
        match tokens.get(*token).and_then(Value::as_float) {
            Some(value) if value == *expected => {}
            Some(value) => failures.push(format!(
                "`{token}` must be {expected} for editor density B, got {value}"
            )),
            None => failures.push(format!("`{token}` must be numeric and declared")),
        }
    }
    for (token, expected) in REQUIRED_VISUAL_DENSITY_TOKEN_ALIASES {
        match tokens.get(*token).and_then(Value::as_str) {
            Some(value) if value == *expected => {}
            Some(value) => failures.push(format!(
                "`{token}` must alias `{expected}` for editor density B, got `{value}`"
            )),
            None => failures.push(format!("`{token}` must alias `{expected}`")),
        }
    }

    assert_number_at(
        &workbench_shell,
        &[
            "components",
            "ActivityRail",
            "root",
            "layout",
            "width",
            "preferred",
        ],
        44.0,
        &mut failures,
    );
    assert_child_controls_max_square(
        &workbench_shell,
        &["components", "ActivityRail", "root"],
        "IconButton",
        32.0,
        &mut failures,
    );
    assert_child_controls_max_square_with_control_prefix(
        &activity_rail,
        &["root"],
        "Container",
        "ActivityRailButton",
        32.0,
        &mut failures,
    );
    assert_child_controls_max_square(&activity_rail, &["root"], "Icon", 18.0, &mut failures);
    assert_child_controls_max_height(
        &workbench_shell,
        &["components", "MenuBar", "root"],
        "IconButton",
        24.0,
        &mut failures,
    );
    assert_child_controls_max_height(
        &inspector_controls,
        &["components", "InspectorSurfaceControls", "root"],
        "reference",
        28.0,
        &mut failures,
    );

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn material_meta_document() -> Value {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    load_document(&repo.join("assets/ui/editor/material_meta_components.ui.toml"))
}

fn material_components(document: &Value) -> &toml::map::Map<String, Value> {
    document
        .get("components")
        .and_then(Value::as_table)
        .expect("material meta components declare [components]")
}

fn load_document(path: &std::path::Path) -> Value {
    let source = std::fs::read_to_string(path).expect("ui asset is readable");
    let source = source.trim_start_matches("stylesheets = []").trim_start();
    toml::from_str(source).expect("ui asset parses as toml")
}

fn stylesheet_selectors(document: &Value) -> Vec<String> {
    document
        .get("stylesheets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|stylesheet| stylesheet.get("rules").and_then(Value::as_array))
        .flatten()
        .filter_map(|rule| rule.get("selector").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

fn material_classes(document: &Value) -> Vec<String> {
    let mut classes = Vec::new();
    visit_value(document, &mut |value| {
        let Some(table) = value.as_table() else {
            return;
        };
        let Some(node_classes) = table.get("classes").and_then(Value::as_array) else {
            return;
        };
        for class in node_classes {
            let Some(class) = class.as_str() else {
                continue;
            };
            if class.starts_with("material-") && !classes.iter().any(|known| known == class) {
                classes.push(class.to_string());
            }
        }
    });
    classes.sort();
    classes
}

fn selector_has_class(selector: &str, class: &str) -> bool {
    selector
        .split(|character: char| character.is_whitespace() || character == '>' || character == '+')
        .any(|part| part.contains(&format!(".{class}")))
}

fn node_has_class(node: &Value, expected: &str) -> bool {
    node.get("classes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|class| class.as_str() == Some(expected))
}

fn node_props(node: &Value) -> Option<&toml::map::Map<String, Value>> {
    node.get("props")
        .or_else(|| node.get("params"))
        .and_then(Value::as_table)
}

fn assert_number_at(document: &Value, path: &[&str], expected: f64, failures: &mut Vec<String>) {
    match value_at(document, path).and_then(value_as_number) {
        Some(value) if value == expected => {}
        Some(value) => failures.push(format!(
            "`{}` must be {expected}, got {value}",
            path.join(".")
        )),
        None => failures.push(format!("`{}` must be numeric", path.join("."))),
    }
}

fn assert_child_controls_max_square(
    document: &Value,
    root_path: &[&str],
    expected_type: &str,
    max_size: f64,
    failures: &mut Vec<String>,
) {
    assert_child_controls_max_axis(
        document,
        root_path,
        expected_type,
        None,
        max_size,
        true,
        failures,
    );
}

fn assert_child_controls_max_square_with_control_prefix(
    document: &Value,
    root_path: &[&str],
    expected_type: &str,
    control_prefix: &str,
    max_size: f64,
    failures: &mut Vec<String>,
) {
    assert_child_controls_max_axis(
        document,
        root_path,
        expected_type,
        Some(control_prefix),
        max_size,
        true,
        failures,
    );
}

fn assert_child_controls_max_height(
    document: &Value,
    root_path: &[&str],
    expected_type: &str,
    max_height: f64,
    failures: &mut Vec<String>,
) {
    assert_child_controls_max_axis(
        document,
        root_path,
        expected_type,
        None,
        max_height,
        false,
        failures,
    );
}

fn assert_child_controls_max_axis(
    document: &Value,
    root_path: &[&str],
    expected_type: &str,
    control_prefix: Option<&str>,
    max_axis: f64,
    require_square: bool,
    failures: &mut Vec<String>,
) {
    let Some(root) = value_at(document, root_path) else {
        failures.push(format!("`{}` must exist", root_path.join(".")));
        return;
    };
    let mut matched = 0usize;
    visit_child_nodes(root, &mut |node| {
        if node_type(node) != Some(expected_type) {
            return;
        }
        let control_id = node
            .get("control_id")
            .and_then(Value::as_str)
            .unwrap_or("<unnamed>");
        if control_prefix.is_some_and(|prefix| !control_id.starts_with(prefix)) {
            return;
        }
        matched += 1;
        let height = value_at(node, &["layout", "height", "preferred"])
            .or_else(|| value_at(node, &["params", "height"]))
            .and_then(value_as_number);
        if height.map_or(true, |height| height > max_axis) {
            failures.push(format!(
                "`{}` {expected_type} `{control_id}` height must be <= {max_axis}, got {:?}",
                root_path.join("."),
                height
            ));
        }
        if !require_square {
            return;
        }
        let width = value_at(node, &["layout", "width", "preferred"]).and_then(value_as_number);
        if width.map_or(true, |width| width > max_axis) {
            failures.push(format!(
                "`{}` {expected_type} `{control_id}` width must be <= {max_axis}, got {:?}",
                root_path.join("."),
                width
            ));
        }
    });
    if matched == 0 {
        failures.push(format!(
            "`{}` must contain at least one {expected_type}",
            root_path.join(".")
        ));
    }
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value, |value, key| value.get(*key))
}

fn value_as_number(value: &Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
}

fn node_type(node: &Value) -> Option<&str> {
    node.get("type")
        .or_else(|| node.get("kind"))
        .and_then(Value::as_str)
}

fn visit_child_nodes(root: &Value, visit: &mut impl FnMut(&Value)) {
    let Some(children) = root.get("children").and_then(Value::as_array) else {
        return;
    };
    for child in children {
        let Some(node) = child.get("node") else {
            continue;
        };
        visit(node);
    }
}

fn visit_value(value: &Value, visit: &mut impl FnMut(&Value)) {
    visit(value);
    match value {
        Value::Array(values) => values.iter().for_each(|value| visit_value(value, visit)),
        Value::Table(values) => values.values().for_each(|value| visit_value(value, visit)),
        _ => {}
    }
}
