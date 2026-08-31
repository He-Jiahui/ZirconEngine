use std::collections::HashSet;

use crate::core::framework::render::DisplayMode;
use crate::core::math::Vec4;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

pub(crate) fn build_wireframe_vertices(
    frame: &ViewportRenderFrame,
    streamer: &ResourceStreamer,
) -> Vec<LineVertex> {
    let display_mode = frame.overlays().display_mode;
    if display_mode == DisplayMode::Shaded {
        return Vec::new();
    }
    let selection = if display_mode == DisplayMode::WireOnly {
        if let Some(highlights) = frame.overlays().highlights.as_ref() {
            let entities = highlights.entities();
            let mut selection = HashSet::with_capacity(entities.len());
            selection.extend(entities.iter().copied());
            selection
        } else {
            HashSet::new()
        }
    } else {
        HashSet::new()
    };

    let mut vertices = Vec::new();
    for mesh_instance in frame.meshes() {
        let Some(model) = streamer.model(&mesh_instance.model.id()) else {
            continue;
        };
        let color = match display_mode {
            DisplayMode::WireOverlay => Vec4::new(0.08, 0.09, 0.1, 0.9),
            DisplayMode::WireOnly => {
                if selection.contains(&mesh_instance.node_id) {
                    Vec4::new(1.0, 0.9, 0.45, 1.0)
                } else {
                    Vec4::new(0.86, 0.88, 0.93, 1.0)
                }
            }
            DisplayMode::Shaded => Vec4::ONE,
        };
        let model_matrix = mesh_instance.transform.matrix();
        for mesh in &model.meshes {
            vertices.reserve(mesh.wire_segments.len().saturating_mul(2));
            for [start, end] in &mesh.wire_segments {
                vertices.push(LineVertex::new(
                    model_matrix.transform_point3(*start),
                    color,
                ));
                vertices.push(LineVertex::new(model_matrix.transform_point3(*end), color));
            }
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const ENTITIES_PER_SAMPLE: usize = 4_096;
    const MESHES_PER_SAMPLE: usize = 32;
    const SEGMENTS_PER_MESH: usize = 256;

    #[test]
    fn shaded_wireframe_skips_vertex_and_selection_build_before_iteration() {
        let source = include_str!("build_wireframe_vertices.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("wireframe primitive implementation");
        let shaded_guard = implementation
            .find("if display_mode == DisplayMode::Shaded")
            .expect("Shaded early return");
        let selection_build = implementation
            .find("let selection: HashSet")
            .expect("WireOnly selection index");
        let mesh_loop = implementation
            .find("for mesh_instance in frame.meshes()")
            .expect("wireframe mesh loop");

        assert!(shaded_guard < selection_build);
        assert!(shaded_guard < mesh_loop);
        assert!(implementation.contains("display_mode == DisplayMode::WireOnly"));
    }

    #[test]
    fn wireframe_build_reserves_selection_and_vertex_capacity() {
        let source = include_str!("build_wireframe_vertices.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("wireframe primitive implementation");

        assert!(implementation.contains("HashSet::with_capacity(entities.len())"));
        assert!(
            implementation.contains("vertices.reserve(mesh.wire_segments.len().saturating_mul(2))")
        );
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cd_runtime_wireframe_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME382_WIREFRAME_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entities_per_sample={ENTITIES_PER_SAMPLE} meshes_per_sample={MESHES_PER_SAMPLE} segments_per_mesh={SEGMENTS_PER_MESH} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut selection = if use_capacity {
            HashSet::with_capacity(ENTITIES_PER_SAMPLE)
        } else {
            HashSet::new()
        };
        selection.extend(0..ENTITIES_PER_SAMPLE);
        let mut vertices = Vec::new();
        for mesh in 0..MESHES_PER_SAMPLE {
            if use_capacity {
                vertices.reserve(SEGMENTS_PER_MESH.saturating_mul(2));
            }
            for segment in 0..SEGMENTS_PER_MESH {
                vertices.push((mesh, segment, 0usize));
                vertices.push((mesh, segment, 1usize));
            }
        }
        std::hint::black_box((selection, vertices));
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
