//! Exercise the narrow property adapter through actual v2/Surface consumers.

use std::sync::Arc;

use crate::ui::asset_editor::{
    preview::preview_host::visibility::{
        UiPreviewVisibilityError as Error, UiPreviewVisibilityOutcome as Outcome,
        UiPreviewVisibilityPublication as Publication,
    },
    UiAssetEditorCommand, UiAssetEditorSession, UiAssetPreviewPreset, UiDesignerToolMode,
};
use zircon_runtime::ui::surface::UiPropertyMutationStatus;
use zircon_runtime_interface::ui::{
    surface::UiSurfaceFrame,
    tree::UiTreeNode,
    window::{UiWindowEvent, UiWindowInputPumpEvent},
};

use super::support::open_design_session;

const URI: &str = "asset://ui/visibility_probe.zui";
const SOURCE: &str = r#"
[asset]
kind = "view"
id = "ui.test.visibility_probe"
version = 2

[root]
node = "root"

[nodes.root]
component = "Panel"
children = [{ node = "child" }]

[nodes.child]
component = "Label"
props = { text = "实际属性 / Visible label" }
"#;

fn session(source: &str) -> UiAssetEditorSession {
    let session = open_design_session(URI, source);
    assert!(
        session.diagnostics().is_empty(),
        "{:?}",
        session.diagnostics()
    );
    session
}

fn node(session: &UiAssetEditorSession) -> &UiTreeNode {
    session
        .preview_host()
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata.as_ref().is_some_and(|metadata| {
                metadata.source_path.as_deref() == Some(URI)
                    && metadata.source_node_id.as_deref() == Some("child")
            })
        })
        .expect("the actual source URI and node must survive compilation")
}

fn frame(session: &UiAssetEditorSession) -> Arc<UiSurfaceFrame> {
    session.preview_host().surface().surface_frame()
}

fn accepted(outcome: Outcome) -> Arc<UiSurfaceFrame> {
    match outcome {
        Outcome::Reported {
            report,
            publication: Publication::Published(frame),
        } => {
            assert_eq!(report.status, UiPropertyMutationStatus::Accepted);
            assert!(report.binding.applied_count > 0);
            assert!(report.invalidation.dirty.render);
            frame
        }
        other => panic!("expected an actual mutation and frame: {other:?}"),
    }
}

#[test]
fn visibility_mutates_real_state_arranged_frame_and_render_without_source_history() {
    let mut session = session(SOURCE);
    let original = session.source_buffer().text().to_string();
    let original_dirty = session.source_buffer().is_dirty();
    let original_undo = session.can_undo();
    let original_redo = session.can_redo();
    let gesture = session
        .begin_preview_visibility("child")
        .expect("native target");
    assert_eq!(gesture.source().document_uri, URI);
    assert_eq!(gesture.source().asset_id, "ui.test.visibility_probe");
    assert!(gesture.target().instance_path.is_empty());
    let initial = frame(&session);
    let id = node(&session).node_id;
    assert!(initial.arranged_tree.get(id).unwrap().is_render_visible());
    assert!(initial
        .render_extract
        .list
        .commands
        .iter()
        .any(|command| command.node_id == id));

    let hidden = accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    assert!(!node(&session).state_flags.visible);
    assert!(!hidden.arranged_tree.get(id).unwrap().is_render_visible());
    assert!(!hidden
        .render_extract
        .list
        .commands
        .iter()
        .any(|command| command.node_id == id));
    assert!(hidden.generation > initial.generation);
    assert!(Arc::ptr_eq(&hidden, &frame(&session)));
    assert_eq!(session.source_buffer().text(), original);
    assert_eq!(session.source_buffer().is_dirty(), original_dirty);
    assert_eq!(session.can_undo(), original_undo);
    assert_eq!(session.can_redo(), original_redo);
}

