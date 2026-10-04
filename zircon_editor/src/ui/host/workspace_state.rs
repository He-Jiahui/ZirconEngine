use std::collections::BTreeSet;

use serde_json::Value;

use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, MainHostPageLayout, MainPageId,
    RestorePolicy, WorkbenchLayout,
};
use crate::ui::workbench::project::ProjectEditorWorkspace;
use crate::ui::workbench::view::{
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
};
use crate::ui::workbench::window_registry::EditorWindowRegistry;

use super::asset_editor_sessions::UI_ASSET_EDITOR_DESCRIPTOR_ID;
use super::builtin_layout::{builtin_hybrid_layout_for_subsystems, ensure_builtin_shell_instances};
use super::editor_error::EditorError;
use super::editor_session_state::EditorSessionState;
use super::editor_ui_host::EditorUiHost;
use super::layout_hosts::{
    active_tab_from_document::active_tab_from_document,
    collect_instance_hosts::collect_instance_hosts,
    repair_builtin_shell_layout::repair_builtin_shell_layout,
};

fn active_main_page_view(session: &EditorSessionState) -> Option<ViewInstanceId> {
    session
        .layout
        .main_pages
        .iter()
        .find(|page| page.id() == &session.layout.active_main_page)
        .and_then(|page| match page {
            MainHostPageLayout::WorkbenchPage { id, .. } => session
                .layout
                .content_workspace_for_page(id)
                .and_then(active_tab_from_document),
            MainHostPageLayout::ExclusiveActivityWindowPage {
                window_instance, ..
            } => Some(window_instance.clone()),
        })
}

fn active_activity_window_template_descriptor_id(
    session: &EditorSessionState,
) -> Option<&ViewDescriptorId> {
    let layout = &session.layout;
    let active_page = layout
        .main_pages
        .iter()
        .find(|page| page.id() == &layout.active_main_page)?;
    match active_page {
        MainHostPageLayout::WorkbenchPage {
            activity_window, ..
        } => layout
            .activity_windows()
            .get(activity_window)
            .map(|window| &window.descriptor_id),
        MainHostPageLayout::ExclusiveActivityWindowPage {
            window_instance, ..
        } => session
            .open_view_instances
            .get(window_instance)
            .map(|instance| &instance.descriptor_id),
    }
}

fn visit_document_focus_target<'a>(
    node: &'a DocumentNode,
    focused: Option<&ViewInstanceId>,
    first: &mut Option<&'a ViewInstanceId>,
    active: &mut Option<&'a ViewInstanceId>,
) -> Option<&'a ViewInstanceId> {
    match node {
        DocumentNode::SplitNode {
            first: first_node,
            second: second_node,
            ..
        } => {
            if let Some(target) = visit_document_focus_target(first_node, focused, first, active) {
                return Some(target);
            }
            visit_document_focus_target(second_node, focused, first, active)
        }
        DocumentNode::Tabs(stack) => {
            for tab in &stack.tabs {
                if first.is_none() {
                    *first = Some(tab);
                }
                if focused == Some(tab) {
                    return Some(tab);
                }
                if active.is_none() && stack.active_tab.as_ref() == Some(tab) {
                    *active = Some(tab);
                }
            }
            None
        }
    }
}

fn floating_window_focus_target_in_layout<'a>(
    layout: &'a WorkbenchLayout,
    window_id: &MainPageId,
) -> Option<&'a ViewInstanceId> {
    let window = layout
        .floating_windows
        .iter()
        .find(|window| &window.window_id == window_id)?;
    let mut first = None;
    let mut active = None;
    visit_document_focus_target(
        &window.workspace,
        window.focused_view.as_ref(),
        &mut first,
        &mut active,
    )
    .or(active)
    .or(first)
}

fn floating_window_id_for_surface_key_in_layout(
    layout: &WorkbenchLayout,
    surface_key: &str,
) -> Option<MainPageId> {
    layout
        .floating_windows
        .iter()
        .find(|window| window.window_id.0 == surface_key)
        .map(|window| window.window_id.clone())
}

