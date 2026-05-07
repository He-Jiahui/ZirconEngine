use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_center_band_frame,
    resolve_root_document_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame,
};
use crate::ui::workbench::autolayout::{ShellFrame, ShellRegionId, WorkbenchChromeMetrics};
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, TabInsertionAnchor, TabInsertionSide, WorkspaceTarget,
};
use crate::ui::workbench::model::{
    DocumentTabModel, PaneTabModel, ToolWindowStackModel, WorkbenchViewModel,
};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};
use zircon_runtime_interface::ui::layout::UiFrame;

use super::resolved_drop::ResolvedTabDrop;
use super::tab_width::{estimate_dock_tab_width, estimate_document_tab_width};

#[derive(Clone)]
struct StripTabEntry {
    instance_id: ViewInstanceId,
    title: String,
    closeable: bool,
    host: ViewHost,
}

#[derive(Clone, Copy)]
enum StripStyle {
    Dock,
    Document,
}

struct TabStripHitBox {
    style: StripStyle,
    start_x: f32,
    end_x: f32,
    y: f32,
    height: f32,
    spacing: f32,
    tabs: Vec<StripTabEntry>,
}

pub(super) fn precise_drop_target(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    dragging_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ResolvedTabDrop> {
    let strip = strip_hit_box(model, metrics, target_group, shared_root_frames)?;
    if pointer_y < strip.y
        || pointer_y > strip.y + strip.height
        || pointer_x < strip.start_x
        || pointer_x > strip.end_x
    {
        return None;
    }

    let tabs: Vec<_> = strip
        .tabs
        .into_iter()
        .filter(|tab| tab.instance_id.0 != dragging_id)
        .collect();
    if tabs.is_empty() {
        return None;
    }

    let mut cursor_x = strip.start_x;
    for tab in &tabs {
        let width = match strip.style {
            StripStyle::Dock => estimate_dock_tab_width(&tab.title),
            StripStyle::Document => estimate_document_tab_width(&tab.title, tab.closeable),
        };
        let tab_end = cursor_x + width;
        let midpoint = cursor_x + width / 2.0;
        if pointer_x <= tab_end {
            return Some(ResolvedTabDrop {
                host: tab.host.clone(),
                anchor: Some(TabInsertionAnchor {
                    target_id: tab.instance_id.clone(),
                    side: if pointer_x < midpoint {
                        TabInsertionSide::Before
                    } else {
                        TabInsertionSide::After
                    },
                }),
            });
        }
        let gap_end = tab_end + strip.spacing;
        if pointer_x < gap_end {
            return Some(ResolvedTabDrop {
                host: tab.host.clone(),
                anchor: Some(TabInsertionAnchor {
                    target_id: tab.instance_id.clone(),
                    side: TabInsertionSide::After,
                }),
            });
        }
        cursor_x = gap_end;
    }

    let last = tabs.last()?;
    Some(ResolvedTabDrop {
        host: last.host.clone(),
        anchor: Some(TabInsertionAnchor {
            target_id: last.instance_id.clone(),
            side: TabInsertionSide::After,
        }),
    })
}

fn strip_hit_box(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    target_group: &str,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<TabStripHitBox> {
    match target_group {
        "left" => tool_strip_hit_box(
            model,
            metrics,
            &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
            ShellRegionId::Left,
            true,
            shared_root_frames,
        ),
        "right" => tool_strip_hit_box(
            model,
            metrics,
            &[
                ActivityDrawerSlot::RightTop,
                ActivityDrawerSlot::RightBottom,
            ],
            ShellRegionId::Right,
            false,
            shared_root_frames,
        ),
        "bottom" => bottom_strip_hit_box(model, metrics, shared_root_frames),
        "document" => document_strip_hit_box(model, shared_root_frames),
        _ => None,
    }
}

fn tool_strip_hit_box(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    slots: &[ActivityDrawerSlot],
    region: ShellRegionId,
    rail_on_left: bool,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<TabStripHitBox> {
    if !group_expanded(model, slots) {
        return None;
    }

    let shared_frame = match region {
        ShellRegionId::Left => resolve_root_left_region_frame(shared_root_frames),
        ShellRegionId::Right => resolve_root_right_region_frame(shared_root_frames),
        ShellRegionId::Bottom => resolve_root_bottom_region_frame(shared_root_frames),
        ShellRegionId::Document => resolve_root_document_region_frame(shared_root_frames),
    };
    let frame = shared_frame;
    if frame.width <= 0.0 {
        return None;
    }

    let tabs = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack
                .tabs
                .iter()
                .map(move |tab| strip_tab_from_pane(tab, stack))
        })
        .collect::<Vec<_>>();
    if tabs.is_empty() {
        return None;
    }

    let start_x = if rail_on_left {
        frame.x + metrics.rail_width + metrics.separator_thickness + 6.0
    } else {
        frame.x + 6.0
    };
    let end_x = if rail_on_left {
        frame.right() - 6.0
    } else {
        frame.right() - metrics.rail_width - metrics.separator_thickness - 6.0
    };
    Some(TabStripHitBox {
        style: StripStyle::Dock,
        start_x,
        end_x,
        y: frame.y + 2.0,
        height: 22.0,
        spacing: 4.0,
        tabs,
    })
}

fn bottom_strip_hit_box(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<TabStripHitBox> {
    if !group_expanded(model, &[ActivityDrawerSlot::Bottom]) {
        return None;
    }

    let frame = resolve_root_bottom_region_frame(shared_root_frames);
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return None;
    }

    let tabs = [ActivityDrawerSlot::Bottom]
        .into_iter()
        .filter_map(|slot| model.tool_windows.get(&slot))
        .flat_map(|stack| {
            stack
                .tabs
                .iter()
                .map(move |tab| strip_tab_from_pane(tab, stack))
        })
        .collect::<Vec<_>>();
    if tabs.is_empty() {
        return None;
    }

    Some(TabStripHitBox {
        style: StripStyle::Dock,
        start_x: frame.x + 6.0,
        end_x: frame.right() - 6.0,
        y: frame.y + 2.0,
        height: (metrics.panel_header_height - 3.0).max(22.0),
        spacing: 4.0,
        tabs,
    })
}

fn document_strip_hit_box(
    model: &WorkbenchViewModel,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<TabStripHitBox> {
    let frame = resolve_direct_document_host_frame(shared_root_frames)
        .unwrap_or_else(|| resolve_root_document_region_frame(shared_root_frames));
    if frame.width <= 0.0 {
        return None;
    }

    let tabs = model
        .document_tabs
        .iter()
        .map(strip_tab_from_document)
        .collect::<Vec<_>>();
    if tabs.is_empty() {
        return None;
    }
    let resolved_center_band_frame = resolve_root_center_band_frame(shared_root_frames);

    Some(TabStripHitBox {
        style: StripStyle::Document,
        start_x: frame.x + 8.0,
        end_x: frame.right() - 8.0,
        y: resolved_center_band_frame.y + 1.0,
        height: 30.0,
        spacing: 2.0,
        tabs,
    })
}

fn strip_tab_from_pane(tab: &PaneTabModel, stack: &ToolWindowStackModel) -> StripTabEntry {
    StripTabEntry {
        instance_id: tab.instance_id.clone(),
        title: tab.title.clone(),
        closeable: tab.closeable,
        host: ViewHost::Drawer(stack.slot),
    }
}

fn strip_tab_from_document(tab: &DocumentTabModel) -> StripTabEntry {
    let host = match &tab.workspace {
        WorkspaceTarget::MainPage(page_id) => {
            ViewHost::Document(page_id.clone(), tab.workspace_path.clone())
        }
        WorkspaceTarget::FloatingWindow(window_id) => {
            ViewHost::FloatingWindow(window_id.clone(), tab.workspace_path.clone())
        }
    };
    StripTabEntry {
        instance_id: tab.instance_id.clone(),
        title: tab.title.clone(),
        closeable: tab.closeable,
        host,
    }
}

fn group_expanded(model: &WorkbenchViewModel, slots: &[ActivityDrawerSlot]) -> bool {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .any(|stack| {
            stack.visible && !stack.tabs.is_empty() && stack.mode != ActivityDrawerMode::Collapsed
        })
}

fn resolve_direct_document_host_frame(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ShellFrame> {
    shared_root_frames
        .and_then(|frames| frames.document_host_frame)
        .map(shell_frame)
        .filter(|frame| frame.width > 0.0 && frame.height > 0.0)
}

fn shell_frame(frame: UiFrame) -> ShellFrame {
    ShellFrame::new(frame.x, frame.y, frame.width, frame.height)
}
