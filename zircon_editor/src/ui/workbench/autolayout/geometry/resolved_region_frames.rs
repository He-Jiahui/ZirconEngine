use std::collections::BTreeMap;

use super::super::{ShellFrame, ShellRegionId};

/// region solver的logical中间结果；供分隔器、视口和浮层计算共用，发布前统一物理缩放。
pub(super) struct ResolvedRegionFrames {
    pub(super) center_band_frame: ShellFrame,
    pub(super) status_bar_frame: ShellFrame,
    pub(super) region_frames: BTreeMap<ShellRegionId, ShellFrame>,
    pub(super) left_frame: ShellFrame,
    pub(super) document_frame: ShellFrame,
    pub(super) right_frame: ShellFrame,
    pub(super) bottom_frame: ShellFrame,
}
