use std::collections::BTreeSet;

use crate::core::math::Vec3;
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareSurfaceCachePageContent,
    HybridGiScenePrepareFrame,
};

const CAPTURE_RESOURCE_CONFIDENCE_QUALITY: f32 = 1.0;
const ATLAS_RESOURCE_CONFIDENCE_QUALITY: f32 = 0.9;
const CAPTURE_SAMPLE_CONFIDENCE_QUALITY: f32 = 0.85;
const ATLAS_SAMPLE_CONFIDENCE_QUALITY: f32 = 0.75;
const SYNTHETIC_REQUEST_CONFIDENCE_QUALITY: f32 = 0.55;

pub(super) fn scene_prepare_surface_cache_fallback_rgb_and_support(
    scene_prepare: &HybridGiScenePrepareFrame,
    probe_position: Vec3,
    probe_radius: f32,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<([f32; 3], f32)> {
    scene_prepare_surface_cache_fallback_rgb_support_and_quality(
        scene_prepare,
        probe_position,
        probe_radius,
        scene_prepare_resources,
    )
    .map(|(rgb, support, _)| (rgb, support))
}

pub(super) fn scene_prepare_surface_cache_fallback_rgb_support_and_quality(
    scene_prepare: &HybridGiScenePrepareFrame,
    probe_position: Vec3,
    probe_radius: f32,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<([f32; 3], f32, f32)> {
    let request_page_ids = scene_prepare
        .card_capture_requests
        .iter()
        .map(|request| request.page_id)
        .collect::<BTreeSet<_>>();
    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    let mut weighted_confidence_quality = 0.0_f32;

    for request in &scene_prepare.card_capture_requests {
        let support = scene_prepare_surface_cache_entry_support(
            probe_position,
            probe_radius,
            request.bounds_center,
            request.bounds_radius,
        );
        if support <= f32::EPSILON {
            continue;
        }

        let (base_rgb, confidence_quality) =
            scene_prepare_card_capture_request_rgb_and_quality(request, scene_prepare_resources);
        weighted_rgb[0] += base_rgb[0] * support;
        weighted_rgb[1] += base_rgb[1] * support;
        weighted_rgb[2] += base_rgb[2] * support;
        total_support += support;
        weighted_confidence_quality += confidence_quality * support;
    }

    for page_content in &scene_prepare.surface_cache_page_contents {
        if request_page_ids.contains(&page_content.page_id) {
            continue;
        }
        let Some((base_rgb, confidence_quality)) =
            scene_prepare_surface_cache_page_rgb_and_quality(page_content, scene_prepare_resources)
        else {
            continue;
        };
        let support = scene_prepare_surface_cache_entry_support(
            probe_position,
            probe_radius,
            page_content.bounds_center,
            page_content.bounds_radius,
        );
        if support <= f32::EPSILON {
            continue;
        }

        weighted_rgb[0] += base_rgb[0] * support;
        weighted_rgb[1] += base_rgb[1] * support;
        weighted_rgb[2] += base_rgb[2] * support;
        total_support += support;
        weighted_confidence_quality += confidence_quality * support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some((
        [
            weighted_rgb[0] / total_support,
            weighted_rgb[1] / total_support,
            weighted_rgb[2] / total_support,
        ],
        total_support,
        (weighted_confidence_quality / total_support).clamp(0.0, 1.0),
    ))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn scene_prepare_surface_cache_owner_rgb(
    scene_prepare: &HybridGiScenePrepareFrame,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    owner_id: u32,
) -> Option<[f32; 3]> {
    scene_prepare_surface_cache_owner_rgb_and_quality(
        scene_prepare,
        scene_prepare_resources,
        owner_id,
    )
    .map(|(rgb, _)| rgb)
}

pub(super) fn scene_prepare_surface_cache_owner_rgb_and_quality(
    scene_prepare: &HybridGiScenePrepareFrame,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    owner_id: u32,
) -> Option<([f32; 3], f32)> {
    if owner_id == 0 {
        return None;
    }

    scene_prepare
        .card_capture_requests
        .iter()
        .find(|request| request.card_id == owner_id)
        .map(|request| {
            scene_prepare_card_capture_request_rgb_and_quality(request, scene_prepare_resources)
        })
        .or_else(|| {
            scene_prepare
                .surface_cache_page_contents
                .iter()
                .find(|page_content| page_content.owner_card_id == owner_id)
                .and_then(|page_content| {
                    scene_prepare_surface_cache_page_rgb_and_quality(
                        page_content,
                        scene_prepare_resources,
                    )
                })
        })
}

fn scene_prepare_card_capture_request_rgb_and_quality(
    request: &HybridGiPrepareCardCaptureRequest,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> ([f32; 3], f32) {
    if let Some(scene_prepare_resources) = scene_prepare_resources {
        if let Some(rgba) = scene_prepare_resources
            .capture_slot_rgba_sample(request.capture_slot_id)
            .filter(|rgba| rgba_sample_is_present(*rgba))
        {
            return (rgba_sample_rgb(rgba), CAPTURE_RESOURCE_CONFIDENCE_QUALITY);
        }

        if let Some(rgba) = scene_prepare_resources
            .atlas_slot_rgba_sample(request.atlas_slot_id)
            .filter(|rgba| rgba_sample_is_present(*rgba))
        {
            return (rgba_sample_rgb(rgba), ATLAS_RESOURCE_CONFIDENCE_QUALITY);
        }
    }

    let bounds_center_x_q = quantized_signed(request.bounds_center.x);
    let bounds_center_z_q = quantized_signed(request.bounds_center.z);
    let bounds_radius_q = quantized_positive(request.bounds_radius, 64.0);

    (
        [
            (96 + ((request.card_id * 17 + request.page_id * 5 + request.capture_slot_id * 3) % 96))
                as f32
                / 255.0,
            (72 + ((request.page_id * 13 + request.atlas_slot_id * 7 + bounds_radius_q) % 80))
                as f32
                / 255.0,
            (40 + ((request.card_id * 11 + bounds_center_x_q + bounds_center_z_q) % 56)) as f32
                / 255.0,
        ],
        SYNTHETIC_REQUEST_CONFIDENCE_QUALITY,
    )
}

fn scene_prepare_surface_cache_page_rgb_and_quality(
    page_content: &HybridGiPrepareSurfaceCachePageContent,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<([f32; 3], f32)> {
    if let Some(scene_prepare_resources) = scene_prepare_resources {
        if let Some(rgba) = scene_prepare_resources
            .capture_slot_rgba_sample(page_content.capture_slot_id)
            .filter(|rgba| rgba_sample_is_present(*rgba))
        {
            return Some((rgba_sample_rgb(rgba), CAPTURE_RESOURCE_CONFIDENCE_QUALITY));
        }

        if let Some(rgba) = scene_prepare_resources
            .atlas_slot_rgba_sample(page_content.atlas_slot_id)
            .filter(|rgba| rgba_sample_is_present(*rgba))
        {
            return Some((rgba_sample_rgb(rgba), ATLAS_RESOURCE_CONFIDENCE_QUALITY));
        }
    }

    if rgba_sample_is_present(page_content.capture_sample_rgba) {
        return Some((
            rgba_sample_rgb(page_content.capture_sample_rgba),
            CAPTURE_SAMPLE_CONFIDENCE_QUALITY,
        ));
    }

    if rgba_sample_is_present(page_content.atlas_sample_rgba) {
        return Some((
            rgba_sample_rgb(page_content.atlas_sample_rgba),
            ATLAS_SAMPLE_CONFIDENCE_QUALITY,
        ));
    }

    None
}

pub(super) fn rgba_sample_is_present(rgba: [u8; 4]) -> bool {
    rgba[3] > 0
}

pub(super) fn rgba_sample_rgb(rgba: [u8; 4]) -> [f32; 3] {
    [
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
    ]
}

fn scene_prepare_surface_cache_entry_support(
    probe_position: Vec3,
    probe_radius: f32,
    bounds_center: Vec3,
    bounds_radius: f32,
) -> f32 {
    let reach = (probe_radius.max(0.05) + bounds_radius.max(0.05) * 2.25).max(0.05);
    let falloff = (1.0 - probe_position.distance(bounds_center) / reach).max(0.0);
    if falloff <= f32::EPSILON {
        return 0.0;
    }

    let bounds_support = (bounds_radius / reach).clamp(0.0, 1.0);
    falloff * (0.28 + bounds_support * 0.72)
}

fn quantized_signed(value: f32) -> u32 {
    ((value * 64.0).round() as i32).wrapping_add(2048) as u32
}

fn quantized_positive(value: f32, scale: f32) -> u32 {
    (value.max(0.0) * scale).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_prepare_surface_cache_owner_rgb_matches_persisted_owner_card_id_not_page_id() {
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 21,
                owner_card_id: 11,
                atlas_slot_id: 0,
                capture_slot_id: 0,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                atlas_sample_rgba: [10, 20, 30, 255],
                capture_sample_rgba: [40, 50, 60, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        };

        assert_eq!(
            scene_prepare_surface_cache_owner_rgb(&scene_prepare, None, 11),
            Some([40.0 / 255.0, 50.0 / 255.0, 60.0 / 255.0])
        );
    }
}
