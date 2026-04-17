use std::collections::BTreeSet;

use zircon_scene::{RenderHybridGiExtract, RenderHybridGiProbe};

use super::super::seed_quantization::{quantized_positive, quantized_signed};

pub(super) const NO_PARENT_PROBE_ID: u32 = u32::MAX;

pub(super) fn pack_rgb8(rgb: [u8; 3]) -> u32 {
    u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16)
}

pub(super) fn probe_position_x_q(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .map(|probe| quantized_signed(probe.position.x))
        .unwrap_or_default()
}

pub(super) fn probe_position_y_q(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .map(|probe| quantized_signed(probe.position.y))
        .unwrap_or_default()
}

pub(super) fn probe_position_z_q(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .map(|probe| quantized_signed(probe.position.z))
        .unwrap_or_default()
}

pub(super) fn probe_radius_q(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .map(|probe| quantized_positive(probe.radius, 96.0))
        .unwrap_or_default()
}

pub(super) fn probe_parent_probe_id(extract: Option<&RenderHybridGiExtract>, probe_id: u32) -> u32 {
    probe_from_extract(extract, probe_id)
        .and_then(|probe| probe.parent_probe_id)
        .unwrap_or(NO_PARENT_PROBE_ID)
}

pub(super) fn probe_resident_ancestor(
    extract: Option<&RenderHybridGiExtract>,
    resident_probe_ids: &BTreeSet<u32>,
    probe_id: u32,
) -> (u32, u32) {
    if resident_probe_ids.is_empty() {
        return (NO_PARENT_PROBE_ID, 0);
    }

    let mut current_probe_id = probe_id;
    let mut resident_ancestor_depth = 0_u32;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);

    loop {
        let Some(parent_probe_id) =
            probe_from_extract(extract, current_probe_id).and_then(|probe| probe.parent_probe_id)
        else {
            return (NO_PARENT_PROBE_ID, 0);
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            return (NO_PARENT_PROBE_ID, 0);
        }

        resident_ancestor_depth = resident_ancestor_depth.saturating_add(1);
        if resident_probe_ids.contains(&parent_probe_id) {
            return (parent_probe_id, resident_ancestor_depth);
        }
        current_probe_id = parent_probe_id;
    }
}

fn probe_from_extract(
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
) -> Option<&RenderHybridGiProbe> {
    extract.and_then(|extract| {
        extract
            .probes
            .iter()
            .find(|probe| probe.probe_id == probe_id)
    })
}