fn active_drawer_toggle_state_in_layout(
    layout: &WorkbenchLayout,
    slot: ActivityDrawerSlot,
    instance_id: &ViewInstanceId,
) -> Result<(ActivityDrawerMode, bool, bool), String> {
    let active_window_id = layout
        .active_activity_window_id()
        .ok_or_else(|| "missing active activity window".to_string())?;
    let active_drawers = &layout
        .activity_windows()
        .get(&active_window_id)
        .ok_or_else(|| format!("missing active activity window {active_window_id:?}"))?
        .activity_drawers;
    let drawer = active_drawers
        .get(&slot)
        .ok_or_else(|| format!("missing drawer {slot:?}"))?;
    let region_was_expanded = active_drawers.iter().any(|(candidate_slot, candidate)| {
        candidate_slot.shares_region(slot)
            && candidate.visible
            && !candidate.tab_stack.tabs.is_empty()
            && candidate.mode != ActivityDrawerMode::Collapsed
    });
    let is_active = drawer
        .tab_stack
        .active_tab
        .as_ref()
        .is_some_and(|active| active == instance_id);
    Ok((drawer.mode, is_active, region_was_expanded))
}

fn active_drawer_mode_in_layout(
    layout: &WorkbenchLayout,
    slot: ActivityDrawerSlot,
) -> Option<ActivityDrawerMode> {
    layout
        .active_activity_window()
        .and_then(|window| window.activity_drawers.get(&slot))
        .map(|drawer| drawer.mode)
}

fn floating_window_exists_in_layout(layout: &WorkbenchLayout, window_id: &MainPageId) -> bool {
    layout
        .floating_windows
        .iter()
        .any(|window| &window.window_id == window_id)
}

fn floating_window_instance_ids_in_layout(
    layout: &WorkbenchLayout,
    window_id: &MainPageId,
) -> Option<Vec<ViewInstanceId>> {
    let window = layout
        .floating_windows
        .iter()
        .find(|window| &window.window_id == window_id)?;
    let mut instance_ids = Vec::with_capacity(window.workspace.instance_count());
    window.workspace.append_instance_ids(&mut instance_ids);
    (!instance_ids.is_empty()).then_some(instance_ids)
}

fn view_host_for_instance_key_in_session(
    session: &EditorSessionState,
    surface_key: &str,
) -> Option<ViewHost> {
    session
        .open_view_instances
        .values()
        .find(|instance| instance.instance_id.0 == surface_key)
        .map(|instance| instance.host.clone())
}

fn view_instance_id_for_descriptor_in_session<'a>(
    session: &'a EditorSessionState,
    descriptor_id: &ViewDescriptorId,
) -> Option<&'a ViewInstanceId> {
    session
        .open_view_instances
        .iter()
        .find_map(|(instance_id, instance)| {
            (&instance.descriptor_id == descriptor_id).then_some(instance_id)
        })
}

fn current_view_instance_ids_in_session(session: &EditorSessionState) -> Vec<ViewInstanceId> {
    session.open_view_instances.keys().cloned().collect()
}

fn view_instance_ids_for_descriptor_key_in_session(
    session: &EditorSessionState,
    descriptor_key: &str,
) -> Vec<ViewInstanceId> {
    let mut instance_ids = Vec::new();
    for (instance_id, instance) in &session.open_view_instances {
        if instance.descriptor_id.0.as_str() != descriptor_key {
            continue;
        }
        if instance_ids.is_empty() {
            instance_ids.reserve(session.open_view_instances.len());
        }
        instance_ids.push(instance_id.clone());
    }
    instance_ids
}

fn view_instance_ids_for_descriptors_in_session(
    session: &EditorSessionState,
    descriptor_ids: &BTreeSet<ViewDescriptorId>,
) -> Vec<ViewInstanceId> {
    let mut instance_ids = Vec::new();
    for (instance_id, instance) in &session.open_view_instances {
        if !descriptor_ids.contains(&instance.descriptor_id) {
            continue;
        }
        if instance_ids.is_empty() {
            instance_ids.reserve(session.open_view_instances.len());
        }
        instance_ids.push(instance_id.clone());
    }
    instance_ids
}

fn editor_pane_instance_ids_in_session(
    session: &EditorSessionState,
    collect_ui_asset_panes: bool,
    collect_animation_panes: bool,
) -> (Vec<ViewInstanceId>, Vec<ViewInstanceId>) {
    let mut ui_asset_instance_ids = Vec::new();
    let mut animation_instance_ids = Vec::new();
    for (instance_id, instance) in &session.open_view_instances {
        match instance.descriptor_id.0.as_str() {
            UI_ASSET_EDITOR_DESCRIPTOR_ID if collect_ui_asset_panes => {
                if ui_asset_instance_ids.is_empty() {
                    ui_asset_instance_ids.reserve(session.open_view_instances.len());
                }
                ui_asset_instance_ids.push(instance_id.clone());
            }
            "editor.animation_sequence" | "editor.animation_graph" if collect_animation_panes => {
                if animation_instance_ids.is_empty() {
                    animation_instance_ids.reserve(session.open_view_instances.len());
                }
                animation_instance_ids.push(instance_id.clone());
            }
            _ => {}
        }
    }
    (ui_asset_instance_ids, animation_instance_ids)
}