#[test]
fn cancel_restores_initial_value_and_explicit_or_absent_attribute() {
    for source in [
        SOURCE.to_string(),
        SOURCE.replace("props = { text", "props = { visible = true, text"),
    ] {
        let mut session = session(&source);
        let before = node(&session)
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("visible")
            .cloned();
        let gesture = session.begin_preview_visibility("child").unwrap();
        let initial = frame(&session);
        let hidden = accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
        let restored = accepted(session.cancel_preview_visibility(&gesture, &hidden, 2));
        assert!(node(&session).state_flags.visible);
        assert_eq!(
            node(&session)
                .template_metadata
                .as_ref()
                .unwrap()
                .attributes
                .get("visible"),
            before.as_ref()
        );
        assert!(Arc::ptr_eq(&restored, &frame(&session)));
        assert!(matches!(
            session.apply_preview_visibility(&gesture, &restored, 3, false),
            Outcome::Rejected(Error::RetiredGesture)
        ));
        assert_eq!(session.source_buffer().text(), source);
        assert!(!session.can_undo());
    }
}

#[test]
fn unchanged_report_does_not_claim_a_new_frame() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    match session.apply_preview_visibility(&gesture, &initial, 1, true) {
        Outcome::Reported {
            report,
            publication: Publication::NotRequested,
        } => {
            assert_eq!(report.status, UiPropertyMutationStatus::Unchanged);
        }
        other => panic!("expected the actual unchanged report: {other:?}"),
    }
    assert!(Arc::ptr_eq(&initial, &frame(&session)));
    assert!(!session.can_undo());
}

#[test]
fn duplicate_sequence_and_old_frame_do_not_write_or_retire_valid_request() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    let hidden = accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    assert!(matches!(
        session.apply_preview_visibility(&gesture, &hidden, 1, true),
        Outcome::Rejected(Error::StaleRequest)
    ));
    assert!(matches!(
        session.apply_preview_visibility(&gesture, &initial, 2, true),
        Outcome::Rejected(Error::StaleFrame)
    ));
    assert!(!node(&session).state_flags.visible);
    accepted(session.apply_preview_visibility(&gesture, &hidden, 2, true));
    assert!(node(&session).state_flags.visible);
}

#[test]
fn another_session_with_identical_source_cannot_consume_the_gesture() {
    let mut original = session(SOURCE);
    let gesture = original.begin_preview_visibility("child").unwrap();
    let mut other = session(SOURCE);
    let other_frame = frame(&other);
    assert!(matches!(
        other.apply_preview_visibility(&gesture, &other_frame, 1, false),
        Outcome::Rejected(Error::RetiredGesture)
    ));
    assert!(node(&other).state_flags.visible);
    let original_frame = frame(&original);
    accepted(original.apply_preview_visibility(&gesture, &original_frame, 1, false));
}

#[test]
fn source_edit_and_undo_never_restore_an_old_baseline_into_current_source() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    let changed = SOURCE.replace("props = { text", "props = { visible = false, text");
    session
        .apply_command(UiAssetEditorCommand::edit_source(&changed))
        .unwrap();
    let changed_frame = frame(&session);
    assert!(matches!(
        session.cancel_preview_visibility(&gesture, &changed_frame, 2),
        Outcome::Rejected(Error::StaleSource)
    ));
    assert!(!node(&session).state_flags.visible);
    assert_eq!(session.source_buffer().text(), changed);
    assert!(session.undo().unwrap());
    assert!(node(&session).state_flags.visible);
    let undone = frame(&session);
    assert!(matches!(
        session.apply_preview_visibility(&gesture, &undone, 3, false),
        Outcome::Rejected(Error::StaleSource)
    ));
    assert!(node(&session).state_flags.visible);
}

#[test]
fn reset_rebuilds_same_source_and_retires_old_gesture_even_with_same_numeric_generation() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    session.reset_preview_visibility().unwrap();
    let reset = frame(&session);
    assert_eq!(initial.tree_id, reset.tree_id);
    assert_eq!(initial.generation, reset.generation);
    assert!(!Arc::ptr_eq(&initial, &reset));
    assert!(node(&session).state_flags.visible);
    assert!(matches!(
        session.apply_preview_visibility(&gesture, &reset, 2, false),
        Outcome::Rejected(Error::RetiredGesture)
    ));
    assert!(!session.can_undo());
}

#[test]
fn preset_rebuild_and_invalid_source_retire_previous_requests() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    assert!(session
        .set_preview_preset(UiAssetPreviewPreset::GameHud)
        .unwrap());
    let resized = frame(&session);
    assert!(matches!(
        session.cancel_preview_visibility(&gesture, &resized, 2),
        Outcome::Rejected(Error::RetiredGesture)
    ));
    let next = session.begin_preview_visibility("child").unwrap();
    session
        .apply_command(UiAssetEditorCommand::edit_source("[asset\n"))
        .unwrap();
    assert!(!session.diagnostics().is_empty());
    let last_valid = frame(&session);
    assert!(matches!(
        session.apply_preview_visibility(&next, &last_valid, 1, false),
        Outcome::Rejected(Error::InvalidSource)
    ));
}

