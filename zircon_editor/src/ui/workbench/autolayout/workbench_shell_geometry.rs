use std::collections::BTreeMap;

use crate::ui::workbench::layout::MainPageId;

use super::{ResolutionContext, ShellFrame, ShellRegionId};

#[derive(Clone, Debug, Default, PartialEq)]
/// 完成解算后给host绘制、命中及native尺寸策略共用的结果；总入口返回时frame为物理坐标。
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
    /// 未占位region返回零frame，调用方还需结合模型决定是否绘制/交互。
    pub fn region_frame(&self, region: ShellRegionId) -> ShellFrame {
        self.region_frames.get(&region).copied().unwrap_or_default()
    }

    pub fn splitter_frame(&self, region: ShellRegionId) -> ShellFrame {
        self.splitter_frames
            .get(&region)
            .copied()
            .unwrap_or_default()
    }

    /// 按布局稳定窗口ID取浮层frame；缺失ID表示本轮没有对应几何。
    pub fn floating_window_frame(&self, window_id: &MainPageId) -> ShellFrame {
        self.floating_window_frames
            .get(window_id)
            .copied()
            .unwrap_or_default()
    }

    /// 复用挂载布局只比较可见frames；native最小尺寸变化由宿主窗口策略独立处理。
    pub(crate) fn shares_mounted_layout_frames_with(&self, other: &Self) -> bool {
        self.center_band_frame == other.center_band_frame
            && self.status_bar_frame == other.status_bar_frame
            && self.region_frames == other.region_frames
            && self.splitter_frames == other.splitter_frames
            && self.floating_window_frames == other.floating_window_frames
            && self.viewport_content_frame == other.viewport_content_frame
    }

    /// Converts the finished logical layout to the physical host coordinate space once.
    /// 总builder完成logical解算后调用一次；重复调用会再次缩放，不能对已发布结果使用。
    pub(crate) fn scaled_to_physical(mut self, resolution: ResolutionContext) -> Self {
        self.window_min_width = resolution.to_physical(self.window_min_width);
        self.window_min_height = resolution.to_physical(self.window_min_height);
        self.center_band_frame = scale_frame(self.center_band_frame, resolution);
        self.status_bar_frame = scale_frame(self.status_bar_frame, resolution);
        self.viewport_content_frame = scale_frame(self.viewport_content_frame, resolution);
        for frame in self.region_frames.values_mut() {
            *frame = scale_frame(*frame, resolution);
        }
        for frame in self.splitter_frames.values_mut() {
            *frame = scale_frame(*frame, resolution);
        }
        for frame in self.floating_window_frames.values_mut() {
            *frame = scale_frame(*frame, resolution);
        }
        self
    }
}

#[cfg(test)]
#[path = "tests/workbench_shell_geometry.rs"]
mod tests;

fn scale_frame(frame: ShellFrame, resolution: ResolutionContext) -> ShellFrame {
    ShellFrame::new(
        resolution.to_physical(frame.x),
        resolution.to_physical(frame.y),
        resolution.to_physical(frame.width),
        resolution.to_physical(frame.height),
    )
}
