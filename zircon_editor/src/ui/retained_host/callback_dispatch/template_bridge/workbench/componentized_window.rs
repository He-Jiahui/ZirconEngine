use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime::ui::surface::{UiSurface, UiVirtualListItemKey};
use zircon_runtime::ui::tree::UiRuntimeTreeRoutingExt;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
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
    EditorUiHostRuntime, RetainedUiHostNodeModel, RetainedUiHostProjection,
    WORKBENCH_WINDOW_DOCUMENT_ID,
};
use crate::ui::workbench::reference::{
    build_editor_workbench_template_surface, EditorWorkbenchReferenceMetrics,
    EditorWorkbenchTemplateControlIds, EditorWorkbenchTemplateFrames,
    EditorWorkbenchTemplateSurface,
};
use crate::ui::workbench::snapshot::InspectorPluginComponentPropertySnapshot;

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
    workbench_extension_workspace_control_id, EXTENSION_MODULE_WORKSPACE_CONTROLS,
};
use super::icon_tooltip::WorkbenchIconTooltipInputState;
use super::layout_frames::{
    bottom_resize_splitter_frame_from_drawer_shell, left_resize_splitter_frame_from_drawer_shell,
    right_resize_splitter_frame_from_drawer_shell, union_visible_frames,
    BuiltinWorkbenchWindowLayoutFrames,
};
use super::module_navigation::{
    workbench_module_workspace_control_id, MODULE_TAB_CONTROLS, MODULE_WORKSPACE_CONTROLS,
};
use super::responsive_layout::apply_workbench_responsive_layout;
use super::scene_hierarchy_projection::SceneHierarchyProjectionState;

mod mounted_layout;
mod refresh_layout;
mod resolution_projection;

pub(super) use resolution_projection::logical_axis_from_physical;
use resolution_projection::{normalized_presentation_scale_factor, scale_frame};

