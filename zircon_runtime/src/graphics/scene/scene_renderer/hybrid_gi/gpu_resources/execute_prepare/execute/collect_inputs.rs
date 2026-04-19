use crate::core::framework::render::RenderHybridGiExtract;

use crate::graphics::types::{HybridGiPrepareFrame, HybridGiResolveRuntime};

use super::super::pending_probe_inputs::pending_probe_inputs;
use super::super::resident_probe_inputs::resident_probe_inputs;
use super::super::trace_region_inputs::trace_region_inputs;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

pub(super) fn collect_inputs(
    prepare: &HybridGiPrepareFrame,
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
) -> HybridGiPrepareExecutionInputs {
    let cache_entries = prepare
        .resident_probes
        .iter()
        .map(|probe| [probe.probe_id, probe.slot])
        .collect::<Vec<_>>();
    let resident_probe_inputs = resident_probe_inputs(prepare, resolve_runtime, extract);
    let pending_probe_inputs = pending_probe_inputs(prepare, resolve_runtime, extract);
    let trace_region_inputs = trace_region_inputs(prepare, extract);

    HybridGiPrepareExecutionInputs {
        cache_word_count: cache_entries.len() * 2,
        completed_probe_word_count: pending_probe_inputs.len() + 1,
        completed_trace_word_count: trace_region_inputs.len() + 1,
        irradiance_word_count: 1
            + (resident_probe_inputs.len() + pending_probe_inputs.len()).max(1) * 2,
        trace_lighting_word_count: 1
            + (resident_probe_inputs.len() + pending_probe_inputs.len()).max(1) * 2,
        cache_entries,
        resident_probe_inputs,
        pending_probe_inputs,
        trace_region_inputs,
    }
}
