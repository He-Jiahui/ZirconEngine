use super::super::data::FrameRect;
use super::HostRedrawRequest;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativePointerDispatchResult {
    redraw: HostRedrawRequest,
}

impl NativePointerDispatchResult {
    pub(in crate::ui::retained_host::host_contract) fn idle() -> Self {
        Self {
            redraw: HostRedrawRequest::none(),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn full_frame() -> Self {
        Self {
            redraw: HostRedrawRequest::full_frame(),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn region(frame: FrameRect) -> Self {
        Self {
            redraw: HostRedrawRequest::region(frame),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn region_with_frame_update(
        frame: FrameRect,
    ) -> Self {
        Self {
            redraw: HostRedrawRequest::region_with_frame_update(frame),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn merge(self, next: Self) -> Self {
        Self {
            redraw: self.redraw.merge(next.redraw),
        }
    }

    pub(crate) fn request_redraw(&self) -> bool {
        self.redraw.request_redraw()
    }

    pub(crate) fn requires_frame_update(&self) -> bool {
        self.redraw.requires_frame_update()
    }

    pub(crate) fn damage_region(&self) -> Option<FrameRect> {
        self.redraw.damage_region().cloned()
    }

    pub(in crate::ui::retained_host::host_contract) fn redraw(self) -> HostRedrawRequest {
        self.redraw
    }
}
