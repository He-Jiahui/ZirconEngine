use super::handles::FrameHistoryHandle;
use crate::core::math::UVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameHistoryInvalidationReason {
    NoPreviousFrame,
    ViewportResized,
    RenderSizeChanged,
    PipelineChanged,
    HistoryBindingChanged,
    FrameInputsChanged,
}

impl FrameHistoryInvalidationReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NoPreviousFrame => "no_previous_frame",
            Self::ViewportResized => "viewport_resized",
            Self::RenderSizeChanged => "render_size_changed",
            Self::PipelineChanged => "pipeline_changed",
            Self::HistoryBindingChanged => "history_binding_changed",
            Self::FrameInputsChanged => "frame_inputs_changed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameHistoryStatus {
    pub current: Option<FrameHistoryHandle>,
    pub previous: Option<FrameHistoryHandle>,
    pub previous_available: bool,
    pub invalidation_reason: Option<FrameHistoryInvalidationReason>,
    pub target_size: UVec2,
    pub render_size: UVec2,
}

impl FrameHistoryStatus {
    pub const fn new(
        current: Option<FrameHistoryHandle>,
        previous: Option<FrameHistoryHandle>,
        previous_available: bool,
        invalidation_reason: Option<FrameHistoryInvalidationReason>,
        target_size: UVec2,
        render_size: UVec2,
    ) -> Self {
        Self {
            current,
            previous,
            previous_available,
            invalidation_reason,
            target_size,
            render_size,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHistoryCopyReport {
    pub history_target_present: bool,
    pub debug_marker_emitted: bool,
    pub target_size: UVec2,
    pub requested_copy_count: usize,
    pub copied_count: usize,
    pub scene_color_copied: bool,
    pub global_illumination_copied: bool,
    pub ambient_occlusion_copied: bool,
    pub screen_space_reflection_copied: bool,
    pub hzb_furthest_copied: bool,
    pub exposure_copied: bool,
}

impl RenderHistoryCopyReport {
    pub fn new(
        history_target_present: bool,
        target_size: UVec2,
        requested_copy_count: usize,
        scene_color_copied: bool,
        global_illumination_copied: bool,
        ambient_occlusion_copied: bool,
        screen_space_reflection_copied: bool,
        hzb_furthest_copied: bool,
        exposure_copied: bool,
    ) -> Self {
        Self {
            history_target_present,
            debug_marker_emitted: history_target_present && requested_copy_count > 0,
            target_size,
            requested_copy_count,
            copied_count: scene_color_copied as usize
                + global_illumination_copied as usize
                + ambient_occlusion_copied as usize
                + screen_space_reflection_copied as usize
                + hzb_furthest_copied as usize
                + exposure_copied as usize,
            scene_color_copied,
            global_illumination_copied,
            ambient_occlusion_copied,
            screen_space_reflection_copied,
            hzb_furthest_copied,
            exposure_copied,
        }
    }
}
