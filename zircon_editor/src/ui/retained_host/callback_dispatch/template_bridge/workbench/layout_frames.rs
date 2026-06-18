use zircon_runtime_interface::ui::layout::UiFrame;

use crate::ui::workbench::autolayout::{ShellRegionId, WorkbenchChromeMetrics};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BuiltinWorkbenchWindowLayoutFrames {
    pub center_band_frame: Option<UiFrame>,
    pub activity_rail_frame: Option<UiFrame>,
    pub left_region_frame: Option<UiFrame>,
    pub left_drawer_shell_frame: Option<UiFrame>,
    pub left_drawer_header_frame: Option<UiFrame>,
    pub left_drawer_content_frame: Option<UiFrame>,
    pub document_tabs_frame: Option<UiFrame>,
    pub document_region_frame: Option<UiFrame>,
    pub right_drawer_shell_frame: Option<UiFrame>,
    pub right_drawer_header_frame: Option<UiFrame>,
    pub right_drawer_content_frame: Option<UiFrame>,
    pub right_region_frame: Option<UiFrame>,
    pub bottom_drawer_shell_frame: Option<UiFrame>,
    pub bottom_drawer_header_frame: Option<UiFrame>,
    pub bottom_drawer_content_frame: Option<UiFrame>,
    pub bottom_region_frame: Option<UiFrame>,
    pub status_bar_frame: Option<UiFrame>,
    pub viewport_toolbar_frame: Option<UiFrame>,
    pub viewport_content_frame: Option<UiFrame>,
    pub left_resize_splitter_frame: Option<UiFrame>,
    pub right_resize_splitter_frame: Option<UiFrame>,
    pub bottom_resize_splitter_frame: Option<UiFrame>,
}

impl BuiltinWorkbenchWindowLayoutFrames {
    pub(crate) fn drawer_shell_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_shell_frame,
            ShellRegionId::Right => self.right_drawer_shell_frame,
            ShellRegionId::Bottom => self.bottom_drawer_shell_frame,
            ShellRegionId::Document => None,
        }
    }

    pub(crate) fn drawer_header_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_header_frame,
            ShellRegionId::Right => self.right_drawer_header_frame,
            ShellRegionId::Bottom => self.bottom_drawer_header_frame,
            ShellRegionId::Document => None,
        }
    }

    pub(crate) fn drawer_content_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_content_frame,
            ShellRegionId::Right => self.right_drawer_content_frame,
            ShellRegionId::Bottom => self.bottom_drawer_content_frame,
            ShellRegionId::Document => None,
        }
    }

    pub(crate) fn resize_splitter_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_resize_splitter_frame,
            ShellRegionId::Right => self.right_resize_splitter_frame,
            ShellRegionId::Bottom => self.bottom_resize_splitter_frame,
            ShellRegionId::Document => None,
        }
    }
}

pub(super) fn union_visible_frames(
    frames: impl IntoIterator<Item = Option<UiFrame>>,
) -> Option<UiFrame> {
    frames
        .into_iter()
        .flatten()
        .filter(ui_frame_is_visible)
        .reduce(union_frames)
}

pub(super) fn left_resize_splitter_frame_from_drawer_shell(
    drawer_shell_frame: Option<UiFrame>,
) -> Option<UiFrame> {
    drawer_shell_frame
        .filter(ui_frame_is_visible)
        .map(|frame| vertical_splitter_frame_at(frame.x + frame.width, frame.y, frame.height))
}

pub(super) fn right_resize_splitter_frame_from_drawer_shell(
    drawer_shell_frame: Option<UiFrame>,
) -> Option<UiFrame> {
    let metrics = WorkbenchChromeMetrics::default();
    drawer_shell_frame.filter(ui_frame_is_visible).map(|frame| {
        vertical_splitter_frame_at(
            frame.x - metrics.separator_thickness.max(0.0),
            frame.y,
            frame.height,
        )
    })
}

pub(super) fn bottom_resize_splitter_frame_from_drawer_shell(
    drawer_shell_frame: Option<UiFrame>,
) -> Option<UiFrame> {
    let metrics = WorkbenchChromeMetrics::default();
    drawer_shell_frame.filter(ui_frame_is_visible).map(|frame| {
        UiFrame::new(
            frame.x,
            frame.y
                - metrics.separator_thickness.max(0.0)
                - metrics.splitter_hit_size.max(0.0) * 0.5,
            frame.width,
            metrics.splitter_hit_size.max(0.0),
        )
    })
}

fn union_frames(left: UiFrame, right: UiFrame) -> UiFrame {
    let min_x = left.x.min(right.x);
    let min_y = left.y.min(right.y);
    let max_x = (left.x + left.width).max(right.x + right.width);
    let max_y = (left.y + left.height).max(right.y + right.height);
    UiFrame::new(
        min_x,
        min_y,
        (max_x - min_x).max(0.0),
        (max_y - min_y).max(0.0),
    )
}

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

fn vertical_splitter_frame_at(edge_x: f32, y: f32, height: f32) -> UiFrame {
    let metrics = WorkbenchChromeMetrics::default();
    UiFrame::new(
        edge_x - metrics.splitter_hit_size.max(0.0) * 0.5,
        y,
        metrics.splitter_hit_size.max(0.0),
        height.max(0.0),
    )
}
