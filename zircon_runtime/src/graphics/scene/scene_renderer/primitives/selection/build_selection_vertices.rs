use std::collections::HashMap;

use crate::core::framework::render::RenderMeshSnapshot;
use crate::core::framework::scene::EntityId;
use crate::core::math::Vec4;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

use super::super::line_geometry::{append_bounding_box_vertices, append_cross};

const INDEXED_SELECTION_LOOKUP_THRESHOLD: usize = 8;

pub(crate) fn build_selection_vertices(
    frame: &ViewportRenderFrame,
    streamer: &ResourceStreamer,
) -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    if let Some(highlights) = frame.overlays().highlights.as_ref() {
        if highlights.attributes().outline_enabled {
            let tint = highlights.attributes().tint_rgba;
            let color = Vec4::new(tint[0], tint[1], tint[2], tint[3]);
            visit_selected_meshes(frame.meshes(), highlights.entities(), |mesh_instance| {
                let Some(model) = streamer.model(&mesh_instance.model.id()) else {
                    return;
                };
                for mesh in &model.meshes {
                    append_bounding_box_vertices(
                        &mut vertices,
                        mesh.bounds_min,
                        mesh.bounds_max,
                        mesh_instance.transform.matrix(),
                        color,
                    );
                }
            });
        }
    }

    let camera = frame.effective_camera();
    for anchor in &frame.overlays().selection_anchors {
        append_cross(
            &mut vertices,
            anchor.position,
            anchor.size,
            anchor.color,
            camera.transform.right(),
            camera.transform.up(),
        );
    }

    vertices
}

fn visit_selected_meshes<'a>(
    meshes: &'a [RenderMeshSnapshot],
    entities: &[EntityId],
    mut visit: impl FnMut(&'a RenderMeshSnapshot),
) {
    if entities.len() <= INDEXED_SELECTION_LOOKUP_THRESHOLD {
        for owner in entities {
            if let Some(mesh) = meshes.iter().find(|mesh| mesh.node_id == *owner) {
                visit(mesh);
            }
        }
        return;
    }

    let mut meshes_by_owner = HashMap::with_capacity(meshes.len());
    for mesh in meshes {
        meshes_by_owner.entry(mesh.node_id).or_insert(mesh);
    }
    for owner in entities {
        if let Some(mesh) = meshes_by_owner.get(owner) {
            visit(mesh);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::core::framework::render::{RenderLayerSet, RenderMeshSnapshot};
    use crate::core::framework::scene::{EntityId, Mobility};
    use crate::core::math::{Transform, Vec4};
    use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

    use super::visit_selected_meshes;

    #[test]
    fn optimization_wave_20260824f_runtime93_selection_lookup_preserves_order_and_first_match() {
        let mut meshes = (0..16)
            .map(|node_id| test_mesh(node_id, node_id * 10))
            .collect::<Vec<_>>();
        meshes.push(test_mesh(7, 7_777));

        let mut small = Vec::new();
        visit_selected_meshes(&meshes, &[7], |mesh| small.push(mesh.stable_instance_key));
        assert_eq!(small, [70]);

        let selected = [9, 3, 7, 15, 4, 8, 2, 14, 1, 99];
        let mut indexed = Vec::new();
        visit_selected_meshes(&meshes, &selected, |mesh| {
            indexed.push(mesh.stable_instance_key)
        });
        assert_eq!(indexed, [90, 30, 70, 150, 40, 80, 20, 140, 10]);
    }

    #[test]
    fn optimization_wave_20260824f_runtime93_selection_lookup_uses_an_adaptive_index() {
        let source = include_str!("build_selection_vertices.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("selection vertex implementation");

        assert!(production.contains("INDEXED_SELECTION_LOOKUP_THRESHOLD"));
        assert!(production.contains("HashMap::with_capacity(meshes.len())"));
        assert!(production.contains(".entry(mesh.node_id)"));
        assert!(production.contains(".or_insert(mesh)"));
    }

    #[test]
    #[ignore = "managed release evidence"]
    fn optimization_wave_20260824f_runtime93_selection_lookup_evidence() {
        const MESH_COUNT: usize = 10_000;
        const SELECTION_COUNT: usize = 2_000;
        const SAMPLE_PAIRS: usize = 11;
        const TARGET: Duration = Duration::from_millis(100);

        let meshes = (0..MESH_COUNT as u64)
            .map(|node_id| test_mesh(node_id, node_id))
            .collect::<Vec<_>>();
        let selected = (MESH_COUNT - SELECTION_COUNT..MESH_COUNT)
            .map(|node_id| node_id as EntityId)
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            let measure_legacy = || measure_ns(|| legacy_selection_checksum(&meshes, &selected));
            let measure_optimized = || {
                measure_ns(|| {
                    let mut checksum = 0_u64;
                    visit_selected_meshes(&meshes, &selected, |mesh| {
                        checksum = checksum.wrapping_add(black_box(mesh.stable_instance_key));
                    });
                    checksum
                })
            };
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let comparisons_before = selected
            .iter()
            .map(|entity| *entity as usize + 1)
            .sum::<usize>();
        let probes_after = MESH_COUNT + SELECTION_COUNT;
        let lookup_work_reduction_percent =
            (1.0 - probes_after as f64 / comparisons_before as f64) * 100.0;

        assert!(
            optimized_p95 <= TARGET.as_nanos(),
            "optimized_p95_ns={optimized_p95} target_ns={}",
            TARGET.as_nanos()
        );
        assert!(
            optimized_p95.saturating_mul(2) <= legacy_p95,
            "optimized_p95_ns={optimized_p95} legacy_p95_ns={legacy_p95}"
        );
        println!(
            "RUNTIME93_SELECTION_LOOKUP_BENCH_V1 meshes={} selected={} comparisons_before={} probes_after={} lookup_work_reduction_percent={:.4} legacy_p95_ns={} optimized_p95_ns={} target_ns={}",
            MESH_COUNT,
            SELECTION_COUNT,
            comparisons_before,
            probes_after,
            lookup_work_reduction_percent,
            legacy_p95,
            optimized_p95,
            TARGET.as_nanos()
        );
    }

    fn legacy_selection_checksum(meshes: &[RenderMeshSnapshot], selected: &[EntityId]) -> u64 {
        let mut checksum = 0_u64;
        for owner in selected {
            if let Some(mesh) = meshes.iter().find(|mesh| mesh.node_id == *owner) {
                checksum = checksum.wrapping_add(black_box(mesh.stable_instance_key));
            }
        }
        checksum
    }

    fn measure_ns(measure: impl FnOnce() -> u64) -> u128 {
        let started = Instant::now();
        black_box(measure());
        started.elapsed().as_nanos()
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn test_mesh(node_id: EntityId, stable_instance_key: u64) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            stable_instance_key,
            transform_revision: 0,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(&format!(
                "builtin://selection-model/{node_id}"
            ))),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                &format!("builtin://selection-material/{node_id}"),
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            common: crate::core::framework::render::RendererCommon {
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                ..Default::default()
            },
        }
    }
}
