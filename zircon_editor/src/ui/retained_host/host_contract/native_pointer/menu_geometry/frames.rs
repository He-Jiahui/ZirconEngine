use super::super::super::data::{FrameRect, HostWindowPresentationData, HostWindowSceneData};

pub(in crate::ui::retained_host::host_contract) fn menu_chrome_frame(
    scene: &HostWindowSceneData,
) -> FrameRect {
    let width = scene
        .layout
        .status_bar_frame
        .width
        .max(scene.layout.center_band_frame.width)
        .max(1.0);
    FrameRect {
        x: 0.0,
        y: 0.0,
        width,
        height: scene.menu_chrome.top_bar_height_px.max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract) fn top_bar_fallback_frame(
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: presentation.host_layout.status_bar_frame.width,
        height: presentation
            .host_scene_data
            .menu_chrome
            .top_bar_height_px
            .max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract) fn popup_blocking_frame(
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    FrameRect {
        x: 0.0,
        y: presentation
            .host_scene_data
            .menu_chrome
            .top_bar_height_px
            .max(0.0),
        width: shell_content_width(presentation),
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
    }
}

pub(in crate::ui::retained_host::host_contract) fn shell_content_width(
    presentation: &HostWindowPresentationData,
) -> f32 {
    presentation
        .host_layout
        .status_bar_frame
        .width
        .max(presentation.host_scene_data.layout.status_bar_frame.width)
        .max(presentation.host_scene_data.layout.center_band_frame.width)
        .max(1.0)
}

pub(in crate::ui::retained_host::host_contract) fn shell_content_height(
    presentation: &HostWindowPresentationData,
) -> f32 {
    presentation
        .host_layout
        .status_bar_frame
        .y
        .max(presentation.host_scene_data.layout.status_bar_frame.y)
        .max(1.0)
}

pub(in crate::ui::retained_host::host_contract) fn constrained_menu_popup_frame(
    presentation: &HostWindowPresentationData,
    menu_frame: &FrameRect,
    width: f32,
    requested_height: f32,
) -> FrameRect {
    let shell_width = shell_content_width(presentation);
    let shell_height = shell_content_height(presentation);
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

pub(in crate::ui::retained_host::host_contract) fn scrolled_menu_frame(
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

pub(in crate::ui::retained_host::host_contract) fn menu_popup_row_frame(
    popup: &FrameRect,
    row: usize,
    scroll_px: f32,
) -> FrameRect {
    FrameRect {
        x: popup.x + 6.0,
        y: popup.y + 6.0 + row as f32 * 30.0 - scroll_px,
        width: (popup.width - 12.0).max(0.0),
        height: 28.0,
    }
}

pub(in crate::ui::retained_host::host_contract) fn menu_popup_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        12.0 + item_count as f32 * 28.0 + (item_count as f32 - 1.0) * 2.0
    }
}

pub(in crate::ui::retained_host::host_contract) fn constrained_submenu_popup_frame(
    presentation: &HostWindowPresentationData,
    anchor: &FrameRect,
    width: f32,
    requested_height: f32,
) -> FrameRect {
    let shell_width = shell_content_width(presentation);
    let shell_height = shell_content_height(presentation);
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
