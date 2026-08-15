use zircon_runtime::graphics::{GraphicsError, RuntimeGpuReadback, RuntimePrepareCollectorContext};

use crate::hybrid_gi::renderer::HybridGiScenePrepareResourcesSnapshot;

use super::HybridGiGpuPendingReadback;

pub(super) struct WordReadback {
    pub(super) word_count: usize,
    readback: RuntimeGpuReadback,
}

impl WordReadback {
    pub(in crate::hybrid_gi::renderer) fn is_ready(&self) -> bool {
        self.readback.is_ready()
    }

    pub(super) fn take(self) -> Result<Vec<u8>, GraphicsError> {
        self.readback
            .try_take()
            .expect("ready hybrid GI readback remains available")
    }
}

pub(in crate::hybrid_gi::renderer) struct HybridGiGpuReadbackFuture {
    pub(super) cache: WordReadback,
    pub(super) completed_probes: WordReadback,
    pub(super) completed_traces: WordReadback,
    pub(super) irradiance: WordReadback,
    pub(super) trace_lighting: WordReadback,
    pub(super) trace_diagnostics: WordReadback,
    pub(super) radiance_cache_dispatch_counts: WordReadback,
    pub(super) scene_prepare_resources: Option<HybridGiScenePrepareResourcesSnapshot>,
    pub(super) atlas_slot_samples: Vec<(u32, RuntimeGpuReadback)>,
    pub(super) capture_slot_samples: Vec<(u32, RuntimeGpuReadback)>,
    pub(super) surface_cache_depth_slot_samples: Vec<(u32, RuntimeGpuReadback)>,
    pub(super) probe_trace_tiles: Option<WordReadback>,
    pub(super) probe_trace_tile_record_count: usize,
    pub(super) probe_trace_indirect_args: Option<WordReadback>,
}

impl HybridGiGpuReadbackFuture {
    pub(in crate::hybrid_gi::renderer) fn is_ready(&self) -> bool {
        self.cache.is_ready()
            && self.completed_probes.is_ready()
            && self.completed_traces.is_ready()
            && self.irradiance.is_ready()
            && self.trace_lighting.is_ready()
            && self.trace_diagnostics.is_ready()
            && self.radiance_cache_dispatch_counts.is_ready()
            && self
                .atlas_slot_samples
                .iter()
                .all(|(_, readback)| readback.is_ready())
            && self
                .capture_slot_samples
                .iter()
                .all(|(_, readback)| readback.is_ready())
            && self
                .surface_cache_depth_slot_samples
                .iter()
                .all(|(_, readback)| readback.is_ready())
            && self
                .probe_trace_tiles
                .as_ref()
                .map_or(true, WordReadback::is_ready)
            && self
                .probe_trace_indirect_args
                .as_ref()
                .map_or(true, WordReadback::is_ready)
    }
}

impl HybridGiGpuPendingReadback {
    pub(in crate::hybrid_gi::renderer) fn enqueue(
        self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<HybridGiGpuReadbackFuture, GraphicsError> {
        let cache = request_words(
            context,
            "hybrid-gi.cache",
            &self.cache_buffer,
            self.cache_word_count,
        )?;
        let completed_probes = request_words(
            context,
            "hybrid-gi.completed-probes",
            &self.completed_probe_buffer,
            self.completed_probe_word_count,
        )?;
        let completed_traces = request_words(
            context,
            "hybrid-gi.completed-traces",
            &self.completed_trace_buffer,
            self.completed_trace_word_count,
        )?;
        let irradiance = request_words(
            context,
            "hybrid-gi.irradiance",
            &self.irradiance_buffer,
            self.irradiance_word_count,
        )?;
        let trace_lighting = request_words(
            context,
            "hybrid-gi.trace-lighting",
            &self.trace_lighting_buffer,
            self.trace_lighting_word_count,
        )?;
        let trace_diagnostics = request_words(
            context,
            "hybrid-gi.trace-diagnostics",
            &self.trace_diagnostic_buffer,
            self.trace_diagnostic_word_count,
        )?;
        let radiance_cache_dispatch_counts = request_word_range(
            context,
            "hybrid-gi.radiance-cache-dispatch-counts",
            &self.radiance_cache_dispatch_counter_buffer,
            self.radiance_cache_dispatch_counter_word_offset,
            self.radiance_cache_dispatch_counter_word_count,
        )?;
        let atlas_slot_samples = request_slot_samples(
            context,
            "hybrid-gi.atlas-slot",
            &self.scene_prepare_atlas_slot_sample_buffers,
        )?;
        let capture_slot_samples = request_slot_samples(
            context,
            "hybrid-gi.capture-slot",
            &self.scene_prepare_capture_slot_sample_buffers,
        )?;
        let surface_cache_depth_slot_samples = request_slot_samples(
            context,
            "hybrid-gi.surface-cache-depth-slot",
            &self.scene_prepare_surface_cache_depth_slot_sample_buffers,
        )?;
        let probe_trace_tiles = self
            .scene_prepare_probe_trace_tile_buffer
            .as_ref()
            .map(|buffer| {
                request_words(
                    context,
                    "hybrid-gi.probe-trace-tiles",
                    buffer,
                    self.scene_prepare_probe_trace_tile_word_count,
                )
            })
            .transpose()?;
        let probe_trace_indirect_args = self
            .scene_prepare_probe_trace_indirect_args_buffer
            .as_ref()
            .map(|buffer| {
                request_words(
                    context,
                    "hybrid-gi.probe-trace-indirect-args",
                    buffer,
                    self.scene_prepare_probe_trace_indirect_arg_word_count,
                )
            })
            .transpose()?;

        Ok(HybridGiGpuReadbackFuture {
            cache,
            completed_probes,
            completed_traces,
            irradiance,
            trace_lighting,
            trace_diagnostics,
            radiance_cache_dispatch_counts,
            scene_prepare_resources: self.scene_prepare_resources,
            atlas_slot_samples,
            capture_slot_samples,
            surface_cache_depth_slot_samples,
            probe_trace_tiles,
            probe_trace_tile_record_count: self.scene_prepare_probe_trace_tile_record_count,
            probe_trace_indirect_args,
        })
    }
}

fn request_words(
    context: &mut RuntimePrepareCollectorContext<'_>,
    name: impl Into<String>,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<WordReadback, GraphicsError> {
    request_word_range(context, name, buffer, 0, word_count)
}

fn request_word_range(
    context: &mut RuntimePrepareCollectorContext<'_>,
    name: impl Into<String>,
    buffer: &wgpu::Buffer,
    word_offset: usize,
    word_count: usize,
) -> Result<WordReadback, GraphicsError> {
    let byte_offset = word_offset as u64 * std::mem::size_of::<u32>() as u64;
    let byte_len = word_count.max(1) as u64 * std::mem::size_of::<u32>() as u64;
    Ok(WordReadback {
        word_count,
        readback: context.request_gpu_readback(
            name,
            buffer,
            byte_offset..byte_offset + byte_len,
        )?,
    })
}

fn request_slot_samples(
    context: &mut RuntimePrepareCollectorContext<'_>,
    name: &str,
    buffers: &[(u32, wgpu::Buffer)],
) -> Result<Vec<(u32, RuntimeGpuReadback)>, GraphicsError> {
    let mut readbacks = Vec::with_capacity(buffers.len());
    for (slot_id, buffer) in buffers {
        readbacks.push((
            *slot_id,
            context.request_gpu_readback(format!("{name}.{slot_id}"), buffer, 0..4)?,
        ));
    }
    Ok(readbacks)
}
