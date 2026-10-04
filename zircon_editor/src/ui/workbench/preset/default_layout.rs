use std::collections::BTreeMap;

use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowHostMode,
    ActivityWindowId, ActivityWindowLayout, DocumentNode, MainHostPageLayout, MainPageId,
    TabStackLayout, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::{
    EditorFunctionalWindowKind, EditorFunctionalWindowPreset, EditorUiDesignStack,
    EditorWindowDockPolicy,
};

impl EditorUiDesignStack {
    /// Assembles the Material/Fyrox/JetBrains/Unreal preset into the neutral
    /// workbench layout model without touching retained-host or runtime scene state.
    pub fn default_workbench_layout(&self) -> WorkbenchLayout {
        let workbench = self
            .window(EditorFunctionalWindowKind::Workbench)
            .expect("default editor UI design stack must contain Workbench");
        WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: workbench.title.clone(),
                activity_window: ActivityWindowId::workbench(),
            }],
            activity_windows: self
                .window_model
                .windows
                .iter()
                .map(|window| {
                    let layout = self.activity_window_layout(window);
                    (layout.window_id.clone(), layout)
                })
                .collect(),
            floating_windows: Vec::new(),
        }
    }

    pub(super) fn activity_window_layout(
        &self,
        window: &EditorFunctionalWindowPreset,
    ) -> ActivityWindowLayout {
        ActivityWindowLayout {
            window_id: activity_window_id(window.kind),
            descriptor_id: window_descriptor_id(window.kind),
            host_mode: host_mode_for_policy(window.dock_policy),
            activity_drawers: self.drawers_for_window_views(window.kind, &window.drawer_views),
            content_workspace: document_tabs_for_views(window.kind, &window.primary_views),
            menu_overflow_mode: Default::default(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        }
    }

    pub(super) fn drawers_for_window_views(
        &self,
        kind: EditorFunctionalWindowKind,
        views: &[String],
    ) -> BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout> {
        let mut drawers = ActivityDrawerSlot::ALL
            .into_iter()
            .map(|slot| (slot, empty_drawer(slot)))
            .collect::<BTreeMap<_, _>>();

        for view in views {
            let slot = self.drawer_slot_for_view(view);
            let drawer = drawers.entry(slot).or_insert_with(|| empty_drawer(slot));
            drawer
                .tab_stack
                .tabs
                .push(view_instance_id_for_window(kind, view));
        }

        for (slot, drawer) in drawers.iter_mut() {
            if let Some(active) = drawer.tab_stack.tabs.first().cloned() {
                drawer.tab_stack.active_tab = Some(active.clone());
                drawer.active_view = Some(active);
                drawer.mode = self
                    .shell
                    .default_mode_for_slot(*slot)
                    .unwrap_or(ActivityDrawerMode::Pinned);
            }
        }

        drawers
    }

    pub(super) fn drawer_slot_for_view(&self, view: &str) -> ActivityDrawerSlot {
        self.shell
            .drawer_slot_for_view(view)
            .unwrap_or_else(|| drawer_slot_for_view(view))
    }
}

fn host_mode_for_policy(policy: EditorWindowDockPolicy) -> ActivityWindowHostMode {
    match policy {
        EditorWindowDockPolicy::FloatingAllowed => ActivityWindowHostMode::NativeWindowHandle,
        EditorWindowDockPolicy::MainWorkbench
        | EditorWindowDockPolicy::DockedDocument
        | EditorWindowDockPolicy::DrawerBacked => ActivityWindowHostMode::EmbeddedMainFrame,
    }
}

pub(super) fn activity_window_id(kind: EditorFunctionalWindowKind) -> ActivityWindowId {
    if kind == EditorFunctionalWindowKind::Workbench {
        ActivityWindowId::workbench()
    } else {
        ActivityWindowId::new(format!("window:{}", kind.slug()))
    }
}

pub(super) fn window_descriptor_id(kind: EditorFunctionalWindowKind) -> ViewDescriptorId {
    if kind == EditorFunctionalWindowKind::Workbench {
        ViewDescriptorId::new("editor.workbench_window")
    } else {
        ViewDescriptorId::new(format!("editor.{}_window", kind.slug()))
    }
}

fn document_tabs_for_views(kind: EditorFunctionalWindowKind, views: &[String]) -> DocumentNode {
    let tabs = views
        .iter()
        .map(|view| view_instance_id_for_window(kind, view))
        .collect::<Vec<_>>();
    DocumentNode::tabs(TabStackLayout {
        active_tab: tabs.first().cloned(),
        tabs,
    })
}

fn empty_drawer(slot: ActivityDrawerSlot) -> ActivityDrawerLayout {
    let mut drawer = ActivityDrawerLayout::new(slot);
    drawer.tab_stack = TabStackLayout::default();
    drawer.active_view = None;
    drawer.mode = ActivityDrawerMode::Collapsed;
    drawer
}

pub(super) fn drawer_slot_for_view(view: &str) -> ActivityDrawerSlot {
    if view.contains("inspector") || view.contains("metadata") {
        ActivityDrawerSlot::RightTop
    } else if view.contains("console") || view.contains("diagnostics") || view.contains("export") {
        ActivityDrawerSlot::Bottom
    } else if view.contains("plugin") {
        ActivityDrawerSlot::LeftBottom
    } else {
        ActivityDrawerSlot::LeftTop
    }
}

pub(super) fn view_instance_id_for_window(
    kind: EditorFunctionalWindowKind,
    view: &str,
) -> ViewInstanceId {
    if kind == EditorFunctionalWindowKind::Workbench {
        ViewInstanceId::new(format!("{view}#1"))
    } else {
        ViewInstanceId::new(format!("{view}#{}", kind.slug()))
    }
}

#[cfg(test)]
#[path = "tests/default_layout.rs"]
mod tests;
