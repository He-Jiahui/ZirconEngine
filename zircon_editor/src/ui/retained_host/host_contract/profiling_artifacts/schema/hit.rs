use serde::Serialize;

use super::frame::UiProfilePoint;

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileHitSample {
    pub(in crate::ui::retained_host::host_contract) id: String,
    pub(in crate::ui::retained_host::host_contract) kind: String,
    pub(in crate::ui::retained_host::host_contract) surface: String,
    pub(in crate::ui::retained_host::host_contract) sample: String,
    pub(in crate::ui::retained_host::host_contract) point: UiProfilePoint,
    pub(in crate::ui::retained_host::host_contract) expected_hit: bool,
    pub(in crate::ui::retained_host::host_contract) route_hit: bool,
}
