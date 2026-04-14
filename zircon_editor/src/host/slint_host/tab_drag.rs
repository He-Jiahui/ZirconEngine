use crate::{
    ActivityDrawerMode, ActivityDrawerSlot, DocumentTabModel, DocumentNode,
    MainHostPageLayout, MainPageId, PaneTabModel, ShellRegionId, TabInsertionAnchor,
    TabInsertionSide, ToolWindowStackModel, ViewHost, ViewInstanceId, WorkbenchChromeMetrics,
    WorkbenchLayout, WorkbenchShellGeometry, WorkbenchViewModel,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedTabDrop {
    pub host: ViewHost,
    pub anchor: Option<TabInsertionAnchor>,
}

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
            &[ActivityDrawerSlot::RightTop, ActivityDrawerSlot::RightBottom],
            ActivityDrawerSlot::RightTop,
        ))),
        "bottom" => Some(ViewHost::Drawer(preferred_drawer_slot(
            layout,
            &[ActivityDrawerSlot::BottomLeft, ActivityDrawerSlot::BottomRight],
            ActivityDrawerSlot::BottomLeft,
        ))),
        "document" => preferred_document_page(layout)
            .map(|page_id| ViewHost::Document(page_id, Vec::new())),
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

pub(crate) fn resolve_tab_drop(
    layout: &WorkbenchLayout,
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    instance_id: &str,
    target_group: &str,
    pointer_x: f32,
    pointer_y: f32,
) -> Option<ResolvedTabDrop> {
    precise_drop_target(
        model,
        geometry,
        metrics,
        instance_id,
        target_group,
        pointer_x,
        pointer_y,
    )
    .or_else(|| {
        drop_host_for_tab(layout, instance_id, target_group).map(|host| ResolvedTabDrop {
            host,
            anchor: None,
        })
    })
}

pub(crate) fn drop_group_label(target_group: &str) -> &'static str {
    match target_group {
        "left" => "left tool stack",
        "right" => "right tool stack",
        "bottom" => "bottom tool stack",
        "document" => "document workspace",
        _ => "unknown target",
    }
}

pub(crate) fn estimate_dock_tab_width(label: &str) -> f32 {
    (estimate_text_width(label, 6.0) + 30.0).max(68.0)
}

pub(crate) fn estimate_document_tab_width(label: &str, closeable: bool) -> f32 {
    let min_width = if closeable { 114.0 } else { 92.0 };
    let chrome_width = if closeable { 54.0 } else { 42.0 };
    (estimate_text_width(label, 6.5) + chrome_width).max(min_width)
}

fn preferred_document_page(layout: &WorkbenchLayout) -> Option<MainPageId> {
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

fn preferred_drawer_slot(
    layout: &WorkbenchLayout,
    slots: &[ActivityDrawerSlot],
    fallback: ActivityDrawerSlot,
) -> ActivityDrawerSlot {
    slots.iter()
        .copied()
        .find(|slot| {
            layout
                .drawers
                .get(slot)
                .is_some_and(|drawer| drawer.visible && drawer.active_view.is_some())
        })
        .or_else(|| {
            slots.iter().copied().find(|slot| {
                layout.drawers.get(slot).is_some_and(|drawer| {
                    drawer.visible && drawer.tab_stack.active_tab.is_some()
                })
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

fn precise_drop_target(
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
            &[ActivityDrawerSlot::RightTop, ActivityDrawerSlot::RightBottom],
            ShellRegionId::Right,
            false,
        ),
        "bottom" => bottom_strip_hit_box(model, geometry, metrics),
        "document" => document_strip_hit_box(model, geometry, metrics),
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
        .flat_map(|stack| stack.tabs.iter().map(move |tab| strip_tab_from_pane(tab, stack)))
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
        &[ActivityDrawerSlot::BottomLeft, ActivityDrawerSlot::BottomRight],
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
    .flat_map(|stack| stack.tabs.iter().map(move |tab| strip_tab_from_pane(tab, stack)))
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
    _metrics: &WorkbenchChromeMetrics,
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
    StripTabEntry {
        instance_id: tab.instance_id.clone(),
        title: tab.title.clone(),
        closeable: tab.closeable,
        host: ViewHost::Document(tab.page_id.clone(), tab.workspace_path.clone()),
    }
}

fn group_expanded(model: &WorkbenchViewModel, slots: &[ActivityDrawerSlot]) -> bool {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .any(|stack| {
            stack.visible
                && !stack.tabs.is_empty()
                && stack.mode != ActivityDrawerMode::Collapsed
        })
}

fn estimate_text_width(label: &str, ascii_char_width: f32) -> f32 {
    label
        .chars()
        .map(|ch| {
            if ch.is_ascii_uppercase() {
                ascii_char_width + 1.0
            } else if ch.is_ascii_whitespace() {
                ascii_char_width * 0.5
            } else if ch.is_ascii_punctuation() {
                ascii_char_width * 0.75
            } else if ch.is_ascii() {
                ascii_char_width
            } else {
                ascii_char_width * 1.8
            }
        })
        .sum::<f32>()
        + 2.0
}
