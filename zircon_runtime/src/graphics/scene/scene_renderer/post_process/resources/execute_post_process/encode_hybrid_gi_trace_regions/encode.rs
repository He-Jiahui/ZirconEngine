use std::collections::{HashMap, HashSet};

use crate::core::framework::render::RenderHybridGiPreparedTraceRegionSceneData;
use crate::core::math::{Mat4, UVec2, Vec3};
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::super::super::super::hybrid_gi_trace_region_gpu::GpuHybridGiTraceRegion;
use super::super::camera_matrices::view_projection;

// 量化尺度由 prepared trace scene sideband 约定；trace 半径和 coverage 使用各自的编码域。
const HYBRID_GI_POSITION_BIAS: i32 = 2048;
const HYBRID_GI_POSITION_SCALE: f32 = 64.0;
const HYBRID_GI_TRACE_RADIUS_SCALE: f32 = 96.0;
const HYBRID_GI_TRACE_COVERAGE_SCALE: f32 = 128.0;

/// 编码本帧调度的 GI trace region 支撑，只消费 runtime 已准备的空间与 RT 光照 sideband。
/// 固定容量限制上传范围；去重、缺失描述或投影失败后，返回计数仍只覆盖有效前缀。
pub(in super::super) fn encode_hybrid_gi_trace_regions(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiTraceRegion; MAX_HYBRID_GI_TRACE_REGIONS], u32) {
    let mut trace_regions = [GpuHybridGiTraceRegion::zeroed(); MAX_HYBRID_GI_TRACE_REGIONS];
    if !enabled {
        return (trace_regions, 0);
    }

    let Some(_hybrid_gi) = frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .filter(|extract| extract.enabled)
    else {
        return (trace_regions, 0);
    };

    let Some(prepared_frame) = frame
        .prepared_runtime_sidebands()
        .hybrid_gi_prepared_frame()
    else {
        return (trace_regions, 0);
    };
    let scene_data_by_id = trace_region_scene_data_by_id(&prepared_frame.trace_region_scene_data);
    let mut encoded_region_ids = HashSet::with_capacity(
        prepared_frame
            .scheduled_trace_region_ids
            .len()
            .min(MAX_HYBRID_GI_TRACE_REGIONS),
    );
    let camera = &frame.extract.view.camera;
    let view_proj = view_projection(camera, viewport_size);
    let camera_position = camera.transform.translation;
    let mut count = 0;

    // TODO: [CR-SCENE-POST-0008] 明确上限约束的是调度候选数量还是成功编码数量；当前先截断，重复、缺失描述和投影失败都会消耗候选预算。
    for region_id in prepared_frame
        .scheduled_trace_region_ids
        .iter()
        .take(MAX_HYBRID_GI_TRACE_REGIONS)
    {
        if !encoded_region_ids.insert(*region_id) {
            continue;
        }
        let Some(region) = scene_data_by_id.get(region_id).copied() else {
            continue;
        };
        let Some(gpu_region) =
            project_prepared_hybrid_gi_trace_region(region, view_proj, camera_position)
        else {
            continue;
        };
        trace_regions[count] = gpu_region;
        count += 1;
    }

    (trace_regions, count as u32)
}

// prepared scene 描述按 ID 建索引；重复 ID 的末条记录获胜，hash-index 回归测试固定此约定。
fn trace_region_scene_data_by_id(
    regions: &[RenderHybridGiPreparedTraceRegionSceneData],
) -> HashMap<u32, &RenderHybridGiPreparedTraceRegionSceneData> {
    regions
        .iter()
        .map(|region| (region.region_id, region))
        .collect()
}

// 将调度区域转换为 shader 的屏幕覆盖和 RT 支撑；其 ID 用作来源信息而非数组索引。
fn project_prepared_hybrid_gi_trace_region(
    region: &RenderHybridGiPreparedTraceRegionSceneData,
    view_proj: Mat4,
    camera_position: Vec3,
) -> Option<GpuHybridGiTraceRegion> {
    let bounds_center = Vec3::new(
        dequantized_signed(region.center_x_q),
        dequantized_signed(region.center_y_q),
        dequantized_signed(region.center_z_q),
    );
    let bounds_radius = region.radius_q as f32 / HYBRID_GI_TRACE_RADIUS_SCALE;
    let screen_coverage = region.coverage_q as f32 / HYBRID_GI_TRACE_COVERAGE_SCALE;
    let (uv_x, uv_y) = project_screen_uv(view_proj, bounds_center)?;
    let screen_radius = projected_screen_radius(bounds_radius, bounds_center, camera_position);
    let rt_lighting = [
        f32::from(region.rt_lighting_rgb[0]) / 255.0,
        f32::from(region.rt_lighting_rgb[1]) / 255.0,
        f32::from(region.rt_lighting_rgb[2]) / 255.0,
    ];

    Some(GpuHybridGiTraceRegion {
        screen_uv_and_radius: [uv_x, uv_y, screen_radius, 0.0],
        boost_and_coverage: [
            1.0,
            screen_coverage.clamp(0.0, 1.0),
            region.region_id as f32,
            0.0,
        ],
        rt_lighting_rgb_and_weight: [rt_lighting[0], rt_lighting[1], rt_lighting[2], 1.0],
    })
}

// 恢复与 probe sideband 共用的量化位置域；传入字段须符合 runtime 编码范围。
fn dequantized_signed(value: u32) -> f32 {
    (value as i32 - HYBRID_GI_POSITION_BIAS) as f32 / HYBRID_GI_POSITION_SCALE
}

// 使用未抖动视图把区域中心映射到当前屏幕权重域，深度不适用的中心不参与组合。
fn project_screen_uv(view_proj: Mat4, position: Vec3) -> Option<(f32, f32)> {
    let clip = view_proj * position.extend(1.0);
    if clip.w.abs() <= f32::EPSILON {
        return None;
    }

    let ndc = clip.truncate() / clip.w;
    if ndc.z < -1.0 || ndc.z > 1.0 {
        return None;
    }

    Some((
        (0.5 + ndc.x * 0.5).clamp(0.0, 1.0),
        (0.5 - ndc.y * 0.5).clamp(0.0, 1.0),
    ))
}

// 距离近似的有界覆盖用于 GI 支撑权重；与 probe 编码保持同一屏幕权重模型。
fn projected_screen_radius(radius: f32, position: Vec3, camera_position: Vec3) -> f32 {
    let distance = (camera_position - position).length().max(1.0);
    (radius.max(0.05) / distance).clamp(0.04, 0.75)
}

#[cfg(test)]
#[path = "tests/encode.rs"]
mod tests;

#[cfg(test)]
#[path = "encode/tests/hash_index_tests.rs"]
mod hash_index_tests;
