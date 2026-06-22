use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::union::union_visible_frame;

pub(super) fn union_host_page_tab_damage(
    mut damage: Option<FrameRect>,
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let page_chrome = &presentation.host_scene_data.page_chrome;

    // Page activation can update selected tab chrome; keep menu/title chrome out.
    damage = union_visible_frame(damage, page_chrome.tab_row_frame.clone());
    damage = union_visible_frame(damage, page_chrome.project_path_frame.clone());
    for row in 0..page_chrome.tab_frames.row_count() {
        let Some(tab) = page_chrome.tab_frames.row_data(row) else {
            continue;
        };
        damage = union_visible_frame(damage, tab.frame.clone());
        damage = union_visible_frame(damage, tab.close_frame.clone());
    }
    damage
}
