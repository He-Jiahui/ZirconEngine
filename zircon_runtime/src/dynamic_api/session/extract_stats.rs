use crate::core::framework::render::{PostProcessPassNode, RenderFrameExtract};
use crate::core::CoreRuntime;

use super::extract_cache::RuntimeFrameExtractCacheStatus;

pub(super) const EXTRACT_REBUILD_CLONES_DIAGNOSTIC: &str = "extract.rebuild_clones";
pub(super) const EXTRACT_OUTPUT_BYTES_DIAGNOSTIC: &str = "extract.output_bytes";
pub(super) const EXTRACT_FULL_CLONES_DIAGNOSTIC: &str = "extract.full_clones";
pub(super) const EXTRACT_FULL_CLONE_BYTES_DIAGNOSTIC: &str = "extract.full_clone_bytes";
pub(super) const EXTRACT_CACHE_HITS_DIAGNOSTIC: &str = "extract.cache_hits";
pub(super) const EXTRACT_CACHE_MISSES_DIAGNOSTIC: &str = "extract.cache_misses";
pub(super) const EXTRACT_STATS_PAYLOAD_SCANS_DIAGNOSTIC: &str = "extract.stats_payload_scans";

/// Immutable statistics computed once alongside a cached extract generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RuntimeFrameExtractDiagnosticsSummary {
    output_bytes: usize,
}

