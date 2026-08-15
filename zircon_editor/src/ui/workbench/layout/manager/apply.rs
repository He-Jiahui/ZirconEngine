use crate::ui::workbench::autolayout::ShellFrame;

use super::super::{
    ActivityDrawerMode, DocumentNode, FloatingWindowLayout, LayoutCommand, LayoutCommandError,
    LayoutDiff, LayoutManager, SplitPlacement, TabStackLayout, WorkbenchLayout,
};

impl LayoutManager {
    pub fn apply(
        &self,
        layout: &mut WorkbenchLayout,
        cmd: LayoutCommand,
    ) -> Result<LayoutDiff, LayoutCommandError> {
        let result = match cmd {
            LayoutCommand::OpenView {
                instance_id,
                target,
            }
            | LayoutCommand::MoveView {
                instance_id,
                target,
            } => {
                self.detach_instance(layout, &instance_id);
                self.attach_instance(layout, instance_id, target, None)?;
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::AttachView {
                instance_id,
                target,
                anchor,
            } => {
                self.detach_instance(layout, &instance_id);
                self.attach_instance(layout, instance_id, target, anchor)?;
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::CloseView { instance_id } => Ok(LayoutDiff {
                changed: self.detach_instance(layout, &instance_id),
            }),
            LayoutCommand::FocusView { instance_id } => Ok(LayoutDiff {
                changed: self.focus_instance(layout, &instance_id),
            }),
            LayoutCommand::DetachViewToWindow {
                instance_id,
                new_window,
            } => {
                self.detach_instance(layout, &instance_id);
                if let Some(window) = layout
                    .floating_windows
                    .iter_mut()
                    .find(|window| window.window_id == new_window)
                {
                    append_instance_to_floating_workspace(
                        &mut window.workspace,
                        instance_id.clone(),
                    );
                    window.focused_view = Some(instance_id);
                    Ok(LayoutDiff { changed: true })
                } else {
                    layout.floating_windows.push(FloatingWindowLayout {
                        window_id: new_window.clone(),
                        title: format!("Window {}", new_window.0),
                        workspace: DocumentNode::Tabs(TabStackLayout {
                            tabs: vec![instance_id.clone()],
                            active_tab: Some(instance_id.clone()),
                        }),
                        focused_view: Some(instance_id),
                        frame: ShellFrame::default(),
                    });
                    Ok(LayoutDiff { changed: true })
                }
            }
            LayoutCommand::CreateSplit {
                workspace,
                path,
                axis,
                placement,
                new_instance,
            } => {
                self.detach_instance(layout, &new_instance);
                let node = self
                    .workspace_node_mut(layout, &workspace, &path)
                    .ok_or_else(|| LayoutCommandError::MissingWorkspacePath {
                        workspace: workspace.clone(),
                        path: path.clone(),
                    })?;
                let previous = node.clone();
                let inserted = DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![new_instance.clone()],
                    active_tab: Some(new_instance),
                });
                let (first, second) = match placement {
                    SplitPlacement::Before => (inserted, previous),
                    SplitPlacement::After => (previous, inserted),
                };
                *node = DocumentNode::SplitNode {
                    axis,
                    ratio: 0.5,
                    first: Box::new(first),
                    second: Box::new(second),
                };
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::ResizeSplit {
                workspace,
                path,
                ratio,
            } => {
                let node = self
                    .workspace_node_mut(layout, &workspace, &path)
                    .ok_or_else(|| LayoutCommandError::MissingSplitPath {
                        workspace: workspace.clone(),
                        path: path.clone(),
                    })?;
                let DocumentNode::SplitNode {
                    ratio: current_ratio,
                    ..
                } = node
                else {
                    return Err(LayoutCommandError::TargetPathIsNotSplitNode { workspace, path });
                };
                let ratio = ratio.clamp(0.1, 0.9);
                let changed = *current_ratio != ratio;
                if changed {
                    *current_ratio = ratio;
                }
                Ok(LayoutDiff { changed })
            }
            LayoutCommand::SetDrawerMode { slot, mode } => {
                let slot = slot.canonical();
                let window = layout
                    .active_activity_window_mut()
                    .ok_or(LayoutCommandError::MissingDrawer { slot })?;
                if !window.activity_drawers.contains_key(&slot) {
                    return Err(LayoutCommandError::MissingDrawer { slot });
                }
                let mut changed = mode != ActivityDrawerMode::Collapsed
                    && window.collapse_drawer_region_siblings(slot);
                let drawer = window
                    .activity_drawers
                    .get_mut(&slot)
                    .expect("validated drawer must remain present");
                changed |= drawer.mode != mode
                    || (mode == ActivityDrawerMode::Collapsed
                        && (drawer.tab_stack.active_tab.is_some() || drawer.active_view.is_some()));
                if !changed {
                    return Ok(LayoutDiff { changed: false });
                }
                drawer.mode = mode;
                if mode == ActivityDrawerMode::Collapsed {
                    drawer.tab_stack.active_tab = None;
                    drawer.active_view = None;
                }
                Ok(LayoutDiff { changed })
            }
            LayoutCommand::SetDrawerExtent { slot, extent } => {
                let slot = slot.canonical();
                let extent = extent.max(120.0);
                let drawer = layout
                    .active_activity_window_mut()
                    .and_then(|window| window.activity_drawers.get_mut(&slot))
                    .ok_or(LayoutCommandError::MissingDrawer { slot })?;
                let changed = drawer.extent != extent;
                if changed {
                    drawer.extent = extent;
                }
                Ok(LayoutDiff { changed })
            }
            LayoutCommand::ActivateDrawerTab { slot, instance_id } => {
                let slot = slot.canonical();
                let window = layout
                    .active_activity_window_mut()
                    .ok_or(LayoutCommandError::MissingDrawer { slot })?;
                let drawer = window
                    .activity_drawers
                    .get(&slot)
                    .ok_or(LayoutCommandError::MissingDrawer { slot })?;
                if !drawer.tab_stack.tabs.contains(&instance_id) {
                    return Err(LayoutCommandError::DrawerMissingTab { slot, instance_id });
                }

                let mut changed = window.collapse_drawer_region_siblings(slot);

                let drawer = window
                    .activity_drawers
                    .get_mut(&slot)
                    .expect("validated drawer must remain present");
                changed |= drawer.tab_stack.active_tab.as_ref() != Some(&instance_id)
                    || drawer.active_view.as_ref() != Some(&instance_id)
                    || drawer.mode == ActivityDrawerMode::Collapsed;
                drawer.tab_stack.active_tab = Some(instance_id.clone());
                drawer.active_view = Some(instance_id);
                if drawer.mode == ActivityDrawerMode::Collapsed {
                    drawer.mode = ActivityDrawerMode::Pinned;
                }
                Ok(LayoutDiff { changed })
            }
            LayoutCommand::ActivateMainPage { page_id } => {
                let changed = layout.active_main_page != page_id;
                if changed {
                    layout.active_main_page = page_id;
                }
                Ok(LayoutDiff { changed })
            }
            LayoutCommand::SavePreset { .. } | LayoutCommand::LoadPreset { .. } => {
                Ok(LayoutDiff { changed: false })
            }
            LayoutCommand::ResetToDefault => {
                *layout = self.default_layout();
                Ok(LayoutDiff { changed: true })
            }
        };

        if matches!(&result, Ok(diff) if diff.changed) {
            normalize_drawer_active_selection(layout);
            layout.sync_legacy_drawers_from_active_activity_window();
        }

        result
    }
}

fn append_instance_to_floating_workspace(
    node: &mut DocumentNode,
    instance_id: crate::ui::workbench::view::ViewInstanceId,
) {
    match node {
        DocumentNode::Tabs(stack) => stack.insert(instance_id, None),
        DocumentNode::SplitNode { first, .. } => {
            append_instance_to_floating_workspace(first, instance_id);
        }
    }
}

fn normalize_drawer_active_selection(layout: &mut WorkbenchLayout) {
    for activity_window in layout.activity_windows.values_mut() {
        for drawer in activity_window.activity_drawers.values_mut() {
            if drawer.mode == ActivityDrawerMode::Collapsed || drawer.tab_stack.tabs.is_empty() {
                drawer.tab_stack.active_tab = None;
                drawer.active_view = None;
                continue;
            }

            let active = drawer
                .tab_stack
                .active_tab
                .clone()
                .filter(|instance_id| drawer.tab_stack.tabs.contains(instance_id))
                .or_else(|| {
                    drawer
                        .active_view
                        .clone()
                        .filter(|instance_id| drawer.tab_stack.tabs.contains(instance_id))
                });
            drawer.tab_stack.active_tab = active.clone();
            drawer.active_view = active;
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use crate::ui::workbench::layout::{
        ActivityDrawerSlot, LayoutCommand, LayoutManager, MainPageId, WorkbenchLayout,
    };
    use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

    #[test]
    fn repeated_layout_commands_report_unchanged() {
        let manager = LayoutManager::default();
        let mut layout = WorkbenchLayout::default();

        assert!(
            !manager
                .apply(
                    &mut layout,
                    LayoutCommand::SetDrawerMode {
                        slot: ActivityDrawerSlot::LeftTop,
                        mode: ActivityDrawerMode::Pinned,
                    },
                )
                .expect("drawer mode")
                .changed
        );
        assert!(
            !manager
                .apply(
                    &mut layout,
                    LayoutCommand::SetDrawerExtent {
                        slot: ActivityDrawerSlot::LeftTop,
                        extent: 260.0,
                    },
                )
                .expect("drawer extent")
                .changed
        );
        assert!(
            !manager
                .apply(
                    &mut layout,
                    LayoutCommand::ActivateMainPage {
                        page_id: MainPageId::workbench(),
                    },
                )
                .expect("main page")
                .changed
        );

        let instance_id = ViewInstanceId::new("editor.scene#performance");
        manager
            .apply(
                &mut layout,
                LayoutCommand::OpenView {
                    instance_id: instance_id.clone(),
                    target: ViewHost::Document(MainPageId::workbench(), Vec::new()),
                },
            )
            .expect("open view");
        assert!(
            !manager
                .apply(&mut layout, LayoutCommand::FocusView { instance_id },)
                .expect("repeat focus")
                .changed
        );
    }

    #[test]
    fn activating_a_drawer_collapses_the_other_drawer_in_the_same_region() {
        let manager = LayoutManager::default();
        let mut layout = WorkbenchLayout::default();
        let hierarchy = ViewInstanceId::new("editor.hierarchy#region");
        let plugins = ViewInstanceId::new("editor.module_plugins#region");
        let window = layout
            .active_activity_window_mut()
            .expect("active workbench window");

        let left_top = window
            .activity_drawers
            .get_mut(&ActivityDrawerSlot::LeftTop)
            .expect("left-top drawer");
        left_top.tab_stack.tabs = vec![hierarchy.clone()];
        left_top.tab_stack.active_tab = Some(hierarchy.clone());
        left_top.active_view = Some(hierarchy.clone());
        left_top.mode = ActivityDrawerMode::Pinned;

        let left_bottom = window
            .activity_drawers
            .get_mut(&ActivityDrawerSlot::LeftBottom)
            .expect("left-bottom drawer");
        left_bottom.tab_stack.tabs = vec![plugins.clone()];
        left_bottom.mode = ActivityDrawerMode::Collapsed;

        assert!(
            manager
                .apply(
                    &mut layout,
                    LayoutCommand::ActivateDrawerTab {
                        slot: ActivityDrawerSlot::LeftBottom,
                        instance_id: plugins.clone(),
                    },
                )
                .expect("activate left-bottom drawer")
                .changed
        );

        let drawers = layout.active_activity_window_drawers();
        let left_top = &drawers[&ActivityDrawerSlot::LeftTop];
        assert_eq!(left_top.mode, ActivityDrawerMode::Collapsed);
        assert_eq!(left_top.tab_stack.active_tab, None);
        assert_eq!(left_top.active_view, None);

        let left_bottom = &drawers[&ActivityDrawerSlot::LeftBottom];
        assert_eq!(left_bottom.mode, ActivityDrawerMode::Pinned);
        assert_eq!(left_bottom.tab_stack.active_tab, Some(plugins.clone()));
        assert_eq!(left_bottom.active_view, Some(plugins));

        assert!(
            manager
                .apply(
                    &mut layout,
                    LayoutCommand::FocusView {
                        instance_id: hierarchy.clone(),
                    },
                )
                .expect("focus left-top drawer")
                .changed
        );
        let drawers = layout.active_activity_window_drawers();
        assert_eq!(
            drawers[&ActivityDrawerSlot::LeftTop].active_view,
            Some(hierarchy)
        );
        assert_eq!(
            drawers[&ActivityDrawerSlot::LeftBottom].mode,
            ActivityDrawerMode::Collapsed
        );
    }
}
