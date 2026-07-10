use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};

use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct SideDockTabSlot {
    x: f32,
    width: f32,
    shows_label: bool,
}

pub(super) fn side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let nodes = fallback_dock_header_nodes(tabs, &"".into(), width, height);
    let slots = side_dock_tab_slots(tabs, width);

    model_rc(
        (0..nodes.row_count())
            .filter_map(|row| nodes.row_data(row))
            .filter_map(|mut node| {
                if let Some(index) = control_slot_index(node.control_id.as_str(), DOCK_TAB_PREFIX) {
                    let slot = slots.get(index).copied().unwrap_or_default();
                    if slot.width <= f32::EPSILON {
                        return None;
                    }
                    node.frame.x = slot.x;
                    node.frame.width = slot.width;
                    if !slot.shows_label {
                        node.text = SharedString::default();
                    }
                    return Some(node);
                }
                if let Some(index) =
                    control_slot_index(node.control_id.as_str(), DOCK_TAB_CLOSE_PREFIX)
                {
                    let slot = slots.get(index).copied().unwrap_or_default();
                    if slot.width <= f32::EPSILON || !slot.shows_label {
                        return None;
                    }
                    node.frame.x = document_tab_close_x(slot.x, slot.width);
                }
                Some(node)
            })
            .collect(),
    )
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

fn control_slot_index(control_id: &str, prefix: &str) -> Option<usize> {
    control_id.strip_prefix(prefix)?.parse().ok()
}
