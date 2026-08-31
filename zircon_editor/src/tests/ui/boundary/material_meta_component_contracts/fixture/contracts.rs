use super::*;

#[test]
fn material_theme_declares_m2_role_tokens_and_styles_material_classes() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let material_theme = load_document(&zui_fixture(&repo, "theme/editor_material.zui"));
    let material_meta = material_meta_document();
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
        "editor_material.zui missing M2 Material role tokens: {}",
        missing_tokens.join(", ")
    );

    let selectors = stylesheet_selectors(&material_theme);
    let mut missing_rules = Vec::new();
    for class in material_classes(material_meta) {
        if !selectors
            .iter()
            .any(|selector| selector_has_class(selector, &class))
        {
            missing_rules.push(class);
        }
    }
    assert!(
        missing_rules.is_empty(),
        "editor_material.zui must style every material_meta_components class: {}",
        missing_rules.join(", ")
    );
}

#[test]
fn material_meta_components_emit_stable_state_metadata() {
    let document = material_meta_document();
    let mut failures = Vec::new();

    for (component, stable_class, required_props) in REQUIRED_STATEFUL_COMPONENTS {
        let Some(root) = component_root_node(document, component) else {
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
    let mut failures = Vec::new();

    for component in REQUIRED_INPUT_CAPABILITY_COMPONENTS {
        let Some(root) = component_root_node(document, component) else {
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
        let Some(root) = component_root_node(document, component) else {
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
    let components = material_components(document);
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
        let Some(root_props) =
            component_root_node_from_document(document, component).and_then(node_props)
        else {
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
    let mut failures = Vec::new();

    for (component_name, required_props) in REQUIRED_SHARED_STYLE_ROOTS {
        let Some(root_props) = component_root_node(document, component_name).and_then(node_props)
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
    let material_meta = material_meta_document();
    let activity_rail = load_document(&repo.join("assets/ui/editor/workbench_activity_rail.zui"));
    let workbench_shell = load_document(&repo.join("assets/ui/editor/host/workbench_shell.zui"));
    let inspector_controls =
        load_document(&repo.join("assets/ui/editor/host/inspector_surface_controls.zui"));
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
        &["nodes", "activity_rail", "layout", "width", "preferred"],
        44.0,
        &mut failures,
    );
    assert_child_controls_max_square(
        &workbench_shell,
        &["nodes", "activity_rail"],
        "WorkbenchRailButton",
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
    assert_number_at(
        &workbench_shell,
        &["nodes", "menu_bar", "layout", "height", "preferred"],
        24.0,
        &mut failures,
    );
    assert_child_controls_max_height(
        &inspector_controls,
        &["root"],
        "InputField",
        28.0,
        &mut failures,
    );
    assert_child_controls_max_height(
        &inspector_controls,
        &["root"],
        "NumberField",
        28.0,
        &mut failures,
    );

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
