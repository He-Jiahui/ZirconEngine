use std::cmp::Ordering;
use std::collections::BTreeSet;

use zircon_math::view_matrix;
use zircon_scene::{
    ProjectionMode, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    ViewportCameraSnapshot,
};

use super::super::culling::{
    orthographic_visible::orthographic_visible, perspective_visible::perspective_visible,
};
use super::super::declarations::{
    VisibilityHistorySnapshot, VisibilityHybridGiFeedback, VisibilityHybridGiProbe,
    VisibilityHybridGiUpdatePlan,
};

pub(crate) fn build_hybrid_gi_plan(
    extract: Option<&RenderHybridGiExtract>,
    visible_entities: &BTreeSet<u64>,
    camera: &ViewportCameraSnapshot,
    previous: Option<&VisibilityHistorySnapshot>,
) -> (
    Vec<VisibilityHybridGiProbe>,
    VisibilityHybridGiUpdatePlan,
    VisibilityHybridGiFeedback,
    Vec<u32>,
) {
    let Some(extract) = extract else {
        return (
            Vec::new(),
            VisibilityHybridGiUpdatePlan::default(),
            VisibilityHybridGiFeedback::default(),
            Vec::new(),
        );
    };

    let resident_probe_ids = extract
        .probes
        .iter()
        .filter(|probe| probe.resident)
        .map(|probe| probe.probe_id)
        .collect::<Vec<_>>();

    let mut active_probes = extract
        .probes
        .iter()
        .filter(|probe| visible_entities.contains(&probe.entity))
        .filter(|probe| hybrid_gi_probe_visible(probe, camera))
        .copied()
        .collect::<Vec<_>>();
    active_probes.sort_by(hybrid_gi_probe_sort_key);

    let hybrid_gi_active_probes = active_probes
        .iter()
        .map(|probe| VisibilityHybridGiProbe {
            entity: probe.entity,
            probe_id: probe.probe_id,
            resident: probe.resident,
            ray_budget: probe.ray_budget,
        })
        .collect::<Vec<_>>();

    let requested_probe_ids = unique_probe_ids(
        hybrid_gi_active_probes
            .iter()
            .filter(|probe| !probe.resident)
            .map(|probe| probe.probe_id),
        extract.probe_budget as usize,
    );
    let previous_requested_probe_ids = previous
        .map(|history| {
            history
                .hybrid_gi_requested_probes
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let dirty_requested_probe_ids = requested_probe_ids
        .iter()
        .copied()
        .filter(|probe_id| !previous_requested_probe_ids.contains(probe_id))
        .collect::<Vec<_>>();

    let mut scheduled_trace_regions = extract
        .trace_regions
        .iter()
        .filter(|region| visible_entities.contains(&region.entity))
        .filter(|region| hybrid_gi_trace_region_visible(region, camera))
        .copied()
        .collect::<Vec<_>>();
    scheduled_trace_regions.sort_by(hybrid_gi_trace_region_sort_key);
    scheduled_trace_regions.truncate(extract.tracing_budget as usize);
    let scheduled_trace_region_ids = scheduled_trace_regions
        .iter()
        .map(|region| region.region_id)
        .collect::<Vec<_>>();

    let active_probe_set = hybrid_gi_active_probes
        .iter()
        .map(|probe| probe.probe_id)
        .collect::<BTreeSet<_>>();
    let evictable_probe_ids = resident_probe_ids
        .iter()
        .copied()
        .filter(|probe_id| !active_probe_set.contains(probe_id))
        .collect::<Vec<_>>();

    let update_plan = VisibilityHybridGiUpdatePlan {
        resident_probe_ids,
        requested_probe_ids: requested_probe_ids.clone(),
        dirty_requested_probe_ids: dirty_requested_probe_ids.clone(),
        scheduled_trace_region_ids: scheduled_trace_region_ids.clone(),
        evictable_probe_ids: evictable_probe_ids.clone(),
    };
    let feedback = VisibilityHybridGiFeedback {
        active_probe_ids: hybrid_gi_active_probes
            .iter()
            .map(|probe| probe.probe_id)
            .collect(),
        requested_probe_ids: requested_probe_ids.clone(),
        scheduled_trace_region_ids: scheduled_trace_region_ids.clone(),
        evictable_probe_ids: evictable_probe_ids.clone(),
    };

    (
        hybrid_gi_active_probes,
        update_plan,
        feedback,
        requested_probe_ids,
    )
}

fn hybrid_gi_probe_visible(probe: &RenderHybridGiProbe, camera: &ViewportCameraSnapshot) -> bool {
    sphere_visible(probe.position, probe.radius, camera)
}

fn hybrid_gi_trace_region_visible(
    region: &RenderHybridGiTraceRegion,
    camera: &ViewportCameraSnapshot,
) -> bool {
    sphere_visible(region.bounds_center, region.bounds_radius, camera)
}

fn sphere_visible(center: zircon_math::Vec3, radius: f32, camera: &ViewportCameraSnapshot) -> bool {
    let view_position = view_matrix(camera.transform).transform_point3(center);
    let depth = -view_position.z;
    let near = camera.z_near.max(0.001);
    let far = camera.z_far.max(near);
    let radius = radius.max(0.0);

    if depth + radius < near || depth - radius > far {
        return false;
    }

    match camera.projection_mode {
        ProjectionMode::Perspective => perspective_visible(view_position, depth, radius, camera),
        ProjectionMode::Orthographic => orthographic_visible(view_position, radius, camera),
    }
}

fn hybrid_gi_probe_sort_key(left: &RenderHybridGiProbe, right: &RenderHybridGiProbe) -> Ordering {
    right
        .ray_budget
        .cmp(&left.ray_budget)
        .then_with(|| left.probe_id.cmp(&right.probe_id))
}

fn hybrid_gi_trace_region_sort_key(
    left: &RenderHybridGiTraceRegion,
    right: &RenderHybridGiTraceRegion,
) -> Ordering {
    right
        .screen_coverage
        .partial_cmp(&left.screen_coverage)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.region_id.cmp(&right.region_id))
}

fn unique_probe_ids(probes: impl IntoIterator<Item = u32>, budget: usize) -> Vec<u32> {
    if budget == 0 {
        return Vec::new();
    }

    let mut seen = BTreeSet::new();
    let mut unique = Vec::new();
    for probe_id in probes {
        if seen.insert(probe_id) {
            unique.push(probe_id);
            if unique.len() == budget {
                break;
            }
        }
    }
    unique
}
