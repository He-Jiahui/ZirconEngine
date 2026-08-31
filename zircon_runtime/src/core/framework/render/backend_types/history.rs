use super::handles::FrameHistoryHandle;
use crate::core::math::UVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderHistoryDomain {
    TaaSceneColor = 0,
    HybridGlobalIllumination = 1,
    AmbientOcclusion = 2,
    ScreenSpaceReflection = 3,
    HzbFurthest = 4,
    Exposure = 5,
    VolumetricScattering = 6,
}

impl RenderHistoryDomain {
    pub const COUNT: usize = 7;
    pub const ALL: [Self; Self::COUNT] = [
        Self::TaaSceneColor,
        Self::HybridGlobalIllumination,
        Self::AmbientOcclusion,
        Self::ScreenSpaceReflection,
        Self::HzbFurthest,
        Self::Exposure,
        Self::VolumetricScattering,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::TaaSceneColor => "taa_scene_color",
            Self::HybridGlobalIllumination => "hybrid_global_illumination",
            Self::AmbientOcclusion => "ambient_occlusion",
            Self::ScreenSpaceReflection => "screen_space_reflection",
            Self::HzbFurthest => "hzb_furthest",
            Self::Exposure => "exposure",
            Self::VolumetricScattering => "volumetric_scattering",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderHistoryDomainResetReason {
    NeverProduced,
    PreviousFrameUnavailable,
    CameraCut,
    AllocationChanged,
    FeatureDisabled,
    SourceUnavailable,
    StructuralCompatibilityChanged,
}

impl RenderHistoryDomainResetReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NeverProduced => "never_produced",
            Self::PreviousFrameUnavailable => "previous_frame_unavailable",
            Self::CameraCut => "camera_cut",
            Self::AllocationChanged => "allocation_changed",
            Self::FeatureDisabled => "feature_disabled",
            Self::SourceUnavailable => "source_unavailable",
            Self::StructuralCompatibilityChanged => "structural_compatibility_changed",
        }
    }

    pub const fn diagnostic_code(self) -> usize {
        match self {
            Self::NeverProduced => 1,
            Self::PreviousFrameUnavailable => 2,
            Self::CameraCut => 3,
            Self::AllocationChanged => 4,
            Self::FeatureDisabled => 5,
            Self::SourceUnavailable => 6,
            Self::StructuralCompatibilityChanged => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHistoryDomainStatus {
    pub generation: u64,
    pub valid: bool,
    pub last_successful_frame: Option<u64>,
    pub active_reset_reason: Option<RenderHistoryDomainResetReason>,
    pub frame_reset_reason: Option<RenderHistoryDomainResetReason>,
}

impl RenderHistoryDomainStatus {
    pub const fn new(
        generation: u64,
        valid: bool,
        last_successful_frame: Option<u64>,
        active_reset_reason: Option<RenderHistoryDomainResetReason>,
        frame_reset_reason: Option<RenderHistoryDomainResetReason>,
    ) -> Self {
        Self {
            generation,
            valid,
            last_successful_frame,
            active_reset_reason,
            frame_reset_reason,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHistoryDomainsReport {
    pub history_target_present: bool,
    states: [RenderHistoryDomainStatus; RenderHistoryDomain::COUNT],
}

impl RenderHistoryDomainsReport {
    pub const fn new(
        history_target_present: bool,
        states: [RenderHistoryDomainStatus; RenderHistoryDomain::COUNT],
    ) -> Self {
        Self {
            history_target_present,
            states,
        }
    }

    pub const fn state(self, domain: RenderHistoryDomain) -> RenderHistoryDomainStatus {
        self.states[domain.index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameHistoryInvalidationReason {
    NoPreviousFrame,
    ViewportResized,
    RenderSizeChanged,
    PipelineChanged,
    HistoryBindingChanged,
    FrameInputsChanged,
    CameraCut,
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
            Self::CameraCut => "camera_cut",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderFrameHistoryInput {
    current: Option<FrameHistoryHandle>,
    previous_available: bool,
    invalidation_reason: Option<FrameHistoryInvalidationReason>,
}

impl RenderFrameHistoryInput {
    pub(crate) const fn new(
        current: Option<FrameHistoryHandle>,
        previous_available: bool,
        invalidation_reason: Option<FrameHistoryInvalidationReason>,
    ) -> Self {
        Self {
            current,
            previous_available,
            invalidation_reason,
        }
    }

    pub(crate) const fn current(self) -> Option<FrameHistoryHandle> {
        self.current
    }

    pub(crate) const fn previous_available(self) -> bool {
        self.previous_available
    }

    pub(crate) const fn invalidation_reason(self) -> Option<FrameHistoryInvalidationReason> {
        self.invalidation_reason
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
    pub volumetric_scattering_copied: bool,
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
        volumetric_scattering_copied: bool,
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
                + exposure_copied as usize
                + volumetric_scattering_copied as usize,
            scene_color_copied,
            global_illumination_copied,
            ambient_occlusion_copied,
            screen_space_reflection_copied,
            hzb_furthest_copied,
            exposure_copied,
            volumetric_scattering_copied,
        }
    }
}
