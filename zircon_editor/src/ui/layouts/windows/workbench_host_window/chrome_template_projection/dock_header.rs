use super::*;
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::document_tabs::{
    document_tab_close_x, document_tab_preferred_width_from_title_width, DOCUMENT_TAB_CLOSE_EXTENT,
    DOCUMENT_TAB_CLOSE_TOP_INSET, DOCUMENT_TAB_GAP, DOCUMENT_TAB_HEIGHT, DOCUMENT_TAB_STRIP_X,
    DOCUMENT_TAB_STRIP_Y, DOCUMENT_TAB_TITLE_FONT_SIZE,
};

mod side;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct DockTabSlot {
    x: f32,
    width: f32,
    shows_label: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct DockTabLayout {
    slots: Vec<DockTabSlot>,
    overflow_frame: Option<ViewTemplateFrameData>,
}

#[cfg(test)]
pub(super) fn clear_side_dock_header_projection_cache_for_tests() {
    side::clear_side_dock_header_projection_cache_for_tests();
}

#[cfg(test)]
pub(super) fn side_dock_header_projection_builds_for_tests() -> usize {
    side::side_dock_header_projection_builds_for_tests()
}

pub(super) fn side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    _panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    side::side_dock_header_nodes(tabs, width, height)
}

pub(super) fn document_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    _panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header_nodes("host.document.dock.header", tabs, subtitle, width, height)
}

pub(super) fn bottom_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    _panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header_nodes("host.bottom.dock.header", tabs, &"".into(), width, height)
}

pub(super) fn floating_window_header_nodes(
    surface_id: &str,
    tabs: &ModelRc<TabData>,
    title: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header_nodes(surface_id, tabs, title, width, height)
}

fn dock_header_nodes(
    surface_id: &str,
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    text_overrides.insert(DOCK_SUBTITLE_CONTROL_ID.to_string(), subtitle.to_string());
    let nodes = tab_template_nodes(
        surface_id,
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        &BTreeMap::new(),
        DOCK_TAB_PREFIX,
        tabs,
    );
    if tab_chrome_needs_fallback(&nodes, DOCK_HEADER_BAR_CONTROL_ID, DOCK_TAB_PREFIX, tabs) {
        return fallback_dock_header_nodes(tabs, subtitle, width, height);
    }
    nodes
}

pub(super) fn dock_header_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, DOCK_HEADER_BAR_CONTROL_ID)
}

pub(super) fn dock_subtitle_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, DOCK_SUBTITLE_CONTROL_ID)
}

pub(super) fn dock_tab_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeTabData> {
    tab_frames(nodes, DOCK_TAB_PREFIX, Some(DOCK_TAB_CLOSE_PREFIX), tabs)
}

pub(super) fn dock_overflow_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, DOCK_TAB_OVERFLOW_CONTROL_ID)
}

