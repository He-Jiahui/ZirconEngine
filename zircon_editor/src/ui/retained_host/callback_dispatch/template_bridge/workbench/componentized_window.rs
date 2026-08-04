use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime::ui::surface::{UiPropertyMutationRequest, UiSurface};
use zircon_runtime::ui::tree::UiRuntimeTreeRoutingExt;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiValue,
    event_ui::UiNodeId,
    layout::{UiFrame, UiSize},
    surface::{UiPointerButton, UiPointerEventKind, UiPointerRoute},
};

use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::TransformHandleKind;
use crate::ui::binding::{
    DockCommand, EditorUiBinding, EditorUiBindingPayload, SelectionCommand, ViewportCommand,
};
use crate::ui::retained_host::callback_dispatch::constants::DOCUMENT_TABS_CONTROL_ID;
use crate::ui::retained_host::callback_dispatch::template_bridge::projection_support::{
    binding_for_control, build_bindings_by_id, project_builtin_document_with_runtime,
};
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, WORKBENCH_WINDOW_DOCUMENT_ID,
};
use crate::ui::workbench::reference::{
    EditorWorkbenchReferenceMetrics, EditorWorkbenchTemplateControlIds,
    EditorWorkbenchTemplateFrames, EditorWorkbenchTemplateSurface,
    build_editor_workbench_template_surface,
};

use super::super::popup_primitives::toml_value_string_list;
#[cfg(test)]
use super::super::projection_support::load_builtin_runtime;
use super::asset_creation_menu::AssetCreationMenuState;
use super::drawer_layout::{
    BOTTOM_DRAWER_CONTENT_CONTROL_ID, BOTTOM_DRAWER_HEADER_CONTROL_ID,
    BOTTOM_DRAWER_SHELL_CONTROL_ID, LEFT_DRAWER_CONTENT_CONTROL_ID, LEFT_DRAWER_HEADER_CONTROL_ID,
    LEFT_DRAWER_SHELL_CONTROL_ID, RIGHT_DRAWER_CONTENT_CONTROL_ID, RIGHT_DRAWER_HEADER_CONTROL_ID,
    RIGHT_DRAWER_SHELL_CONTROL_ID,
};
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::extension_module_navigation::{
    EXTENSION_MODULE_WORKSPACE_CONTROLS, workbench_extension_workspace_control_id,
};
use super::layout_frames::{
    BuiltinWorkbenchWindowLayoutFrames, bottom_resize_splitter_frame_from_drawer_shell,
    left_resize_splitter_frame_from_drawer_shell, right_resize_splitter_frame_from_drawer_shell,
    union_visible_frames,
};
use super::module_navigation::{
    MODULE_TAB_CONTROLS, MODULE_WORKSPACE_CONTROLS, workbench_module_workspace_control_id,
};

mod refresh_layout;

