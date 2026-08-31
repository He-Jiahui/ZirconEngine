use std::sync::OnceLock;

use super::super::{
    AntiAliasSettings, AoSourceSettings, DisplayMode, FallbackSkyboxKind, PostProcessPassGraph,
    PostProcessStackDescriptor, PostProcessVolumeExtract, PreviewEnvironmentExtract,
    RenderBloomSettings, RenderColorGradingSettings, RenderExposureSettings, RenderLayerSet,
    RenderPostProcessEffectStackSettings, RenderResolvedPostProcessSettings, VolumeEvaluationError,
    VolumeEvaluationRequest, VolumeEvaluator, DEFAULT_CAMERA_EXPOSURE_EV100,
};

#[derive(Clone, Debug, PartialEq)]
pub struct PostProcessExtract {
    pub preview: PreviewEnvironmentExtract,
    pub display_mode: DisplayMode,
    pub ambient_occlusion: AoSourceSettings,
    pub bloom: RenderBloomSettings,
    pub exposure: RenderExposureSettings,
    pub color_grading: RenderColorGradingSettings,
    pub effect_stack: RenderPostProcessEffectStackSettings,
    pub volumes: Vec<PostProcessVolumeExtract>,
    pub stack: PostProcessStackDescriptor,
    pub graph: PostProcessPassGraph,
}

impl Default for PostProcessExtract {
    fn default() -> Self {
        let bloom = RenderBloomSettings::default();
        let color_grading = RenderColorGradingSettings::default();
        Self::from_parts_with_effect_stack(
            PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: crate::core::math::Vec4::ZERO,
            },
            DisplayMode::Shaded,
            bloom,
            color_grading,
            RenderPostProcessEffectStackSettings::default(),
            false,
            false,
        )
    }
}

impl PostProcessExtract {
    pub fn from_parts(
        preview: PreviewEnvironmentExtract,
        display_mode: DisplayMode,
        bloom: RenderBloomSettings,
        color_grading: RenderColorGradingSettings,
        temporal_history_enabled: bool,
        history_available: bool,
    ) -> Self {
        Self::from_parts_with_effect_stack(
            preview,
            display_mode,
            bloom,
            color_grading,
            RenderPostProcessEffectStackSettings::default(),
            temporal_history_enabled,
            history_available,
        )
    }

    pub fn from_parts_with_effect_stack(
        preview: PreviewEnvironmentExtract,
        display_mode: DisplayMode,
        bloom: RenderBloomSettings,
        color_grading: RenderColorGradingSettings,
        effect_stack: RenderPostProcessEffectStackSettings,
        temporal_history_enabled: bool,
        history_available: bool,
    ) -> Self {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
                &bloom,
                &color_grading,
                RenderExposureSettings::manual_ev100(DEFAULT_CAMERA_EXPOSURE_EV100),
                &effect_stack,
                temporal_history_enabled,
                history_available,
                &AntiAliasSettings::off(),
            );
        let graph = stack.validated_graph();
        Self {
            preview,
            display_mode,
            ambient_occlusion: AoSourceSettings::default(),
            bloom,
            exposure: RenderExposureSettings::manual_ev100(DEFAULT_CAMERA_EXPOSURE_EV100),
            color_grading,
            effect_stack,
            volumes: Vec::new(),
            stack,
            graph,
        }
    }

    pub fn rebuild_graph(&mut self, temporal_history_enabled: bool, history_available: bool) {
        self.stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
                &self.bloom,
                &self.color_grading,
                self.exposure,
                &self.effect_stack,
                temporal_history_enabled,
                history_available,
                &AntiAliasSettings::off(),
            );
        self.graph = self.stack.validated_graph();
    }

    pub fn rebuild_graph_with_anti_alias(
        &mut self,
        temporal_history_enabled: bool,
        history_available: bool,
        anti_alias: &AntiAliasSettings,
    ) {
        self.stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
                &self.bloom,
                &self.color_grading,
                self.exposure,
                &self.effect_stack,
                temporal_history_enabled,
                history_available,
                anti_alias,
            );
        self.graph = self.stack.validated_graph();
    }

    pub fn resolved_settings_for_camera(
        &self,
        camera_position: crate::core::math::Vec3,
        camera_volume_mask: &RenderLayerSet,
    ) -> Result<RenderResolvedPostProcessSettings, VolumeEvaluationError> {
        builtin_volume_evaluator().evaluate(VolumeEvaluationRequest {
            camera_position,
            camera_volume_mask,
            base_ambient_occlusion: self.ambient_occlusion,
            base_bloom: self.bloom,
            base_exposure: self.exposure,
            base_color_grading: self.color_grading,
            base_effect_stack: self.effect_stack,
            volumes: &self.volumes,
        })
    }
}

fn builtin_volume_evaluator() -> &'static VolumeEvaluator {
    static EVALUATOR: OnceLock<VolumeEvaluator> = OnceLock::new();
    EVALUATOR.get_or_init(VolumeEvaluator::default)
}
