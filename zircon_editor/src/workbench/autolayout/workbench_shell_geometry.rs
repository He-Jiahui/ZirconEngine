use std::collections::BTreeMap;

use crate::layout::MainPageId;

use super::{ShellFrame, ShellRegionId};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkbenchShellGeometry {
    pub window_min_width: f32,
    pub window_min_height: f32,
    pub center_band_frame: ShellFrame,
    pub status_bar_frame: ShellFrame,
    pub region_frames: BTreeMap<ShellRegionId, ShellFrame>,
    pub splitter_frames: BTreeMap<ShellRegionId, ShellFrame>,
    pub floating_window_frames: BTreeMap<MainPageId, ShellFrame>,
    pub viewport_content_frame: ShellFrame,
}

impl WorkbenchShellGeometry {
    pub fn region_frame(&self, region: ShellRegionId) -> ShellFrame {
        self.region_frames.get(&region).copied().unwrap_or_default()
    }

    pub fn splitter_frame(&self, region: ShellRegionId) -> ShellFrame {
        self.splitter_frames
            .get(&region)
            .copied()
            .unwrap_or_default()
    }

    pub fn floating_window_frame(&self, window_id: &MainPageId) -> ShellFrame {
        self.floating_window_frames
            .get(window_id)
            .copied()
            .unwrap_or_default()
    }
}
