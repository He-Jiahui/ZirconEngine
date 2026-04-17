use std::collections::BTreeMap;

use super::super::{ShellFrame, ShellRegionId};

pub(super) struct ResolvedRegionFrames {
    pub(super) center_band_frame: ShellFrame,
    pub(super) region_frames: BTreeMap<ShellRegionId, ShellFrame>,
    pub(super) left_frame: ShellFrame,
    pub(super) document_frame: ShellFrame,
    pub(super) right_frame: ShellFrame,
    pub(super) bottom_frame: ShellFrame,
}
