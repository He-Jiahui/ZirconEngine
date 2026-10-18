use zircon_runtime::ui::v2::UiV2SourceFileReceipt;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiSize};

use crate::ui::retained_host::callback_dispatch::template_bridge::projection_support::build_bindings_by_id;

use super::super::{
    error::BuiltinHostWindowTemplateBridgeError,
    extension_module_navigation::{
        workbench_extension_workspace_control_id, workbench_extension_workspace_document,
    },
    module_navigation::{
        workbench_module_workspace_control_id, workbench_module_workspace_document,
    },
    responsive_layout::apply_workbench_responsive_layout,
};
use super::BuiltinWorkbenchWindowTemplateSurfaceBridge;

pub(super) struct MountedModuleWorkspace {
    pub(super) action_id: String,
    pub(super) root_id: UiNodeId,
    pub(super) workspace_control_id: &'static str,
    // Keep the selected asset closure's receipt identity beside its mounted tree.
    pub(super) _source_receipts: Vec<UiV2SourceFileReceipt>,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    #[cfg(test)]
    pub(crate) fn mounted_module_source_receipts_for_test(&self) -> Vec<UiV2SourceFileReceipt> {
        self.module_workspace
            .as_ref()
            .map(|workspace| workspace._source_receipts.clone())
            .unwrap_or_default()
    }

    /// Use the committed selected root and its actual source receipts, including unavailable nodes.
    pub(in super::super) fn mounted_module_root_for_source_publication(
        &self,
    ) -> Result<Option<(UiNodeId, &'static str)>, BuiltinHostWindowTemplateBridgeError> {
        let Some(mounted) = self.module_workspace.as_ref() else {
            return Ok(None);
        };
        let invalid = || {
            crate::ui::workbench::reference::EditorWorkbenchTemplateSurfaceError::MissingControl {
                control_id: mounted.workspace_control_id,
            }
        };
        let root = self
            .surface()
            .tree
            .node(mounted.root_id)
            .ok_or_else(invalid)?;
        let metadata = root.template_metadata.as_ref().ok_or_else(invalid)?;
        if self.control_node_id(mounted.workspace_control_id) != Some(mounted.root_id)
            || metadata.control_id.as_deref() != Some(mounted.workspace_control_id)
            || root.parent != self.control_node_id("UnrealSceneViewViewport")
            || root.parent.is_none()
            || !mounted._source_receipts.iter().any(|receipt| {
                receipt.source_path.as_deref().is_some()
                    && receipt.source_path.as_deref() == metadata.source_path.as_deref()
            })
        {
            return Err(invalid().into());
        }
        Ok(Some((mounted.root_id, mounted.workspace_control_id)))
    }

