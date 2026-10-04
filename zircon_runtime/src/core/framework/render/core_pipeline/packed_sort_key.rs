use super::{RenderPhase, RenderPhaseSortComponents};

pub const SORT_KEY_CAMERA_ORDER_SHIFT: u32 = 56;
pub const SORT_KEY_QUEUE_SHIFT: u32 = 43;
pub const SORT_KEY_DOMAIN_SHIFT: u32 = 10;

pub(super) const SORT_KEY_CAMERA_ORDER_MASK: u64 = 0xff;
pub(super) const SORT_KEY_QUEUE_MASK: u64 = 0x1fff;
pub(super) const SORT_KEY_DOMAIN_MASK: u64 = 0x1_ffff_ffff;
pub(super) const SORT_KEY_TIE_MASK: u64 = 0x03ff;
pub(super) const Y_SORT_UNITS: f32 = 16.0;

/// 不透明阶段优先批次聚类，透明 3D 优先远近顺序；不同阶段不能直接比较此键。
pub fn packed_sort_key_u64(
    phase: RenderPhase,
    components: RenderPhaseSortComponents,
    pipeline_variant: u32,
    material_discriminant: u16,
) -> u64 {
    ((camera_order_key(components.camera_order) & SORT_KEY_CAMERA_ORDER_MASK)
        << SORT_KEY_CAMERA_ORDER_SHIFT)
        | (queue_key(components.queue) << SORT_KEY_QUEUE_SHIFT)
        | ((domain_key(phase, components, pipeline_variant, material_discriminant)
            & SORT_KEY_DOMAIN_MASK)
            << SORT_KEY_DOMAIN_SHIFT)
        | entity_tie_breaker_key(components.entity_tie_breaker)
}

pub(super) fn camera_order_key(value: i32) -> u64 {
    signed_lane(value, -128, 127)
}

pub(super) fn queue_key(value: super::RenderQueueValue) -> u64 {
    u64::from(value.raw()).min(SORT_KEY_QUEUE_MASK)
}

pub(super) fn domain_key(
    phase: RenderPhase,
    components: RenderPhaseSortComponents,
    pipeline_variant: u32,
    material_discriminant: u16,
) -> u64 {
    match phase {
        RenderPhase::Transparent3d => {
            (transparent_depth_key(components.effective_depth()) << 10)
                | pipeline_cluster_key(pipeline_variant)
        }
        RenderPhase::Opaque2d | RenderPhase::AlphaMask2d | RenderPhase::Transparent2d => {
            (sorting_layer_key(components.sorting_layer) << 25)
                | (order_in_layer_key(components.order_in_layer) << 10)
                | y_sort_key(components.y_sort)
        }
        RenderPhase::Ui | RenderPhase::Overlay => ui_z_index_key(components.ui_z_index) << 10,
        _ => {
            (pipeline_cluster_key(pipeline_variant) << 23)
                | (material_cluster_key(material_discriminant) << 15)
                | opaque_depth_key(components.effective_depth())
        }
    }
}

pub(super) fn pipeline_cluster_key(value: u32) -> u64 {
    u64::from(value) & 0x03ff
}

pub(super) fn material_cluster_key(value: u16) -> u64 {
    u64::from(((value >> 8) ^ value) & 0x00ff)
}

pub(super) fn opaque_depth_key(effective_depth: f32) -> u64 {
    quantized_non_negative_depth(effective_depth, 8.0, 0x7fff)
}

pub(super) fn transparent_depth_key(effective_depth: f32) -> u64 {
    0x7f_ffff - quantized_non_negative_depth(effective_depth, 1000.0, 0x7f_ffff)
}

pub(super) fn sorting_layer_key(value: i32) -> u64 {
    signed_lane(value, -128, 127)
}

pub(super) fn order_in_layer_key(value: i32) -> u64 {
    signed_lane(value, -16_384, 16_383)
}

pub(super) fn y_sort_key(value: Option<f32>) -> u64 {
    value
        .filter(|value| value.is_finite())
        .map(|value| {
            let rounded = (value * Y_SORT_UNITS).round() as i32;
            signed_lane(rounded, -512, 511)
        })
        .unwrap_or(512)
}

pub(super) fn ui_z_index_key(value: i32) -> u64 {
    signed_lane(value, -4_194_304, 4_194_303)
}

pub(super) fn entity_tie_breaker_key(value: u64) -> u64 {
    value & SORT_KEY_TIE_MASK
}

pub(super) fn depth_sort_key(effective_depth: f32) -> i64 {
    if effective_depth.is_finite() {
        (effective_depth * 1000.0).round() as i64
    } else {
        0
    }
}

// BUG: [CR-RENDER-CORE-0001] 极大负深度量化为 i64::MIN 后，透明阶段取负在调试构建中溢出；诊断拆解可触发。
pub(super) fn ordered_depth_key(phase: RenderPhase, depth_key: i64) -> i64 {
    if phase == RenderPhase::Transparent3d {
        -depth_key
    } else {
        depth_key
    }
}

fn signed_lane(value: i32, min: i32, max: i32) -> u64 {
    (value.clamp(min, max) - min) as u64
}

fn quantized_non_negative_depth(effective_depth: f32, units: f32, max: u64) -> u64 {
    if !effective_depth.is_finite() {
        return 0;
    }
    ((effective_depth * units).round() as i64).clamp(0, max as i64) as u64
}

#[cfg(test)]
#[path = "tests/packed_sort_key.rs"]
mod tests;