impl RuntimeFrameExtractDiagnosticsSummary {
    pub(super) fn from_extract(extract: &RenderFrameExtract) -> Self {
        Self {
            output_bytes: estimate_extract_output_bytes(extract),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RuntimeFrameExtractStats {
    pub rebuild_clones: u64,
    pub output_bytes: usize,
    pub full_clones: u64,
    pub full_clone_bytes: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub payload_stats_scans: u64,
}

impl RuntimeFrameExtractStats {
    pub(super) fn from_summary(
        summary: RuntimeFrameExtractDiagnosticsSummary,
        status: RuntimeFrameExtractCacheStatus,
    ) -> Self {
        Self {
            rebuild_clones: match status {
                RuntimeFrameExtractCacheStatus::Rebuilt => 1,
                RuntimeFrameExtractCacheStatus::Reused => 0,
            },
            output_bytes: summary.output_bytes,
            // Cache population and cache reuse each perform one deep RenderFrameExtract clone.
            full_clones: 1,
            full_clone_bytes: summary.output_bytes,
            cache_hits: match status {
                RuntimeFrameExtractCacheStatus::Rebuilt => 0,
                RuntimeFrameExtractCacheStatus::Reused => 1,
            },
            cache_misses: match status {
                RuntimeFrameExtractCacheStatus::Rebuilt => 1,
                RuntimeFrameExtractCacheStatus::Reused => 0,
            },
            payload_stats_scans: match status {
                RuntimeFrameExtractCacheStatus::Rebuilt => 1,
                RuntimeFrameExtractCacheStatus::Reused => 0,
            },
        }
    }

    pub fn record_diagnostics(&self, runtime: &CoreRuntime) {
        let frame_index = runtime.real_time().frame_index();
        runtime.record_diagnostic(
            EXTRACT_REBUILD_CLONES_DIAGNOSTIC,
            frame_index,
            self.rebuild_clones as f64,
            Some("count"),
            ["runtime", "extract"],
        );
        runtime.record_diagnostic(
            EXTRACT_OUTPUT_BYTES_DIAGNOSTIC,
            frame_index,
            self.output_bytes as f64,
            Some("byte"),
            ["runtime", "extract"],
        );
        runtime.record_diagnostic(
            EXTRACT_FULL_CLONES_DIAGNOSTIC,
            frame_index,
            self.full_clones as f64,
            Some("count"),
            ["runtime", "extract"],
        );
        runtime.record_diagnostic(
            EXTRACT_FULL_CLONE_BYTES_DIAGNOSTIC,
            frame_index,
            self.full_clone_bytes as f64,
            Some("byte"),
            ["runtime", "extract"],
        );
        runtime.record_diagnostic(
            EXTRACT_CACHE_HITS_DIAGNOSTIC,
            frame_index,
            self.cache_hits as f64,
            Some("count"),
            ["runtime", "extract"],
        );
        runtime.record_diagnostic(
            EXTRACT_CACHE_MISSES_DIAGNOSTIC,
            frame_index,
            self.cache_misses as f64,
            Some("count"),
            ["runtime", "extract"],
        );
        runtime.record_diagnostic(
            EXTRACT_STATS_PAYLOAD_SCANS_DIAGNOSTIC,
            frame_index,
            self.payload_stats_scans as f64,
            Some("count"),
            ["runtime", "extract"],
        );
    }
}

pub(super) fn record_frame_extract_stats(
    runtime: &CoreRuntime,
    summary: RuntimeFrameExtractDiagnosticsSummary,
    status: RuntimeFrameExtractCacheStatus,
) {
    RuntimeFrameExtractStats::from_summary(summary, status).record_diagnostics(runtime);
}

fn estimate_extract_output_bytes(extract: &RenderFrameExtract) -> usize {
    let geometry = &extract.geometry;
    let mut bytes = 0usize;
    bytes += slice_bytes(&geometry.meshes);
    bytes += geometry
        .meshes
        .iter()
        .map(|mesh| slice_bytes(&mesh.morph_weights))
        .sum::<usize>();
    bytes += slice_bytes(&geometry.phase_inputs);
    bytes += slice_bytes(&geometry.phase_queue.items);
    bytes += slice_bytes(&geometry.static_batches);
    bytes += geometry
        .static_batches
        .iter()
        .map(|batch| slice_bytes(&batch.mesh_indices) + slice_bytes(&batch.entities))
        .sum::<usize>();
    if let Some(virtual_geometry) = &geometry.virtual_geometry {
        bytes += slice_bytes(&virtual_geometry.clusters);
        bytes += slice_bytes(&virtual_geometry.hierarchy_nodes);
        bytes += slice_bytes(&virtual_geometry.hierarchy_child_ids);
        bytes += slice_bytes(&virtual_geometry.pages);
        bytes += slice_bytes(&virtual_geometry.page_dependencies);
        bytes += virtual_geometry
            .page_dependencies
            .iter()
            .map(|dependency| slice_bytes(&dependency.child_page_ids))
            .sum::<usize>();
        bytes += slice_bytes(&virtual_geometry.instances);
        bytes += virtual_geometry
            .instances
            .iter()
            .map(|instance| {
                option_string_bytes(&instance.mesh_name)
                    + option_string_bytes(&instance.source_hint)
            })
            .sum::<usize>();
    }

    bytes += slice_bytes(&extract.animation_poses);
    bytes += slice_bytes(&extract.lighting.directional_lights);
    bytes += slice_bytes(&extract.lighting.point_lights);
    bytes += slice_bytes(&extract.lighting.spot_lights);
    bytes += slice_bytes(&extract.lighting.ambient_lights);
    bytes += extract
        .lighting
        .ambient_lights
        .iter()
        .map(|light| option_string_bytes(&light.degradation_reason))
        .sum::<usize>();
    bytes += slice_bytes(&extract.lighting.rect_lights);
    bytes += extract
        .lighting
        .rect_lights
        .iter()
        .map(|light| option_string_bytes(&light.degradation_reason))
        .sum::<usize>();
    bytes += slice_bytes(&extract.environment.probes);
    if let Some(lightmaps) = extract.environment.baked_lighting() {
        bytes += std::mem::size_of_val(lightmaps);
        bytes += lightmaps.slot_capacity()
            * std::mem::size_of::<(u64, crate::core::framework::render::LightmapInstanceSlot)>();
    }
    if let Some(probe_grid) = extract.environment.light_probe_grid() {
        bytes += std::mem::size_of_val(probe_grid);
        bytes += probe_grid.sh.capacity()
            * std::mem::size_of::<crate::core::framework::render::ShL2Rgb>();
    }
    if let Some(hybrid_gi) = &extract.lighting.hybrid_global_illumination {
        bytes += std::mem::size_of_val(hybrid_gi);
    }

    bytes += slice_bytes(&extract.post_process.volumes);
    bytes += strings_bytes(&extract.post_process.stack.initial_resources);
    bytes += slice_bytes(&extract.post_process.stack.effects);
    bytes += post_process_nodes_bytes(&extract.post_process.graph.nodes);
    bytes += post_process_nodes_bytes(&extract.post_process.graph.skipped_nodes);
    bytes += option_string_bytes(&extract.post_process.graph.output_transfer_node);

    let overlays = &extract.debug.overlays;
    bytes += slice_bytes(&overlays.selection);
    bytes += slice_bytes(&overlays.selection_anchors);
    bytes += slice_bytes(&overlays.handles);
    bytes += overlays
        .handles
        .iter()
        .map(|handle| slice_bytes(&handle.elements))
        .sum::<usize>();
    bytes += slice_bytes(&overlays.scene_gizmos);
    bytes += overlays
        .scene_gizmos
        .iter()
        .map(|gizmo| {
            slice_bytes(&gizmo.lines)
                + slice_bytes(&gizmo.wire_shapes)
                + slice_bytes(&gizmo.icons)
                + slice_bytes(&gizmo.pick_shapes)
        })
        .sum::<usize>();

    bytes += slice_bytes(&extract.sprites.sprites);
    bytes += slice_bytes(&extract.sprites.phase_queue.items);
    bytes += slice_bytes(&extract.particles.emitters);
    bytes += slice_bytes(&extract.particles.sprites);
    bytes += slice_bytes(&extract.particles.bounds);
    bytes += extract
        .particles
        .gpu_frame
        .as_ref()
        .map_or(0, std::mem::size_of_val);
    bytes += slice_bytes(&extract.visibility.renderable_entities);
    bytes += slice_bytes(&extract.visibility.static_entities);
    bytes += slice_bytes(&extract.visibility.dynamic_entities);
    bytes += slice_bytes(&extract.visibility.renderables);
    bytes
}

fn post_process_nodes_bytes(nodes: &[PostProcessPassNode]) -> usize {
    slice_bytes(nodes)
        + nodes
            .iter()
            .map(|node| {
                node.name.len()
                    + strings_bytes(&node.required_inputs)
                    + strings_bytes(&node.produced_outputs)
                    + slice_bytes(&node.after)
            })
            .sum::<usize>()
}

fn strings_bytes(values: &[String]) -> usize {
    slice_bytes(values) + values.iter().map(String::len).sum::<usize>()
}

fn option_string_bytes(value: &Option<String>) -> usize {
    value.as_ref().map_or(0, String::len)
}

fn slice_bytes<T>(values: &[T]) -> usize {
    std::mem::size_of_val(values)
}
