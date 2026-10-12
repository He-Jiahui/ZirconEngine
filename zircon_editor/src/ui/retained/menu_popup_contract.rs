use super::popup_anchor_metrics::{POPUP_ANCHOR_GAP, POPUP_EDGE_MARGIN};

pub(in crate::ui::retained_host) const WINDOW_MENU_INDEX: usize = 5;
pub(crate) const MENU_POPUP_PADDING: f32 = 6.0;
pub(crate) const MENU_POPUP_ROW_HEIGHT: f32 = 28.0;
pub(crate) const MENU_POPUP_ROW_GAP: f32 = 2.0;
pub(crate) const MENU_POPUP_ANCHOR_GAP: f32 = POPUP_ANCHOR_GAP;
pub(crate) const MENU_POPUP_EDGE_MARGIN: f32 = POPUP_EDGE_MARGIN;
pub(crate) const MENU_POPUP_MIN_HEIGHT: f32 = 72.0;
const MENU_POPUP_HORIZONTAL_CONTENT_PADDING: f32 = 28.0;
pub(crate) const MENU_POPUP_LABEL_SHORTCUT_GAP: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host) struct RootMenuPopupViewport {
    pub height: f32,
    pub scroll: f32,
}

pub(crate) fn content_measured_menu_popup_width<'a>(
    fallback_width: f32,
    available_width: f32,
    rows: impl IntoIterator<Item = (&'a str, &'a str)>,
    measure: impl Fn(&str) -> f32,
) -> f32 {
    content_measured_menu_popup_width_with_trailing_reserve(
        fallback_width,
        available_width,
        rows.into_iter()
            .map(|(label, shortcut)| (label, shortcut, 0.0)),
        measure,
    )
}

pub(crate) fn content_measured_menu_popup_width_with_trailing_reserve<'a>(
    fallback_width: f32,
    available_width: f32,
    rows: impl IntoIterator<Item = (&'a str, &'a str, f32)>,
    measure: impl Fn(&str) -> f32,
) -> f32 {
    let measured_width = rows
        .into_iter()
        .map(|(label, shortcut, trailing_reserve)| {
            let shortcut_width = if shortcut.is_empty() {
                0.0
            } else {
                MENU_POPUP_LABEL_SHORTCUT_GAP + measure(shortcut)
            };
            measure(label)
                + shortcut_width
                + MENU_POPUP_HORIZONTAL_CONTENT_PADDING
                + trailing_reserve.max(0.0)
        })
        .fold(fallback_width.max(1.0), f32::max);

    measured_width.min(available_width.max(1.0)).max(1.0)
}

pub(crate) fn content_measured_structured_menu_popup_width<'a>(
    fallback_width: f32,
    available_width: f32,
    items: impl IntoIterator<Item = &'a str>,
    trailing_adornment_reserve: f32,
    measure: impl Fn(&str) -> f32,
) -> f32 {
    content_measured_menu_popup_width_with_trailing_reserve(
        fallback_width,
        available_width,
        items.into_iter().filter_map(|item| {
            structured_menu_popup_measurement_row(item, trailing_adornment_reserve)
        }),
        measure,
    )
}

fn structured_menu_popup_measurement_row(
    item: &str,
    trailing_adornment_reserve: f32,
) -> Option<(&str, &str, f32)> {
    let mut fields = item.splitn(3, '|');
    let label = fields.next()?;
    if label == "---" {
        return None;
    }
    let flags = fields.next().unwrap_or_default();
    let shortcut = fields.next().unwrap_or_default();
    let has_adornment = flags
        .split(',')
        .any(|flag| matches!(flag, "checked" | "submenu") || flag.strip_prefix("icon=").is_some());
    Some((
        label,
        shortcut,
        if has_adornment {
            trailing_adornment_reserve
        } else {
            0.0
        },
    ))
}

pub(in crate::ui::retained_host) fn root_menu_popup_viewport(
    menu_index: usize,
    content_height: f32,
    window_viewport_height: f32,
    window_scroll: f32,
) -> RootMenuPopupViewport {
    if menu_index == WINDOW_MENU_INDEX && window_viewport_height > 0.0 {
        let height = window_viewport_height.min(content_height.max(1.0)).max(1.0);
        let max_scroll = (content_height - height).max(0.0);
        return RootMenuPopupViewport {
            height,
            scroll: window_scroll.clamp(0.0, max_scroll),
        };
    }

    RootMenuPopupViewport {
        height: content_height,
        scroll: 0.0,
    }
}

pub(crate) fn menu_popup_content_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        MENU_POPUP_PADDING * 2.0
            + item_count as f32 * MENU_POPUP_ROW_HEIGHT
            + (item_count as f32 - 1.0) * MENU_POPUP_ROW_GAP
    }
}

#[cfg(test)]
#[path = "tests/menu_popup_contract.rs"]
mod tests;
