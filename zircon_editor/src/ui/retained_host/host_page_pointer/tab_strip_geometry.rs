use std::collections::BTreeSet;

use zircon_runtime_interface::ui::layout::UiFrame;

use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::page_tabs::{
    main_page_project_path_width, main_page_tab_preferred_width_from_title_width,
    main_page_tab_visible_cap_for_width, MAIN_PAGE_TAB_GAP, MAIN_PAGE_TAB_HEIGHT,
    MAIN_PAGE_TAB_MAX_WIDTH, MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_OVERFLOW_WIDTH,
    MAIN_PAGE_TAB_STRIP_X, MAIN_PAGE_TAB_STRIP_Y, MAIN_PAGE_TAB_TITLE_FONT_SIZE,
};

use super::host_page_pointer_item::HostPagePointerItem;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostPageTabSlot {
    pub page_index: usize,
    pub page_id: String,
    pub frame: UiFrame,
}

// Overflow keeps hidden page indices separate from the visible tab slots so the
// pointer bridge, renderer, and future popup menu all read the same allocation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostPageOverflowSlot {
    pub frame: UiFrame,
    pub hidden_page_indices: Vec<usize>,
}

pub(super) fn allocate_host_page_tabs(
    strip_frame: UiFrame,
    pages: &[HostPagePointerItem],
    active_index: Option<usize>,
) -> (Vec<HostPageTabSlot>, Option<HostPageOverflowSlot>) {
    let visible_indices = visible_indices(strip_frame.width, pages, active_index);
    let visible_set = visible_indices.iter().copied().collect::<BTreeSet<_>>();
    let hidden_page_indices = (0..pages.len())
        .filter(|index| !visible_set.contains(index))
        .collect::<Vec<_>>();
    let has_overflow = !hidden_page_indices.is_empty();
    let mut x = strip_frame.x + MAIN_PAGE_TAB_STRIP_X;
    let project_path_width = main_page_project_path_width(strip_frame.width);
    let tab_lane_right =
        (strip_frame.x + strip_frame.width - MAIN_PAGE_TAB_STRIP_X - project_path_width).max(x);
    let tab_area_right = if has_overflow {
        (tab_lane_right - MAIN_PAGE_TAB_OVERFLOW_WIDTH - MAIN_PAGE_TAB_GAP).max(x)
    } else {
        tab_lane_right
    };
    let tabs = visible_indices
        .into_iter()
        .filter_map(|page_index| {
            let page = pages.get(page_index)?;
            let tab_width = page_tab_width(page)
                .min((tab_area_right - x).max(MAIN_PAGE_TAB_MIN_WIDTH))
                .clamp(MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_MAX_WIDTH);
            let frame = UiFrame::new(
                x,
                strip_frame.y + MAIN_PAGE_TAB_STRIP_Y,
                tab_width,
                MAIN_PAGE_TAB_HEIGHT,
            );
            x += tab_width + MAIN_PAGE_TAB_GAP;
            Some(HostPageTabSlot {
                page_index,
                page_id: page.page_id.clone(),
                frame,
            })
        })
        .collect::<Vec<_>>();

    let overflow = has_overflow.then(|| {
        let strip_left = strip_frame.x + MAIN_PAGE_TAB_STRIP_X;
        HostPageOverflowSlot {
            frame: UiFrame::new(
                x.min((tab_lane_right - MAIN_PAGE_TAB_OVERFLOW_WIDTH).max(strip_left)),
                strip_frame.y + MAIN_PAGE_TAB_STRIP_Y,
                MAIN_PAGE_TAB_OVERFLOW_WIDTH,
                MAIN_PAGE_TAB_HEIGHT,
            ),
            hidden_page_indices,
        }
    });

    (tabs, overflow)
}

fn visible_indices(
    strip_width: f32,
    pages: &[HostPagePointerItem],
    active_index: Option<usize>,
) -> Vec<usize> {
    let page_count = pages.len();
    if page_count == 0 {
        return Vec::new();
    }
    let visible_cap = main_page_tab_visible_cap_for_width(strip_width, page_count);
    let force_overflow = visible_cap < page_count;
    let max_tab_right =
        (strip_width - MAIN_PAGE_TAB_STRIP_X - main_page_project_path_width(strip_width))
            .max(MAIN_PAGE_TAB_STRIP_X);
    let mut x = MAIN_PAGE_TAB_STRIP_X;
    let mut indices = Vec::new();
    for page_index in 0..page_count {
        if indices.len() >= visible_cap {
            break;
        }
        let remaining_after_page = page_count.saturating_sub(page_index + 1);
        let overflow_reserve = if remaining_after_page > 0 || force_overflow {
            MAIN_PAGE_TAB_OVERFLOW_WIDTH + MAIN_PAGE_TAB_GAP
        } else {
            0.0
        };
        let tab_width = page_tab_width(&pages[page_index]);
        if !indices.is_empty() && x + tab_width + overflow_reserve > max_tab_right {
            break;
        }
        indices.push(page_index);
        x += tab_width + MAIN_PAGE_TAB_GAP;
    }
    if let Some(active_index) = active_index.filter(|index| *index < page_count) {
        if !indices.contains(&active_index) {
            if let Some(last) = indices.last_mut() {
                *last = active_index;
            } else {
                indices.push(active_index);
            }
        }
    }
    indices.sort_unstable();
    indices.dedup();
    indices
}

