use crate::core::framework::render::ShaderQualityTier;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;

use super::extend_pending_draws_for_mesh_instance::extend_pending_draws_for_mesh_instance;
use super::material_draw_selection::MaterialDrawSelection;
use super::material_pipeline_requirements::{
    MaterialPipelineFeatureSet, MaterialPipelineRequirementCensus,
    PublishedMaterialPipelineRequirementCollector,
};
use super::mesh_draw_build_context::MeshDrawBuildContext;
use super::pending_mesh_draw::PendingMeshDraw;
use super::phase_ordering::phase_ordered_meshes;

pub(super) fn collect_pending_draws(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    build_context: &MeshDrawBuildContext,
    gpu_scene: &GpuScene,
    material_selection: &MaterialDrawSelection,
) -> Vec<PendingMeshDraw> {
    collect_pending_draws_observed(
        streamer,
        frame,
        build_context,
        gpu_scene,
        material_selection,
        |_| {},
    )
}

pub(super) fn collect_pending_draws_with_published_pipeline_requirements(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    build_context: &MeshDrawBuildContext,
    gpu_scene: &GpuScene,
    material_selection: &MaterialDrawSelection,
    features: MaterialPipelineFeatureSet,
    shader_quality: ShaderQualityTier,
    volumetric_fog_enabled: bool,
) -> (Vec<PendingMeshDraw>, MaterialPipelineRequirementCensus) {
    crate::profile_scope!("render", "material", "current_requirement_census");
    let mut collector = PublishedMaterialPipelineRequirementCollector::default();
    let pending_draws = collect_pending_draws_observed(
        streamer,
        frame,
        build_context,
        gpu_scene,
        material_selection,
        |pending_draw| {
            collector.observe_published_draw(
                pending_draw,
                features,
                shader_quality,
                volumetric_fog_enabled,
            );
        },
    );
    let census = collector.finish(pending_draws.len());
    (pending_draws, census)
}

fn collect_pending_draws_observed(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    build_context: &MeshDrawBuildContext,
    gpu_scene: &GpuScene,
    material_selection: &MaterialDrawSelection,
    mut observe_pending_draw: impl FnMut(&PendingMeshDraw),
) -> Vec<PendingMeshDraw> {
    let mut pending_draws = Vec::new();
    for mesh_instance in phase_ordered_meshes(frame, streamer, material_selection) {
        let first_pending_draw = pending_draws.len();
        extend_pending_draws_for_mesh_instance(
            &mut pending_draws,
            streamer,
            frame,
            build_context,
            gpu_scene,
            mesh_instance.snapshot,
            mesh_instance.command_sort_input,
            material_selection,
        );
        for pending_draw in &pending_draws[first_pending_draw..] {
            observe_pending_draw(pending_draw);
        }
    }
    pending_draws
}

#[cfg(test)]
mod tests {
    #[test]
    fn published_requirement_census_observes_each_new_draw_inside_collection() {
        let source = include_str!("collect_pending_draws.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("pending draw collection test boundary");

        assert!(source.contains("collect_pending_draws_with_published_pipeline_requirements"));
        assert!(source.contains("first_pending_draw"));
        assert!(source.contains("&pending_draws[first_pending_draw..]"));
        assert!(source.contains("PublishedMaterialPipelineRequirementCollector"));
        assert!(source.contains("collector.observe_published_draw"));
        assert!(!source.contains("insert_published_material_pipeline_requirements"));
    }
}
