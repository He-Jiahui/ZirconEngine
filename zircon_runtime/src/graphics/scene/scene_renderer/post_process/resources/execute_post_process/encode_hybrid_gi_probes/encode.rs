use crate::core::framework::render::{
    RenderHybridGiPreparedFrame, RenderHybridGiPreparedProbe,
    RenderHybridGiPreparedProbeRtLighting, RenderHybridGiPreparedProbeSceneData,
};
use crate::core::math::{Mat4, UVec2, Vec3};
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_PROBES;
use super::super::super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;
use super::super::camera_matrices::view_projection;

// 与 runtime prepared scene sideband 的量化字段相配套，不能作为任意世界坐标的独立编码格式。
const HYBRID_GI_POSITION_BIAS: i32 = 2048;
const HYBRID_GI_POSITION_SCALE: f32 = 64.0;
const HYBRID_GI_RADIUS_SCALE: f32 = 64.0;
const PREPARED_RT_LIGHTING_WEIGHT: f32 = 0.75;

// 在一次探针编码中共享场景和 RT 光照的查找约定；规范升序输入可复用二分，非规范输入保留首个匹配语义。
struct PreparedProbeSidebandLookup<'a> {
    scene_data: &'a [RenderHybridGiPreparedProbeSceneData],
    rt_lighting: &'a [RenderHybridGiPreparedProbeRtLighting],
    scene_data_is_canonical: bool,
    rt_lighting_is_canonical: bool,
}

impl<'a> PreparedProbeSidebandLookup<'a> {
    // 一次验证两种 sideband 的规范性，使 resident probe 循环不重复判定排序/重复键。
    fn new(prepared_frame: &'a RenderHybridGiPreparedFrame) -> Self {
        let scene_data = prepared_frame.probe_scene_data.as_slice();
        let rt_lighting = prepared_frame.probe_rt_lighting_rgb.as_slice();
        Self {
            scene_data,
            rt_lighting,
            scene_data_is_canonical: strictly_increasing_by_key(scene_data, scene_data_probe_id),
            rt_lighting_is_canonical: strictly_increasing_by_key(rt_lighting, rt_lighting_probe_id),
        }
    }

    // 缺少场景位置的 resident probe 不能投影，由上层丢弃该探针。
    fn scene_data(&self, probe_id: u32) -> Option<&RenderHybridGiPreparedProbeSceneData> {
        lookup_by_probe_id(
            self.scene_data,
            probe_id,
            self.scene_data_is_canonical,
            scene_data_probe_id,
        )
    }

    // RT 光照是可选支撑，缺失时保留探针自身辐照度并使用中性 RT 权重。
    fn rt_lighting(&self, probe_id: u32) -> Option<&RenderHybridGiPreparedProbeRtLighting> {
        lookup_by_probe_id(
            self.rt_lighting,
            probe_id,
            self.rt_lighting_is_canonical,
            rt_lighting_probe_id,
        )
    }
}

/// 将 runtime 已准备的 resident probes 投影成当前视图可上传的有效前缀。
/// 编译特性、extract 启用与 prepared sideband 三者共同约束可用性；返回计数是 shader 可读范围。
pub(in super::super) fn encode_hybrid_gi_probes(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiProbe; MAX_HYBRID_GI_PROBES], u32) {
    let mut probes = [GpuHybridGiProbe::zeroed(); MAX_HYBRID_GI_PROBES];
    if !enabled {
        return (probes, 0);
    }

    let Some(_hybrid_gi) = frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .filter(|extract| extract.enabled)
    else {
        return (probes, 0);
    };

    let camera = &frame.extract.view.camera;
    let view_proj = view_projection(camera, viewport_size);
    let camera_position = camera.transform.translation;
    let mut count = 0;

    if let Some(prepared_frame) = frame
        .prepared_runtime_sidebands()
        .hybrid_gi_prepared_frame()
    {
        let sidebands = PreparedProbeSidebandLookup::new(prepared_frame);
        for prepared_probe in &prepared_frame.resident_probes {
            if count >= MAX_HYBRID_GI_PROBES {
                break;
            }
            let Some(gpu_probe) = project_prepared_hybrid_gi_probe(
                prepared_probe,
                prepared_frame,
                &sidebands,
                view_proj,
                camera_position,
            ) else {
                continue;
            };
            probes[count] = gpu_probe;
            count += 1;
        }
    }

    (probes, count as u32)
}