fn page_tab_width(page: &HostPagePointerItem) -> f32 {
    let title_width =
        measure_runtime_text_width(page.title.as_str(), MAIN_PAGE_TAB_TITLE_FONT_SIZE);
    main_page_tab_preferred_width_from_title_width(title_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_items(count: usize) -> Vec<HostPagePointerItem> {
        (0..count)
            .map(|index| HostPagePointerItem {
                page_id: format!("page-{index}"),
                title: format!("Page {index}"),
            })
            .collect()
    }

    fn page_item(page_id: &str, title: &str) -> HostPagePointerItem {
        HostPagePointerItem {
            page_id: page_id.to_string(),
            title: title.to_string(),
        }
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.01,
            "expected {expected:.3}, got {actual:.3}",
        );
    }

    #[test]
    fn all_tabs_fit_when_strip_is_wide() {
        let strip = UiFrame::new(0.0, 24.0, 1280.0, 32.0);
        let (tabs, overflow) = allocate_host_page_tabs(strip, &page_items(3), Some(1));

        assert_eq!(tabs.len(), 3);
        assert!(overflow.is_none());
        assert!(tabs
            .iter()
            .all(|tab| tab.frame.width >= MAIN_PAGE_TAB_MIN_WIDTH));
        assert!(tabs
            .iter()
            .all(|tab| tab.frame.width <= MAIN_PAGE_TAB_MAX_WIDTH));
    }

    #[test]
    fn tabs_overflow_without_shrinking_below_min_width() {
        let strip = UiFrame::new(0.0, 24.0, 360.0, 32.0);
        let (tabs, overflow) = allocate_host_page_tabs(strip, &page_items(6), Some(0));

        assert!(tabs.len() < 6);
        assert!(tabs
            .iter()
            .all(|tab| tab.frame.width >= MAIN_PAGE_TAB_MIN_WIDTH));
        assert_eq!(
            overflow
                .expect("overflow slot")
                .hidden_page_indices
                .last()
                .copied(),
            Some(5)
        );
    }

    #[test]
    fn active_tab_is_kept_visible_when_overflowing() {
        let strip = UiFrame::new(0.0, 24.0, 360.0, 32.0);
        let (tabs, overflow) = allocate_host_page_tabs(strip, &page_items(6), Some(5));

        assert!(tabs.iter().any(|tab| tab.page_index == 5));
        assert!(!overflow
            .expect("overflow slot")
            .hidden_page_indices
            .contains(&5));
    }

    #[test]
    fn narrow_tier_caps_visible_tabs_before_overflow() {
        let strip = UiFrame::new(0.0, 24.0, 640.0, 32.0);
        let (tabs, overflow) = allocate_host_page_tabs(strip, &page_items(5), Some(4));

        assert_eq!(
            tabs.len(),
            2,
            "narrow layout tier should keep the host page strip to two readable tabs"
        );
        assert!(tabs.iter().any(|tab| tab.page_index == 4));
        assert!(overflow.is_some());
    }

    #[test]
    fn host_page_pointer_tabs_use_runtime_measured_title_widths() {
        let pages = vec![
            page_item("narrow", "iiiiiiii"),
            page_item("wide", "WWWWWWWW"),
        ];
        let strip = UiFrame::new(0.0, 24.0, 900.0, 32.0);

        let (tabs, overflow) = allocate_host_page_tabs(strip, &pages, Some(0));

        assert!(overflow.is_none());
        assert_eq!(tabs.len(), 2);
        let expected_narrow = main_page_tab_preferred_width_from_title_width(
            measure_runtime_text_width("iiiiiiii", MAIN_PAGE_TAB_TITLE_FONT_SIZE),
        );
        let expected_wide = main_page_tab_preferred_width_from_title_width(
            measure_runtime_text_width("WWWWWWWW", MAIN_PAGE_TAB_TITLE_FONT_SIZE),
        );
        assert_close(tabs[0].frame.width, expected_narrow);
        assert_close(tabs[1].frame.width, expected_wide);
        assert!(
            tabs[1].frame.width > tabs[0].frame.width,
            "runtime-measured pointer hitbox should keep wide glyph titles wider"
        );
    }
}
