use super::super::support::*;

const COMMAND_PALETTE_CONTROL_ID: &str = "WorkbenchCommandPalette";
const COMMAND_PALETTE_COMMIT_BINDING_ID: &str = "CommandPalette/Commit";
const COMMAND_PALETTE_QUERY_BINDING_ID: &str = "CommandPalette/QueryChanged";
const COMMAND_PALETTE_WINDOW_BINDING_ID: &str = "CommandPalette/WindowRequested";

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
            if command_id == "editor.command.palette"
    ));
    let query_binding = bridge
        .binding_for_control(COMMAND_PALETTE_CONTROL_ID, UiEventKind::Change)
        .expect("workbench command palette should expose query change binding");
    assert_eq!(query_binding.path().event_kind, EditorUiEventKind::Change);
    assert_eq!(
        bridge
            .binding_id_for_action_id(COMMAND_PALETTE_QUERY_BINDING_ID)
            .as_deref(),
        Some(COMMAND_PALETTE_QUERY_BINDING_ID)
    );
    assert_eq!(
        bridge
            .binding_id_for_action_id(COMMAND_PALETTE_WINDOW_BINDING_ID)
            .as_deref(),
        Some(COMMAND_PALETTE_WINDOW_BINDING_ID)
    );

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
            query: String::new(),
            commands: UiValue::Array(vec![
                UiValue::String("file.project.open|label=Open Project".to_string()),
                UiValue::String("file.project.save|label=Save Project".to_string()),
            ]),
            filtered_commands: UiValue::Array(vec![
                UiValue::String("file.project.open".to_string()),
                UiValue::String("file.project.save".to_string()),
            ]),
            selected_command_id: "file.project.open".to_string(),
            focused_index: 0,
            catalog_generation: 7,
            total_match_count: 2,
            window_offset: 0,
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
        control_integer_attribute(&bridge, "catalog_generation"),
        Some(7)
    );
    assert_eq!(control_integer_attribute(&bridge, "match_count"), Some(2));
    assert_eq!(
        control_string_list_attribute(&bridge, "filtered_commands"),
        vec!["file.project.open", "file.project.save"]
    );

    bridge
        .update_command_palette_query(WorkbenchCommandPaletteOpenState {
            query: "save".to_string(),
            commands: UiValue::Array(vec![UiValue::String(
                "file.project.save|label=Save Project".to_string(),
            )]),
            filtered_commands: UiValue::Array(vec![UiValue::String(
                "file.project.save".to_string(),
            )]),
            selected_command_id: "file.project.save".to_string(),
            focused_index: 0,
            catalog_generation: 7,
            total_match_count: 1,
            window_offset: 0,
        })
        .expect("query state should update while palette stays open");

    assert!(bridge.command_palette_open());
    assert_eq!(
        control_string_attribute(&bridge, "query").as_deref(),
        Some("save")
    );
    assert_eq!(
        control_string_list_attribute(&bridge, "filtered_commands"),
        vec!["file.project.save"]
    );
    assert!(
        bridge
            .close_command_palette()
            .expect("command palette should close")
    );
    assert!(!bridge.command_palette_open());
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("collapsed")
    );
}

#[test]
fn scene_picker_chrome_is_scoped_to_its_palette_open() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    bridge
        .open_command_palette_with_chrome(
            palette_state("scene-picker-open-0", "res://levels/main.scene.toml"),
            "scene-picker-open",
            "Search project scenes",
            "No scene assets found in this project",
            "Open project scene",
            "Choose a project-owned scene asset",
        )
        .expect("scene picker chrome should open");

    assert_eq!(bridge.command_palette_source(), "scene-picker-open");
    assert_eq!(
        control_string_attribute(&bridge, "placeholder").as_deref(),
        Some("Search project scenes")
    );
    assert_eq!(
        control_string_attribute(&bridge, "empty_text").as_deref(),
        Some("No scene assets found in this project")
    );

    bridge
        .open_command_palette(palette_state("file.project.open", "Open Project"))
        .expect("normal command palette should reopen");

    assert_eq!(bridge.command_palette_source(), "workbench");
    assert_eq!(
        control_string_attribute(&bridge, "placeholder").as_deref(),
        Some("Search commands")
    );
    assert_eq!(
        control_string_attribute(&bridge, "empty_text").as_deref(),
        Some("No commands found")
    );
}

#[test]
fn command_palette_anchor_tracks_the_current_workbench_size() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    bridge
        .recompute_layout(UiSize::new(360.0, 640.0))
        .expect("narrow workbench layout should recompute");
    bridge
        .open_command_palette(palette_state("file.project.open", "Open Project"))
        .expect("command palette should open after a narrow layout recompute");

    assert_float_attribute(&bridge, "popup_anchor_x", 14.4);
    assert_float_attribute(&bridge, "popup_anchor_y", 64.0);
    assert_float_attribute(&bridge, "popup_anchor_width", 331.2);

    bridge
        .recompute_layout(UiSize::new(1200.0, 900.0))
        .expect("desktop workbench layout should recompute");

    assert_float_attribute(&bridge, "popup_anchor_x", 32.0);
    assert_float_attribute(&bridge, "popup_anchor_y", 72.0);
    assert_float_attribute(&bridge, "popup_anchor_width", 1136.0);
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
        "file.project.open",
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
    assert_eq!(
        harness
            .runtime
            .command_palette_mru()
            .entries()
            .first()
            .map(|command| command.as_str()),
        Some("file.project.open")
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
        "editor.command.palette",
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

fn palette_state(command_id: &str, label: &str) -> WorkbenchCommandPaletteOpenState {
    WorkbenchCommandPaletteOpenState {
        query: String::new(),
        commands: UiValue::Array(vec![UiValue::Map(std::collections::BTreeMap::from([
            ("id".to_string(), UiValue::String(command_id.to_string())),
            ("label".to_string(), UiValue::String(label.to_string())),
        ]))]),
        filtered_commands: UiValue::Array(vec![UiValue::String(command_id.to_string())]),
        selected_command_id: command_id.to_string(),
        focused_index: 0,
        catalog_generation: 1,
        total_match_count: 1,
        window_offset: 0,
    }
}

fn control_bool_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<bool> {
    control_attribute(bridge, property).and_then(toml::Value::as_bool)
}

fn control_integer_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<i64> {
    control_attribute(bridge, property).and_then(toml::Value::as_integer)
}

fn assert_float_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
    expected: f64,
) {
    let actual = control_attribute(bridge, property)
        .and_then(toml::Value::as_float)
        .expect("command palette float property should exist");
    assert!(
        (actual - expected).abs() < 0.001,
        "{property} expected {expected}, got {actual}"
    );
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
