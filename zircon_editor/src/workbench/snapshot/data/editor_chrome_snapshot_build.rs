use std::collections::{BTreeMap, HashMap};

use crate::layout::{MainHostPageLayout, WorkbenchLayout};
use crate::view::{ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId};

use super::super::workbench::{
    resolve_document_workspace, resolve_view_tab, ActivityDrawerSnapshot, FloatingWindowSnapshot,
    MainPageSnapshot, WorkbenchSnapshot,
};
use super::{EditorChromeSnapshot, EditorDataSnapshot};

impl EditorChromeSnapshot {
    pub fn build(
        data: EditorDataSnapshot,
        layout: &WorkbenchLayout,
        instances: Vec<ViewInstance>,
        descriptors: Vec<ViewDescriptor>,
    ) -> Self {
        let instances_by_id: HashMap<ViewInstanceId, ViewInstance> = instances
            .into_iter()
            .map(|instance| (instance.instance_id.clone(), instance))
            .collect();
        let descriptors_by_id: HashMap<ViewDescriptorId, ViewDescriptor> = descriptors
            .into_iter()
            .map(|descriptor| (descriptor.descriptor_id.clone(), descriptor))
            .collect();

        let drawers = build_drawers(layout, &instances_by_id, &descriptors_by_id);
        let main_pages = build_main_pages(layout, &instances_by_id, &descriptors_by_id);
        let floating_windows = build_floating_windows(layout, &instances_by_id, &descriptors_by_id);

        Self {
            workbench: WorkbenchSnapshot {
                active_main_page: layout.active_main_page.clone(),
                main_pages,
                drawers,
                floating_windows,
            },
            scene_entries: data.scene_entries,
            inspector: data.inspector,
            status_line: data.status_line,
            hovered_axis: data.hovered_axis,
            viewport_size: data.viewport_size,
            scene_viewport_settings: data.scene_viewport_settings,
            mesh_import_path: data.mesh_import_path,
            project_overview: data.project_overview,
            asset_activity: data.asset_activity,
            asset_browser: data.asset_browser,
            project_path: data.project_path,
            session_mode: data.session_mode,
            welcome: data.welcome,
            project_open: data.project_open,
            can_undo: data.can_undo,
            can_redo: data.can_redo,
        }
    }
}

fn build_drawers(
    layout: &WorkbenchLayout,
    instances: &HashMap<ViewInstanceId, ViewInstance>,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
) -> BTreeMap<crate::layout::ActivityDrawerSlot, ActivityDrawerSnapshot> {
    layout
        .drawers
        .iter()
        .map(|(slot, drawer)| {
            (
                *slot,
                ActivityDrawerSnapshot {
                    slot: *slot,
                    tabs: drawer
                        .tab_stack
                        .tabs
                        .iter()
                        .map(|instance_id| resolve_view_tab(instance_id, instances, descriptors))
                        .collect(),
                    active_tab: drawer.tab_stack.active_tab.clone(),
                    active_view: drawer.active_view.clone(),
                    mode: drawer.mode,
                    extent: drawer.extent,
                    visible: drawer.visible,
                },
            )
        })
        .collect()
}

fn build_main_pages(
    layout: &WorkbenchLayout,
    instances: &HashMap<ViewInstanceId, ViewInstance>,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
) -> Vec<MainPageSnapshot> {
    layout
        .main_pages
        .iter()
        .map(|page| match page {
            MainHostPageLayout::WorkbenchPage {
                id,
                title,
                document_workspace,
            } => MainPageSnapshot::Workbench {
                id: id.clone(),
                title: title.clone(),
                workspace: resolve_document_workspace(document_workspace, instances, descriptors),
            },
            MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                title,
                window_instance,
            } => MainPageSnapshot::Exclusive {
                id: id.clone(),
                title: title.clone(),
                view: resolve_view_tab(window_instance, instances, descriptors),
            },
        })
        .collect()
}

fn build_floating_windows(
    layout: &WorkbenchLayout,
    instances: &HashMap<ViewInstanceId, ViewInstance>,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
) -> Vec<FloatingWindowSnapshot> {
    layout
        .floating_windows
        .iter()
        .map(|window| FloatingWindowSnapshot {
            window_id: window.window_id.clone(),
            title: window.title.clone(),
            workspace: resolve_document_workspace(&window.workspace, instances, descriptors),
            focused_view: window.focused_view.clone(),
        })
        .collect()
}
