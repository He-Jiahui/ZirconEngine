use super::*;

#[test]
fn template_tree_builder_projects_template_instance_into_shared_ui_tree_with_metadata() {
    let document = UiTemplateLoader::load_toml_str(WORKBENCH_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree =
        UiTemplateTreeBuilder::build_tree(UiTreeId::new("workbench.template"), &instance).unwrap();

    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.nodes.len(), 6);

    let root = tree.node(tree.roots[0]).unwrap();
    let root_template = root.template_metadata.as_ref().expect("root metadata");
    assert_eq!(root_template.component, "WorkbenchShell");
    assert_eq!(root_template.control_id, None);

    let open_project = tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OpenProject")
        })
        .unwrap();
    let open_project_template = open_project.template_metadata.as_ref().unwrap();
    assert_eq!(open_project_template.component, "IconButton");
    assert_eq!(
        open_project_template
            .attributes
            .get("icon")
            .unwrap()
            .as_str(),
        Some("folder-open-outline")
    );
    assert_eq!(
        open_project_template
            .attributes
            .get("label")
            .unwrap()
            .as_str(),
        Some("Open")
    );
    assert_eq!(open_project_template.bindings.len(), 1);
    assert_eq!(
        open_project_template.bindings[0].id,
        "WorkbenchMenuBar/OpenProject"
    );
    assert_eq!(
        open_project_template.bindings[0].route.as_deref(),
        Some("MenuAction.OpenProject")
    );
    assert!(open_project.node_path.0.contains("OpenProject"));
    assert!(open_project.state_flags.clickable);
    assert!(open_project.state_flags.hoverable);
    assert!(open_project.state_flags.focusable);
    assert_eq!(open_project.input_policy, UiInputPolicy::Receive);
}

#[test]
fn template_tree_builder_marks_custom_bound_component_interactive() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "ScriptActionChip", control_id = "ActionChip", bindings = [{ id = "Demo/Action", event = "Click", route = "Demo.Action" }] }"#,
    ));
    let node = only_root_node(&tree);

    assert_eq!(node.input_policy, UiInputPolicy::Receive);
    assert!(node.state_flags.clickable);
    assert!(node.state_flags.hoverable);
    assert!(node.state_flags.focusable);
}

#[test]
fn template_tree_builder_infers_scroll_binding_as_receive_input_only() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "ScrollPanel", control_id = "Scrollable", bindings = [{ id = "Demo/Scroll", event = "Scroll", route = "Demo.Scroll" }] }"#,
    ));
    let node = only_root_node(&tree);

    assert_eq!(node.input_policy, UiInputPolicy::Receive);
    assert!(!node.state_flags.clickable);
    assert!(!node.state_flags.hoverable);
    assert!(!node.state_flags.focusable);
}

#[test]
fn template_tree_builder_infers_focus_binding_as_focusable_only() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "FocusProbe", control_id = "FocusProbe", bindings = [{ id = "Demo/Focus", event = "Focus", route = "Demo.Focus" }] }"#,
    ));
    let node = only_root_node(&tree);

    assert_eq!(node.input_policy, UiInputPolicy::Receive);
    assert!(!node.state_flags.clickable);
    assert!(!node.state_flags.hoverable);
    assert!(node.state_flags.focusable);
}

#[test]
fn template_tree_builder_infers_hover_binding_as_hoverable_only() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "HoverProbe", control_id = "HoverProbe", bindings = [{ id = "Demo/Hover", event = "Hover", route = "Demo.Hover" }] }"#,
    ));
    let node = only_root_node(&tree);

    assert_eq!(node.input_policy, UiInputPolicy::Receive);
    assert!(!node.state_flags.clickable);
    assert!(node.state_flags.hoverable);
    assert!(!node.state_flags.focusable);
}

#[test]
fn template_tree_builder_keeps_click_binding_button_like_by_default() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "ActionChip", control_id = "ActionChip", bindings = [{ id = "Demo/Click", event = "Click", route = "Demo.Click" }] }"#,
    ));
    let node = only_root_node(&tree);

    assert_eq!(node.input_policy, UiInputPolicy::Receive);
    assert!(node.state_flags.clickable);
    assert!(node.state_flags.hoverable);
    assert!(node.state_flags.focusable);
}

#[test]
fn template_tree_builder_allows_explicit_focusable_input_metadata() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "FocusRingTarget", control_id = "Focusable", attributes = { input_focusable = true } }"#,
    ));
    let node = only_root_node(&tree);

    assert_eq!(node.input_policy, UiInputPolicy::Receive);
    assert!(!node.state_flags.clickable);
    assert!(!node.state_flags.hoverable);
    assert!(node.state_flags.focusable);
    assert_eq!(
        node.template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("input_focusable"),
        Some(&Value::Boolean(true))
    );
}

#[test]
fn template_tree_builder_keeps_unbound_visual_component_non_interactive() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "DecorativeBadge", control_id = "VisualOnly", attributes = { label = "Info" } }"#,
    ));
    let node = only_root_node(&tree);

    assert_eq!(node.input_policy, UiInputPolicy::Inherit);
    assert!(!node.state_flags.clickable);
    assert!(!node.state_flags.hoverable);
    assert!(!node.state_flags.focusable);
}

#[test]
fn template_tree_builder_honors_explicit_legacy_button_input_opt_out() {
    let tree = tree_from_root_toml(root_with_inline_node(
        r#"{ component = "Button", control_id = "ExplicitOptOut", attributes = { input_interactive = false, input_clickable = false, input_hoverable = false, input_focusable = false } }"#,
    ));
    let node = only_root_node(&tree);

    assert_eq!(node.input_policy, UiInputPolicy::Inherit);
    assert!(!node.state_flags.clickable);
    assert!(!node.state_flags.hoverable);
    assert!(!node.state_flags.focusable);
}
