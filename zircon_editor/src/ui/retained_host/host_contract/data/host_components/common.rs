use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct TabData {
    pub id: SharedString,
    pub slot: SharedString,
    pub title: SharedString,
    pub icon_key: SharedString,
    pub active: bool,
    pub closeable: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FrameRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl FrameRect {
    pub(crate) fn right(&self) -> f32 {
        self.x + self.width
    }

    pub(crate) fn bottom(&self) -> f32 {
        self.y + self.height
    }
}

#[derive(Clone, Default)]
pub(crate) struct HostChromeControlFrameData {
    pub control_id: SharedString,
    pub frame: FrameRect,
}

#[derive(Clone, Default)]
pub(crate) struct HostChromeTabData {
    pub control_id: SharedString,
    pub tab: TabData,
    pub frame: FrameRect,
    pub close_frame: FrameRect,
}