pub(super) fn fallback_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let header_height = height.max(DOCK_HEADER_HEIGHT_PX);
    let controls = EditorControlTokens::workbench_dense();
    let preferred_widths = tabs
        .iter()
        .map(|tab| {
            let title_width =
                measure_runtime_text_width(tab.title.as_str(), DOCUMENT_TAB_TITLE_FONT_SIZE);
            document_tab_preferred_width_from_title_width(title_width, tab.closeable)
        })
        .collect::<Vec<_>>();
    let layout = adaptive_dock_tab_layout(
        tabs,
        width,
        controls.default_height,
        header_height,
        &preferred_widths,
    );
    let slots = &layout.slots;
    let close_count = tabs
        .iter()
        .enumerate()
        .filter(|(row, tab)| tab.closeable && slots.get(*row).is_some_and(|slot| slot.shows_label))
        .count();
    let mut nodes = Vec::with_capacity(
        tabs.row_count() + close_count + usize::from(layout.overflow_frame.is_some()) + 2,
    );
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackDockHeaderBar".into(),
        control_id: DOCK_HEADER_BAR_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: header_height,
        },
        ..ViewTemplateNodeData::default()
    });

    let mut max_tab_right = DOCUMENT_TAB_STRIP_X;
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.get(row) else {
            continue;
        };
        let slot = slots.get(row).copied().unwrap_or_default();
        if slot.width <= f32::EPSILON {
            continue;
        }
        let text_tone = if tab.active { "default" } else { "subtle" };
        let font_weight = if tab.active { 600 } else { 400 };
        let icon_name = chrome_tab_icon_name(tab);
        let mut tab_node = ViewTemplateNodeData {
            node_id: format!("FallbackDockTab{row}").into(),
            control_id: format!("{DOCK_TAB_PREFIX}{row}").into(),
            role: "Button".into(),
            text: if slot.shows_label {
                tab.title.clone()
            } else {
                SharedString::default()
            },
            text_tone: text_tone.into(),
            font_size: DOCUMENT_TAB_TITLE_FONT_SIZE,
            font_weight,
            surface_variant: if tab.active { "inset" } else { "transparent" }.into(),
            button_variant: "ghost".into(),
            corner_radius: fallback_chrome_control_radius(),
            selected: tab.active,
            focused: false,
            frame: ViewTemplateFrameData {
                x: slot.x,
                y: DOCUMENT_TAB_STRIP_Y,
                width: slot.width,
                height: DOCUMENT_TAB_HEIGHT.min(header_height.max(DOCUMENT_TAB_HEIGHT)),
            },
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut tab_node, &icon_name);
        nodes.push(tab_node);
        if tab.closeable && slot.shows_label {
            let mut close_node = ViewTemplateNodeData {
                node_id: format!("FallbackDockTabClose{row}").into(),
                control_id: format!("{DOCK_TAB_CLOSE_PREFIX}{row}").into(),
                role: "IconButton".into(),
                text_tone: "muted".into(),
                font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
                surface_variant: "transparent".into(),
                button_variant: "ghost".into(),
                corner_radius: fallback_chrome_control_radius(),
                value_number: 14.0,
                frame: ViewTemplateFrameData {
                    x: document_tab_close_x(slot.x, slot.width),
                    y: DOCUMENT_TAB_CLOSE_TOP_INSET,
                    width: DOCUMENT_TAB_CLOSE_EXTENT,
                    height: DOCUMENT_TAB_CLOSE_EXTENT,
                },
                ..ViewTemplateNodeData::default()
            };
            apply_template_icon(&mut close_node, DOCK_TAB_CLOSE_ICON);
            nodes.push(close_node);
        }
        max_tab_right = max_tab_right.max(slot.x + slot.width);
    }

    if let Some(overflow_frame) = layout.overflow_frame.clone() {
        let mut overflow_node = ViewTemplateNodeData {
            node_id: "FallbackDockTabOverflow".into(),
            control_id: DOCK_TAB_OVERFLOW_CONTROL_ID.into(),
            role: "IconButton".into(),
            text_tone: "subtle".into(),
            font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
            surface_variant: "transparent".into(),
            button_variant: "ghost".into(),
            corner_radius: fallback_chrome_control_radius(),
            frame: overflow_frame,
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut overflow_node, "ellipsis-horizontal-outline");
        nodes.push(overflow_node);
    }

    if !subtitle.is_empty() {
        nodes.push(ViewTemplateNodeData {
            node_id: "FallbackDockSubtitle".into(),
            control_id: DOCK_SUBTITLE_CONTROL_ID.into(),
            role: "Text".into(),
            text: subtitle.clone(),
            text_tone: "muted".into(),
            font_size: EditorTypographyTokens::WORKBENCH_CAPTION_SIZE,
            frame: ViewTemplateFrameData {
                x: (max_tab_right + 8.0).min(width.max(1.0)),
                y: 7.0,
                width: (layout
                    .overflow_frame
                    .as_ref()
                    .map(|frame| frame.x)
                    .unwrap_or(width)
                    - max_tab_right
                    - 16.0)
                    .max(0.0),
                height: 16.0,
            },
            ..ViewTemplateNodeData::default()
        });
    }

    model_rc(nodes)
}