impl EditorUiHost {
    pub(super) fn current_layout(&self) -> WorkbenchLayout {
        self.lock_session().layout.clone()
    }

    pub(super) fn current_view_instances(&self) -> Vec<ViewInstance> {
        self.lock_session()
            .open_view_instances
            .values()
            .cloned()
            .collect()
    }

    pub(super) fn current_focused_view(&self) -> Option<ViewInstanceId> {
        self.lock_session().focused_view.clone()
    }

    pub(super) fn current_focused_view_matches(&self, descriptor_id: &ViewDescriptorId) -> bool {
        let session = self.lock_session();
        let Some(focused) = session.focused_view.as_ref() else {
            return false;
        };
        session
            .open_view_instances
            .get(focused)
            .is_some_and(|instance| &instance.descriptor_id == descriptor_id)
    }

    pub(super) fn active_activity_window_template_document_is(&self, document_id: &str) -> bool {
        let session = self.lock_session();
        let Some(descriptor_id) = active_activity_window_template_descriptor_id(&session) else {
            return false;
        };
        let registry = self.lock_view_registry();
        let Some(descriptor) = registry.descriptor(descriptor_id) else {
            return false;
        };
        if registry.descriptor_capability_error(descriptor).is_some() {
            return false;
        }
        descriptor
            .activity_window_template
            .as_ref()
            .is_some_and(|template| template.document_id.as_str() == document_id)
    }

    pub(super) fn floating_window_focus_target(
        &self,
        window_id: &MainPageId,
    ) -> Option<ViewInstanceId> {
        let session = self.lock_session();
        floating_window_focus_target_in_layout(&session.layout, window_id).cloned()
    }

    pub(super) fn floating_window_id_for_surface_key(
        &self,
        surface_key: &str,
    ) -> Option<MainPageId> {
        let session = self.lock_session();
        floating_window_id_for_surface_key_in_layout(&session.layout, surface_key)
    }

    pub(super) fn active_drawer_toggle_state(
        &self,
        slot: ActivityDrawerSlot,
        instance_id: &ViewInstanceId,
    ) -> Result<(ActivityDrawerMode, bool, bool), String> {
        let session = self.lock_session();
        active_drawer_toggle_state_in_layout(&session.layout, slot, instance_id)
    }

    pub(super) fn active_drawer_mode(
        &self,
        slot: ActivityDrawerSlot,
    ) -> Option<ActivityDrawerMode> {
        let session = self.lock_session();
        active_drawer_mode_in_layout(&session.layout, slot)
    }

    pub(super) fn floating_window_exists(&self, window_id: &MainPageId) -> bool {
        let session = self.lock_session();
        floating_window_exists_in_layout(&session.layout, window_id)
    }

    pub(super) fn floating_window_instance_ids(
        &self,
        window_id: &MainPageId,
    ) -> Option<Vec<ViewInstanceId>> {
        let session = self.lock_session();
        floating_window_instance_ids_in_layout(&session.layout, window_id)
    }

    pub(super) fn view_host_for_instance_key(&self, surface_key: &str) -> Option<ViewHost> {
        let session = self.lock_session();
        view_host_for_instance_key_in_session(&session, surface_key)
    }

    pub(super) fn view_instance_id_for_descriptor(
        &self,
        descriptor_id: &ViewDescriptorId,
    ) -> Option<ViewInstanceId> {
        let session = self.lock_session();
        view_instance_id_for_descriptor_in_session(&session, descriptor_id).cloned()
    }

    pub(super) fn current_view_instance_ids(&self) -> Vec<ViewInstanceId> {
        let session = self.lock_session();
        current_view_instance_ids_in_session(&session)
    }

    pub(super) fn view_instance_ids_for_descriptor_key(
        &self,
        descriptor_key: &str,
    ) -> Vec<ViewInstanceId> {
        let session = self.lock_session();
        view_instance_ids_for_descriptor_key_in_session(&session, descriptor_key)
    }

