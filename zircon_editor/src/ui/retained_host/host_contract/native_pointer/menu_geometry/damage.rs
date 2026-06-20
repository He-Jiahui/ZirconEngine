use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{
    FrameRect, HostMenuChromeItemData, HostMenuStateData, HostWindowPresentationData,
};
use super::frames::{
    constrained_menu_popup_frame, constrained_submenu_popup_frame, menu_popup_height,
    menu_popup_row_frame, scrolled_menu_frame, shell_content_width,
};

pub(in crate::ui::retained_host::host_contract) fn menu_damage_frame(
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    menu_damage_frame_with_state(presentation, &presentation.menu_state)
}

pub(in crate::ui::retained_host::host_contract) fn menu_damage_frame_with_state(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
) -> FrameRect {
    let scene = &presentation.host_scene_data;
    let width = shell_content_width(presentation);
    let base_height = scene.menu_chrome.top_bar_height_px.max(0.0);
    let popup_bottom = if menu_state.open_menu_index >= 0 {
        scene
            .menu_chrome
            .menus
            .row_data(menu_state.open_menu_index as usize)
            .and_then(|menu| {
                let menu_frame = scene
                    .menu_chrome
                    .menu_frames
                    .row_data(menu_state.open_menu_index as usize)?;
                let menu_frame_rect = scrolled_menu_frame(&menu_frame.frame, presentation);
                let popup = constrained_menu_popup_frame(
                    presentation,
                    &menu_frame_rect,
                    menu.popup_width_px.max(menu_frame_rect.width).max(1.0),
                    menu.popup_height_px.max(1.0),
                );
                Some(menu_popup_stack_bottom(
                    presentation,
                    menu_state,
                    menu.items.clone(),
                    popup,
                ))
            })
            .unwrap_or(base_height)
    } else {
        base_height
    };
    FrameRect {
        x: 0.0,
        y: 0.0,
        width,
        height: (popup_bottom + 4.0).max(base_height),
    }
}

fn menu_popup_stack_bottom(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    mut items: ModelRc<HostMenuChromeItemData>,
    mut parent_popup: FrameRect,
) -> f32 {
    let mut bottom = parent_popup.y + parent_popup.height;
    for (level, selected_index) in menu_state.open_submenu_path.iter().copied().enumerate() {
        let Some(branch) = items.row_data(selected_index) else {
            break;
        };
        if branch.children.row_count() == 0 {
            break;
        }
        let scroll_px = if level == 0 {
            menu_state.window_menu_scroll_px
        } else {
            0.0
        };
        let anchor = menu_popup_row_frame(&parent_popup, selected_index, scroll_px);
        let popup = constrained_submenu_popup_frame(
            presentation,
            &anchor,
            parent_popup.width.max(1.0),
            menu_popup_height(branch.children.row_count()).max(1.0),
        );
        bottom = bottom.max(popup.y + popup.height);
        items = branch.children.clone();
        parent_popup = popup;
    }
    bottom
}
