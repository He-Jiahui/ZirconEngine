use crate::{
    ActivityDrawerMode, ActivityDrawerSlot, DocumentTabModel, PaneTabModel, ShellRegionId,
    TabInsertionAnchor, TabInsertionSide, ToolWindowStackModel, ViewHost, ViewInstanceId,
    WorkbenchChromeMetrics, WorkbenchShellGeometry, WorkbenchViewModel, WorkspaceTarget,
};

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
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    dragging_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
) -> Option<ResolvedTabDrop> {
    let strip = strip_hit_box(model, geometry, metrics, target_group)?;
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
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    target_group: &str,
) -> Option<TabStripHitBox> {
    match target_group {
        "left" => tool_strip_hit_box(
            model,
            geometry,
            metrics,
            &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
            ShellRegionId::Left,
            true,
        ),
        "right" => tool_strip_hit_box(
            model,
            geometry,
            metrics,
            &[
                ActivityDrawerSlot::RightTop,
                ActivityDrawerSlot::RightBottom,
            ],
            ShellRegionId::Right,
            false,
        ),
        "bottom" => bottom_strip_hit_box(model, geometry, metrics),
        "document" => document_strip_hit_box(model, geometry),
        _ => None,
    }
}

fn tool_strip_hit_box(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    slots: &[ActivityDrawerSlot],
    region: ShellRegionId,
    rail_on_left: bool,
) -> Option<TabStripHitBox> {
    if !group_expanded(model, slots) {
        return None;
    }

    let frame = geometry.region_frame(region);
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
        y: geometry.center_band_frame.y + 2.0,
        height: 22.0,
        spacing: 4.0,
        tabs,
    })
}

fn bottom_strip_hit_box(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
) -> Option<TabStripHitBox> {
    if !group_expanded(
        model,
        &[
            ActivityDrawerSlot::BottomLeft,
            ActivityDrawerSlot::BottomRight,
        ],
    ) {
        return None;
    }

    let frame = geometry.region_frame(ShellRegionId::Bottom);
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return None;
    }

    let tabs = [
        ActivityDrawerSlot::BottomLeft,
        ActivityDrawerSlot::BottomRight,
    ]
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
    geometry: &WorkbenchShellGeometry,
) -> Option<TabStripHitBox> {
    let frame = geometry.region_frame(ShellRegionId::Document);
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

    Some(TabStripHitBox {
        style: StripStyle::Document,
        start_x: frame.x + 8.0,
        end_x: frame.right() - 8.0,
        y: geometry.center_band_frame.y + 1.0,
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
