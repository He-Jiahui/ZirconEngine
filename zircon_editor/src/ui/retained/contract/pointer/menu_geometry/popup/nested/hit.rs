use crate::ui::retained_host::host_contract::data::{FrameRect, HostMenuChromeItemData};
use crate::ui::retained_host::primitives::ModelRc;

pub(super) struct NestedMenuLevelHit {
    pub(super) items: ModelRc<HostMenuChromeItemData>,
    pub(super) popup: FrameRect,
    pub(super) contains_point: bool,
}
