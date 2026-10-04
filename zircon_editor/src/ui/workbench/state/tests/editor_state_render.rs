use std::sync::Arc;

use crate::core::gateway::{EditorRuntimeGateway, GatewayError};
use zircon_runtime::scene::{DefaultLevelManager, LevelSystem, World};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use super::*;

#[test]
fn render_submission_borrows_the_viewport_controller() {
    let source = include_str!("../editor_state_render.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(!implementation.contains("clone_for_render()"));
    assert!(implementation.contains("let controller = &self.viewport_controller"));
}

#[test]
fn render_submission_binds_the_extract_to_the_scene_generation() {
    let source = include_str!("../editor_state_render.rs");
    let implementation = source
        .split_once("#[cfg(test)]")
        .map_or(source, |(production, _)| production);

    assert!(implementation.contains("RenderWorldSnapshotHandle::new(scene.world_generation())"));
    assert!(!implementation.contains("RenderWorldSnapshotHandle::new(0)"));
}

#[test]
fn renderer_visible_snapshot_is_only_adopted_through_the_current_scene() {
    let source = include_str!("../editor_state_render.rs");
    let implementation = source
        .split_once("#[cfg(test)]")
        .map_or(source, |(production, _)| production);

    assert!(implementation.contains("with_world(|scene|"));
    assert!(implementation.contains("clear_renderer_visible_spatial_snapshot"));
}

#[test]
fn render_frame_submission_keeps_the_base_scene_when_highlight_delivery_fails() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    let state = EditorState::with_default_selection(level.clone(), UVec2::new(1280, 720));
    state
        .context
        .authoring_gateway()
        .replace(Arc::new(HighlightFailingGateway { level }))
        .expect("install highlight delivery fault gateway");

    let submission = state
        .render_frame_submission()
        .expect("a highlight delivery fault must not discard the base scene frame");

    assert_eq!(
        submission.extract.world.raw(),
        state
            .world
            .with_world(|scene| scene.world_generation())
            .expect("world gateway should be available")
            .expect("authoring world should be bound")
    );
}

struct HighlightFailingGateway {
    level: LevelSystem,
}

impl EditorRuntimeGateway for HighlightFailingGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn session_identity(&self) -> crate::core::gateway::GatewaySessionIdentity {
        crate::core::gateway::GatewaySessionIdentity::detached()
    }

    fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        self.level.with_world(read);
        Ok(())
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        self.level.with_world_mut(write);
        Ok(())
    }

    fn submit_highlight_set(
        &self,
        _set: crate::core::gateway::EditorRuntimeHighlightSet,
    ) -> Result<(), GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.editor_overlay.highlight_set",
        })
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}
