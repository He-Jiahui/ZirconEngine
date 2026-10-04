use super::{PostProcessChainSlot, PostProcessEffectKind, RenderPipelinePhase};

#[test]
fn render_post_chain_backbone_order_is_stable() {
    let labels = PostProcessChainSlot::fixed_backbone()
        .iter()
        .map(|slot| slot.label())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "depth-of-field",
            "taa-resolve",
            "motion-blur",
            "bloom",
            "exposure-histogram",
            "exposure-resolve",
            "scene-composite",
            "blur",
            "color-lut-bake",
            "uber",
            "terminal-anti-alias",
            "primary-upscale",
            "secondary-upscale",
            "output-transfer",
        ]
    );
}

#[test]
fn render_post_chain_current_effect_kinds_have_migration_slots() {
    let mappings = [
        (
            PostProcessEffectKind::TaaResolve,
            PostProcessChainSlot::TaaResolve,
        ),
        (PostProcessEffectKind::Bloom, PostProcessChainSlot::Bloom),
        (
            PostProcessEffectKind::DepthOfField,
            PostProcessChainSlot::DepthOfField,
        ),
        (
            PostProcessEffectKind::MotionBlur,
            PostProcessChainSlot::MotionBlur,
        ),
        (
            PostProcessEffectKind::ExposureHistogram,
            PostProcessChainSlot::ExposureHistogram,
        ),
        (
            PostProcessEffectKind::ExposureResolve,
            PostProcessChainSlot::ExposureResolve,
        ),
        (
            PostProcessEffectKind::SceneComposite,
            PostProcessChainSlot::SceneComposite,
        ),
        (PostProcessEffectKind::Blur, PostProcessChainSlot::Blur),
        (
            PostProcessEffectKind::ColorLutBake,
            PostProcessChainSlot::ColorLutBake,
        ),
        (PostProcessEffectKind::Uber, PostProcessChainSlot::Uber),
        (
            PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid,
            PostProcessChainSlot::SceneComposite,
        ),
        (
            PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse,
            PostProcessChainSlot::SceneComposite,
        ),
        (
            PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion,
            PostProcessChainSlot::SceneComposite,
        ),
        (
            PostProcessEffectKind::ScreenSpaceReflectionResolve,
            PostProcessChainSlot::SceneComposite,
        ),
        (
            PostProcessEffectKind::PrimaryUpscale,
            PostProcessChainSlot::PrimaryUpscale,
        ),
        (
            PostProcessEffectKind::SecondaryUpscale,
            PostProcessChainSlot::SecondaryUpscale,
        ),
        (
            PostProcessEffectKind::OutputTransfer,
            PostProcessChainSlot::OutputTransfer,
        ),
        (
            PostProcessEffectKind::Fxaa,
            PostProcessChainSlot::TerminalAntiAlias,
        ),
        (
            PostProcessEffectKind::Smaa,
            PostProcessChainSlot::TerminalAntiAlias,
        ),
    ];

    for (kind, expected_slot) in mappings {
        assert_eq!(
            PostProcessChainSlot::from_current_effect_kind(kind),
            expected_slot,
            "{kind} should be assigned to the PP-M1 migration slot"
        );
    }
}

#[test]
fn render_post_chain_planned_executor_ids_are_stable() {
    let executor_ids = PostProcessChainSlot::fixed_backbone()
        .iter()
        .map(|slot| slot.planned_executor_id())
        .collect::<Vec<_>>();

    assert_eq!(
        executor_ids,
        vec![
            "post.depth-of-field",
            "temporal.taa-resolve",
            "post.motion-blur",
            "post.bloom",
            "post.exposure.histogram",
            "post.exposure.resolve",
            "post.scene-composite",
            "post.blur",
            "post.color-lut-bake",
            "post.uber",
            "post.terminal-aa",
            "post.primary-upscale",
            "post.secondary-upscale",
            "post.output-transfer",
        ]
    );
}

#[test]
fn render_post_chain_slots_map_to_view_family_phases() {
    let mappings = [
        (
            PostProcessChainSlot::DepthOfField,
            RenderPipelinePhase::PreReconstructionScenePostProcess,
        ),
        (
            PostProcessChainSlot::TaaResolve,
            RenderPipelinePhase::TemporalReconstruction,
        ),
        (
            PostProcessChainSlot::MotionBlur,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
        ),
        (
            PostProcessChainSlot::Bloom,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
        ),
        (
            PostProcessChainSlot::ExposureHistogram,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
        ),
        (
            PostProcessChainSlot::ExposureResolve,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
        ),
        (
            PostProcessChainSlot::SceneComposite,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
        ),
        (
            PostProcessChainSlot::Blur,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
        ),
        (
            PostProcessChainSlot::ColorLutBake,
            RenderPipelinePhase::DisplayMapping,
        ),
        (
            PostProcessChainSlot::Uber,
            RenderPipelinePhase::DisplayMapping,
        ),
        (
            PostProcessChainSlot::TerminalAntiAlias,
            RenderPipelinePhase::DisplayPostProcess,
        ),
        (
            PostProcessChainSlot::PrimaryUpscale,
            RenderPipelinePhase::PrimarySpatialUpscale,
        ),
        (
            PostProcessChainSlot::SecondaryUpscale,
            RenderPipelinePhase::SecondarySpatialUpscale,
        ),
        (
            PostProcessChainSlot::OutputTransfer,
            RenderPipelinePhase::OutputTransform,
        ),
    ];

    for (slot, expected_phase) in mappings {
        assert_eq!(slot.pipeline_phase(), expected_phase, "{slot}");
    }
}
