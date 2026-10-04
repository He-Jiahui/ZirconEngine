use crate::core::framework::render::{FrameHistoryInvalidationReason, RenderFrameHistoryInput};

use super::{spatial_history_reset_reason, SceneHistoryResetReason};

#[test]
fn recreated_history_returns_initialization_commands_for_the_scene_packet() {
    let source = include_str!("../prepare_history_textures.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("new_with_requirements_and_initialization("));
    assert!(production.contains("history_initialization_command_buffer"));
    assert!(!production.contains("record_pre_scene_submission("));
    assert!(!production.contains("RenderFrameSubmissionProducer::HistoryInitialization"));
    assert!(!production.contains("RenderFrameSubmissionTransaction"));
}

#[test]
fn preparation_uses_domain_transactions_instead_of_global_history_validity() {
    let source = include_str!("../prepare_history_textures.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("SceneHistoryFrameTransaction"));
    assert!(production.contains("invalidate_spatial"));
    assert!(production.contains("SceneHistoryDomain::Exposure"));
    assert!(!production.contains("fn history_is_available"));
}

#[test]
fn spatial_history_reset_preserves_the_shared_camera_cut_reason() {
    let camera_cut =
        RenderFrameHistoryInput::new(None, false, Some(FrameHistoryInvalidationReason::CameraCut));
    let structural_change = RenderFrameHistoryInput::new(
        None,
        false,
        Some(FrameHistoryInvalidationReason::FrameInputsChanged),
    );
    let unavailable = RenderFrameHistoryInput::new(None, false, None);
    let available = RenderFrameHistoryInput::new(None, true, None);

    assert_eq!(
        spatial_history_reset_reason(camera_cut),
        Some(SceneHistoryResetReason::CameraCut)
    );
    assert_eq!(
        spatial_history_reset_reason(unavailable),
        Some(SceneHistoryResetReason::PreviousFrameUnavailable)
    );
    assert_eq!(
        spatial_history_reset_reason(structural_change),
        Some(SceneHistoryResetReason::StructuralCompatibilityChanged)
    );
    assert_eq!(spatial_history_reset_reason(available), None);
}

#[test]
fn empty_requirements_release_persistent_physical_history() {
    let source = include_str!("../prepare_history_textures.rs");

    assert!(source.contains("requirements.is_empty()"));
    assert!(source.contains("history_targets.remove(&handle)"));
}

#[test]
fn spatial_ambient_occlusion_does_not_activate_shared_frame_history() {
    let source = include_str!("../prepare_history_textures.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let requirements = production
        .split("SceneFrameHistoryRequirements::new(")
        .nth(1)
        .and_then(|tail| tail.split(");").next())
        .expect("compiled history requirements");

    assert!(!requirements.contains("ssao_enabled"));
    assert!(production.contains("(SceneHistoryDomain::AmbientOcclusion, false)"));
}

#[test]
fn physical_history_allocation_is_driven_by_compiled_requirements() {
    let source = include_str!("../prepare_history_textures.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("SceneFrameHistoryRequirements::new("));
    assert!(production.contains("new_with_requirements_and_initialization("));
    assert!(!production.contains("new_with_volumetric_history_and_submission("));
}

#[test]
fn occupied_history_is_reconciled_without_whole_aggregate_replacement() {
    let source = include_str!("../prepare_history_textures.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let occupied = production
        .split("std::collections::hash_map::Entry::Occupied")
        .nth(1)
        .and_then(|tail| {
            tail.split("std::collections::hash_map::Entry::Vacant")
                .next()
        })
        .expect("occupied history branch");

    assert!(occupied.contains("reconcile_with_requirements_and_initialization("));
    assert!(!occupied.contains("*history = replacement"));
    assert!(production.contains("allocation_changes.changed(domain)"));
    assert!(production.contains("SceneHistoryResetReason::AllocationChanged"));
}