    pub(super) fn view_instance_ids_for_descriptors(
        &self,
        descriptor_ids: &BTreeSet<ViewDescriptorId>,
    ) -> Vec<ViewInstanceId> {
        let session = self.lock_session();
        view_instance_ids_for_descriptors_in_session(&session, descriptor_ids)
    }

    pub(super) fn editor_pane_instance_ids(
        &self,
        collect_ui_asset_panes: bool,
        collect_animation_panes: bool,
    ) -> (Vec<ViewInstanceId>, Vec<ViewInstanceId>) {
        let session = self.lock_session();
        editor_pane_instance_ids_in_session(
            &session,
            collect_ui_asset_panes,
            collect_animation_panes,
        )
    }

    pub(super) fn update_view_instance_metadata(
        &self,
        instance_id: &ViewInstanceId,
        title: Option<String>,
        dirty: Option<bool>,
        payload: Option<Value>,
    ) -> Result<(), EditorError> {
        let mut session = self.lock_session();
        let instance = session
            .open_view_instances
            .get_mut(instance_id)
            .ok_or_else(|| {
                EditorError::Registry(format!("missing view instance {}", instance_id.0))
            })?;
        if let Some(title) = title {
            instance.title = title;
        }
        if let Some(dirty) = dirty {
            instance.dirty = dirty;
        }
        if let Some(payload) = payload {
            instance.serializable_payload = payload;
        }
        Ok(())
    }

    pub(super) fn native_window_hosts(
        &self,
    ) -> Vec<super::window_host_manager::NativeWindowHostState> {
        self.lock_window_host_manager().states()
    }

    pub(super) fn sync_native_window_projection_bounds(
        &self,
        window_id: &MainPageId,
        bounds: [f32; 4],
    ) {
        self.lock_window_host_manager()
            .sync_window_bounds(window_id, bounds);
    }

    pub(super) fn descriptors(&self) -> Vec<ViewDescriptor> {
        self.lock_view_registry().list_descriptors()
    }

    pub(super) fn restore_workspace(
        &self,
        policy: RestorePolicy,
    ) -> Result<WorkbenchLayout, EditorError> {
        let global = self.load_global_default_layout();
        let workspace = self.project_workspace();
        let restored = self
            .layout_manager
            .restore_workspace(policy, Some(workspace), global)
            .map_err(EditorError::Layout)?;
        let mut session = self.lock_session();
        session.layout = restored.clone();
        self.recompute_session_metadata(&mut session);
        Ok(restored)
    }

