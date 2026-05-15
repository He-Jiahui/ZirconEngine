use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiSize},
    surface::UiSurfaceFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use crate::ui::binding::EditorUiBinding;
use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiProjection,
};

#[cfg(test)]
use super::super::projection_support::load_builtin_runtime;
use super::super::projection_support::{
    binding_for_control, build_bindings_by_id, project_builtin_document_with_runtime,
};
use super::error::BuiltinViewportToolbarTemplateBridgeError;
use super::host_projection::{
    build_builtin_viewport_toolbar_surface, project_builtin_viewport_toolbar_host_projection,
    rebuild_builtin_viewport_toolbar_surface,
};

pub(crate) struct BuiltinViewportToolbarTemplateBridge {
    runtime: Arc<EditorUiHostRuntime>,
    projection: RetainedUiProjection,
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    surface: UiSurface,
    host_projection: RetainedUiHostProjection,
}

impl BuiltinViewportToolbarTemplateBridge {
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, BuiltinViewportToolbarTemplateBridgeError> {
        let runtime = Arc::new(load_builtin_runtime()?);
        Self::new_with_runtime(runtime)
    }

    pub(crate) fn new_with_runtime(
        runtime: Arc<EditorUiHostRuntime>,
    ) -> Result<Self, BuiltinViewportToolbarTemplateBridgeError> {
        let projection =
            project_builtin_document_with_runtime(&runtime, BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
        let bindings_by_id = build_bindings_by_id(&projection);
        let surface =
            build_builtin_viewport_toolbar_surface(runtime.as_ref(), UiSize::new(1280.0, 28.0))?;
        let host_projection = project_builtin_viewport_toolbar_host_projection(
            runtime.as_ref(),
            &projection,
            &surface,
        )?;
        Ok(Self {
            runtime,
            projection,
            bindings_by_id,
            surface,
            host_projection,
        })
    }

    pub(crate) fn recompute_layout(
        &mut self,
        surface_size: UiSize,
    ) -> Result<(), BuiltinViewportToolbarTemplateBridgeError> {
        rebuild_builtin_viewport_toolbar_surface(&mut self.surface, surface_size)?;
        self.host_projection = project_builtin_viewport_toolbar_host_projection(
            self.runtime.as_ref(),
            &self.projection,
            &self.surface,
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

    pub(crate) fn control_frame_for_control(&self, control_id: &str) -> Option<UiFrame> {
        self.host_projection
            .node_by_control_id(control_id)
            .map(|node| node.frame)
    }

    pub(crate) fn surface_frame_for_projection_controls<F>(
        &self,
        surface_key: &str,
        surface_size: UiSize,
        mut hit_control_id: F,
    ) -> UiSurfaceFrame
    where
        F: FnMut(&str) -> Option<String>,
    {
        let mut surface = UiSurface::new(UiTreeId::new(format!(
            "zircon.editor.viewport_toolbar.{surface_key}"
        )));
        let root_frame = UiFrame::new(
            0.0,
            0.0,
            surface_size.width.max(1.0),
            surface_size.height.max(1.0),
        );
        let mut root = UiTreeNode::new(
            UiNodeId::new(1),
            UiNodePath::new(format!("viewport_toolbar/{surface_key}/root")),
        )
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
        root.layout_cache.clip_frame = Some(root_frame);
        surface.tree.insert_root(root);

        let mut next_node_id = 2;
        for projection_node in &self.host_projection.nodes {
            let Some(projection_control_id) = projection_node.control_id.as_deref() else {
                continue;
            };
            if projection_node.routes.is_empty() || projection_node.disabled {
                continue;
            }
            let Some(control_id) = hit_control_id(projection_control_id) else {
                continue;
            };
            let mut metadata = UiTemplateNodeMetadata {
                component: projection_node.component.clone(),
                control_id: Some(control_id.clone()),
                ..Default::default()
            };
            metadata.attributes.insert(
                "source".to_string(),
                toml::Value::String("viewport_toolbar".to_string()),
            );
            metadata.attributes.insert(
                "projection_control_id".to_string(),
                toml::Value::String(projection_control_id.to_string()),
            );
            let node = UiTreeNode::new(
                UiNodeId::new(next_node_id),
                UiNodePath::new(format!(
                    "viewport_toolbar/{surface_key}/{projection_control_id}"
                )),
            )
            .with_frame(projection_node.frame)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: true,
                pressed: false,
                checked: false,
                dirty: false,
            })
            .with_input_policy(UiInputPolicy::Receive)
            .with_template_metadata(metadata);
            let _ = surface.tree.insert_child(UiNodeId::new(1), node);
            next_node_id += 1;
        }

        surface.rebuild();
        surface.surface_frame()
    }
}
