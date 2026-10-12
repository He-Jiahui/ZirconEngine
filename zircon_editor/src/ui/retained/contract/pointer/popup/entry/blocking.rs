use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::super::routing::contains;
use super::super::super::frames::popup_blocking_frame;

pub(super) fn popup_blocking_region_handles_point(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> bool {
    contains(&popup_blocking_frame(presentation), x, y)
}