fn adaptive_dock_tab_slots(
    tabs: &ModelRc<TabData>,
    width: f32,
    compact_width: f32,
    preferred_widths: &[f32],
) -> Vec<DockTabSlot> {
    let available_width = (width.max(0.0) - DOCUMENT_TAB_STRIP_X * 2.0).max(0.0);
    let tab_count = tabs.row_count();
    let mut widths = vec![0.0; tab_count];
    let active_index = tabs.iter().position(|tab| tab.active);

    let mut remaining = available_width;
    let mut visible_count = 0_usize;
    if let Some(index) = active_index {
        let active_width = preferred_widths
            .get(index)
            .copied()
            .unwrap_or(compact_width)
            .min(remaining);
        if active_width > f32::EPSILON {
            widths[index] = active_width;
            remaining = (remaining - active_width).max(0.0);
            visible_count = 1;
        }
    }
    for index in 0..tab_count {
        if Some(index) == active_index {
            continue;
        }
        let gap = if visible_count == 0 {
            0.0
        } else {
            DOCUMENT_TAB_GAP
        };
        if compact_width + gap <= remaining + f32::EPSILON {
            widths[index] = compact_width;
            remaining = (remaining - compact_width - gap).max(0.0);
            visible_count += 1;
        }
    }
    for index in 0..tab_count {
        if Some(index) != active_index && widths[index] > f32::EPSILON {
            expand_dock_tab_slot(index, &mut widths, preferred_widths, &mut remaining);
        }
    }

    let mut x = DOCUMENT_TAB_STRIP_X;
    let mut has_visible_slot = false;
    widths
        .into_iter()
        .enumerate()
        .map(|(index, slot_width)| {
            if slot_width > f32::EPSILON && has_visible_slot {
                x += DOCUMENT_TAB_GAP;
            }
            let preferred_width = preferred_widths
                .get(index)
                .copied()
                .unwrap_or(compact_width);
            let slot = DockTabSlot {
                x,
                width: slot_width,
                shows_label: slot_width + f32::EPSILON >= preferred_width,
            };
            if slot_width > f32::EPSILON {
                x += slot_width;
                has_visible_slot = true;
            }
            slot
        })
        .collect()
}

fn adaptive_dock_tab_layout(
    tabs: &ModelRc<TabData>,
    width: f32,
    compact_width: f32,
    header_height: f32,
    preferred_widths: &[f32],
) -> DockTabLayout {
    let slots = adaptive_dock_tab_slots(tabs, width, compact_width, preferred_widths);
    if slots.iter().all(|slot| slot.width > f32::EPSILON) {
        return DockTabLayout {
            slots,
            overflow_frame: None,
        };
    }

    let overflow_width = compact_width.min((width.max(0.0) - DOCUMENT_TAB_STRIP_X * 2.0).max(0.0));
    let overflow_reserve = if overflow_width > f32::EPSILON {
        overflow_width + DOCUMENT_TAB_GAP
    } else {
        0.0
    };
    let tab_width = (width - overflow_reserve).max(0.0);
    let slots = adaptive_dock_tab_slots(tabs, tab_width, compact_width, preferred_widths);
    let overflow_frame = (overflow_width > f32::EPSILON).then_some(ViewTemplateFrameData {
        x: (width.max(0.0) - DOCUMENT_TAB_STRIP_X - overflow_width).max(DOCUMENT_TAB_STRIP_X),
        y: DOCUMENT_TAB_STRIP_Y,
        width: overflow_width,
        height: DOCUMENT_TAB_HEIGHT.min(header_height.max(DOCUMENT_TAB_HEIGHT)),
    });

    DockTabLayout {
        slots,
        overflow_frame,
    }
}

fn expand_dock_tab_slot(
    index: usize,
    widths: &mut [f32],
    preferred_widths: &[f32],
    remaining: &mut f32,
) {
    let extra = preferred_widths
        .get(index)
        .copied()
        .unwrap_or(widths[index])
        - widths[index];
    let extra = extra.max(0.0);
    if extra <= *remaining + f32::EPSILON {
        widths[index] += extra;
        *remaining = (*remaining - extra).max(0.0);
    }
}
