use super::super::*;

#[test]
fn command_palette_command_requests_open_effect() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_command_palette_open");
    let binding = EditorUiBinding::new(
        "CommandPalette",
        "OpenCommandPalette",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::editor_command("editor.command.palette"),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("command palette editor command should dispatch");

    assert_eq!(
        record.event,
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette)
    );
    assert_eq!(
        record.effects,
        vec![EditorEventEffect::CommandPaletteOpenRequested]
    );
}

#[test]
fn material_component_lab_binding_records_feedback_without_business_effects() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_material_lab_feedback");
    let binding = EditorUiBinding::new(
        "MaterialComponentLab",
        "MaterialLabChips",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::Custom(UiBindingCall::new("MaterialComponentLab")),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("Material Lab prototype binding should dispatch");

    assert!(record.effects.is_empty());
    assert_eq!(record.before_revision, record.after_revision);
    assert_eq!(
        record.operation_group.as_deref(),
        Some("MaterialComponentLab")
    );
    assert!(record.result.error.is_none());

    let effects = dispatch_builtin_template_binding(&runtime.runtime, "MaterialLab/Chips/Click")
        .expect("Material Lab builtin binding should exist")
        .expect("Material Lab builtin binding should dispatch");
    let dirty_domains = effects.dirty_domains();
    assert!(dirty_domains.contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!dirty_domains.requires_layout());
    assert!(!dirty_domains.requires_presentation());
}

#[test]
fn retained_preset_menu_actions_normalize_to_layout_events_with_expected_names() {
    let save = retained_menu_action("workbench.layout.preset.save.rider").unwrap();
    let load = retained_menu_action("workbench.layout.preset.load.").unwrap();
    let legacy_save = retained_menu_action("SavePreset.rider").unwrap();
    let legacy_load = retained_menu_action("LoadPreset.").unwrap();

    assert_eq!(
        save.event,
        EditorEvent::Layout(LayoutCommand::SavePreset {
            name: "rider".to_string(),
        })
    );
    assert_eq!(
        load.event,
        EditorEvent::Layout(LayoutCommand::LoadPreset {
            name: "current".to_string(),
        })
    );
    assert_eq!(legacy_save.event, save.event);
    assert_eq!(legacy_load.event, load.event);
}

#[test]
fn scene_menu_actions_dispatch_through_runtime_and_request_picker_presentation() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_scene_menu_actions");
    let open_record = runtime
        .runtime
        .dispatch_binding(
            menu_action_binding(&MenuAction::OpenScene),
            EditorEventSource::Headless,
        )
        .unwrap();
    let create_record = runtime
        .runtime
        .dispatch_binding(
            menu_action_binding(&MenuAction::CreateScene),
            EditorEventSource::Headless,
        )
        .unwrap();

    assert_eq!(
        open_record.event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenScene)
    );
    assert_eq!(
        create_record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateScene)
    );
    assert!(open_record
        .effects
        .contains(&EditorEventEffect::OpenScenePickerRequested));
    assert!(create_record
        .effects
        .contains(&EditorEventEffect::CreateScenePickerRequested));
    assert!(!open_record
        .effects
        .contains(&EditorEventEffect::LayoutChanged));
    assert!(!create_record
        .effects
        .contains(&EditorEventEffect::LayoutChanged));
}

#[test]
fn close_view_layout_event_removes_the_view_instance_from_runtime_registry_state() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_close_view");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::WorkbenchMenu(MenuAction::OpenView(EventViewDescriptorId::new(
                "editor.asset_browser",
            ))),
        )
        .unwrap();

    let opened_instance = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.asset_browser"))
        .expect("asset browser view should open");

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Layout(LayoutCommand::CloseView {
                instance_id: EventViewInstanceId::new(opened_instance.instance_id.0.clone()),
            }),
        )
        .unwrap();

    assert!(
        runtime
            .runtime
            .current_view_instances()
            .into_iter()
            .all(|instance| instance.instance_id != opened_instance.instance_id),
        "closed view instance should be removed from runtime session registry"
    );
}

#[test]
fn draft_inspector_binding_normalizes_and_updates_live_snapshot() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_draft_inspector");
    let binding = EditorUiBinding::parse_native_binding(
        r#"InspectorView/NameField:onChange(DraftCommand.SetInspectorField("entity://selected","name","Draft Cube"))"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("draft inspector binding should dispatch through runtime");

    assert_eq!(
        runtime
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Draft Cube")
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(!record.effects.contains(&EditorEventEffect::RenderChanged));
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
}

#[test]
fn draft_mesh_import_path_binding_normalizes_and_updates_live_snapshot() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_draft_mesh_import");
    let binding = EditorUiBinding::parse_native_binding(
        r#"AssetsView/MeshImportPathEdited:onChange(DraftCommand.SetMeshImportPath("E:/Models/cube.glb"))"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("mesh import path draft binding should dispatch through runtime");

    assert_eq!(
        runtime.runtime.editor_snapshot().mesh_import_path,
        "E:/Models/cube.glb"
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(!record.effects.contains(&EditorEventEffect::RenderChanged));
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
}

#[test]
fn asset_import_binding_normalizes_to_runtime_host_request() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_asset_import");
    let binding = EditorUiBinding::parse_native_binding(
        r#"AssetsView/ImportModel:onClick(AssetCommand.ImportModel())"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("asset import binding should dispatch through runtime");

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::ImportModel)
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::ImportModelRequested));
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
    assert!(!record.effects.contains(&EditorEventEffect::RenderChanged));
}
