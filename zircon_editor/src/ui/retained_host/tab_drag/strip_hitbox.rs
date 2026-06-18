#[cfg(test)]
use crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
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

#[cfg(test)]
pub(super) fn precise_drop_target(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    dragging_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> Option<ResolvedTabDrop> {
    let workbench_layout_frames =
        test_workbench_layout_frames_from_root_frames(shared_root_frames, metrics);
    precise_drop_target_with_workbench_layout_frames(
        model,
        metrics,
        dragging_id,
        target_group,
        pointer_x,
        pointer_y,
        workbench_layout_frames,
    )
}

pub(super) fn precise_drop_target_with_workbench_layout_frames(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    dragging_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<ResolvedTabDrop> {
    let strip = strip_hit_box(
        model,
        metrics,
        target_group,
        componentized_workbench_layout_frames,
    )?;
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
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<TabStripHitBox> {
    match target_group {
        "left" => tool_strip_hit_box(
            model,
            metrics,
            &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
            ShellRegionId::Left,
            true,
            componentized_workbench_layout_frames,
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
            componentized_workbench_layout_frames,
        ),
        "bottom" => bottom_strip_hit_box(model, metrics, componentized_workbench_layout_frames),
        "document" => document_strip_hit_box(model, componentized_workbench_layout_frames),
        _ => None,
    }
}

fn tool_strip_hit_box(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    slots: &[ActivityDrawerSlot],
    region: ShellRegionId,
    rail_on_left: bool,
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<TabStripHitBox> {
    if !group_expanded(model, slots) {
        return None;
    }

    let frame = layout_region_frame(componentized_workbench_layout_frames, region);
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
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<TabStripHitBox> {
    if !group_expanded(model, &[ActivityDrawerSlot::Bottom]) {
        return None;
    }

    let frame = layout_region_frame(componentized_workbench_layout_frames, ShellRegionId::Bottom);
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
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> Option<TabStripHitBox> {
    let document_tabs_frame = componentized_workbench_layout_frames
        .document_tabs_frame
        .filter(ui_frame_is_visible)
        .map(shell_frame);
    let frame = document_tabs_frame.or_else(|| {
        visible_workbench_shell_frame(componentized_workbench_layout_frames.document_region_frame)
    })?;
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
    let resolved_center_band_frame =
        visible_workbench_shell_frame(componentized_workbench_layout_frames.center_band_frame)
            .unwrap_or_default();

    Some(TabStripHitBox {
        style: StripStyle::Document,
        start_x: frame.x + 8.0,
        end_x: frame.right() - 8.0,
        y: document_tabs_frame
            .map(|frame| frame.y)
            .unwrap_or(resolved_center_band_frame.y + 1.0),
        height: document_tabs_frame
            .map(|frame| frame.height.max(0.0))
            .unwrap_or(30.0),
        spacing: 2.0,
        tabs,
    })
}

fn layout_region_frame(
    componentized_workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    region: ShellRegionId,
) -> ShellFrame {
    let componentized_frame = match region {
        ShellRegionId::Left => componentized_workbench_layout_frames.left_region_frame,
        ShellRegionId::Right => componentized_workbench_layout_frames.right_region_frame,
        ShellRegionId::Bottom => componentized_workbench_layout_frames.bottom_region_frame,
        ShellRegionId::Document => componentized_workbench_layout_frames.document_region_frame,
    };
    componentized_frame
        .filter(ui_frame_is_visible)
        .map(shell_frame)
        .unwrap_or_default()
}

fn visible_workbench_shell_frame(frame: Option<UiFrame>) -> Option<ShellFrame> {
    frame.filter(ui_frame_is_visible).map(shell_frame)
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

fn shell_frame(frame: UiFrame) -> ShellFrame {
    ShellFrame::new(frame.x, frame.y, frame.width, frame.height)
}

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

#[cfg(test)]
fn test_workbench_layout_frames_from_root_frames(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    metrics: &WorkbenchChromeMetrics,
) -> BuiltinWorkbenchWindowLayoutFrames {
    let document_region_frame = shared_root_frames.and_then(|frames| frames.document_host_frame);
    let document_tabs_frame = shared_root_frames
        .and_then(|frames| frames.document_tabs_frame)
        .or_else(|| {
            document_region_frame
                .filter(ui_frame_is_visible)
                .map(|frame| {
                    UiFrame::new(
                        frame.x,
                        frame.y,
                        frame.width,
                        metrics.document_header_height.max(0.0),
                    )
                })
        });

    BuiltinWorkbenchWindowLayoutFrames {
        center_band_frame: shared_root_frames.and_then(|frames| frames.host_body_frame),
        document_tabs_frame,
        document_region_frame,
        ..BuiltinWorkbenchWindowLayoutFrames::default()
    }
}