// 将同一 probe ID 的空间、辐照度和 RT sideband 组合为屏幕影响权重；缺少空间描述或投影失败即排除。
fn project_prepared_hybrid_gi_probe(
    probe: &RenderHybridGiPreparedProbe,
    prepared_frame: &RenderHybridGiPreparedFrame,
    sidebands: &PreparedProbeSidebandLookup<'_>,
    view_proj: Mat4,
    camera_position: Vec3,
) -> Option<GpuHybridGiProbe> {
    let scene_data = sidebands.scene_data(probe.probe_id)?;
    let position = dequantized_probe_position(scene_data);
    let radius = dequantized_probe_radius(scene_data);
    let (uv_x, uv_y) = project_screen_uv(view_proj, position)?;
    let screen_radius = projected_screen_radius(radius, position, camera_position);
    let budget_weight = ((probe.ray_budget.max(1) as f32) / 128.0).clamp(0.25, 1.5);
    let temporal_signature = probe_temporal_signature(probe, prepared_frame);
    let irradiance = rgb8_to_unit(probe.irradiance_rgb);
    let rt_lighting = sidebands
        .rt_lighting(probe.probe_id)
        .map(|rt_lighting| rgb8_to_unit(rt_lighting.rt_lighting_rgb))
        .unwrap_or([0.0; 3]);
    let rt_lighting_weight = if rt_lighting.iter().any(|channel| *channel > 0.0) {
        PREPARED_RT_LIGHTING_WEIGHT
    } else {
        0.0
    };

    Some(GpuHybridGiProbe {
        screen_uv_and_radius: [uv_x, uv_y, screen_radius, budget_weight],
        irradiance_and_intensity: [irradiance[0], irradiance[1], irradiance[2], 1.0],
        hierarchy_irradiance_rgb_and_weight: [0.0, 0.0, 0.0, 0.0],
        hierarchy_rt_lighting_rgb_and_weight: [
            rt_lighting[0],
            rt_lighting[1],
            rt_lighting[2],
            rt_lighting_weight,
        ],
        temporal_signature_and_padding: [
            temporal_signature,
            1.0,
            probe.source_mask as f32,
            f32::from(probe.dynamic_weight_q8) / 255.0,
        ],
    })
}

// 规范性标志必须对应此切片；非规范路径保留测试固定的重复键首条记录，不能随意改成末条覆盖。
fn lookup_by_probe_id<T>(
    entries: &[T],
    probe_id: u32,
    is_canonical: bool,
    key: fn(&T) -> u32,
) -> Option<&T> {
    if is_canonical {
        let index = entries.binary_search_by_key(&probe_id, key).ok()?;
        return entries.get(index);
    }
    entries.iter().find(|entry| key(entry) == probe_id)
}

// 严格升序同时排除重复键，让二分查找与历史首条匹配约定一致。
fn strictly_increasing_by_key<T>(entries: &[T], key: fn(&T) -> u32) -> bool {
    entries.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

fn scene_data_probe_id(entry: &RenderHybridGiPreparedProbeSceneData) -> u32 {
    entry.probe_id
}

fn rt_lighting_probe_id(entry: &RenderHybridGiPreparedProbeRtLighting) -> u32 {
    entry.probe_id
}

// 给 GI 历史写入的主导探针提供粗粒度来源签名；策略 epoch/烘焙代际变化也参与该签名。
// 有限 bucket 用于降低误复用概率，不能作为无碰撞 probe 标识。
fn probe_temporal_signature(
    probe: &RenderHybridGiPreparedProbe,
    prepared_frame: &RenderHybridGiPreparedFrame,
) -> f32 {
    let policy = prepared_frame.composite_policy;
    let generation = policy.baked_light_set_generation().unwrap_or_default();
    let mut signature = probe.probe_id
        ^ probe.stable_instance_key as u32
        ^ (probe.stable_instance_key >> 32) as u32
        ^ probe.source_mask.rotate_left(7)
        ^ (policy.participation_epoch() as u32).rotate_left(13)
        ^ (generation as u32).rotate_left(19)
        ^ ((generation >> 32) as u32).rotate_left(23);
    signature ^= signature >> 16;
    signature = signature.wrapping_mul(0x7FEB_352D);
    signature ^= signature >> 15;
    let bucket = signature % 1023 + 1;
    bucket as f32 / 1024.0
}

// 将有效深度中心映射为稳定的屏幕影响中心，屏幕边缘夹取属于此近似权重模型。
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

// 有界的距离近似用于组合权重的屏幕覆盖，避免近处探针覆盖无限扩大；不是几何体的精确投影半径。
fn projected_screen_radius(radius: f32, position: Vec3, camera_position: Vec3) -> f32 {
    let distance = (camera_position - position).length().max(1.0);
    (radius.max(0.05) / distance).clamp(0.04, 0.75)
}

// 恢复 prepared scene sideband 的世界空间中心，随后与当前视图矩阵一同使用。
fn dequantized_probe_position(scene_data: &RenderHybridGiPreparedProbeSceneData) -> Vec3 {
    Vec3::new(
        dequantized_signed(scene_data.position_x_q),
        dequantized_signed(scene_data.position_y_q),
        dequantized_signed(scene_data.position_z_q),
    )
}

// 输入须满足 runtime 量化坐标域；与上述 bias/scale 和 sideband 生产者保持一致。
fn dequantized_signed(value: u32) -> f32 {
    (value as i32 - HYBRID_GI_POSITION_BIAS) as f32 / HYBRID_GI_POSITION_SCALE
}

// 半径采用 prepared probe 的专用量化尺度，与 trace region 的半径尺度分别维护。
fn dequantized_probe_radius(scene_data: &RenderHybridGiPreparedProbeSceneData) -> f32 {
    scene_data.radius_q as f32 / HYBRID_GI_RADIUS_SCALE
}

// Runtime sideband 的八位光照通道进入 GPU 浮点权重域，共享于探针辐照度与 RT 辅助照明。
fn rgb8_to_unit(rgb: [u8; 3]) -> [f32; 3] {
    [
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
    ]
}

#[cfg(test)]
#[path = "tests/encode.rs"]
mod tests;
