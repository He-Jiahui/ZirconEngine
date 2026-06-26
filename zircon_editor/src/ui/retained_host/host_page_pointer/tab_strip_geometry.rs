use std::collections::BTreeSet;

use zircon_runtime_interface::ui::layout::UiFrame;

use crate::ui::workbench::page_tabs::{
    main_page_project_path_width, main_page_tab_visible_cap_for_width, MAIN_PAGE_TAB_GAP,
    MAIN_PAGE_TAB_HEIGHT, MAIN_PAGE_TAB_MAX_WIDTH, MAIN_PAGE_TAB_MIN_WIDTH,
    MAIN_PAGE_TAB_OVERFLOW_WIDTH, MAIN_PAGE_TAB_STRIP_X, MAIN_PAGE_TAB_STRIP_Y,
};

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
    page_ids: &[String],
    active_index: Option<usize>,
) -> (Vec<HostPageTabSlot>, Option<HostPageOverflowSlot>) {
    let visible_indices = visible_indices(strip_frame.width, page_ids.len(), active_index);
    let visible_set = visible_indices.iter().copied().collect::<BTreeSet<_>>();
    let hidden_page_indices = (0..page_ids.len())
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
    let tab_count = visible_indices.len().max(1) as f32;
    let tab_width = ((tab_area_right - x - MAIN_PAGE_TAB_GAP * (tab_count - 1.0)) / tab_count)
        .clamp(MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_MAX_WIDTH);
    let tabs = visible_indices
        .into_iter()
        .filter_map(|page_index| {
            let page_id = page_ids.get(page_index)?;
            let frame = UiFrame::new(
                x,
                strip_frame.y + MAIN_PAGE_TAB_STRIP_Y,
                tab_width,
                MAIN_PAGE_TAB_HEIGHT,
            );
            x += tab_width + MAIN_PAGE_TAB_GAP;
            Some(HostPageTabSlot {
                page_index,
                page_id: page_id.clone(),
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

fn visible_indices(strip_width: f32, page_count: usize, active_index: Option<usize>) -> Vec<usize> {
    if page_count == 0 {
        return Vec::new();
    }
    let visible_cap = main_page_tab_visible_cap_for_width(strip_width, page_count);
    let available =
        (strip_width - MAIN_PAGE_TAB_STRIP_X * 2.0 - main_page_project_path_width(strip_width))
            .max(0.0);
    let need_all = page_count as f32 * MAIN_PAGE_TAB_MIN_WIDTH
        + page_count.saturating_sub(1) as f32 * MAIN_PAGE_TAB_GAP;
    if need_all <= available && visible_cap >= page_count {
        return (0..page_count).collect();
    }

    let available_for_tabs =
        (available - MAIN_PAGE_TAB_OVERFLOW_WIDTH - MAIN_PAGE_TAB_GAP).max(MAIN_PAGE_TAB_MIN_WIDTH);
    let visible_count = ((available_for_tabs + MAIN_PAGE_TAB_GAP)
        / (MAIN_PAGE_TAB_MIN_WIDTH + MAIN_PAGE_TAB_GAP))
        .floor() as usize;
    let visible_count = visible_count.min(visible_cap).clamp(1, page_count);
    let mut indices = (0..visible_count).collect::<Vec<_>>();
    if let Some(active_index) = active_index.filter(|index| *index < page_count) {
        if !indices.contains(&active_index) {
            if let Some(last) = indices.last_mut() {
                *last = active_index;
            }
        }
    }
    indices.sort_unstable();
    indices.dedup();
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_ids(count: usize) -> Vec<String> {
        (0..count).map(|index| format!("page-{index}")).collect()
    }

    #[test]
    fn all_tabs_fit_when_strip_is_wide() {
        let strip = UiFrame::new(0.0, 24.0, 1280.0, 32.0);
        let (tabs, overflow) = allocate_host_page_tabs(strip, &page_ids(3), Some(1));

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
        let (tabs, overflow) = allocate_host_page_tabs(strip, &page_ids(6), Some(0));

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
        let (tabs, overflow) = allocate_host_page_tabs(strip, &page_ids(6), Some(5));

        assert!(tabs.iter().any(|tab| tab.page_index == 5));
        assert!(!overflow
            .expect("overflow slot")
            .hidden_page_indices
            .contains(&5));
    }

    #[test]
    fn narrow_tier_caps_visible_tabs_before_overflow() {
        let strip = UiFrame::new(0.0, 24.0, 640.0, 32.0);
        let (tabs, overflow) = allocate_host_page_tabs(strip, &page_ids(5), Some(4));

        assert_eq!(
            tabs.len(),
            2,
            "narrow layout tier should keep the host page strip to two readable tabs"
        );
        assert!(tabs.iter().any(|tab| tab.page_index == 4));
        assert!(overflow.is_some());
    }
}
