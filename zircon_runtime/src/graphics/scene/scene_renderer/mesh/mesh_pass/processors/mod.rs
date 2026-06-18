mod depth_prepass;
mod opaque_base;
mod shadow;
mod taa_reactive_mask;
mod transparent;
mod velocity;

pub(crate) use depth_prepass::DepthPrepassProcessor;
pub(crate) use opaque_base::OpaqueBasePassProcessor;
pub(crate) use shadow::ShadowPassProcessor;
pub(crate) use taa_reactive_mask::TaaReactiveMaskPassProcessor;
pub(crate) use transparent::TransparentPassProcessor;
pub(crate) use velocity::VelocityPassProcessor;

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        packed_sort_key_u64, CorePipelineKind, PrimitiveRelevance, RenderLayerSet,
        RenderMaterialAlphaMode, RenderPhase, RenderPhaseSortComponents,
    };
    use crate::core::framework::scene::Mobility;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
        MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DepthPrepassProcessor, MeshBatchRef, MeshBindHandle, MeshDrawArgs, MeshDrawCommandList,
        MeshGeometryHandle, MeshPassBuildContext, MeshPassPipelineKind, MeshPassProcessor,
        OpaqueBasePassProcessor, ShadowPassProcessor, TaaReactiveMaskPassProcessor,
        TransparentPassProcessor, VelocityPassProcessor,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantRegistry;

    #[test]
    fn processors_emit_expected_mesh_phases() {
        let mut list = MeshDrawCommandList::new();
        let mut variants = MeshPipelineVariantRegistry::default();
        let mut context = MeshPassBuildContext::with_default_quality(&mut variants);
        let opaque = batch(MeshDrawQueuePhase::Opaque, 10)
            .with_casts_shadow(true)
            .with_taa_reactive_mask_strength(0.75);
        let alpha = batch(MeshDrawQueuePhase::AlphaMask, 20)
            .with_casts_shadow(true)
            .with_taa_reactive_mask_strength(0.5);
        let transparent = batch(MeshDrawQueuePhase::Transparent, 30);

        DepthPrepassProcessor.add_mesh_batch(&opaque, &mut context, &mut list);
        OpaqueBasePassProcessor.add_mesh_batch(&opaque, &mut context, &mut list);
        OpaqueBasePassProcessor.add_mesh_batch(&alpha, &mut context, &mut list);
        TransparentPassProcessor.add_mesh_batch(&transparent, &mut context, &mut list);
        ShadowPassProcessor.add_mesh_batch(&alpha, &mut context, &mut list);
        TaaReactiveMaskPassProcessor.add_mesh_batch(&opaque, &mut context, &mut list);
        TaaReactiveMaskPassProcessor.add_mesh_batch(&alpha, &mut context, &mut list);
        TaaReactiveMaskPassProcessor.add_mesh_batch(&transparent, &mut context, &mut list);
        list.sort();

        let phases = list
            .commands()
            .iter()
            .map(|command| (command.phase, command.pipeline_kind))
            .collect::<Vec<_>>();

        assert_eq!(
            phases,
            vec![
                (RenderPhase::Prepass, MeshPassPipelineKind::DepthPrepass),
                (
                    RenderPhase::Shadow,
                    MeshPassPipelineKind::ShadowDepthAlphaMask
                ),
                (RenderPhase::Opaque3d, MeshPassPipelineKind::Base),
                (RenderPhase::AlphaMask3d, MeshPassPipelineKind::Base),
                (RenderPhase::Transparent3d, MeshPassPipelineKind::Base),
                (
                    RenderPhase::PostProcess,
                    MeshPassPipelineKind::TaaReactiveMaterialMask
                ),
                (
                    RenderPhase::PostProcess,
                    MeshPassPipelineKind::TaaReactiveMaterialMask
                ),
                (
                    RenderPhase::PostProcess,
                    MeshPassPipelineKind::TaaReactiveMask
                ),
            ]
        );
    }

    #[test]
    fn taa_reactive_mask_processor_draws_visible_main_view_batches_by_mask_semantics() {
        let mut list = MeshDrawCommandList::new();
        let mut variants = MeshPipelineVariantRegistry::default();
        let mut context = MeshPassBuildContext::with_default_quality(&mut variants);
        let opaque = batch(MeshDrawQueuePhase::Opaque, 1).with_taa_reactive_mask_strength(0.25);
        let alpha = batch(MeshDrawQueuePhase::AlphaMask, 4).with_taa_reactive_mask_strength(0.5);
        let transparent = batch(MeshDrawQueuePhase::Transparent, 2);
        let unflagged_opaque = batch(MeshDrawQueuePhase::Opaque, 6);
        let hidden_transparent = batch(MeshDrawQueuePhase::Transparent, 3).with_visibility(
            Some(transparent_relevance()),
            false,
            false,
        );
        let hidden_opaque = batch(MeshDrawQueuePhase::Opaque, 5)
            .with_taa_reactive_mask_strength(0.25)
            .with_visibility(None, false, false);

        TaaReactiveMaskPassProcessor.add_mesh_batch(&opaque, &mut context, &mut list);
        TaaReactiveMaskPassProcessor.add_mesh_batch(&alpha, &mut context, &mut list);
        TaaReactiveMaskPassProcessor.add_mesh_batch(&transparent, &mut context, &mut list);
        TaaReactiveMaskPassProcessor.add_mesh_batch(&unflagged_opaque, &mut context, &mut list);
        TaaReactiveMaskPassProcessor.add_mesh_batch(&hidden_transparent, &mut context, &mut list);
        TaaReactiveMaskPassProcessor.add_mesh_batch(&hidden_opaque, &mut context, &mut list);
        list.sort();

        let commands = list.commands();
        assert_eq!(commands.len(), 3);
        assert_eq!(
            commands
                .iter()
                .map(|command| (command.phase, command.pipeline_kind))
                .collect::<Vec<_>>(),
            vec![
                (
                    RenderPhase::PostProcess,
                    MeshPassPipelineKind::TaaReactiveMaterialMask
                ),
                (
                    RenderPhase::PostProcess,
                    MeshPassPipelineKind::TaaReactiveMaterialMask
                ),
                (
                    RenderPhase::PostProcess,
                    MeshPassPipelineKind::TaaReactiveMask
                ),
            ]
        );
    }

    #[test]
    fn velocity_processor_requires_velocity_history_and_previous_transform() {
        let mut list = MeshDrawCommandList::new();
        let mut variants = MeshPipelineVariantRegistry::default();
        let mut context = MeshPassBuildContext::with_default_quality(&mut variants);
        let dynamic_without_previous = batch(MeshDrawQueuePhase::Opaque, 1);
        let dynamic_with_previous =
            batch(MeshDrawQueuePhase::Opaque, 2).with_previous_velocity_transform(true);
        let transparent_with_previous =
            batch(MeshDrawQueuePhase::Transparent, 4).with_previous_velocity_transform(true);
        let static_with_previous = MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::Prepared,
                Mobility::Static,
                false,
                false,
                false,
            ),
            default_pipeline_key(),
            RenderPhaseSortComponents::new(3.0, 3),
            MeshGeometryHandle::test(3),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_gpu_scene_instance_span(3, 1)
        .with_previous_velocity_transform(true);

        VelocityPassProcessor.add_mesh_batch(&dynamic_without_previous, &mut context, &mut list);
        VelocityPassProcessor.add_mesh_batch(&dynamic_with_previous, &mut context, &mut list);
        VelocityPassProcessor.add_mesh_batch(&transparent_with_previous, &mut context, &mut list);
        VelocityPassProcessor.add_mesh_batch(&static_with_previous, &mut context, &mut list);

        let commands = list.commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].phase, RenderPhase::PostProcess);
        assert_eq!(commands[0].pipeline_kind, MeshPassPipelineKind::Velocity);
        assert_eq!(
            commands[0].sort_key,
            packed_sort_key_u64(
                RenderPhase::PostProcess,
                RenderPhaseSortComponents::new(2.0, 2),
                commands[0].pipeline_variant_id.value(),
                202,
            )
        );
    }

    #[test]
    fn processors_keep_shadow_candidate_when_main_view_layer_filters_mesh() {
        let mut list = MeshDrawCommandList::new();
        let mut variants = MeshPipelineVariantRegistry::default();
        let mut context = MeshPassBuildContext::with_default_quality(&mut variants);
        let hidden_main_shadow_caster = batch(MeshDrawQueuePhase::AlphaMask, 10)
            .with_casts_shadow(true)
            .with_visibility(Some(hidden_alpha_mask_relevance()), false, true);

        DepthPrepassProcessor.add_mesh_batch(&hidden_main_shadow_caster, &mut context, &mut list);
        OpaqueBasePassProcessor.add_mesh_batch(&hidden_main_shadow_caster, &mut context, &mut list);
        ShadowPassProcessor.add_mesh_batch(&hidden_main_shadow_caster, &mut context, &mut list);
        list.sort();

        let phases = list
            .commands()
            .iter()
            .map(|command| (command.phase, command.pipeline_kind))
            .collect::<Vec<_>>();

        assert_eq!(
            phases,
            vec![(
                RenderPhase::Shadow,
                MeshPassPipelineKind::ShadowDepthAlphaMask
            )]
        );
    }

    #[test]
    fn shadow_processor_respects_shadow_view_visibility() {
        let mut list = MeshDrawCommandList::new();
        let mut variants = MeshPipelineVariantRegistry::default();
        let mut context = MeshPassBuildContext::with_default_quality(&mut variants);
        let shadow_culled = batch(MeshDrawQueuePhase::Opaque, 10)
            .with_casts_shadow(true)
            .with_visibility(Some(visible_opaque_relevance()), true, false);

        ShadowPassProcessor.add_mesh_batch(&shadow_culled, &mut context, &mut list);

        assert!(list.commands().is_empty());
    }

    fn batch(phase: MeshDrawQueuePhase, sort_key: u64) -> MeshBatchRef {
        MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                phase,
                MeshDrawGeometrySource::Prepared,
                Mobility::Dynamic,
                false,
                false,
                false,
            ),
            default_pipeline_key(),
            RenderPhaseSortComponents::new(sort_key as f32, sort_key),
            MeshGeometryHandle::test(sort_key),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_gpu_scene_instance_span(sort_key as u32, 1)
        .with_material_textures(MeshBindHandle::test(sort_key + 100))
        .with_material(MeshBindHandle::test(sort_key + 200))
        .with_standard_material(MeshBindHandle::test(sort_key + 300))
    }

    fn visible_opaque_relevance() -> PrimitiveRelevance {
        PrimitiveRelevance::for_mesh_view(
            &RenderLayerSet::layer(0),
            CorePipelineKind::Core3d,
            1,
            Mobility::Dynamic,
            RenderMaterialAlphaMode::Opaque,
        )
    }

    fn hidden_alpha_mask_relevance() -> PrimitiveRelevance {
        PrimitiveRelevance::for_mesh_view(
            &RenderLayerSet::layer(0),
            CorePipelineKind::Core3d,
            1 << 4,
            Mobility::Static,
            RenderMaterialAlphaMode::Mask { cutoff: 0.5 },
        )
    }

    fn transparent_relevance() -> PrimitiveRelevance {
        PrimitiveRelevance::for_mesh_view(
            &RenderLayerSet::layer(0),
            CorePipelineKind::Core3d,
            1,
            Mobility::Dynamic,
            RenderMaterialAlphaMode::Blend,
        )
    }
}
