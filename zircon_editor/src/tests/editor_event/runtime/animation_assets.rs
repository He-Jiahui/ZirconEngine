use super::*;

#[test]
fn animation_binding_without_active_sequence_editor_reports_ignored_status_line() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_animation_binding");
    let binding = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "CreateTrackButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTrack {
            track_path: "Root/Hero:AnimationPlayer.weight".to_string(),
        }),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("animation binding should dispatch through runtime");

    assert_eq!(
        record.event,
        EditorEvent::Animation(EditorAnimationEvent::CreateTrack {
            track_path: AnimationTrackPath::parse("Root/Hero:AnimationPlayer.weight").unwrap(),
        })
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(record
        .effects
        .contains(&EditorEventEffect::ReflectionChanged));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Ignored animation command because active center tab is not an animation sequence editor"
    );
}

#[test]
fn animation_graph_and_state_machine_bindings_without_open_editor_report_ignored_status_line() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_animation_graph_binding");
    let binding = EditorUiBinding::new(
        "AnimationGraphEditorView",
        "CreateTransition",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTransition {
            state_machine_path: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 8,
        }),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("graph/state-machine animation binding should dispatch through runtime");

    assert_eq!(
        record.event,
        EditorEvent::Animation(EditorAnimationEvent::CreateTransition {
            state_machine_path: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 8,
        })
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(record
        .effects
        .contains(&EditorEventEffect::ReflectionChanged));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Ignored animation command because active center tab is not an animation graph editor"
    );
}

#[test]
fn workbench_menu_open_ui_asset_opens_ui_asset_editor_for_shared_asset() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_menu_open_ui_asset");
    let ui_asset_path = std::env::temp_dir().join("zircon_editor_event_menu_open_ui_asset.zui");
    fs::write(
        &ui_asset_path,
        r#"
[asset]
kind = "view"
id = "editor.tests.menu_ui_asset"
version = 2
display_name = "Menu UI Asset"

[root]
node = "root"

[nodes.root]
component = "Label"
props = { text = "Menu" }
"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: ui_asset_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("menu open ui asset");

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_path: ui_asset_path.to_string_lossy().into_owned(),
        })
    );
    assert!(record.effects.contains(&EditorEventEffect::LayoutChanged));
    assert!(runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .any(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset")));

    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn asset_open_event_routes_animation_assets_to_animation_editor_views() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_animation_asset_open");
    let sequence_path =
        std::env::temp_dir().join("zircon_editor_event_animation_asset_open.sequence.zranim");
    let graph_path =
        std::env::temp_dir().join("zircon_editor_event_animation_asset_open.graph.zranim");
    let state_machine_path =
        std::env::temp_dir().join("zircon_editor_event_animation_asset_open.state_machine.zranim");
    fs::write(&sequence_path, b"").unwrap();
    fs::write(&graph_path, b"").unwrap();
    fs::write(&state_machine_path, b"").unwrap();

    let sequence_record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: sequence_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open animation sequence asset");
    assert!(sequence_record
        .effects
        .contains(&EditorEventEffect::LayoutChanged));

    let instances = runtime.runtime.current_view_instances();
    let sequence_view = instances
        .iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("animation sequence view should open");
    assert_eq!(
        sequence_view.serializable_payload["path"],
        json!(sequence_path.to_string_lossy().to_string())
    );

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: graph_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open animation graph asset");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: state_machine_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open animation state machine asset");

    let graph_views = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .filter(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph")
        })
        .collect::<Vec<_>>();
    assert_eq!(graph_views.len(), 2);
    assert!(graph_views.iter().any(|instance| {
        instance.serializable_payload["path"] == json!(graph_path.to_string_lossy().to_string())
    }));
    assert!(graph_views.iter().any(|instance| {
        instance.serializable_payload["path"]
            == json!(state_machine_path.to_string_lossy().to_string())
    }));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        format!(
            "Opened animation graph editor for {}",
            state_machine_path.to_string_lossy()
        )
    );

    let _ = fs::remove_file(sequence_path);
    let _ = fs::remove_file(graph_path);
    let _ = fs::remove_file(state_machine_path);
}

#[test]
fn asset_kind_filter_event_accepts_physics_and_animation_asset_kinds() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_asset_kind_filters");
    for (kind, expected) in [
        ("PhysicsMaterial", ResourceKind::PhysicsMaterial),
        ("AnimationSequence", ResourceKind::AnimationSequence),
        ("AnimationGraph", ResourceKind::AnimationGraph),
        ("AnimationStateMachine", ResourceKind::AnimationStateMachine),
    ] {
        let record = runtime
            .runtime
            .dispatch_event(
                EditorEventSource::Headless,
                EditorEvent::Asset(EditorAssetEvent::SetKindFilter {
                    kind: Some(kind.to_string()),
                }),
            )
            .expect("asset kind filter event");

        assert_eq!(
            runtime.runtime.editor_snapshot().asset_activity.kind_filter,
            Some(expected)
        );
        assert_eq!(
            runtime.runtime.editor_snapshot().asset_browser.kind_filter,
            Some(expected)
        );
        assert!(record
            .effects
            .contains(&EditorEventEffect::PresentationChanged));
    }
}
