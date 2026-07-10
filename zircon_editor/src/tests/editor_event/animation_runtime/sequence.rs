use super::*;

#[test]
fn animation_sequence_binding_marks_active_sequence_editor_dirty_and_updates_session_state() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_sequence_dirty");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_sequence_asset")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();

    let binding = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "CreateTrackButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTrack {
            track_path: "Root/Hero:Transform.translation".to_string(),
        }),
    );
    harness
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("sequence editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should be queryable after command");

    assert!(
        instance.dirty,
        "animation authoring command should mark instance dirty"
    );
    assert!(pane
        .track_items
        .contains(&"Root/Hero:Transform.translation".to_string()));
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Created animation track Root/Hero:Transform.translation"
    );
}

#[test]
fn animation_sequence_ignores_timeline_selection_for_missing_track() {
    let _guard = env_lock().lock().unwrap();
    let harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_sequence_missing_selection");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_sequence_missing_selection")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SelectTimelineSpan {
                    track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:Transform.rotation",
                        )
                        .unwrap(),
                    start_frame: 24,
                    end_frame: 48,
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("sequence editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should remain queryable after invalid selection");

    assert!(
        pane.selection_summary.is_empty(),
        "missing-track selection should not create a phantom timeline selection"
    );
    assert!(
        !instance.dirty,
        "missing-track selection should remain a no-op for the document"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_sequence_removing_selected_track_clears_selection_summary() {
    let _guard = env_lock().lock().unwrap();
    let harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_sequence_remove_selected");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_sequence_remove_selected")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset_with_multiple_tracks(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SelectTimelineSpan {
                    track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:Transform.translation",
                        )
                        .unwrap(),
                    start_frame: 24,
                    end_frame: 48,
                },
            ),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::RemoveTrack {
                    track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:Transform.translation",
                        )
                        .unwrap(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("sequence editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should remain queryable after removing a selected track");

    assert!(
        pane.track_items
            .iter()
            .all(|item| item != "Root/Hero:Transform.translation"),
        "removed track should disappear from the sequence session"
    );
    assert!(
        pane.selection_summary.is_empty(),
        "removing the selected track should clear the stale timeline selection"
    );
    assert!(
        instance.dirty,
        "removing the selected track should mark the document dirty"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Removed animation track Root/Hero:Transform.translation"
    );
}