#[test]
fn root_interactive_and_bound_documents_are_rejected_without_mutation() {
    let mut plain = session(SOURCE);
    assert!(matches!(
        plain.begin_preview_visibility("root"),
        Err(Error::UnsupportedTarget)
    ));
    assert!(matches!(
        plain.begin_preview_visibility("missing"),
        Err(Error::UnsupportedTarget)
    ));
    for source in [
        SOURCE.replace("component = \"Label\"", "component = \"Button\""),
        format!(
            "{SOURCE}\nevents = [{{ id = \"probe.click\", event = \"Click\", route = \"probe.invoke\" }}]\n"
        ),
    ] {
        let mut session = session(&source);
        assert!(matches!(
            session.begin_preview_visibility("child"),
            Err(Error::UnsupportedDocument)
        ));
        assert!(node(&session).state_flags.visible);
        assert_eq!(session.source_buffer().text(), source);
        assert!(!session.can_undo());
    }
}

#[test]
fn actual_window_input_retires_gesture_and_reset_restores_current_source() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    let hidden = accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    session.set_designer_tool_mode(UiDesignerToolMode::PreviewInteract);
    session
        .dispatch_preview_input(
            &hidden,
            UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(
                Default::default(),
                false,
            )),
        )
        .unwrap();
    let after_input = frame(&session);
    assert!(matches!(
        session.cancel_preview_visibility(&gesture, &after_input, 2),
        Outcome::Rejected(Error::RetiredGesture)
    ));
    assert!(!node(&session).state_flags.visible);
    session.reset_preview_visibility().unwrap();
    assert!(node(&session).state_flags.visible);
    assert_eq!(session.source_buffer().text(), SOURCE);
    assert!(!session.can_undo());
}

fn visibility_channel() -> zircon_runtime::core::framework::animation::AnimationChannelAsset {
    use zircon_runtime::core::framework::animation::{
        AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
        AnimationInterpolationAsset,
    };
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: [(0.0, true), (1.0, false), (2.0, false)]
            .into_iter()
            .map(|(time_seconds, visible)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Bool(visible),
                in_tangent: None,
                out_tangent: None,
            })
            .collect(),
    }
}

#[test]
fn public_channel_sample_uses_existing_step_boundaries_and_real_surface_outcome() {
    let mut session = session(SOURCE);
    let channel = visibility_channel();
    let before = channel.clone();
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    let held = session
        .sample_preview_visibility(&gesture, &initial, 1, &channel, 1.0)
        .unwrap();
    assert_eq!(held.requested_seconds, 1.0);
    assert!(held.sampled_visible); // Exact interior key holds the preceding interval.
    assert!(matches!(
        held.outcome,
        Outcome::Reported {
            report,
            publication: Publication::NotRequested
        } if report.status == UiPropertyMutationStatus::Unchanged
    ));
    assert!(Arc::ptr_eq(&initial, &frame(&session)));
    let sample = session
        .sample_preview_visibility(&gesture, &initial, 2, &channel, 1.25)
        .unwrap();
    assert_eq!(sample.requested_seconds, 1.25);
    assert!(!sample.sampled_visible);
    let hidden = accepted(sample.outcome);
    assert!(!node(&session).state_flags.visible);
    assert!(!hidden
        .arranged_tree
        .get(node(&session).node_id)
        .unwrap()
        .is_render_visible());
    assert_eq!(channel, before);
    assert_eq!(session.source_buffer().text(), SOURCE);
    assert!(!session.source_buffer().is_dirty());
    assert!(!session.can_undo());
}

#[test]
fn rejected_channel_or_nonfinite_time_never_writes_or_consumes_valid_request() {
    use zircon_runtime::core::framework::animation::AnimationInterpolationAsset;
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    let channel = visibility_channel();
    let mut invalid = channel.clone();
    invalid.interpolation = AnimationInterpolationAsset::Linear;
    for (candidate, seconds) in [
        (&invalid, 0.5),
        (&channel, zircon_runtime::core::math::Real::NAN),
    ] {
        assert!(matches!(
            session.sample_preview_visibility(&gesture, &initial, 1, candidate, seconds),
            Err(Error::UnsupportedChannelSample)
        ));
        assert!(node(&session).state_flags.visible);
        assert!(Arc::ptr_eq(&initial, &frame(&session)));
    }
    let sample = session
        .sample_preview_visibility(&gesture, &initial, 1, &channel, 2.0)
        .unwrap();
    accepted(sample.outcome);
    assert!(!node(&session).state_flags.visible);
}