    pub(super) fn mount_workbench_module_workspace(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if !self.preview_features_enabled && action_id != "workbench.module.scene.select" {
            return self.show_unimplemented_preview();
        }
        if self
            .module_workspace
            .as_ref()
            .is_some_and(|mounted| mounted.action_id == action_id)
        {
            return Ok(());
        }
        let workspace_control_id = workbench_module_workspace_control_id(action_id)
            .or_else(|| workbench_extension_workspace_control_id(action_id));
        let incoming = if let Some((document_id, component_id)) =
            workbench_module_workspace_document(action_id)
                .or_else(|| workbench_extension_workspace_document(action_id))
        {
            let (projection, surface, receipts) = self
                .runtime
                .project_workbench_module_workspace(document_id, component_id)?;
            if projection.root.control_id.as_deref() != workspace_control_id {
                return Err(crate::ui::workbench::reference::EditorWorkbenchTemplateSurfaceError::MissingControl {
                    control_id: workspace_control_id.unwrap_or(component_id),
                }.into());
            }
            Some((projection, surface, receipts))
        } else if action_id == "workbench.module.scene.select" {
            if self.module_workspace.is_none() {
                return Ok(());
            }
            None
        } else {
            return Ok(());
        };
        let previous_root = self
            .module_workspace
            .as_ref()
            .map(|mounted| mounted.root_id);
        let previous_control = self
            .module_workspace
            .as_ref()
            .map(|mounted| mounted.workspace_control_id);
        let (incoming, receipts) = match incoming {
            Some((projection, surface, receipts)) => (Some((projection, surface)), receipts),
            None => (None, Vec::new()),
        };
        let (staged, root_id, proof) = self.template_surface.stage_module_workspace(
            self.runtime.as_ref(),
            previous_root,
            previous_control,
            incoming,
        )?;
        let retired = proof.retired_nodes();
        let bindings = build_bindings_by_id(staged.source_projection());
        let mut previous_surface = std::mem::replace(&mut self.template_surface, staged);
        let previous_bindings = std::mem::replace(&mut self.bindings_by_id, bindings);
        let previous_tooltip_input = std::mem::take(&mut self.icon_tooltip_input);
        let prepared = (|| {
            self.dismiss_workbench_icon_tooltip()?;
            match action_id {
                "workbench.module.effect.select" => self.initialize_effect_workspace_state()?,
                "workbench.module.ability.select" => self.initialize_ability_workspace_state()?,
                "workbench.module.tags.select" => self.initialize_tags_workspace_state()?,
                "workbench.module.perception.select" => {
                    self.initialize_perception_workspace_state()?
                }
                "workbench.module.material.select" => self.initialize_material_workspace_state()?,
                "workbench.module.behavior.select" => self.initialize_behavior_workspace_state()?,
                "workbench.module.render.select" => self.initialize_render_workspace_state()?,
                "workbench.module.assets.select" => self.initialize_assets_workspace_state()?,
                "workbench.module.vfx.select" => self.initialize_vfx_workspace_state()?,
                "workbench.module.hud.select" => self.initialize_hud_workspace_state()?,
                _ => {}
            }
            if let Some(control_id) = workspace_control_id {
                self.set_visible(control_id, true)?;
            }
            let scene_active = root_id.is_none();
            self.set_visible("WorkbenchViewportSurface", scene_active)?;
            self.set_visible("UnrealViewportToolbar", scene_active)?;
            apply_workbench_responsive_layout(
                &mut self.template_surface.surface,
                UiSize::new(self.mount_frame.width, self.mount_frame.height),
                self.presentation_scale_factor,
                self.compact_module_details_drawer_open,
            )?;
            self.template_surface
                .refresh_after_state_change(self.runtime.as_ref())?;
            Ok::<(), BuiltinHostWindowTemplateBridgeError>(())
        })();
        if let Err(error) = prepared {
            self.template_surface = previous_surface;
            self.bindings_by_id = previous_bindings;
            self.icon_tooltip_input = previous_tooltip_input;
            return Err(error);
        }
        let retired_controls = retired
            .iter()
            .filter_map(|id| {
                previous_surface
                    .surface
                    .tree
                    .node(*id)?
                    .template_metadata
                    .as_ref()?
                    .control_id
                    .clone()
            })
            .collect::<std::collections::BTreeSet<_>>();
        if let Err(error) = self
            .template_surface
            .commit_module_workspace(&mut previous_surface, &proof)
        {
            self.template_surface = previous_surface;
            self.bindings_by_id = previous_bindings;
            self.icon_tooltip_input = previous_tooltip_input;
            return Err(error.into());
        }
        if self
            .content_drawer_native_previous_focus
            .as_ref()
            .is_some_and(|focus| retired_controls.contains(focus.control_id.as_str()))
        {
            self.content_drawer_native_previous_focus = None;
        }
        if self
            .content_drawer_previous_focus
            .is_some_and(|id| retired.contains(&id))
        {
            self.content_drawer_previous_focus = None;
        }
        self.module_workspace = root_id.map(|root_id| MountedModuleWorkspace {
            action_id: action_id.to_string(),
            root_id,
            workspace_control_id: workspace_control_id
                .expect("mounted workspace has a validated root control"),
            _source_receipts: receipts,
        });
        Ok(())
    }

    pub(in super::super) fn workbench_module_is_selected(&self, control_id: &str) -> bool {
        let action_id = self
            .module_workspace
            .as_ref()
            .map(|mounted| mounted.action_id.as_str())
            .unwrap_or("workbench.module.scene.select");
        super::super::module_navigation::workbench_module_tab_control_id(action_id)
            == Some(control_id)
    }
}
