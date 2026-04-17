use zircon_scene::RenderHybridGiExtract;

use crate::types::HybridGiPrepareFrame;

use super::super::gpu_resident_probe_input::GpuResidentProbeInput;
use super::probe_quantization::{
    pack_rgb8, probe_parent_probe_id, probe_position_x_q, probe_position_y_q, probe_position_z_q,
    probe_radius_q,
};

pub(super) fn resident_probe_inputs(
    prepare: &HybridGiPrepareFrame,
    extract: Option<&RenderHybridGiExtract>,
) -> Vec<GpuResidentProbeInput> {
    prepare
        .resident_probes
        .iter()
        .map(|probe| GpuResidentProbeInput {
            probe_id: probe.probe_id,
            slot: probe.slot,
            ray_budget: probe.ray_budget,
            position_x_q: probe_position_x_q(extract, probe.probe_id),
            position_y_q: probe_position_y_q(extract, probe.probe_id),
            position_z_q: probe_position_z_q(extract, probe.probe_id),
            radius_q: probe_radius_q(extract, probe.probe_id),
            previous_irradiance_rgb: pack_rgb8(probe.irradiance_rgb),
            parent_probe_id: probe_parent_probe_id(extract, probe.probe_id),
            _padding: 0,
        })
        .collect()
}
