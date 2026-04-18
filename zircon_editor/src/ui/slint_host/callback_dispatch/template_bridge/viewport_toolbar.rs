use std::collections::BTreeMap;

use thiserror::Error;
use crate::ui::EditorUiBinding;
use zircon_ui::{UiEventKind, UiFrame, UiSize};

use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, SlintUiHostProjection, SlintUiProjection,
};

use super::{binding_for_control, build_bindings_by_id, load_builtin_runtime_projection};

#[derive(Debug, Error)]
pub(crate) enum BuiltinViewportToolbarTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    Layout(#[from] zircon_ui::UiTreeError),
}

pub(crate) struct BuiltinViewportToolbarTemplateBridge {
    runtime: EditorUiHostRuntime,
    projection: SlintUiProjection,
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: SlintUiHostProjection,
}

impl BuiltinViewportToolbarTemplateBridge {
    pub(crate) fn new() -> Result<Self, BuiltinViewportToolbarTemplateBridgeError> {
        let (runtime, projection) =
            load_builtin_runtime_projection(BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
        let bindings_by_id = build_bindings_by_id(&projection);
        let host_projection = build_builtin_viewport_toolbar_host_projection(
            &runtime,
            &projection,
            UiSize::new(1280.0, 28.0),
        )?;
        Ok(Self {
            runtime,
            projection,
            bindings_by_id,
            host_projection,
        })
    }

    pub(crate) fn recompute_layout(
        &mut self,
        surface_size: UiSize,
    ) -> Result<(), BuiltinViewportToolbarTemplateBridgeError> {
        self.host_projection = build_builtin_viewport_toolbar_host_projection(
            &self.runtime,
            &self.projection,
            surface_size,
        )?;
        Ok(())
    }

    pub(crate) fn binding_for_control(
        &self,
        control_id: &str,
        event_kind: UiEventKind,
    ) -> Option<&EditorUiBinding> {
        binding_for_control(
            &self.bindings_by_id,
            &self.host_projection,
            control_id,
            event_kind,
        )
    }

    pub(crate) fn control_frame_for_action(&self, control_id: &str) -> Option<UiFrame> {
        let projection_control_id = projection_control_for_action(control_id)?;
        self.host_projection
            .node_by_control_id(projection_control_id)
            .map(|node| node.frame)
    }
}

fn build_builtin_viewport_toolbar_host_projection(
    runtime: &EditorUiHostRuntime,
    projection: &SlintUiProjection,
    surface_size: UiSize,
) -> Result<SlintUiHostProjection, BuiltinViewportToolbarTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
    surface.compute_layout(surface_size)?;
    Ok(runtime.build_slint_host_projection_with_surface(projection, &surface)?)
}

fn projection_control_for_action(control_id: &str) -> Option<&'static str> {
    match control_id {
        "tool.drag" | "tool.move" | "tool.rotate" | "tool.scale" => Some("SetTool"),
        "space.local" | "space.global" | "transform.local" | "transform.global" => {
            Some("SetTransformSpace")
        }
        "projection.perspective" | "projection.orthographic" => Some("SetProjectionMode"),
        "align.pos_x" | "align.neg_x" | "align.pos_y" | "align.neg_y" | "align.pos_z"
        | "align.neg_z" => Some("AlignView"),
        "display.cycle" => Some("SetDisplayMode"),
        "grid.cycle" => Some("SetGridMode"),
        "snap.translate" | "translate_snap.cycle" => Some("SetTranslateSnap"),
        "snap.rotate" | "rotate_snap.cycle" => Some("SetRotateSnapDegrees"),
        "snap.scale" | "scale_snap.cycle" => Some("SetScaleSnap"),
        "toggle.lighting" | "preview_lighting.toggle" => Some("SetPreviewLighting"),
        "toggle.skybox" | "preview_skybox.toggle" => Some("SetPreviewSkybox"),
        "toggle.gizmos" | "gizmos.toggle" => Some("SetGizmosEnabled"),
        "frame.selection" | "frame_selection" => Some("FrameSelection"),
        _ => None,
    }
}
