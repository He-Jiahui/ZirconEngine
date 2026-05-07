use crate::ui::workbench::autolayout::ShellFrame;

use super::super::{
    ActivityDrawerMode, DocumentNode, FloatingWindowLayout, LayoutCommand, LayoutDiff,
    LayoutManager, SplitPlacement, TabStackLayout, WorkbenchLayout,
};

impl LayoutManager {
    pub fn apply(
        &self,
        layout: &mut WorkbenchLayout,
        cmd: LayoutCommand,
    ) -> Result<LayoutDiff, String> {
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
                    .ok_or_else(|| format!("missing workspace path for {:?}", workspace))?;
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
                    .ok_or_else(|| format!("missing split path for {:?}", workspace))?;
                let DocumentNode::SplitNode {
                    ratio: current_ratio,
                    ..
                } = node
                else {
                    return Err("target path is not a split node".to_string());
                };
                *current_ratio = ratio.clamp(0.1, 0.9);
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::SetDrawerMode { slot, mode } => {
                let slot = slot.canonical();
                let drawer = layout
                    .active_activity_window_mut()
                    .and_then(|window| window.activity_drawers.get_mut(&slot))
                    .ok_or_else(|| format!("missing drawer {:?}", slot))?;
                drawer.mode = mode;
                if mode == ActivityDrawerMode::Collapsed {
                    drawer.tab_stack.active_tab = None;
                    drawer.active_view = None;
                }
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::SetDrawerExtent { slot, extent } => {
                let slot = slot.canonical();
                let extent = extent.max(120.0);
                let drawer = layout
                    .active_activity_window_mut()
                    .and_then(|window| window.activity_drawers.get_mut(&slot))
                    .ok_or_else(|| format!("missing drawer {:?}", slot))?;
                drawer.extent = extent;
                Ok(LayoutDiff { changed: true })
            }
            LayoutCommand::ActivateDrawerTab { slot, instance_id } => {
                let slot = slot.canonical();
                let drawer = layout
                    .active_activity_window_mut()
                    .and_then(|window| window.activity_drawers.get_mut(&slot))
                    .ok_or_else(|| format!("missing drawer {:?}", slot))?;
                if drawer.tab_stack.tabs.contains(&instance_id) {
                    drawer.tab_stack.active_tab = Some(instance_id.clone());
                    drawer.active_view = Some(instance_id);
                    if drawer.mode == ActivityDrawerMode::Collapsed {
                        drawer.mode = ActivityDrawerMode::Pinned;
                    }
                    Ok(LayoutDiff { changed: true })
                } else {
                    Err("drawer does not contain target tab".to_string())
                }
            }
            LayoutCommand::ActivateMainPage { page_id } => {
                layout.active_main_page = page_id;
                Ok(LayoutDiff { changed: true })
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
