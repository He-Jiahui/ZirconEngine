use super::data::{
    FrameRect, HostChromeTabData, HostDockOverflowMenuStateData, HostWindowPresentationData,
    TabData,
};
use super::menu_popup_metrics::{
    menu_popup_outer_padding, menu_popup_row_stride, menu_popup_shell_padding,
    menu_popup_visible_row_range, MENU_POPUP_ANCHOR_GAP, MENU_POPUP_EDGE_INSET,
    MENU_POPUP_ROW_HEIGHT, MENU_POPUP_SHELL_MARGIN, MENU_POPUP_TEXT_INSET_X,
};
use super::paint_theme::current_host_metrics;
use crate::ui::retained_host::popup_anchor_metrics::clamp_popup_x_to_bounds;
use crate::ui::retained_host::{measure_runtime_text_width, primitives::ModelRc};
use crate::ui::workbench::page_tabs::MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH;

pub(in crate::ui::retained_host::host_contract) struct HostDockOverflowProjection<'a> {
    pub surface_key: &'a str,
    pub anchor_frame: FrameRect,
    pub tabs: &'a ModelRc<TabData>,
    pub tab_frames: &'a ModelRc<HostChromeTabData>,
    pub drawer: bool,
}

pub(in crate::ui::retained_host::host_contract) struct HostDockOverflowRowHit {
    pub tab_index: usize,
    pub frame: FrameRect,
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_projection<'a>(
    presentation: &'a HostWindowPresentationData,
    state: &HostDockOverflowMenuStateData,
) -> Option<HostDockOverflowProjection<'a>> {
    if !state.open || state.surface_key.is_empty() {
        return None;
    }
    let scene = &presentation.host_scene_data;
    let surface_key = state.surface_key.as_str();
    let projection = if surface_key == scene.document_dock.surface_key.as_str() {
        HostDockOverflowProjection {
            surface_key: scene.document_dock.surface_key.as_str(),
            anchor_frame: translated(
                &scene.document_dock.overflow_frame,
                scene.document_dock.region_frame.x,
                scene.document_dock.region_frame.y,
            ),
            tabs: &scene.document_dock.tabs,
            tab_frames: &scene.document_dock.tab_frames,
            drawer: false,
        }
    } else if surface_key == scene.left_dock.surface_key.as_str()
        || surface_key == scene.right_dock.surface_key.as_str()
    {
        let dock = if surface_key == scene.left_dock.surface_key.as_str() {
            &scene.left_dock
        } else {
            &scene.right_dock
        };
        let panel_x = if dock.rail_before_panel {
            dock.region_frame.x + dock.rail_width_px
        } else {
            dock.region_frame.x
        };
        HostDockOverflowProjection {
            surface_key: dock.surface_key.as_str(),
            anchor_frame: translated(&dock.overflow_frame, panel_x, dock.region_frame.y),
            tabs: &dock.tabs,
            tab_frames: &dock.tab_frames,
            drawer: true,
        }
    } else if surface_key == scene.bottom_dock.surface_key.as_str() {
        HostDockOverflowProjection {
            surface_key: scene.bottom_dock.surface_key.as_str(),
            anchor_frame: translated(
                &scene.bottom_dock.overflow_frame,
                scene.bottom_dock.region_frame.x,
                scene.bottom_dock.region_frame.y,
            ),
            tabs: &scene.bottom_dock.tabs,
            tab_frames: &scene.bottom_dock.tab_frames,
            drawer: true,
        }
    } else {
        let window = scene
            .floating_layer
            .floating_windows
            .iter()
            .find(|window| window.window_id.as_str() == surface_key)?;
        HostDockOverflowProjection {
            surface_key: window.window_id.as_str(),
            anchor_frame: translated(&window.overflow_frame, window.frame.x, window.frame.y),
            tabs: &window.tabs,
            tab_frames: &window.tab_frames,
            drawer: false,
        }
    };
    valid_frame(&projection.anchor_frame).then_some(projection)
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_hidden_indices(
    projection: &HostDockOverflowProjection<'_>,
) -> Vec<usize> {
    projection
        .tab_frames
        .iter()
        .enumerate()
        .filter_map(|(index, tab)| (tab.frame.width <= f32::EPSILON).then_some(index))
        .collect()
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_popup_frame_with_state(
    presentation: &HostWindowPresentationData,
    state: &HostDockOverflowMenuStateData,
) -> Option<FrameRect> {
    let projection = host_dock_overflow_projection(presentation, state)?;
    let hidden_indices = host_dock_overflow_hidden_indices(&projection);
    if hidden_indices.is_empty() {
        return None;
    }
    let content_height = content_extent(hidden_indices.len());
    let shell_bottom = shell_bottom(presentation);
    let (y, height) = vertical_placement(&projection.anchor_frame, shell_bottom, content_height);
    if !height.is_finite() || height <= 0.0 {
        return None;
    }
    let (shell_x, shell_width) = shell_bounds(presentation, &projection.anchor_frame);
    let scrollbar_reserve = scrollbar_reserve(hidden_indices.len(), height);
    let metrics = current_host_metrics();
    let widest_title = hidden_indices
        .iter()
        .filter_map(|index| projection.tabs.get(*index))
        .map(|tab| measure_runtime_text_width(tab.title.as_str(), metrics.font_body))
        .fold(0.0_f32, f32::max);
    let title_chrome = MENU_POPUP_EDGE_INSET * 2.0
        + MENU_POPUP_TEXT_INSET_X * 2.0
        + metrics.selection_indicator_width
        + metrics.gap_s
        + scrollbar_reserve;
    let preferred_width = (widest_title + title_chrome).max(MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH);
    let popup_width = preferred_width.min((shell_width - menu_popup_shell_padding()).max(0.0));
    if !popup_width.is_finite() || popup_width <= 0.0 {
        return None;
    }
    let x = clamp_popup_x_to_bounds(
        projection.anchor_frame.right() - popup_width,
        shell_x,
        shell_width,
        popup_width,
    );
    let popup = FrameRect {
        x,
        y,
        width: popup_width,
        height,
    };
    let viewport = host_dock_overflow_content_viewport_frame(&popup);
    (viewport.width >= MENU_POPUP_EDGE_INSET && viewport.height >= MENU_POPUP_EDGE_INSET)
        .then_some(popup)
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_row_frame_with_state(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    row: usize,
    state: &HostDockOverflowMenuStateData,
) -> FrameRect {
    let scroll_offset = clamp_scroll_offset(presentation, popup, state, state.scroll_offset);
    row_frame(popup, row, scroll_offset)
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_visible_row_range_with_state(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    state: &HostDockOverflowMenuStateData,
) -> std::ops::Range<usize> {
    let item_count = host_dock_overflow_projection(presentation, state)
        .map(|projection| host_dock_overflow_hidden_indices(&projection).len())
        .unwrap_or(0);
    menu_popup_visible_row_range(
        item_count,
        host_dock_overflow_content_viewport_frame(popup).height,
        clamp_scroll_offset(presentation, popup, state, state.scroll_offset),
        0.0,
    )
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_row_hit_in_popup(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    state: &HostDockOverflowMenuStateData,
    x: f32,
    y: f32,
) -> Option<HostDockOverflowRowHit> {
    let projection = host_dock_overflow_projection(presentation, state)?;
    let hidden_indices = host_dock_overflow_hidden_indices(&projection);
    let scroll_offset = clamp_scroll_offset(presentation, popup, state, state.scroll_offset);
    let mut viewport = host_dock_overflow_content_viewport_frame(popup);
    let gutter = scrollbar_reserve(hidden_indices.len(), popup.height).min(viewport.width);
    viewport.width = (viewport.width - gutter).max(0.0);
    if !contains(&viewport, x, y) {
        return None;
    }
    let position = (y - popup.y - MENU_POPUP_EDGE_INSET + scroll_offset) / menu_popup_row_stride();
    if !position.is_finite() || position < 0.0 {
        return None;
    }
    let row = position.floor() as usize;
    if !host_dock_overflow_visible_row_range_with_state(presentation, popup, state).contains(&row) {
        return None;
    }
    let frame = row_frame(popup, row, scroll_offset);
    if !contains(&frame, x, y) {
        return None;
    }
    Some(HostDockOverflowRowHit {
        tab_index: *hidden_indices.get(row)?,
        frame,
    })
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_scroll_offset_for_delta(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    state: &HostDockOverflowMenuStateData,
    delta: f32,
) -> f32 {
    let current = clamp_scroll_offset(presentation, popup, state, state.scroll_offset);
    if !delta.is_finite() {
        return current;
    }
    (current + delta).clamp(0.0, max_scroll(presentation, popup, state))
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_scroll_offset_for_tab(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    state: &HostDockOverflowMenuStateData,
    tab_index: usize,
) -> f32 {
    let Some(projection) = host_dock_overflow_projection(presentation, state) else {
        return 0.0;
    };
    let Some(row) = host_dock_overflow_hidden_indices(&projection)
        .iter()
        .position(|index| *index == tab_index)
    else {
        return clamp_scroll_offset(presentation, popup, state, state.scroll_offset);
    };
    let current = clamp_scroll_offset(presentation, popup, state, state.scroll_offset);
    let viewport_height = host_dock_overflow_content_viewport_frame(popup).height;
    let row_top = row as f32 * menu_popup_row_stride();
    let row_bottom = row_top + MENU_POPUP_ROW_HEIGHT;
    let offset = if row_top < current {
        row_top
    } else if row_bottom > current + viewport_height {
        row_bottom - viewport_height
    } else {
        current
    };
    offset.clamp(0.0, max_scroll(presentation, popup, state))
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_content_viewport_frame(
    popup: &FrameRect,
) -> FrameRect {
    let x_inset = MENU_POPUP_EDGE_INSET.min(popup.width.max(0.0) * 0.5);
    let y_inset = MENU_POPUP_EDGE_INSET.min(popup.height.max(0.0) * 0.5);
    FrameRect {
        x: popup.x + x_inset,
        y: popup.y + y_inset,
        width: (popup.width - x_inset * 2.0).max(0.0),
        height: (popup.height - y_inset * 2.0).max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_scroll_content_extent(
    presentation: &HostWindowPresentationData,
    state: &HostDockOverflowMenuStateData,
) -> f32 {
    let count = host_dock_overflow_projection(presentation, state)
        .map(|projection| host_dock_overflow_hidden_indices(&projection).len())
        .unwrap_or(0);
    (content_extent(count) - menu_popup_outer_padding()).max(0.0)
}

pub(in crate::ui::retained_host::host_contract) fn host_dock_overflow_scrollbar_reserve(
    presentation: &HostWindowPresentationData,
    popup_height: f32,
    state: &HostDockOverflowMenuStateData,
) -> f32 {
    let count = host_dock_overflow_projection(presentation, state)
        .map(|projection| host_dock_overflow_hidden_indices(&projection).len())
        .unwrap_or(0);
    scrollbar_reserve(count, popup_height)
}

fn row_frame(popup: &FrameRect, row: usize, scroll_offset: f32) -> FrameRect {
    FrameRect {
        x: popup.x + MENU_POPUP_EDGE_INSET,
        y: popup.y + MENU_POPUP_EDGE_INSET + row as f32 * menu_popup_row_stride() - scroll_offset,
        width: (popup.width - MENU_POPUP_EDGE_INSET * 2.0).max(0.0),
        height: MENU_POPUP_ROW_HEIGHT,
    }
}

fn content_extent(item_count: usize) -> f32 {
    menu_popup_outer_padding()
        + item_count as f32 * MENU_POPUP_ROW_HEIGHT
        + item_count.saturating_sub(1) as f32 * (menu_popup_row_stride() - MENU_POPUP_ROW_HEIGHT)
}

fn scrollbar_reserve(item_count: usize, popup_height: f32) -> f32 {
    if content_extent(item_count) <= popup_height {
        return 0.0;
    }
    let metrics = current_host_metrics();
    metrics.scrollbar_thickness + metrics.gap_s
}

fn max_scroll(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    state: &HostDockOverflowMenuStateData,
) -> f32 {
    (host_dock_overflow_scroll_content_extent(presentation, state)
        - host_dock_overflow_content_viewport_frame(popup).height)
        .max(0.0)
}

fn clamp_scroll_offset(
    presentation: &HostWindowPresentationData,
    popup: &FrameRect,
    state: &HostDockOverflowMenuStateData,
    offset: f32,
) -> f32 {
    if !offset.is_finite() {
        return 0.0;
    }
    offset.clamp(0.0, max_scroll(presentation, popup, state))
}

fn vertical_placement(anchor: &FrameRect, shell_bottom: f32, content_height: f32) -> (f32, f32) {
    let below_y = anchor.bottom() + MENU_POPUP_ANCHOR_GAP;
    if !below_y.is_finite() || !content_height.is_finite() || content_height <= 0.0 {
        return (0.0, 0.0);
    }
    if shell_bottom <= 0.0 {
        return (below_y, content_height);
    }
    let shell_top = MENU_POPUP_SHELL_MARGIN.min((shell_bottom * 0.5).max(0.0));
    let shell_bottom = (shell_bottom - MENU_POPUP_SHELL_MARGIN).max(shell_top);
    let below_y = below_y.clamp(shell_top, shell_bottom);
    let above_bottom = (anchor.y - MENU_POPUP_ANCHOR_GAP).clamp(shell_top, shell_bottom);
    let below_height = (shell_bottom - below_y).max(0.0);
    let above_height = (above_bottom - shell_top).max(0.0);
    let place_below = content_height <= below_height
        || (content_height > above_height && below_height >= above_height);
    let available = if place_below {
        below_height
    } else {
        above_height
    };
    let height = content_height.min(available);
    let y = if place_below {
        below_y
    } else {
        (above_bottom - height).max(shell_top)
    };
    (y, height)
}

fn shell_bottom(presentation: &HostWindowPresentationData) -> f32 {
    [
        presentation.host_layout.status_bar_frame.y,
        presentation.host_scene_data.layout.status_bar_frame.y,
    ]
    .into_iter()
    .filter(|value| value.is_finite() && *value > 0.0)
    .fold(0.0_f32, f32::max)
}

fn shell_bounds(presentation: &HostWindowPresentationData, anchor: &FrameRect) -> (f32, f32) {
    let mut left = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    for frame in [
        &presentation.host_layout.status_bar_frame,
        &presentation.host_layout.center_band_frame,
    ] {
        if valid_frame(frame) {
            left = left.min(frame.x);
            right = right.max(frame.right());
        }
    }
    if left.is_finite() && right.is_finite() && right > left {
        return (left, right - left);
    }
    (0.0, anchor.right().max(0.0))
}

fn translated(frame: &FrameRect, x: f32, y: f32) -> FrameRect {
    FrameRect {
        x: frame.x + x,
        y: frame.y + y,
        width: frame.width,
        height: frame.height,
    }
}

fn valid_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(in crate::ui::retained_host::host_contract) fn contains(
    frame: &FrameRect,
    x: f32,
    y: f32,
) -> bool {
    x >= frame.x && y >= frame.y && x <= frame.right() && y <= frame.bottom()
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::ui::retained_host::primitives::VecModel;

    #[test]
    fn document_overflow_uses_published_anchor_and_hidden_frame_indices() {
        let mut presentation = overflow_presentation("document", 2);
        presentation.host_scene_data.document_dock.region_frame = frame(100.0, 40.0, 320.0, 260.0);
        presentation.host_scene_data.document_dock.overflow_frame = frame(280.0, 3.0, 28.0, 28.0);
        let state = open_state("document");

        let projection = host_dock_overflow_projection(&presentation, &state)
            .expect("published document overflow projection");
        assert_eq!(projection.anchor_frame, frame(380.0, 43.0, 28.0, 28.0));
        assert_eq!(host_dock_overflow_hidden_indices(&projection), vec![1, 2]);

        let popup = host_dock_overflow_popup_frame_with_state(&presentation, &state)
            .expect("hidden tabs should expose a popup");
        let row = host_dock_overflow_row_frame_with_state(&presentation, &popup, 0, &state);
        let hit = host_dock_overflow_row_hit_in_popup(
            &presentation,
            &popup,
            &state,
            row.x + 2.0,
            row.y + 2.0,
        )
        .expect("first hidden tab row hit");
        assert_eq!(hit.tab_index, 1);
    }

    #[test]
    fn left_drawer_overflow_anchor_includes_activity_rail_width() {
        let mut presentation = overflow_presentation("left", 1);
        let dock = &mut presentation.host_scene_data.left_dock;
        dock.region_frame = frame(10.0, 20.0, 240.0, 300.0);
        dock.rail_before_panel = true;
        dock.rail_width_px = 36.0;
        dock.overflow_frame = frame(150.0, 2.0, 28.0, 28.0);
        let state = open_state("left");

        let projection = host_dock_overflow_projection(&presentation, &state)
            .expect("left drawer overflow projection");

        assert!(projection.drawer);
        assert_eq!(projection.anchor_frame, frame(196.0, 22.0, 28.0, 28.0));
    }

    #[test]
    fn scroll_offset_is_bounded_by_the_same_content_extent_used_for_rows() {
        let presentation = overflow_presentation("document", 24);
        let state = open_state("document");
        let popup = host_dock_overflow_popup_frame_with_state(&presentation, &state)
            .expect("bounded overflow popup");

        let end =
            host_dock_overflow_scroll_offset_for_delta(&presentation, &popup, &state, f32::MAX);
        let repeated = host_dock_overflow_scroll_offset_for_delta(
            &presentation,
            &popup,
            &HostDockOverflowMenuStateData {
                scroll_offset: end,
                ..state
            },
            100.0,
        );

        assert!(end > 0.0);
        assert_eq!(repeated, end);
    }

    fn overflow_presentation(surface_key: &str, hidden_count: usize) -> HostWindowPresentationData {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_layout.center_band_frame = frame(0.0, 0.0, 640.0, 420.0);
        presentation.host_layout.status_bar_frame = frame(0.0, 400.0, 640.0, 20.0);
        let tabs = model_rc(
            (0..=hidden_count)
                .map(|index| TabData {
                    id: format!("tab-{index}").into(),
                    title: format!("Tab {index}").into(),
                    active: index == 0,
                    ..TabData::default()
                })
                .collect(),
        );
        let frames = model_rc(
            tabs.iter()
                .enumerate()
                .map(|(index, tab)| HostChromeTabData {
                    control_id: format!("DockTab{index}").into(),
                    tab,
                    frame: if index == 0 {
                        frame(8.0, 3.0, 100.0, 28.0)
                    } else {
                        FrameRect::default()
                    },
                    ..HostChromeTabData::default()
                })
                .collect(),
        );
        match surface_key {
            "left" => {
                presentation.host_scene_data.left_dock.surface_key = "left".into();
                presentation.host_scene_data.left_dock.tabs = tabs;
                presentation.host_scene_data.left_dock.tab_frames = frames;
            }
            _ => {
                presentation.host_scene_data.document_dock.surface_key = "document".into();
                presentation.host_scene_data.document_dock.region_frame =
                    frame(0.0, 40.0, 640.0, 360.0);
                presentation.host_scene_data.document_dock.overflow_frame =
                    frame(600.0, 2.0, 28.0, 28.0);
                presentation.host_scene_data.document_dock.tabs = tabs;
                presentation.host_scene_data.document_dock.tab_frames = frames;
            }
        }
        presentation
    }

    fn open_state(surface_key: &str) -> HostDockOverflowMenuStateData {
        HostDockOverflowMenuStateData {
            open: true,
            surface_key: surface_key.into(),
            ..HostDockOverflowMenuStateData::default()
        }
    }

    fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
        ModelRc::from(Rc::new(VecModel::from(rows)))
    }

    fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
