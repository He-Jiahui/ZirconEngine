use std::cell::RefCell;

use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};

use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct SideDockTabSlot {
    x: f32,
    width: f32,
    shows_label: bool,
}

const SIDE_DOCK_HEADER_CACHE_CAPACITY: usize = 12;

struct SideDockHeaderProjectionCacheEntry {
    tabs: ModelRc<TabData>,
    width_bits: u32,
    height_bits: u32,
    nodes: ModelRc<ViewTemplateNodeData>,
}

#[derive(Default)]
struct SideDockHeaderProjectionCache {
    entries: Vec<SideDockHeaderProjectionCacheEntry>,
    #[cfg(test)]
    builds: usize,
}

thread_local! {
    static SIDE_DOCK_HEADER_PROJECTION_CACHE: RefCell<SideDockHeaderProjectionCache> =
        RefCell::new(SideDockHeaderProjectionCache::default());
}

pub(super) fn side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let width_bits = width.to_bits();
    let height_bits = height.to_bits();
    SIDE_DOCK_HEADER_PROJECTION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(entry) = cache.entries.iter().find(|entry| {
            entry.width_bits == width_bits
                && entry.height_bits == height_bits
                && entry.tabs.shares_values_with(tabs)
        }) {
            return entry.nodes.clone();
        }

        let nodes = build_side_dock_header_nodes(tabs, width, height);
        if cache.entries.len() == SIDE_DOCK_HEADER_CACHE_CAPACITY {
            cache.entries.remove(0);
        }
        cache.entries.push(SideDockHeaderProjectionCacheEntry {
            tabs: tabs.clone(),
            width_bits,
            height_bits,
            nodes: nodes.clone(),
        });
        #[cfg(test)]
        {
            cache.builds += 1;
        }
        nodes
    })
}

fn build_side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let header_height = height.max(DOCK_HEADER_HEIGHT_PX);
    let slots = side_dock_tab_slots(tabs, width);
    let close_count = (0..tabs.row_count())
        .filter(|row| {
            tabs.row_data(*row).is_some_and(|tab| {
                tab.closeable && slots.get(*row).is_some_and(|slot| slot.shows_label)
            })
        })
        .count();
    let mut nodes = Vec::with_capacity(tabs.row_count() + close_count + 1);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackSideDockHeaderBar".into(),
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

    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let slot = slots.get(row).copied().unwrap_or_default();
        if slot.width <= f32::EPSILON {
            continue;
        }
        let text_tone = if tab.active { "default" } else { "subtle" };
        let font_weight = if tab.active { 600 } else { 400 };
        let mut tab_node = ViewTemplateNodeData {
            node_id: format!("FallbackSideDockTab{row}").into(),
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
            surface_variant: if tab.active { "inset" } else { "" }.into(),
            button_variant: "ghost".into(),
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
        apply_template_icon(&mut tab_node, &chrome_tab_icon_name(&tab));
        nodes.push(tab_node);
        if tab.closeable && slot.shows_label {
            let mut close_node = ViewTemplateNodeData {
                node_id: format!("FallbackSideDockTabClose{row}").into(),
                control_id: format!("{DOCK_TAB_CLOSE_PREFIX}{row}").into(),
                role: "IconButton".into(),
                text_tone: "muted".into(),
                font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
                button_variant: "ghost".into(),
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
    }
    model_rc(nodes)
}

fn side_dock_tab_slots(tabs: &ModelRc<TabData>, width: f32) -> Vec<SideDockTabSlot> {
    let controls = EditorControlTokens::workbench_dense();
    let density = EditorDensityTokens::workbench_dense();
    let compact_width = controls.default_height;
    let available_width = (width.max(0.0) - DOCUMENT_TAB_STRIP_X * 2.0).max(0.0);
    let tab_count = tabs.row_count();
    let slot_budget = available_width;
    let mut widths = vec![0.0; tab_count];
    let preferred_widths = (0..tab_count)
        .map(|index| {
            tabs.row_data(index)
                .map(|tab| side_dock_tab_preferred_width(tab.title.as_str(), controls, density))
                .unwrap_or(compact_width)
        })
        .collect::<Vec<_>>();
    let active_index =
        (0..tab_count).find(|index| tabs.row_data(*index).map(|tab| tab.active).unwrap_or(false));

    let mut remaining = slot_budget;
    let mut visible_count = 0_usize;
    if let Some(index) = active_index {
        let active_width = preferred_widths[index].min(remaining);
        if active_width > f32::EPSILON {
            widths[index] = active_width;
            remaining = (remaining - active_width).max(0.0);
            visible_count = 1;
        }
    }
    for index in 0..tab_count {
        if Some(index) != active_index {
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
    }
    for index in 0..tab_count {
        if Some(index) != active_index && widths[index] > f32::EPSILON {
            expand_slot(index, &mut widths, &preferred_widths, &mut remaining);
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
            let slot = SideDockTabSlot {
                x,
                width: slot_width,
                shows_label: slot_width + f32::EPSILON >= preferred_widths[index],
            };
            if slot_width > f32::EPSILON {
                x += slot_width;
                has_visible_slot = true;
            }
            slot
        })
        .collect()
}

fn side_dock_tab_preferred_width(
    title: &str,
    controls: EditorControlTokens,
    density: EditorDensityTokens,
) -> f32 {
    let icon_width = controls.default_height * 0.5;
    let horizontal_padding = density.gap_medium * 2.0;
    (measure_runtime_text_width(title, DOCUMENT_TAB_TITLE_FONT_SIZE)
        + icon_width
        + density.gap_small
        + horizontal_padding)
        .max(controls.default_height * 3.0 + density.gap_medium * 2.0)
}

fn expand_slot(index: usize, widths: &mut [f32], preferred_widths: &[f32], remaining: &mut f32) {
    let extra = (preferred_widths[index] - widths[index]).max(0.0);
    if extra <= *remaining + f32::EPSILON {
        widths[index] += extra;
        *remaining = (*remaining - extra).max(0.0);
    }
}

#[cfg(test)]
pub(super) fn clear_side_dock_header_projection_cache_for_tests() {
    SIDE_DOCK_HEADER_PROJECTION_CACHE.with(|cache| *cache.borrow_mut() = Default::default());
}

#[cfg(test)]
pub(super) fn side_dock_header_projection_builds_for_tests() -> usize {
    SIDE_DOCK_HEADER_PROJECTION_CACHE.with(|cache| cache.borrow().builds)
}
