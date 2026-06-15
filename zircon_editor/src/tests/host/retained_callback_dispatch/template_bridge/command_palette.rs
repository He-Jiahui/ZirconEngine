use super::super::support::*;

const COMMAND_PALETTE_CONTROL_ID: &str = "WorkbenchCommandPalette";
const COMMAND_PALETTE_COMMIT_BINDING_ID: &str = "CommandPalette/Commit";

#[test]
fn workbench_command_palette_surface_exposes_commit_route() {
    let _guard = env_lock().lock().unwrap();
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    assert!(bridge.has_control(COMMAND_PALETTE_CONTROL_ID));
    let binding = bridge
        .binding_for_control(COMMAND_PALETTE_CONTROL_ID, UiEventKind::Submit)
        .expect("workbench command palette should expose submit binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::EditorCommand { command_id }
            if command_id == "editor.command_palette"
    ));

    let node = bridge
        .host_projection()
        .node_by_control_id(COMMAND_PALETTE_CONTROL_ID)
        .expect("command palette should be projected");
    assert!(node.routes.iter().any(|route| {
        route.binding_id == COMMAND_PALETTE_COMMIT_BINDING_ID
            && route.event_kind == UiEventKind::Submit
    }));
}

#[test]
fn workbench_command_palette_open_state_populates_visible_overlay() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    assert!(!bridge.command_palette_open());
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("collapsed")
    );

    let opened = bridge
        .open_command_palette(WorkbenchCommandPaletteOpenState {
            commands: UiValue::Array(vec![
                UiValue::String("workbench.project.open|label=Open Project".to_string()),
                UiValue::String("workbench.project.save|label=Save Project".to_string()),
            ]),
            filtered_commands: UiValue::Array(vec![
                UiValue::String("workbench.project.open".to_string()),
                UiValue::String("workbench.project.save".to_string()),
            ]),
            disabled_commands: UiValue::Array(vec![UiValue::String(
                "workbench.project.save".to_string(),
            )]),
            selected_command_id: "workbench.project.open".to_string(),
            focused_index: 0,
        })
        .expect("command palette should open");

    assert!(opened);
    assert!(bridge.command_palette_open());
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("visible")
    );
    assert_eq!(control_bool_attribute(&bridge, "popup_open"), Some(true));
    assert_eq!(
        control_string_list_attribute(&bridge, "filtered_commands"),
        vec!["workbench.project.open", "workbench.project.save"]
    );
    assert_eq!(
        control_string_list_attribute(&bridge, "disabled_commands"),
        vec!["workbench.project.save"]
    );

    assert!(bridge
        .close_command_palette()
        .expect("command palette should close"));
    assert!(!bridge.command_palette_open());
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("collapsed")
    );
}

#[test]
fn workbench_command_palette_commit_dispatches_editor_command() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_workbench_command_palette_commit_route");
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    dispatch_componentized_workbench_command_palette_committed(
        &harness.runtime,
        &bridge,
        COMMAND_PALETTE_CONTROL_ID,
        COMMAND_PALETTE_COMMIT_BINDING_ID,
        "workbench.project.open",
    )
    .expect("command palette route should be recognized")
    .expect("command palette command should dispatch");

    let journal = harness.runtime.journal();
    let record = journal
        .records()
        .last()
        .expect("command palette commit should append an editor event");
    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenProject)
    );
}

#[test]
fn workbench_command_palette_commit_can_request_palette_open() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_workbench_command_palette_commit_open_route");
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    dispatch_componentized_workbench_command_palette_committed(
        &harness.runtime,
        &bridge,
        COMMAND_PALETTE_CONTROL_ID,
        COMMAND_PALETTE_COMMIT_BINDING_ID,
        "editor.command_palette",
    )
    .expect("command palette route should be recognized")
    .expect("command palette command should dispatch");

    let journal = harness.runtime.journal();
    let record = journal
        .records()
        .last()
        .expect("command palette commit should append an editor event");
    assert_eq!(
        record.event,
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette)
    );
}

fn control_string_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<String> {
    control_attribute(bridge, property)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn control_bool_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<bool> {
    control_attribute(bridge, property).and_then(toml::Value::as_bool)
}

fn control_string_list_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Vec<String> {
    control_attribute(bridge, property)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}

fn control_attribute<'a>(
    bridge: &'a BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<&'a toml::Value> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(COMMAND_PALETTE_CONTROL_ID))
            .and_then(|metadata| metadata.attributes.get(property))
    })
}
