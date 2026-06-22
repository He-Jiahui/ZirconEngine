use super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn zero_origin() -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    }
}

pub(in crate::ui::retained_host::host_contract) struct RootFrames {
    pub(in crate::ui::retained_host::host_contract) top_bar: FrameRect,
    pub(in crate::ui::retained_host::host_contract) center_band: FrameRect,
    pub(in crate::ui::retained_host::host_contract) status_bar: FrameRect,
    pub(in crate::ui::retained_host::host_contract) left_region: FrameRect,
    pub(in crate::ui::retained_host::host_contract) right_region: FrameRect,
    pub(in crate::ui::retained_host::host_contract) bottom_region: FrameRect,
    pub(in crate::ui::retained_host::host_contract) document_region: FrameRect,
    pub(in crate::ui::retained_host::host_contract) viewport_region: FrameRect,
}