pub(crate) struct BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) runtime: Arc<EditorUiHostRuntime>,
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    pub(super) template_surface: EditorWorkbenchTemplateSurface,
    pub(super) mount_frame: UiFrame,
    pub(super) asset_creation_menu: AssetCreationMenuState,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    #[cfg(test)]
    pub(crate) fn new(shell_size: UiSize) -> Result<Self, BuiltinHostWindowTemplateBridgeError> {
        let runtime = Arc::new(load_builtin_runtime()?);
        Self::new_with_runtime(runtime, shell_size)
    }

    pub(crate) fn new_with_runtime(
        runtime: Arc<EditorUiHostRuntime>,
        shell_size: UiSize,
    ) -> Result<Self, BuiltinHostWindowTemplateBridgeError> {
        Self::new_mounted_with_runtime(
            runtime,
            UiFrame::new(0.0, 0.0, shell_size.width, shell_size.height),
        )
    }

    pub(crate) fn new_mounted_with_runtime(
        runtime: Arc<EditorUiHostRuntime>,
        mount_frame: UiFrame,
    ) -> Result<Self, BuiltinHostWindowTemplateBridgeError> {
        let mount_frame = normalized_mount_frame(mount_frame);
        let shell_size = UiSize::new(mount_frame.width, mount_frame.height);
        let projection =
            project_builtin_document_with_runtime(&runtime, WORKBENCH_WINDOW_DOCUMENT_ID)?;
        let bindings_by_id = build_bindings_by_id(&projection);
        let mut template_surface = build_editor_workbench_template_surface(
            runtime.as_ref(),
            EditorWorkbenchReferenceMetrics::default(),
        )?;
        if shell_size != template_surface.metrics.target_size() {
            template_surface.recompute_layout(runtime.as_ref(), shell_size)?;
        }

        let mut bridge = Self {
            runtime,
            bindings_by_id,
            template_surface,
            mount_frame,
            asset_creation_menu: AssetCreationMenuState::default(),
        };
        bridge.apply_responsive_toolbar_layout(shell_size)?;
        bridge
            .template_surface
            .refresh_after_state_change(bridge.runtime.as_ref())?;
        Ok(bridge)
    }

    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mount_frame =
            normalized_mount_frame(UiFrame::new(0.0, 0.0, shell_size.width, shell_size.height));
        refresh_layout::recompute(self, shell_size)
    }

    pub(crate) fn recompute_layout_at_mount(
        &mut self,
        mount_frame: UiFrame,
    ) -> Result<UiSize, BuiltinHostWindowTemplateBridgeError> {
        self.mount_frame = normalized_mount_frame(mount_frame);
        let shell_size = UiSize::new(self.mount_frame.width, self.mount_frame.height);
        refresh_layout::recompute(self, shell_size)?;
        Ok(shell_size)
    }

    pub(crate) fn surface(&self) -> &UiSurface {
        &self.template_surface.surface
    }

    pub(crate) fn frames(&self) -> EditorWorkbenchTemplateFrames {
        self.template_surface.frames
    }

    pub(crate) fn host_projection(&self) -> &RetainedUiHostProjection {
        &self.template_surface.host_projection
    }

    pub(crate) fn control_frame(&self, control_id: &str) -> Option<UiFrame> {
        self.template_surface.visible_control_frame(control_id)
    }

    pub(super) fn control_node_id(&self, control_id: &str) -> Option<UiNodeId> {
        self.template_surface.control_node_id(control_id)
    }

    pub(crate) fn layout_frames(&self) -> BuiltinWorkbenchWindowLayoutFrames {
        let mounted = |control_id| self.mounted_control_frame(control_id);
        BuiltinWorkbenchWindowLayoutFrames {
            mount_frame: Some(self.mount_frame),
            center_band_frame: mounted(EditorWorkbenchTemplateControlIds::MAIN_BAND),
            activity_rail_frame: mounted(EditorWorkbenchTemplateControlIds::ACTIVITY_RAIL),
            left_region_frame: union_visible_frames([
                mounted(EditorWorkbenchTemplateControlIds::ACTIVITY_RAIL),
                mounted(EditorWorkbenchTemplateControlIds::SCENE_TREE),
            ]),
            left_drawer_shell_frame: mounted(LEFT_DRAWER_SHELL_CONTROL_ID),
            left_drawer_header_frame: mounted(LEFT_DRAWER_HEADER_CONTROL_ID),
            left_drawer_content_frame: mounted(LEFT_DRAWER_CONTENT_CONTROL_ID),
            document_tabs_frame: mounted(DOCUMENT_TABS_CONTROL_ID),
            document_region_frame: mounted(EditorWorkbenchTemplateControlIds::VIEWPORT),
            right_drawer_shell_frame: mounted(RIGHT_DRAWER_SHELL_CONTROL_ID),
            right_drawer_header_frame: mounted(RIGHT_DRAWER_HEADER_CONTROL_ID),
            right_drawer_content_frame: mounted(RIGHT_DRAWER_CONTENT_CONTROL_ID),
            right_region_frame: mounted(EditorWorkbenchTemplateControlIds::INSPECTOR),
            bottom_drawer_shell_frame: mounted(BOTTOM_DRAWER_SHELL_CONTROL_ID),
            bottom_drawer_header_frame: mounted(BOTTOM_DRAWER_HEADER_CONTROL_ID),
            bottom_drawer_content_frame: mounted(BOTTOM_DRAWER_CONTENT_CONTROL_ID),
            bottom_region_frame: mounted(EditorWorkbenchTemplateControlIds::COMPONENT_DRAWER),
            status_bar_frame: mounted(EditorWorkbenchTemplateControlIds::STATUS_BAR),
            viewport_toolbar_frame: mounted(EditorWorkbenchTemplateControlIds::VIEWPORT_TOOLBAR),
            viewport_content_frame: mounted(EditorWorkbenchTemplateControlIds::VIEWPORT_SURFACE),
            left_resize_splitter_frame: left_resize_splitter_frame_from_drawer_shell(mounted(
                LEFT_DRAWER_SHELL_CONTROL_ID,
            )),
            right_resize_splitter_frame: right_resize_splitter_frame_from_drawer_shell(mounted(
                RIGHT_DRAWER_SHELL_CONTROL_ID,
            )),
            bottom_resize_splitter_frame: bottom_resize_splitter_frame_from_drawer_shell(mounted(
                BOTTOM_DRAWER_SHELL_CONTROL_ID,
            )),
        }
    }

    fn mounted_control_frame(&self, control_id: &str) -> Option<UiFrame> {
        self.control_frame(control_id).map(|frame| {
            UiFrame::new(
                frame.x + self.mount_frame.x,
                frame.y + self.mount_frame.y,
                frame.width,
                frame.height,
            )
        })
    }

    pub(crate) fn binding_for_control(
        &self,
        control_id: &str,
        event_kind: UiEventKind,
    ) -> Option<&EditorUiBinding> {
        binding_for_control(
            &self.bindings_by_id,
            &self.template_surface.host_projection,
            control_id,
            event_kind,
        )
    }

    pub(crate) fn has_control(&self, control_id: &str) -> bool {
        self.template_surface
            .host_projection
            .node_by_control_id(control_id)
            .is_some()
    }

    pub(crate) fn binding_by_id(&self, binding_id: &str) -> Option<&EditorUiBinding> {
        self.bindings_by_id.get(binding_id)
    }

    pub(crate) fn binding_id_for_action_id(&self, action_id: &str) -> Option<String> {
        if self.binding_by_id(action_id).is_some() {
            return Some(action_id.to_string());
        }
        self.template_surface
            .host_projection
            .nodes
            .iter()
            .flat_map(|node| node.routes.iter())
            .find(|route| binding_path_action_id(&route.binding_id) == action_id)
            .map(|route| route.binding_id.clone())
    }

    pub(crate) fn route_pointer_event(
        &mut self,
        mut event: zircon_runtime_interface::ui::dispatch::UiPointerEvent,
    ) -> Result<UiPointerRoute, BuiltinHostWindowTemplateBridgeError> {
        event.point.x -= self.mount_frame.x;
        event.point.y -= self.mount_frame.y;
        let route = match event.button {
            Some(button) => self
                .template_surface
                .surface
                .route_pointer_event_with_button(event.kind, event.point, button)?,
            None => self
                .template_surface
                .surface
                .route_pointer_event(event.kind, event.point)?,
        };
        Ok(route)
    }

    pub(crate) fn activation_route_for_pointer_route(
        &self,
        route: &UiPointerRoute,
    ) -> Option<(String, UiEventKind)> {
        if route.kind != UiPointerEventKind::Up || route.button != Some(UiPointerButton::Primary) {
            return None;
        }
        let click_target = route.click_target?;
        self.activation_route_for_node(click_target)
    }

    pub(crate) fn dispatch_control_state(
        &mut self,
        control_id: &str,
        event_kind: UiEventKind,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        let Some(binding) = self.binding_for_control(control_id, event_kind).cloned() else {
            return Ok(None);
        };
        self.apply_binding_state(control_id, &binding)?;
        Ok(Some(binding))
    }

    fn apply_binding_state(
        &mut self,
        source_control_id: &str,
        binding: &EditorUiBinding,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        match binding.payload() {
            EditorUiBindingPayload::ViewportCommand(ViewportCommand::ActivateSceneMode(mode)) => {
                self.select_exclusive(TOOL_CONTROLS, scene_mode_control_id(mode))?;
            }
            EditorUiBindingPayload::ViewportCommand(ViewportCommand::SetGridMode(_)) => {
                self.set_control_active("WorkbenchToolSnap", true)?;
            }
            EditorUiBindingPayload::DockCommand(command) => {
                if let Some(control_id) = dock_command_control_id(command) {
                    self.select_exclusive(RAIL_CONTROLS, control_id)?;
                }
            }
            EditorUiBindingPayload::SelectionCommand(SelectionCommand::SelectSceneNode {
                ..
            }) => {
                let selected_control_id = if self.is_scene_tree_control(source_control_id)? {
                    source_control_id.to_string()
                } else {
                    "WorkbenchScenePropsItem".to_string()
                };
                for control_id in self.scene_tree_control_ids()? {
                    self.set_selected(&control_id, control_id == selected_control_id)?;
                }
            }
            EditorUiBindingPayload::MenuAction { action_id } => {
                self.apply_reference_menu_action(source_control_id, action_id)?;
            }
            _ => {}
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }

    pub(crate) fn dispatch_binding_state_for_control(
        &mut self,
        source_control_id: &str,
        binding_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        let Some(binding) = self.binding_by_id(binding_id).cloned() else {
            return Ok(None);
        };
        let control_id = if source_control_id.is_empty() {
            binding.path().control_id.as_str()
        } else {
            source_control_id
        };
        self.apply_binding_state(&control_id, &binding)?;
        Ok(Some(binding))
    }

    pub(super) fn select_exclusive(
        &mut self,
        controls: &[&str],
        selected_control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for control_id in controls {
            self.set_control_active(control_id, *control_id == selected_control_id)?;
        }
        Ok(())
    }

    pub(super) fn select_exclusive_selected(
        &mut self,
        controls: &[&str],
        selected_control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for control_id in controls {
            self.set_selected(control_id, *control_id == selected_control_id)?;
        }
        Ok(())
    }

    pub(super) fn apply_workbench_module_workspace(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let scene_active = action_id == "workbench.module.scene.select";
        self.set_visible("WorkbenchSceneWorkspace", true)?;
        self.set_visible("WorkbenchMainBandModuleWorkspace", !scene_active)?;
        self.set_visible("WorkbenchModuleWorkspace", !scene_active)?;
        self.set_visible("WorkbenchExtensionModuleWorkspaces", false)?;
        self.set_visible("WorkbenchExtensionModuleWorkspacesHost", false)?;
        self.close_workbench_generated_bottom_drawer()?;
        for control_id in MODULE_WORKSPACE_CONTROLS {
            self.set_visible(
                control_id,
                Some(*control_id) == workbench_module_workspace_control_id(action_id),
            )?;
        }
        Ok(())
    }

    pub(super) fn apply_workbench_extension_workspace(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive(MODULE_TAB_CONTROLS, "WorkbenchModuleRender")?;
        self.set_visible("WorkbenchSceneWorkspace", true)?;
        self.set_visible("WorkbenchMainBandModuleWorkspace", true)?;
        self.set_visible("WorkbenchModuleWorkspace", true)?;
        for control_id in MODULE_WORKSPACE_CONTROLS {
            self.set_visible(control_id, false)?;
        }
        self.close_workbench_generated_bottom_drawer()?;
        self.set_visible("WorkbenchExtensionModuleWorkspaces", true)?;
        self.set_visible("WorkbenchExtensionModuleWorkspacesHost", true)?;
        for control_id in EXTENSION_MODULE_WORKSPACE_CONTROLS {
            self.set_visible(
                control_id,
                Some(*control_id) == workbench_extension_workspace_control_id(action_id),
            )?;
        }
        Ok(())
    }

    pub(super) fn should_open_dropdown_for_module_field_action(
        &self,
        source_control_id: &str,
        action_id: &str,
    ) -> bool {
        action_id.ends_with(".edit")
            && !self
                .control_string_array(source_control_id, "options")
                .is_empty()
    }

    pub(super) fn set_control_active(
        &mut self,
        control_id: &str,
        selected: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let node_ids =
            control_node_ids_with_descendants(&self.template_surface.surface, control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        let value = UiValue::Bool(selected);
        for node_id in &node_ids {
            self.mutate_node_bool(*node_id, "selected", selected)?;
            self.mutate_node_bool(*node_id, "checked", selected)?;
        }
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(node_ids[0], "value", value))?;
        Ok(())
    }

    pub(super) fn set_selected(
        &mut self,
        control_id: &str,
        selected: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let node_ids =
            control_node_ids_with_descendants(&self.template_surface.surface, control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        for node_id in node_ids {
            self.mutate_node_bool(node_id, "selected", selected)?;
        }
        Ok(())
    }

    pub(super) fn toggle_checked(
        &mut self,
        control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let node_ids =
            control_node_ids_with_descendants(&self.template_surface.surface, control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        let checked = !self.control_bool(control_id, "checked");
        for node_id in &node_ids {
            self.mutate_node_bool(*node_id, "checked", checked)?;
            self.mutate_node_bool(*node_id, "selected", checked)?;
        }
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(
                node_ids[0],
                "value",
                UiValue::Bool(checked),
            ))?;
        Ok(())
    }

    pub(super) fn set_visible(
        &mut self,
        control_id: &str,
        visible: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let node_ids = control_node_ids(&self.template_surface.surface, control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        let visibility = if visible { "visible" } else { "collapsed" };
        for node_id in node_ids {
            let _ =
                self.template_surface
                    .surface
                    .mutate_property(UiPropertyMutationRequest::new(
                        node_id,
                        "visibility",
                        UiValue::String(visibility.to_string()),
                    ))?;
            if visible {
                let _ = self.template_surface.surface.mutate_property(
                    UiPropertyMutationRequest::new(node_id, "visible", UiValue::Bool(true)),
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn cycle_string_property(
        &mut self,
        control_id: &str,
        property: &str,
        values: &[&str],
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if values.is_empty() {
            return Ok(());
        }
        let Some(node_id) = self.control_node_id(control_id) else {
            return Ok(());
        };
        let current = self
            .control_string(control_id, property)
            .unwrap_or_else(|| values[0].to_string());
        let current_index = values
            .iter()
            .position(|value| *value == current)
            .unwrap_or(0);
        let next = values[(current_index + 1) % values.len()];
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(
                node_id,
                property,
                UiValue::String(next.to_string()),
            ))?;
        Ok(())
    }

    pub(super) fn mutate_control_property(
        &mut self,
        control_id: &str,
        property: &str,
        value: UiValue,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(node_id) = self.control_node_id(control_id) else {
            return Ok(());
        };
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(node_id, property, value))?;
        Ok(())
    }

    pub(super) fn mutate_node_bool(
        &mut self,
        node_id: UiNodeId,
        property: &str,
        value: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(
                node_id,
                property,
                UiValue::Bool(value),
            ))
            .map_err(
                |source| BuiltinHostWindowTemplateBridgeError::LayoutMutation {
                    node_id,
                    property: property.to_string(),
                    source,
                },
            )?;
        Ok(())
    }

    pub(super) fn control_bool(&self, control_id: &str, property: &str) -> bool {
        let Some(node_id) = self.control_node_id(control_id) else {
            return false;
        };
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.attributes.get(property))
                    .and_then(toml::Value::as_bool)
            })
            .unwrap_or(false)
    }

    pub(super) fn control_string(&self, control_id: &str, property: &str) -> Option<String> {
        let node_id = self.control_node_id(control_id)?;
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.attributes.get(property))
                    .and_then(toml::Value::as_str)
                    .map(str::to_string)
            })
    }

    pub(super) fn control_string_array(&self, control_id: &str, property: &str) -> Vec<String> {
        let Some(node_id) = self.control_node_id(control_id) else {
            return Vec::new();
        };
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.attributes.get(property))
                    .map(toml_value_string_list)
            })
            .unwrap_or_default()
    }

    fn activation_route_for_node(&self, node_id: UiNodeId) -> Option<(String, UiEventKind)> {
        let bubble_route = self
            .template_surface
            .surface
            .tree
            .bubble_route(node_id)
            .ok()?;
        for candidate_id in bubble_route {
            let node = self
                .template_surface
                .surface
                .tree
                .nodes
                .get(&candidate_id)?;
            let Some(metadata) = node.template_metadata.as_ref() else {
                continue;
            };
            let Some(control_id) = metadata.control_id.as_deref() else {
                continue;
            };
            if let Some(event_kind) = authored_primary_activation_event(metadata) {
                return Some((control_id.to_string(), event_kind));
            }
        }
        None
    }
}

