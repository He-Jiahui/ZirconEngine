use super::*;

#[test]
fn animation_rebind_to_existing_track_keeps_original_sequence_tracks_intact() {
    let _guard = env_lock().lock().unwrap();
    let mut harness = EventRuntimeHarness::new("zircon_editor_event_animation_rebind_duplicate");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_rebind_duplicate_project",
        "res://animation/hero.sequence.zranim",
        write_sequence_asset_with_multiple_tracks,
    );

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset { asset_locator }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_binding(
            EditorUiBinding::new(
                "AnimationSequenceEditorView",
                "RebindTrackButton",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::animation_command(AnimationCommand::RebindTrack {
                    from_track_path: "Root/Hero:AnimationPlayer.weight".to_string(),
                    to_track_path: "Root/Hero:Transform.translation".to_string(),
                }),
            ),
            EditorEventSource::Headless,
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
        .expect("sequence session should remain queryable after duplicate rebind");

    assert_eq!(
        pane.track_items,
        vec![
            "Root/Hero:AnimationPlayer.weight".to_string(),
            "Root/Hero:Transform.translation".to_string(),
        ],
        "duplicate rebind should not delete the original source track"
    );
    assert!(
        !instance.dirty,
        "duplicate rebind should remain a no-op instead of marking the document dirty"
    );
}

#[test]
fn animation_rebind_updates_selected_timeline_span_to_new_track_path() {
    let _guard = env_lock().lock().unwrap();
    let mut harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_rebind_updates_selection");
    let asset_locator = open_indexed_animation_asset(
        &mut harness,
        "zircon_editor_event_animation_rebind_updates_selection_project",
        "res://animation/hero.sequence.zranim",
        write_sequence_asset_with_multiple_tracks,
    );

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset { asset_locator }),
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
                            "Root/Hero:AnimationPlayer.weight",
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
                crate::core::editor_event::EditorAnimationEvent::RebindTrack {
                    from_track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:AnimationPlayer.weight",
                        )
                        .unwrap(),
                    to_track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:Transform.rotation",
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
        .expect("sequence session should remain queryable after rebind");

    assert!(
        pane.track_items
            .iter()
            .any(|item| item == "Root/Hero:Transform.rotation"),
        "rebound track should appear under the destination track path"
    );
    assert_eq!(
        pane.selection_summary, "Root/Hero:Transform.rotation [24..48]",
        "rebind should migrate the selected timeline span to the destination path"
    );
    assert!(
        instance.dirty,
        "successful rebind should mark the document dirty"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Rebound animation track Root/Hero:AnimationPlayer.weight -> Root/Hero:Transform.rotation"
    );
}
