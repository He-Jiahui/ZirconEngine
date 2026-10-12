use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, HostWindowSceneData,
};

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
