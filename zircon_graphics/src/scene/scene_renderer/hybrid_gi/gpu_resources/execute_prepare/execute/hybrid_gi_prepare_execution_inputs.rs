use super::super::super::gpu_pending_probe_input::GpuPendingProbeInput;
use super::super::super::gpu_resident_probe_input::GpuResidentProbeInput;
use super::super::super::gpu_trace_region_input::GpuTraceRegionInput;

pub(super) struct HybridGiPrepareExecutionInputs {
    pub(super) cache_entries: Vec<[u32; 2]>,
    pub(super) resident_probe_inputs: Vec<GpuResidentProbeInput>,
    pub(super) pending_probe_inputs: Vec<GpuPendingProbeInput>,
    pub(super) trace_region_inputs: Vec<GpuTraceRegionInput>,
    pub(super) cache_word_count: usize,
    pub(super) completed_probe_word_count: usize,
    pub(super) completed_trace_word_count: usize,
    pub(super) irradiance_word_count: usize,
}
