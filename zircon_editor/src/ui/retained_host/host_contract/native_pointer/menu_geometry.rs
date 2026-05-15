use crate::ui::retained_host::primitives::ModelRc;

use super::super::data::{
    FrameRect, HostMenuChromeItemData, HostMenuStateData, HostWindowPresentationData,
};
use super::contains;

pub(super) fn menu_handles_point(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> bool {
    let scene = &presentation.host_scene_data;
    if contains(&scene.menu_chrome_frame(), x, y) {
        return true;
    }
    if scene.menu_chrome.menu_frames.row_count() == 0 {
        return contains(
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: presentation.host_layout.status_bar_frame.width,
                height: scene.menu_chrome.top_bar_height_px.max(0.0),
            },
            x,
            y,
        );
    }
    (0..scene.menu_chrome.menu_frames.row_count()).any(|row| {
        scene
            .menu_chrome
            .menu_frames
            .row_data(row)
            .is_some_and(|control| {
                contains(&scrolled_menu_frame(&control.frame, presentation), x, y)
            })
    })
}

pub(super) fn menu_popup_handles_point(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> bool {
    let state = &presentation.menu_state;
    if state.open_menu_index < 0 {
        return false;
    }
    let menu_index = state.open_menu_index as usize;
    let Some(menu_frame) = presentation
        .host_scene_data
        .menu_chrome
        .menu_frames
        .row_data(menu_index)
    else {
        return false;
    };
    let Some(menu) = presentation
        .host_scene_data
        .menu_chrome
        .menus
        .row_data(menu_index)
    else {
        return false;
    };
    let menu_frame_rect = scrolled_menu_frame(&menu_frame.frame, presentation);
    let popup = constrained_menu_popup_frame(
        presentation,
        &menu_frame_rect,
        menu.popup_width_px.max(menu_frame_rect.width).max(1.0),
        menu.popup_height_px.max(1.0),
    );
    contains(&popup, x, y)
        || nested_menu_popup_handles_point(presentation, menu.items.clone(), popup, x, y)
        || contains(
            &FrameRect {
                x: 0.0,
                y: presentation
                    .host_scene_data
                    .menu_chrome
                    .top_bar_height_px
                    .max(0.0),
                width: presentation
                    .host_layout
                    .status_bar_frame
                    .width
                    .max(presentation.host_scene_data.layout.status_bar_frame.width),
                height: (presentation
                    .host_layout
                    .status_bar_frame
                    .y
                    .max(presentation.host_scene_data.layout.status_bar_frame.y)
                    - presentation
                        .host_scene_data
                        .menu_chrome
                        .top_bar_height_px
                        .max(0.0))
                .max(0.0),
            },
            x,
            y,
        )
}

pub(super) fn menu_damage_frame(presentation: &HostWindowPresentationData) -> FrameRect {
    menu_damage_frame_with_state(presentation, &presentation.menu_state)
}

pub(super) fn menu_damage_frame_with_state(
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
) -> FrameRect {
    let scene = &presentation.host_scene_data;
    let width = presentation
        .host_layout
        .status_bar_frame
        .width
        .max(scene.layout.status_bar_frame.width)
        .max(scene.layout.center_band_frame.width)
        .max(1.0);
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

fn constrained_menu_popup_frame(
    presentation: &HostWindowPresentationData,
    menu_frame: &FrameRect,
    width: f32,
    requested_height: f32,
) -> FrameRect {
    let shell_width = presentation
        .host_layout
        .status_bar_frame
        .width
        .max(presentation.host_scene_data.layout.status_bar_frame.width)
        .max(presentation.host_scene_data.layout.center_band_frame.width)
        .max(1.0);
    let shell_height = presentation
        .host_layout
        .status_bar_frame
        .y
        .max(presentation.host_scene_data.layout.status_bar_frame.y)
        .max(1.0);
    let width = width.min(shell_width).max(1.0);
    let popup_y = menu_frame.y + menu_frame.height + 3.0;
    let x = menu_frame.x.clamp(0.0, (shell_width - width).max(0.0));
    let available_below = (shell_height - popup_y - 8.0).max(0.0);
    let available_above = (menu_frame.y - 8.0).max(0.0);
    let available_height = available_below
        .max(available_above)
        .max(72.0)
        .min(shell_height);
    let height = requested_height.min(available_height).max(1.0);
    let y = if popup_y + height <= shell_height {
        popup_y
    } else {
        (menu_frame.y - height - 3.0).max(0.0)
    };
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn scrolled_menu_frame(
    menu_frame: &FrameRect,
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    FrameRect {
        x: menu_frame.x - presentation.menu_state.menu_bar_scroll_px,
        y: menu_frame.y,
        width: menu_frame.width,
        height: menu_frame.height,
    }
}

fn nested_menu_popup_handles_point(
    presentation: &HostWindowPresentationData,
    mut items: ModelRc<HostMenuChromeItemData>,
    mut parent_popup: FrameRect,
    x: f32,
    y: f32,
) -> bool {
    for (level, selected_index) in presentation
        .menu_state
        .open_submenu_path
        .iter()
        .copied()
        .enumerate()
    {
        let Some(branch) = items.row_data(selected_index) else {
            return false;
        };
        if branch.children.row_count() == 0 {
            return false;
        }
        let scroll_px = if level == 0 {
            presentation.menu_state.window_menu_scroll_px
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
        if contains(&popup, x, y) {
            return true;
        }
        items = branch.children.clone();
        parent_popup = popup;
    }
    false
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

fn menu_popup_row_frame(popup: &FrameRect, row: usize, scroll_px: f32) -> FrameRect {
    FrameRect {
        x: popup.x + 6.0,
        y: popup.y + 6.0 + row as f32 * 30.0 - scroll_px,
        width: (popup.width - 12.0).max(0.0),
        height: 28.0,
    }
}

fn menu_popup_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        12.0 + item_count as f32 * 28.0 + (item_count as f32 - 1.0) * 2.0
    }
}

fn constrained_submenu_popup_frame(
    presentation: &HostWindowPresentationData,
    anchor: &FrameRect,
    width: f32,
    requested_height: f32,
) -> FrameRect {
    let shell_width = presentation
        .host_layout
        .status_bar_frame
        .width
        .max(presentation.host_scene_data.layout.status_bar_frame.width)
        .max(presentation.host_scene_data.layout.center_band_frame.width)
        .max(1.0);
    let shell_height = presentation
        .host_layout
        .status_bar_frame
        .y
        .max(presentation.host_scene_data.layout.status_bar_frame.y)
        .max(1.0);
    let width = width.min((shell_width - 16.0).max(1.0)).max(1.0);
    let min_x = 8.0;
    let max_x = (shell_width - width - 8.0).max(min_x);
    let right_x = anchor.x + anchor.width + 3.0;
    let left_x = anchor.x - width - 3.0;
    let x = if right_x + width <= shell_width - 8.0 {
        right_x.clamp(min_x, max_x)
    } else {
        left_x.clamp(min_x, max_x)
    };
    let height = requested_height
        .min((shell_height - 16.0).max(1.0))
        .max(1.0);
    let min_y = 8.0;
    let max_y = (shell_height - height - 8.0).max(min_y);
    let y = anchor.y.clamp(min_y, max_y);
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

trait HostWindowPresentationMenuFrame {
    fn menu_chrome_frame(&self) -> FrameRect;
}

impl HostWindowPresentationMenuFrame for super::super::data::HostWindowSceneData {
    fn menu_chrome_frame(&self) -> FrameRect {
        let width = self
            .layout
            .status_bar_frame
            .width
            .max(self.layout.center_band_frame.width)
            .max(1.0);
        FrameRect {
            x: 0.0,
            y: 0.0,
            width,
            height: self.menu_chrome.top_bar_height_px.max(0.0),
        }
    }
}
