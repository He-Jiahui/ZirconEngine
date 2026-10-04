use std::collections::{BTreeMap, HashMap};

use crate::ui::workbench::layout::{ActivityWindowId, MainHostPageLayout, WorkbenchLayout};
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId,
};

use super::super::workbench::{
    resolve_document_workspace, resolve_view_tab, ActivityDrawerSnapshot, FloatingWindowSnapshot,
    MainPageSnapshot, WorkbenchSnapshot,
};
use super::{EditorChromeSnapshot, EditorDataSnapshot};

impl EditorChromeSnapshot {
    /// 本轮data联接layout与registry；缺失view保持tab位置并投影明确占位诊断。
    pub fn build(
        data: EditorDataSnapshot,
        layout: &WorkbenchLayout,
        instances: Vec<ViewInstance>,
        descriptors: Vec<ViewDescriptor>,
        focused_view: Option<&ViewInstanceId>,
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
        let menu_overflow_mode = active_menu_overflow_mode(layout);
        let focused_document_kind = focused_view
            .and_then(|instance_id| instances_by_id.get(instance_id))
            .and_then(|instance| descriptors_by_id.get(&instance.descriptor_id))
            .and_then(|descriptor| descriptor.document_kind.clone());

        // TODO: [CR-EDITOR-WORKBENCH-0006] 业务快照携bridge诊断矩阵，但此chrome投影没有字段；确认应向诊断pane发布，还是由独立只读入口消费。
        Self {
            focused_document_kind,
            workbench: WorkbenchSnapshot {
                active_main_page: layout.active_main_page.clone(),
                main_pages,
                drawers,
                floating_windows,
            },
            scene_entries: data.scene_entries,
            inspector: data.inspector,
            status_line: data.status_line,
            console_output: data.console_output,
            status_task_progress: data.status_task_progress,
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
            menu_overflow_mode,
        }
    }
}

fn active_menu_overflow_mode(
    layout: &WorkbenchLayout,
) -> crate::ui::workbench::window_registry::MenuOverflowMode {
    let Some(active_window_id) = layout.active_activity_window_id() else {
        return Default::default();
    };
    layout
        .activity_windows()
        .get(&active_window_id)
        .map(|window| window.menu_overflow_mode)
        .unwrap_or_default()
}

/// 仅当前活动activity window的抽屉进入chrome，不混入其它窗口状态。
fn build_drawers(
    layout: &WorkbenchLayout,
    instances: &HashMap<ViewInstanceId, ViewInstance>,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
) -> BTreeMap<crate::ui::workbench::layout::ActivityDrawerSlot, ActivityDrawerSnapshot> {
    let Some(active_window_id) = layout.active_activity_window_id() else {
        return BTreeMap::new();
    };
    let activity_windows = layout.activity_windows();
    let Some(window) = activity_windows.get(&active_window_id) else {
        return BTreeMap::new();
    };

    window
        .activity_drawers
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

/// 保持Workbench/Exclusive页面身份与顺序，缺失工作区使用空document树。
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
                activity_window,
            } => MainPageSnapshot::Workbench {
                id: id.clone(),
                title: title.clone(),
                activity_window: activity_window.clone(),
                activity_window_template: activity_window_template(
                    layout,
                    descriptors,
                    activity_window,
                ),
                workspace: layout
                    .content_workspace_for_page(id)
                    .map(|workspace| resolve_document_workspace(workspace, instances, descriptors))
                    .unwrap_or_else(|| {
                        resolve_document_workspace(
                            &crate::ui::workbench::layout::DocumentNode::default(),
                            instances,
                            descriptors,
                        )
                    }),
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

/// activity window的模板来自其descriptor，缺注册声明时保留无模板回退。
fn activity_window_template(
    layout: &WorkbenchLayout,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
    activity_window: &ActivityWindowId,
) -> Option<ActivityWindowTemplateSpec> {
    let windows = layout.activity_windows();
    let window = windows.get(activity_window)?;
    descriptors
        .get(&window.descriptor_id)
        .and_then(|descriptor| descriptor.activity_window_template.clone())
}

/// 浮层保留稳定ID、文档树与请求位置；实际frame由autolayout限制。
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
            requested_frame: window.frame,
            workspace: resolve_document_workspace(&window.workspace, instances, descriptors),
            focused_view: window.focused_view.clone(),
        })
        .collect()
}
