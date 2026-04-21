use crate::ui::workbench::layout::{
    ActivityDrawerSlot, DocumentNode, MainHostPageLayout, MainPageId, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

pub(crate) fn drop_host_for_group(
    layout: &WorkbenchLayout,
    target_group: &str,
) -> Option<ViewHost> {
    match target_group {
        "left" => Some(ViewHost::Drawer(preferred_drawer_slot(
            layout,
            &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
            ActivityDrawerSlot::LeftTop,
        ))),
        "right" => Some(ViewHost::Drawer(preferred_drawer_slot(
            layout,
            &[
                ActivityDrawerSlot::RightTop,
                ActivityDrawerSlot::RightBottom,
            ],
            ActivityDrawerSlot::RightTop,
        ))),
        "bottom" => Some(ViewHost::Drawer(preferred_drawer_slot(
            layout,
            &[
                ActivityDrawerSlot::BottomLeft,
                ActivityDrawerSlot::BottomRight,
            ],
            ActivityDrawerSlot::BottomLeft,
        ))),
        "document" => {
            preferred_document_page(layout).map(|page_id| ViewHost::Document(page_id, Vec::new()))
        }
        _ => None,
    }
}

pub(crate) fn drop_host_for_tab(
    layout: &WorkbenchLayout,
    instance_id: &str,
    target_group: &str,
) -> Option<ViewHost> {
    let current_host = find_instance_host(layout, &ViewInstanceId::new(instance_id));
    if current_host
        .as_ref()
        .and_then(host_group)
        .is_some_and(|group| group == target_group)
    {
        return current_host;
    }

    drop_host_for_group(layout, target_group)
}

pub(super) fn preferred_document_page(layout: &WorkbenchLayout) -> Option<MainPageId> {
    if layout.main_pages.iter().any(|page| {
        matches!(
            page,
            MainHostPageLayout::WorkbenchPage { id, .. } if id == &layout.active_main_page
        )
    }) {
        return Some(layout.active_main_page.clone());
    }

    layout.main_pages.iter().find_map(|page| match page {
        MainHostPageLayout::WorkbenchPage { id, .. } => Some(id.clone()),
        MainHostPageLayout::ExclusiveActivityWindowPage { .. } => None,
    })
}

pub(super) fn active_floating_window_path(
    layout: &WorkbenchLayout,
    window_id: &MainPageId,
) -> Option<Vec<usize>> {
    let window = layout
        .floating_windows
        .iter()
        .find(|window| &window.window_id == window_id)?;
    window
        .focused_view
        .as_ref()
        .and_then(|instance_id| find_document_path(&window.workspace, instance_id))
        .or_else(|| preferred_workspace_path(&window.workspace))
}

fn preferred_drawer_slot(
    layout: &WorkbenchLayout,
    slots: &[ActivityDrawerSlot],
    fallback: ActivityDrawerSlot,
) -> ActivityDrawerSlot {
    slots
        .iter()
        .copied()
        .find(|slot| {
            layout
                .drawers
                .get(slot)
                .is_some_and(|drawer| drawer.visible && drawer.active_view.is_some())
        })
        .or_else(|| {
            slots.iter().copied().find(|slot| {
                layout
                    .drawers
                    .get(slot)
                    .is_some_and(|drawer| drawer.visible && drawer.tab_stack.active_tab.is_some())
            })
        })
        .or_else(|| {
            slots.iter().copied().find(|slot| {
                layout
                    .drawers
                    .get(slot)
                    .is_some_and(|drawer| drawer.visible && !drawer.tab_stack.tabs.is_empty())
            })
        })
        .or_else(|| {
            slots.iter().copied().find(|slot| {
                layout
                    .drawers
                    .get(slot)
                    .is_some_and(|drawer| drawer.visible)
            })
        })
        .unwrap_or(fallback)
}

fn preferred_workspace_path(node: &DocumentNode) -> Option<Vec<usize>> {
    fn visit(node: &DocumentNode, path: &mut Vec<usize>) -> Option<Vec<usize>> {
        match node {
            DocumentNode::Tabs(stack) => {
                (stack.active_tab.is_some() || !stack.tabs.is_empty()).then(|| path.clone())
            }
            DocumentNode::SplitNode { first, second, .. } => {
                path.push(0);
                let first_result = visit(first, path);
                path.pop();
                if first_result.is_some() {
                    return first_result;
                }

                path.push(1);
                let second_result = visit(second, path);
                path.pop();
                second_result
            }
        }
    }

    visit(node, &mut Vec::new())
}

fn find_instance_host(layout: &WorkbenchLayout, instance_id: &ViewInstanceId) -> Option<ViewHost> {
    for (slot, drawer) in &layout.drawers {
        if drawer.tab_stack.tabs.contains(instance_id) {
            return Some(ViewHost::Drawer(*slot));
        }
    }

    for page in &layout.main_pages {
        match page {
            MainHostPageLayout::WorkbenchPage {
                id,
                document_workspace,
                ..
            } => {
                if let Some(path) = find_document_path(document_workspace, instance_id) {
                    return Some(ViewHost::Document(id.clone(), path));
                }
            }
            MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                window_instance,
                ..
            } if window_instance == instance_id => {
                return Some(ViewHost::ExclusivePage(id.clone()));
            }
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => {}
        }
    }

    for window in &layout.floating_windows {
        if let Some(path) = find_document_path(&window.workspace, instance_id) {
            return Some(ViewHost::FloatingWindow(window.window_id.clone(), path));
        }
    }

    None
}

fn find_document_path(node: &DocumentNode, instance_id: &ViewInstanceId) -> Option<Vec<usize>> {
    fn visit(
        node: &DocumentNode,
        instance_id: &ViewInstanceId,
        path: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        match node {
            DocumentNode::Tabs(stack) => stack.tabs.contains(instance_id).then(|| path.clone()),
            DocumentNode::SplitNode { first, second, .. } => {
                path.push(0);
                let first_result = visit(first, instance_id, path);
                path.pop();
                if first_result.is_some() {
                    return first_result;
                }

                path.push(1);
                let second_result = visit(second, instance_id, path);
                path.pop();
                second_result
            }
        }
    }

    visit(node, instance_id, &mut Vec::new())
}

fn host_group(host: &ViewHost) -> Option<&'static str> {
    match host {
        ViewHost::Drawer(ActivityDrawerSlot::LeftTop | ActivityDrawerSlot::LeftBottom) => {
            Some("left")
        }
        ViewHost::Drawer(ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom) => {
            Some("right")
        }
        ViewHost::Drawer(ActivityDrawerSlot::BottomLeft | ActivityDrawerSlot::BottomRight) => {
            Some("bottom")
        }
        ViewHost::Document(..) => Some("document"),
        ViewHost::FloatingWindow(..) | ViewHost::ExclusivePage(..) => None,
    }
}
