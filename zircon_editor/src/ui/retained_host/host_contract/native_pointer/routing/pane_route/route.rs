use crate::ui::retained_host::host_contract::data::FrameRect;

use super::target::PanePointerTarget;

pub(in crate::ui::retained_host::host_contract) struct PanePointerRoute<'a> {
    pub(in crate::ui::retained_host::host_contract) hierarchy_row_metrics:
        Option<crate::ui::retained_host::hierarchy_pointer::HierarchyRowMetrics>,
    pub(in crate::ui::retained_host::host_contract) target: PanePointerTarget<'a>,
    pub(in crate::ui::retained_host::host_contract) frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract) local_x: f32,
    pub(in crate::ui::retained_host::host_contract) local_y: f32,
    pub(in crate::ui::retained_host::host_contract) width: f32,
    pub(in crate::ui::retained_host::host_contract) height: f32,
}

impl<'a> PanePointerRoute<'a> {
    pub(in crate::ui::retained_host::host_contract) fn new(
        target: PanePointerTarget<'a>,
        frame: &FrameRect,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            hierarchy_row_metrics: None,
            target,
            frame: frame.clone(),
            local_x: x - frame.x,
            local_y: y - frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}
