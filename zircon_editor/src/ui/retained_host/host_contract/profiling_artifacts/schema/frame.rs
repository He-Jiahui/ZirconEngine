use serde::Serialize;

use super::super::super::data::FrameRect;

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileFrame {
    pub(in crate::ui::retained_host::host_contract) x: f32,
    pub(in crate::ui::retained_host::host_contract) y: f32,
    pub(in crate::ui::retained_host::host_contract) width: f32,
    pub(in crate::ui::retained_host::host_contract) height: f32,
}

impl From<FrameRect> for UiProfileFrame {
    fn from(frame: FrameRect) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

impl From<&FrameRect> for UiProfileFrame {
    fn from(frame: &FrameRect) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfilePoint {
    pub(in crate::ui::retained_host::host_contract) x: f32,
    pub(in crate::ui::retained_host::host_contract) y: f32,
}

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileSize {
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
}

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileLayout {
    pub(in crate::ui::retained_host::host_contract) center_band: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) document_region: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) left_region: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) right_region: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) bottom_region: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) status_bar: UiProfileFrame,
}
