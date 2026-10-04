use serde::{Deserialize, Serialize};

use crate::core::math::Real;

pub const OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 3;
pub const OIT_GPU_LAYER_SIZE_BYTES: u64 = 8;
pub const OIT_GPU_COUNT_SIZE_BYTES: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OitSettings {
    pub fragments_per_pixel_average: Real,
    pub sorted_fragment_max_count: u32,
    pub alpha_threshold: Real,
}

impl OitSettings {
    pub const DEFAULT: Self = Self {
        fragments_per_pixel_average: 4.0,
        sorted_fragment_max_count: 8,
        alpha_threshold: 0.0,
    };
}

impl Default for OitSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OitCapabilityProfile {
    pub fragment_writable_storage: bool,
    pub max_storage_buffers_per_shader_stage: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OitSupport {
    Supported,
    FallbackSorted { diagnostic: &'static str },
}

pub const fn oit_support(capabilities: OitCapabilityProfile) -> OitSupport {
    if !capabilities.fragment_writable_storage {
        return OitSupport::FallbackSorted {
            diagnostic: "OIT unavailable: fragment writable storage is not supported; using sorted transparency",
        };
    }
    if capabilities.max_storage_buffers_per_shader_stage
        < OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
    {
        return OitSupport::FallbackSorted {
            diagnostic: "OIT unavailable: max_storage_buffers_per_shader_stage is below 3; using sorted transparency",
        };
    }
    OitSupport::Supported
}

/// 容量计划按视口尺寸和每像素配置推导缓冲上限；它描述分配输入，不表示实际片元数或 GPU 写入量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OitBufferPlan {
    pub pixel_count: u64,
    pub fragments_per_pixel_capacity: u32,
    pub fragment_capacity: u64,
    pub layer_buffer_size_bytes: u64,
    pub count_buffer_size_bytes: u64,
}

impl OitBufferPlan {
    pub fn for_view(view_size: [u32; 2], settings: OitSettings) -> Self {
        let pixel_count = u64::from(view_size[0].max(1)) * u64::from(view_size[1].max(1));
        let fragments_per_pixel_capacity = if settings.fragments_per_pixel_average.is_finite() {
            settings.fragments_per_pixel_average.ceil().max(1.0) as u32
        } else {
            OitSettings::DEFAULT.fragments_per_pixel_average as u32
        };
        let fragment_capacity = pixel_count.saturating_mul(u64::from(fragments_per_pixel_capacity));
        Self {
            pixel_count,
            fragments_per_pixel_capacity,
            fragment_capacity,
            layer_buffer_size_bytes: fragment_capacity.saturating_mul(OIT_GPU_LAYER_SIZE_BYTES),
            count_buffer_size_bytes: pixel_count.saturating_mul(OIT_GPU_COUNT_SIZE_BYTES),
        }
    }

    pub(crate) const fn fits_storage_binding_size_limit(self, max_binding_size_bytes: u64) -> bool {
        max_binding_size_bytes > 0
            && self.layer_buffer_size_bytes <= max_binding_size_bytes
            && self.count_buffer_size_bytes <= max_binding_size_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OitFragment {
    pub color: [Real; 4],
    pub depth: Real,
}

impl OitFragment {
    pub const fn new(color: [Real; 4], depth: Real) -> Self {
        Self { color, depth }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OitResolveResult {
    pub color: [Real; 4],
    pub sorted_depths: Vec<Real>,
    pub merged_fragment_count: usize,
}

/// CPU 参考合成按深度保留配置数量的精确前缀，再合并尾部片元；结果用于验证算法语义，不是 GPU 回读。
pub fn resolve_oit_fragments(
    fragments: &[OitFragment],
    sorted_fragment_max_count: u32,
) -> OitResolveResult {
    let ordered = ordered_oit_fragment_indices(fragments);

    let exact_count = ordered
        .len()
        .min(sorted_fragment_max_count.try_into().unwrap_or(usize::MAX));
    let mut premultiplied = [0.0; 4];
    for &(_, fragment_index) in &ordered[..exact_count] {
        let fragment = &fragments[fragment_index];
        blend_front_to_back(&mut premultiplied, fragment.color);
    }
    let mut merged_tail = [0.0; 4];
    for &(_, fragment_index) in &ordered[exact_count..] {
        let fragment = &fragments[fragment_index];
        blend_front_to_back(&mut merged_tail, fragment.color);
    }
    blend_front_to_back(&mut premultiplied, merged_tail);

    OitResolveResult {
        color: premultiplied,
        sorted_depths: ordered[..exact_count]
            .iter()
            .map(|(_, fragment_index)| fragments[*fragment_index].depth)
            .collect(),
        merged_fragment_count: ordered.len().saturating_sub(exact_count),
    }
}

fn ordered_oit_fragment_indices(fragments: &[OitFragment]) -> Vec<(u32, usize)> {
    let mut ordered = fragments
        .iter()
        .enumerate()
        .map(|(fragment_index, fragment)| (total_f32_sort_key(fragment.depth), fragment_index))
        .collect::<Vec<_>>();
    ordered.sort_unstable();
    ordered
}

fn total_f32_sort_key(value: f32) -> u32 {
    let bits = value.to_bits();
    if bits & (1 << 31) == 0 {
        bits ^ (1 << 31)
    } else {
        !bits
    }
}

fn blend_front_to_back(accumulated: &mut [Real; 4], color: [Real; 4]) {
    let alpha = color[3].clamp(0.0, 1.0);
    let remaining = 1.0 - accumulated[3];
    accumulated[0] += remaining * color[0].clamp(0.0, 1.0) * alpha;
    accumulated[1] += remaining * color[1].clamp(0.0, 1.0) * alpha;
    accumulated[2] += remaining * color[2].clamp(0.0, 1.0) * alpha;
    accumulated[3] += remaining * alpha;
}

#[cfg(test)]
#[path = "tests/oit.rs"]
mod tests;