#[test]
fn finish_retains_value_releases_lease_and_old_completion_cannot_retire_new_gesture() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    let hidden = accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    assert!(matches!(
        session.begin_preview_visibility("child"),
        Err(Error::GestureInProgress)
    ));
    let finished = session
        .finish_preview_visibility(&gesture, &hidden, 2)
        .unwrap();
    assert!(Arc::ptr_eq(&finished, &hidden));
    assert!(!node(&session).state_flags.visible);
    let next = session.begin_preview_visibility("child").unwrap();
    assert!(matches!(
        session.finish_preview_visibility(&gesture, &hidden, 3),
        Err(Error::RetiredGesture)
    ));
    assert!(matches!(
        session.cancel_preview_visibility(&next, &hidden, 1),
        Outcome::Reported {
            report,
            publication: Publication::NotRequested
        } if report.status == UiPropertyMutationStatus::Unchanged
    ));
    assert!(!node(&session).state_flags.visible);
    assert_eq!(session.source_buffer().text(), SOURCE);
    assert!(!session.can_undo());
}

#[test]
fn finish_rejects_old_frame_and_sequence_without_retiring_the_current_lease() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    let hidden = accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    assert!(matches!(
        session.finish_preview_visibility(&gesture, &initial, 2),
        Err(Error::StaleFrame)
    ));
    assert!(matches!(
        session.finish_preview_visibility(&gesture, &hidden, 1),
        Err(Error::StaleRequest)
    ));
    let finished = session
        .finish_preview_visibility(&gesture, &hidden, 2)
        .unwrap();
    assert!(Arc::ptr_eq(&hidden, &finished));
}

#[test]
fn sampled_or_finished_old_source_never_writes_into_the_new_source_or_retires_its_lease() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let changed = SOURCE.replace("props = { text", "props = { visible = false, text");
    session
        .apply_command(UiAssetEditorCommand::edit_source(&changed))
        .unwrap();
    let current = frame(&session);
    let next = session.begin_preview_visibility("child").unwrap();
    assert!(matches!(
        session.sample_preview_visibility(&gesture, &current, 1, &visibility_channel(), 0.0),
        Err(Error::StaleSource)
    ));
    assert!(matches!(
        session.finish_preview_visibility(&gesture, &current, 2),
        Err(Error::StaleSource)
    ));
    assert!(!node(&session).state_flags.visible);
    accepted(session.apply_preview_visibility(&next, &current, 1, true));
    assert_eq!(session.source_buffer().text(), changed);
}

#[test]
fn cancel_before_deactivation_restores_but_deactivation_first_requires_current_source_reset() {
    let mut session = session(SOURCE);
    let gesture = session.begin_preview_visibility("child").unwrap();
    let initial = frame(&session);
    let hidden = accepted(session.apply_preview_visibility(&gesture, &initial, 1, false));
    accepted(session.cancel_preview_visibility(&gesture, &hidden, 2));
    session
        .deactivate_preview_input(Default::default())
        .unwrap();
    assert!(node(&session).state_flags.visible);

    let next = session.begin_preview_visibility("child").unwrap();
    let current = frame(&session);
    accepted(session.apply_preview_visibility(&next, &current, 1, false));
    session
        .deactivate_preview_input(Default::default())
        .unwrap();
    let deactivated = frame(&session);
    assert!(matches!(
        session.cancel_preview_visibility(&next, &deactivated, 2),
        Outcome::Rejected(Error::RetiredGesture)
    ));
    assert!(matches!(
        session.finish_preview_visibility(&next, &deactivated, 3),
        Err(Error::RetiredGesture)
    ));
    assert!(!node(&session).state_flags.visible);
    session.reset_preview_visibility().unwrap();
    assert!(node(&session).state_flags.visible);
    assert_eq!(session.source_buffer().text(), SOURCE);
    assert!(!session.can_undo());
}