    pub(super) fn apply_project_workspace_state(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Result<Vec<ViewInstance>, EditorError> {
        let Some(workspace) = workspace else {
            self.bootstrap_default_layout()?;
            return Ok(Vec::new());
        };

        self.clear_document_toolkits()?;

        let mut session = self.lock_session();
        let mut registry = self.lock_view_registry();
        registry.clear_instances();
        self.lock_animation_editor_sessions().clear();
        self.lock_ui_asset_sessions().clear();
        self.lock_ui_asset_dependency_generation().clear();

        session.layout = workspace.workbench;
        session.open_view_instances.clear();
        for instance in workspace.open_view_instances {
            let restored = registry
                .restore_instance(instance)
                .map_err(EditorError::Registry)?;
            session
                .open_view_instances
                .insert(restored.instance_id.clone(), restored);
        }
        let snapshot = self.lock_capability_snapshot().clone();
        let subsystem_report = self.lock_subsystem_report().clone();
        ensure_builtin_shell_instances(&mut registry, &mut session, &snapshot)?;
        let open_instances = session
            .open_view_instances
            .values()
            .cloned()
            .collect::<Vec<_>>();
        repair_builtin_shell_layout(&mut session.layout, &open_instances, &subsystem_report);
        session.focused_view = workspace.focused_view;
        session.active_drawers = workspace.active_drawers;
        self.layout_manager
            .normalize(&mut session.layout, &registry);
        self.recompute_session_metadata(&mut session);
        let ui_asset_instances = session
            .open_view_instances
            .values()
            .filter(|instance| instance.descriptor_id.0 == UI_ASSET_EDITOR_DESCRIPTOR_ID)
            .cloned()
            .collect::<Vec<_>>();
        drop(registry);
        drop(session);
        Ok(ui_asset_instances)
    }

    pub(super) fn apply_project_workspace(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Result<(), EditorError> {
        for instance in self.apply_project_workspace_state(workspace)? {
            self.restore_ui_asset_editor_instance(&instance)?;
        }
        Ok(())
    }

    pub(super) fn project_workspace(&self) -> ProjectEditorWorkspace {
        let session = self.lock_session();
        ProjectEditorWorkspace {
            workbench: session.layout.clone(),
            open_view_instances: session.open_view_instances.values().cloned().collect(),
            focused_view: session.focused_view.clone(),
            active_drawers: session.active_drawers.clone(),
            scene_viewport_sessions: std::collections::BTreeMap::new(),
        }
    }

    pub(super) fn bootstrap_default_layout(&self) -> Result<(), EditorError> {
        self.clear_document_toolkits()?;
        let mut registry = self.lock_view_registry();
        registry.clear_instances();
        self.lock_animation_editor_sessions().clear();
        self.lock_ui_asset_sessions().clear();
        self.lock_ui_asset_dependency_generation().clear();
        let mut session = EditorSessionState::default();
        let snapshot = self.lock_capability_snapshot().clone();
        let subsystem_report = self.lock_subsystem_report().clone();
        ensure_builtin_shell_instances(&mut registry, &mut session, &snapshot)?;
        session.layout = builtin_hybrid_layout_for_subsystems(&subsystem_report);
        self.layout_manager
            .normalize(&mut session.layout, &registry);
        *self.lock_session() = session;

        if let Some(layout) = self.load_global_default_layout() {
            let mut session = self.lock_session();
            session.layout = layout;
            let open_instances = session
                .open_view_instances
                .values()
                .cloned()
                .collect::<Vec<_>>();
            repair_builtin_shell_layout(&mut session.layout, &open_instances, &subsystem_report);
            self.layout_manager
                .normalize(&mut session.layout, &registry);
            self.recompute_session_metadata(&mut session);
        } else {
            let mut session = self.lock_session();
            self.recompute_session_metadata(&mut session);
        }
        Ok(())
    }

    pub(super) fn recompute_session_metadata(&self, session: &mut EditorSessionState) {
        let placements = collect_instance_hosts(&session.layout);
        session
            .open_view_instances
            .retain(|instance_id, _| placements.contains_key(instance_id));
        for (instance_id, host) in placements {
            if let Some(instance) = session.open_view_instances.get_mut(&instance_id) {
                instance.host = host;
            }
        }

        let open_instances = session
            .open_view_instances
            .values()
            .cloned()
            .collect::<Vec<_>>();
        *self.lock_window_registry() =
            EditorWindowRegistry::sync_from_layout(&session.layout, &open_instances);
        session.active_drawers = session
            .layout
            .active_activity_window_drawers()
            .iter()
            .filter_map(|(slot, drawer)| drawer.visible.then_some(*slot))
            .collect();
        if session
            .focused_view
            .as_ref()
            .is_some_and(|instance_id| !session.open_view_instances.contains_key(instance_id))
        {
            session.focused_view = active_main_page_view(session);
        }
        self.lock_animation_editor_sessions()
            .retain(|instance_id, _| session.open_view_instances.contains_key(instance_id));
        let removed_ui_asset_instances = {
            let mut sessions = self.lock_ui_asset_sessions();
            let removed = sessions
                .keys()
                .filter(|instance_id| !session.open_view_instances.contains_key(*instance_id))
                .cloned()
                .collect::<Vec<_>>();
            sessions.retain(|instance_id, _| session.open_view_instances.contains_key(instance_id));
            removed
        };
        let mut dependency_generation = self.lock_ui_asset_dependency_generation();
        for instance_id in removed_ui_asset_instances {
            dependency_generation.remove(&instance_id);
        }
        self.lock_window_host_manager()
            .sync_layout_windows(&session.layout);
    }
}

#[cfg(test)]
#[path = "tests/workspace_state_active_template_query_tests.rs"]
mod workspace_state_active_template_query_tests;

#[cfg(test)]
#[path = "tests/workspace_state_floating_focus_query_tests.rs"]
mod workspace_state_floating_focus_query_tests;

#[cfg(test)]
#[path = "tests/workspace_state_surface_window_query_tests.rs"]
mod workspace_state_surface_window_query_tests;

#[cfg(test)]
#[path = "tests/workspace_state_direct_query_batch_tests.rs"]
mod workspace_state_direct_query_batch_tests;

#[cfg(test)]
#[path = "tests/workspace_state_identity_projection_tests.rs"]
mod workspace_state_identity_projection_tests;
