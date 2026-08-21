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
        let drawers = self.drawers_for_window_views(workbench.kind, &workbench.drawer_views);

        WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: workbench.title.clone(),
                activity_window: ActivityWindowId::workbench(),
                document_workspace: document_tabs_for_views(
                    workbench.kind,
                    &workbench.primary_views,
                ),
            }],
            drawers: drawers.clone(),
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
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
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
    DocumentNode::Tabs(TabStackLayout {
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
mod tests {
    use crate::ui::workbench::layout::{LayoutCommand, LayoutManager};
    use crate::ui::workbench::view::ViewHost;

    use super::*;

    #[test]
    fn default_layout_places_scene_game_documents_and_fyrox_panels() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let layout = stack.default_workbench_layout();

        let [MainHostPageLayout::WorkbenchPage {
            document_workspace, ..
        }] = layout.main_pages.as_slice()
        else {
            panic!("default layout should expose one main workbench page");
        };
        let DocumentNode::Tabs(documents) = document_workspace else {
            panic!("workbench document area should be a tab stack");
        };
        assert_eq!(
            documents.tabs,
            vec![
                ViewInstanceId::new("editor.scene#1"),
                ViewInstanceId::new("editor.game#1")
            ]
        );
        assert_eq!(
            documents.active_tab,
            Some(ViewInstanceId::new("editor.scene#1"))
        );

        assert_drawer_tabs(
            &layout,
            ActivityDrawerSlot::LeftTop,
            &["editor.hierarchy#1", "editor.assets#1"],
        );
        assert_drawer_tabs(
            &layout,
            ActivityDrawerSlot::RightTop,
            &["editor.inspector#1"],
        );
        assert_drawer_tabs(
            &layout,
            ActivityDrawerSlot::Bottom,
            &[
                "editor.console#1",
                "editor.runtime_diagnostics#1",
                "editor.build_export_desktop#1",
            ],
        );
        assert_drawer_tabs(
            &layout,
            ActivityDrawerSlot::LeftBottom,
            &["editor.module_plugins#1"],
        );
        assert_eq!(
            layout.drawers[&ActivityDrawerSlot::LeftTop].mode,
            ActivityDrawerMode::Pinned
        );
        assert_eq!(
            layout.drawers[&ActivityDrawerSlot::LeftBottom].mode,
            ActivityDrawerMode::Collapsed
        );
        assert_eq!(
            layout.activity_windows[&ActivityWindowId::workbench()].activity_drawers
                [&ActivityDrawerSlot::LeftBottom]
                .mode,
            ActivityDrawerMode::Collapsed
        );
    }

    #[test]
    fn default_layout_registers_functional_windows_as_independent_units() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let layout = stack.default_workbench_layout();

        assert_eq!(
            layout.activity_windows.len(),
            stack.window_model.windows.len()
        );

        let material = layout
            .activity_windows
            .get(&ActivityWindowId::new("window:material_editor"))
            .expect("material editor activity window");
        assert_eq!(
            material.host_mode,
            ActivityWindowHostMode::NativeWindowHandle
        );
        let DocumentNode::Tabs(material_tabs) = &material.content_workspace else {
            panic!("material editor should use primary document tabs");
        };
        assert_eq!(
            material_tabs.tabs,
            vec![
                ViewInstanceId::new("editor.material.graph#material_editor"),
                ViewInstanceId::new("editor.material.preview#material_editor")
            ]
        );
        assert_drawer_tabs_in_window(
            material,
            ActivityDrawerSlot::RightTop,
            &["editor.inspector#material_editor"],
        );
        assert_drawer_tabs_in_window(
            material,
            ActivityDrawerSlot::LeftTop,
            &["editor.asset_browser#material_editor"],
        );
    }

    #[test]
    fn preset_layout_supports_drawer_selection_detach_attach_and_focus() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let mut layout = stack.default_workbench_layout();
        let manager = LayoutManager;

        manager
            .apply(
                &mut layout,
                LayoutCommand::ActivateDrawerTab {
                    slot: ActivityDrawerSlot::LeftTop,
                    instance_id: ViewInstanceId::new("editor.assets#1"),
                },
            )
            .unwrap();
        assert_eq!(
            layout.drawers[&ActivityDrawerSlot::LeftTop].active_view,
            Some(ViewInstanceId::new("editor.assets#1"))
        );

        manager
            .apply(
                &mut layout,
                LayoutCommand::DetachViewToWindow {
                    instance_id: ViewInstanceId::new("editor.scene#1"),
                    new_window: MainPageId::new("floating:scene"),
                },
            )
            .unwrap();
        assert_eq!(layout.floating_windows.len(), 1);
        assert_eq!(
            layout.floating_windows[0].focused_view,
            Some(ViewInstanceId::new("editor.scene#1"))
        );

        manager
            .apply(
                &mut layout,
                LayoutCommand::AttachView {
                    instance_id: ViewInstanceId::new("editor.scene#1"),
                    target: ViewHost::Document(MainPageId::workbench(), vec![]),
                    anchor: None,
                },
            )
            .unwrap();
        assert!(layout.floating_windows.is_empty());
        let MainHostPageLayout::WorkbenchPage {
            document_workspace, ..
        } = &layout.main_pages[0]
        else {
            panic!("main page should remain the workbench page");
        };
        assert!(document_workspace.contains(&ViewInstanceId::new("editor.scene#1")));
    }

    #[test]
    fn preset_layout_focus_restores_collapsed_drawer_selection() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let mut layout = stack.default_workbench_layout();
        let manager = LayoutManager;

        assert_eq!(
            layout.drawers[&ActivityDrawerSlot::LeftBottom].mode,
            ActivityDrawerMode::Collapsed
        );

        manager
            .apply(
                &mut layout,
                LayoutCommand::FocusView {
                    instance_id: ViewInstanceId::new("editor.module_plugins#1"),
                },
            )
            .unwrap();

        let root_drawer = &layout.drawers[&ActivityDrawerSlot::LeftBottom];
        assert_eq!(root_drawer.mode, ActivityDrawerMode::Pinned);
        assert_eq!(
            root_drawer.active_view,
            Some(ViewInstanceId::new("editor.module_plugins#1"))
        );
        let activity_drawer = &layout.activity_windows[&ActivityWindowId::workbench()]
            .activity_drawers[&ActivityDrawerSlot::LeftBottom];
        assert_eq!(activity_drawer.mode, ActivityDrawerMode::Pinned);
        assert_eq!(
            activity_drawer.active_view,
            Some(ViewInstanceId::new("editor.module_plugins#1"))
        );
    }

    fn assert_drawer_tabs(layout: &WorkbenchLayout, slot: ActivityDrawerSlot, expected: &[&str]) {
        assert_drawer_tabs_in_window(
            &layout.activity_windows[&ActivityWindowId::workbench()],
            slot,
            expected,
        );
        assert_drawer_tabs_in_map(&layout.drawers, slot, expected);
    }

    fn assert_drawer_tabs_in_window(
        window: &ActivityWindowLayout,
        slot: ActivityDrawerSlot,
        expected: &[&str],
    ) {
        assert_drawer_tabs_in_map(&window.activity_drawers, slot, expected);
    }

    fn assert_drawer_tabs_in_map(
        drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
        slot: ActivityDrawerSlot,
        expected: &[&str],
    ) {
        let expected = expected
            .iter()
            .copied()
            .map(ViewInstanceId::new)
            .collect::<Vec<_>>();
        assert_eq!(drawers[&slot].tab_stack.tabs, expected);
    }
}
