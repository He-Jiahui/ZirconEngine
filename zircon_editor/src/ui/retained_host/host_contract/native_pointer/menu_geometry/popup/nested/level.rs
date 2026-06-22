use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostMenuChromeItemData, HostWindowPresentationData,
};
use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::routing::contains;
use super::super::submenu_frame::nested_submenu_popup_frame;
use super::hit::NestedMenuLevelHit;

pub(super) fn nested_menu_level_hit(
    presentation: &HostWindowPresentationData,
    items: &ModelRc<HostMenuChromeItemData>,
    parent_popup: &FrameRect,
    selected_index: usize,
    level: usize,
    x: f32,
    y: f32,
) -> Option<NestedMenuLevelHit> {
    let branch = items.row_data(selected_index)?;
    if branch.children.row_count() == 0 {
        return None;
    }
    let popup = nested_submenu_popup_frame(
        presentation,
        parent_popup,
        selected_index,
        branch.children.row_count(),
        level,
    );
    Some(NestedMenuLevelHit {
        items: branch.children.clone(),
        contains_point: contains(&popup, x, y),
        popup,
    })
}
