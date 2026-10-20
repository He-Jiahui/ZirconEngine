//! 模板绑定只声明所需输入能力，构树时推断初始点击、悬停与焦点；显式输入属性能覆盖旧组件名兜底。

use super::*;

#[test]
fn template_tree_builder_projects_template_instance_into_shared_ui_tree_with_metadata() {
    let instance = compiled_instance_from_toml(WORKBENCH_TEMPLATE_TOML);

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
    assert!(node.state_flags.hoverable);
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

#[test]
fn template_scroll_only_child_routes_scroll_without_click_or_focus() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use zircon_runtime_interface::ui::{
        binding::UiEventKind,
        dispatch::{UiPointerDispatchEffect, UiPointerEvent},
        layout::{UiPoint, UiSize},
        surface::{UiPointerButton, UiPointerEventKind},
    };

    for explicit_opt_out in [false, true] {
        let source = r#"
[root]
component = "VerticalBox"
attributes = { layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox" } } }
children = [
    { component = "Container", control_id = "ScrollOnly", bindings = [{ id = "Demo/Scroll", event = "Scroll", route = "Demo.Scroll" }], attributes = { HOVER_OVERRIDE layout = { width = { stretch = "Stretch" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } } } },
    { component = "Container", control_id = "VisualOnly", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } } } },
    { component = "Container", control_id = "ClickOnly", bindings = [{ id = "Demo/Click", event = "Click", route = "Demo.Click" }], attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } } } }
]
"#.replace("HOVER_OVERRIDE", if explicit_opt_out { "input_hoverable = false," } else { "" });
        let instance = compiled_instance_from_toml(&source);
        let program = crate::ui::template::compile_binding_program(
            &instance.root,
            "runtime.ui.scroll.legacy",
        )
        .unwrap();
        let instance = UiTemplateInstance::with_binding_program(instance.root, program);
        let mut surface = UiTemplateSurfaceBuilder::build_surface(
            UiTreeId::new("runtime.ui.scroll.legacy"),
            &instance,
        )
        .unwrap();

        surface.compute_layout(UiSize::new(128.0, 128.0)).unwrap();
        let child = surface
            .tree
            .nodes
            .values()
            .find(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some("ScrollOnly")
            })
            .unwrap()
            .node_id;
        let visual = surface
            .tree
            .nodes
            .values()
            .find(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some("VisualOnly")
            })
            .unwrap()
            .node_id;
        let click = surface
            .tree
            .nodes
            .values()
            .find(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some("ClickOnly")
            })
            .unwrap()
            .node_id;
        let node = surface.tree.node(child).unwrap();
        assert_eq!(
            node.parent,
            Some(surface.tree.roots[0]),
            "regression requires a nonroot child"
        );
        assert_eq!(node.input_policy, UiInputPolicy::Receive);
        assert_eq!(node.state_flags.hoverable, !explicit_opt_out);
        assert_eq!(node.supports_pointer(), !explicit_opt_out);
        assert!(!node.state_flags.clickable);
        assert!(!node.state_flags.focusable);
        assert!(!surface.tree.node(visual).unwrap().supports_pointer());
        let click_flags = &surface.tree.node(click).unwrap().state_flags;
        assert!(click_flags.clickable && click_flags.hoverable && click_flags.focusable);
        assert_eq!(
            surface.compiled_binding_event_source_count_for_test(child, UiEventKind::Scroll),
            Some(1),
        );
        let frame = surface.arranged_tree.get(child).unwrap().frame;
        assert!(frame.width > 2.0 && frame.height > 2.0);
        let point = UiPoint::new(frame.x + 1.0, frame.y + 1.0);
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let mut dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
        dispatcher.register(child, UiPointerEventKind::Scroll, move |context| {
            assert_eq!(context.node_id, child);
            assert_eq!(context.route.scroll_delta, 48.0);
            observed.fetch_add(1, Ordering::SeqCst);
            UiPointerDispatchEffect::Handled
        });
        let scrolled = surface
            .dispatch_pointer_event(
                &dispatcher,
                UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(48.0),
            )
            .unwrap();
        if explicit_opt_out {
            assert_ne!(scrolled.route.target, Some(child));
            assert_eq!(scrolled.handled_by, None);
            assert_eq!(calls.load(Ordering::SeqCst), 0);
        } else {
            assert_eq!(scrolled.route.target, Some(child));
            assert_eq!(scrolled.handled_by, Some(child));
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        }
        for kind in [UiPointerEventKind::Down, UiPointerEventKind::Up] {
            let result = surface
                .dispatch_pointer_event(
                    &dispatcher,
                    UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
                )
                .unwrap();
            assert_eq!(result.route.click_target, None);
            assert_eq!(result.focus_changed_to, None);
            assert!(!result.component_events.iter().any(|event| {
                matches!(event.event_kind, UiEventKind::Click | UiEventKind::Focus)
            }));
            assert_eq!(surface.focus.focused, None);
        }
        assert_eq!(calls.load(Ordering::SeqCst), usize::from(!explicit_opt_out));
        let visual_calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&visual_calls);
        dispatcher.register(visual, UiPointerEventKind::Scroll, move |_| {
            observed.fetch_add(1, Ordering::SeqCst);
            UiPointerDispatchEffect::Handled
        });
        let frame = surface.arranged_tree.get(visual).unwrap().frame;
        let result = surface
            .dispatch_pointer_event(
                &dispatcher,
                UiPointerEvent::new(
                    UiPointerEventKind::Scroll,
                    UiPoint::new(frame.x + 1.0, frame.y + 1.0),
                )
                .with_scroll_delta(48.0),
            )
            .unwrap();
        assert_ne!(result.route.target, Some(visual));
        assert_eq!(visual_calls.load(Ordering::SeqCst), 0);
    }
}