pub(crate) struct BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) runtime: Arc<EditorUiHostRuntime>,
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    pub(super) template_surface: EditorWorkbenchTemplateSurface,
    pub(super) mount_frame: UiFrame,
    pub(super) presentation_scale_factor: f32,
    committed_mount_origin: (f32, f32),
    committed_presentation_scale_factor: f32,
    pub(super) asset_creation_menu: AssetCreationMenuState,
    pub(super) scene_hierarchy_projection: SceneHierarchyProjectionState,
    pub(super) inspector_source_properties: Arc<[InspectorPluginComponentPropertySnapshot]>,
    pub(super) inspector_component_label: String,
    pub(super) inspector_has_selection: bool,
    pub(super) inspector_has_component: bool,
    pub(super) component_properties: Arc<[InspectorPluginComponentPropertySnapshot]>,
    pub(super) component_property_keys: Arc<[UiVirtualListItemKey]>,
    pub(super) component_customization_available: bool,
    pub(super) icon_tooltip_input: WorkbenchIconTooltipInputState,
    pub(super) compact_module_details_drawer_open: bool,
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
            presentation_scale_factor: 1.0,
            committed_mount_origin: (mount_frame.x, mount_frame.y),
            committed_presentation_scale_factor: 1.0,
            asset_creation_menu: AssetCreationMenuState::default(),
            scene_hierarchy_projection: SceneHierarchyProjectionState::default(),
            inspector_source_properties: Arc::from([]),
            inspector_component_label: String::new(),
            inspector_has_selection: false,
            inspector_has_component: false,
            component_properties: Arc::from([]),
            component_property_keys: Arc::from([]),
            component_customization_available: false,
            icon_tooltip_input: WorkbenchIconTooltipInputState::default(),
            compact_module_details_drawer_open: false,
        };
        bridge.initialize_live_control_state()?;
        apply_workbench_responsive_layout(
            &mut bridge.template_surface.surface,
            shell_size,
            1.0,
            bridge.compact_module_details_drawer_open,
        )?;
        bridge.apply_responsive_toolbar_layout(shell_size)?;
        bridge
            .template_surface
            .refresh_after_state_change(bridge.runtime.as_ref())?;
        Ok(bridge)
    }

    pub(crate) fn surface(&self) -> &UiSurface {
        &self.template_surface.surface
    }

    #[cfg(test)]
    pub(crate) fn layout_pass_count(&self) -> u64 {
        self.template_surface.layout_pass_count()
    }

    pub(crate) fn frames(&self) -> EditorWorkbenchTemplateFrames {
        let frames = self.template_surface.frames;
        EditorWorkbenchTemplateFrames {
            root: scale_frame(frames.root, self.presentation_scale_factor),
            top_toolbar: scale_frame(frames.top_toolbar, self.presentation_scale_factor),
            main_band: scale_frame(frames.main_band, self.presentation_scale_factor),
            activity_rail: scale_frame(frames.activity_rail, self.presentation_scale_factor),
            scene_tree: scale_frame(frames.scene_tree, self.presentation_scale_factor),
            viewport: scale_frame(frames.viewport, self.presentation_scale_factor),
            inspector: scale_frame(frames.inspector, self.presentation_scale_factor),
            component_drawer: scale_frame(frames.component_drawer, self.presentation_scale_factor),
            status_bar: scale_frame(frames.status_bar, self.presentation_scale_factor),
        }
    }

    pub(crate) fn host_projection(&self) -> &RetainedUiHostProjection {
        &self.template_surface.host_projection
    }

    pub(crate) fn pending_host_projection_patch_nodes(
        &self,
    ) -> Option<Vec<crate::ui::template_runtime::RetainedUiHostNodeModel>> {
        self.template_surface.pending_host_projection_patch_nodes()
    }

    pub(crate) fn pending_host_projection_geometry_patch_indices(&self) -> Option<Vec<usize>> {
        if self.committed_mount_origin != (self.mount_frame.x, self.mount_frame.y)
            || self.committed_presentation_scale_factor != self.presentation_scale_factor
        {
            return None;
        }
        self.template_surface
            .pending_host_projection_geometry_patch_indices()
    }

    pub(crate) fn has_pending_host_projection_commit(&self) -> bool {
        self.template_surface.has_pending_host_projection_commit()
    }

    pub(crate) fn has_pending_surface_state_change(&self) -> bool {
        self.template_surface
            .surface
            .pending_invalidation_changed_node_count()
            > 0
    }

    pub(crate) fn mark_host_projection_committed(&mut self) {
        self.template_surface.mark_host_projection_committed();
        self.committed_mount_origin = (self.mount_frame.x, self.mount_frame.y);
        self.committed_presentation_scale_factor = self.presentation_scale_factor;
    }

    pub(crate) fn refresh_prepared_state_change(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }

    pub(crate) fn presentation_scale_factor(&self) -> f32 {
        self.presentation_scale_factor
    }

    pub(crate) fn host_projection_nodes_for_controls(
        &self,
        control_ids: &[String],
    ) -> Option<Vec<RetainedUiHostNodeModel>> {
        control_ids
            .iter()
            .map(|control_id| {
                self.template_surface
                    .host_projection_node_for_control(control_id)
                    .cloned()
            })
            .collect()
    }

    pub(crate) fn control_frame(&self, control_id: &str) -> Option<UiFrame> {
        self.template_surface
            .visible_control_frame(control_id)
            .map(|frame| scale_frame(frame, self.presentation_scale_factor))
    }

    pub(super) fn control_node_id(&self, control_id: &str) -> Option<UiNodeId> {
        self.template_surface.control_node_id(control_id)
    }

    fn control_parent_id(&self, control_id: &str) -> Option<UiNodeId> {
        let node_id = self.control_node_id(control_id)?;
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.parent)
    }

    #[cfg(test)]
    pub(crate) fn has_control_index_entry(&self, control_id: &str) -> bool {
        self.template_surface.has_control_index_entry(control_id)
    }

    pub(crate) fn scene_entity_for_control(&self, control_id: &str) -> Option<u64> {
        self.scene_hierarchy_projection
            .entity_for_control(control_id)
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
        event.point.x = logical_axis_from_physical(
            event.point.x,
            self.mount_frame.x,
            self.presentation_scale_factor,
        );
        event.point.y = logical_axis_from_physical(
            event.point.y,
            self.mount_frame.y,
            self.presentation_scale_factor,
        );
        let route = self
            .template_surface
            .surface
            .route_pointer_input_event(event)?;
        let _ = self
            .template_surface
            .surface
            .apply_default_pointer_scroll(&route)?;
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
                let previously_selected = self
                    .scene_hierarchy_projection
                    .selected_entities()
                    .iter()
                    .filter_map(|entity| {
                        self.scene_hierarchy_projection
                            .control_for(*entity)
                            .map(str::to_string)
                    })
                    .collect::<Vec<_>>();
                for control_id in previously_selected {
                    if control_id != selected_control_id {
                        self.set_selected(&control_id, false)?;
                    }
                }
                self.set_selected(&selected_control_id, true)?;
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

    fn initialize_live_control_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive(MODULE_TAB_CONTROLS, "WorkbenchModuleEffect")?;
        self.select_exclusive(TOOL_CONTROLS, "WorkbenchToolSelect")?;
        self.select_exclusive(RAIL_CONTROLS, "WorkbenchRailScene")?;
        self.initialize_panel_live_control_state()?;
        self.initialize_blend_space_transport_state()?;
        self.initialize_run_mode_menu_indicator()?;
        self.initialize_layout_menu_indicator()?;
        self.set_control_active("WorkbenchModuleDetailsDrawerToggle", false)?;
        self.apply_workbench_module_workspace("workbench.module.effect.select")
    }

    pub(super) fn select_exclusive_selected(
        &mut self,
        controls: &[&str],
        selected_control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let selection_parent = self.control_parent_id(selected_control_id);
        for control_id in controls {
            if selection_parent.is_some() && self.control_parent_id(control_id) != selection_parent
            {
                continue;
            }
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
        self.refresh_compact_module_details_toggle_visibility()
    }

    pub(super) fn toggle_compact_module_details_drawer(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.compact_module_details_drawer_open = !self.compact_module_details_drawer_open;
        self.set_control_active(
            "WorkbenchModuleDetailsDrawerToggle",
            self.compact_module_details_drawer_open,
        )?;
        apply_workbench_responsive_layout(
            &mut self.template_surface.surface,
            UiSize::new(self.mount_frame.width, self.mount_frame.height),
            self.presentation_scale_factor,
            self.compact_module_details_drawer_open,
        )?;
        let roots = self.template_surface.surface.tree.roots.clone();
        for root_id in roots {
            self.template_surface
                .surface
                .tree
                .mark_layout_dirty(root_id)?;
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
        self.refresh_compact_module_details_toggle_visibility()
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