fn normalized_mount_frame(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x.max(0.0),
        frame.y.max(0.0),
        frame.width.max(0.0),
        frame.height.max(0.0),
    )
}

const TOOL_CONTROLS: &[&str] = &[
    "WorkbenchToolSelect",
    "WorkbenchToolMove",
    "WorkbenchToolRotate",
    "WorkbenchToolScale",
];

const RAIL_CONTROLS: &[&str] = &[
    "WorkbenchRailScene",
    "WorkbenchRailCube",
    "WorkbenchRailGraph",
    "WorkbenchRailImage",
    "WorkbenchRailAudio",
    "WorkbenchRailCode",
];

fn scene_mode_control_id(mode: &SceneModeActivation) -> &'static str {
    match mode {
        SceneModeActivation::Select | SceneModeActivation::Custom(_) => "WorkbenchToolSelect",
        SceneModeActivation::Transform(TransformHandleKind::Move) => "WorkbenchToolMove",
        SceneModeActivation::Transform(TransformHandleKind::Rotate) => "WorkbenchToolRotate",
        SceneModeActivation::Transform(TransformHandleKind::Scale) => "WorkbenchToolScale",
    }
}

fn dock_command_control_id(command: &DockCommand) -> Option<&'static str> {
    match command {
        DockCommand::FocusView { instance_id } if instance_id == "editor.scene#1" => {
            Some("WorkbenchRailScene")
        }
        DockCommand::ActivateDrawerTab { instance_id, .. }
            if instance_id == "editor.hierarchy#1" =>
        {
            Some("WorkbenchRailCube")
        }
        DockCommand::FocusView { instance_id } if instance_id == "editor.graph#1" => {
            Some("WorkbenchRailGraph")
        }
        DockCommand::ActivateDrawerTab { instance_id, .. } if instance_id == "editor.assets#1" => {
            Some("WorkbenchRailImage")
        }
        DockCommand::FocusView { instance_id } if instance_id == "editor.audio#1" => {
            Some("WorkbenchRailAudio")
        }
        DockCommand::FocusView { instance_id } if instance_id == "editor.code#1" => {
            Some("WorkbenchRailCode")
        }
        _ => None,
    }
}

fn control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    control_node_ids(surface, control_id).into_iter().next()
}

fn control_node_ids(surface: &UiSurface, control_id: &str) -> Vec<UiNodeId> {
    surface
        .tree
        .nodes
        .values()
        .filter_map(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                .filter(|candidate| *candidate == control_id)
                .map(|_| node.node_id)
        })
        .collect()
}

fn control_node_ids_with_descendants(surface: &UiSurface, control_id: &str) -> Vec<UiNodeId> {
    let Some(root_id) = control_node_id(surface, control_id) else {
        return Vec::new();
    };

    let mut node_ids = Vec::new();
    let mut stack = vec![root_id];
    while let Some(node_id) = stack.pop() {
        node_ids.push(node_id);
        if let Some(node) = surface.tree.nodes.get(&node_id) {
            for child_id in node.children.iter().rev() {
                stack.push(*child_id);
            }
        }
    }
    node_ids
}

fn authored_primary_activation_event(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> Option<UiEventKind> {
    [UiEventKind::Click, UiEventKind::Toggle, UiEventKind::Change]
        .into_iter()
        .find(|event_kind| {
            metadata
                .bindings
                .iter()
                .any(|binding| binding.event == *event_kind)
        })
}

fn binding_path_action_id(binding_id: &str) -> String {
    binding_id
        .split(['/', '.', ':'])
        .filter(|segment| !segment.is_empty())
        .map(camel_to_snake_segment)
        .collect::<Vec<_>>()
        .join(".")
}

fn camel_to_snake_segment(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}
