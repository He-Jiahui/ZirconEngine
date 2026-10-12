use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::ShadowPcfQuality;
use crate::core::math::Mat4;

use super::atlas::ShadowSlotAllocation;
use super::cascade::{CascadeRange, MAX_SHADOW_CASCADES};

pub(crate) const GPU_SHADOW_SLOT_STRIDE: usize = 96;
pub(crate) const GPU_SHADOW_GLOBALS_STRIDE: usize = 48;
pub(crate) const GPU_SHADOW_SLOT_FLAG_VALID: u32 = 1 << 0;
pub(crate) const GPU_SHADOW_SLOT_FLAG_DIRECTIONAL_CASCADE: u32 = 1 << 1;
pub(crate) const GPU_SHADOW_SLOT_FLAG_SPOT: u32 = 1 << 2;
pub(crate) const GPU_SHADOW_SLOT_FLAG_POINT_FACE: u32 = 1 << 3;
pub(crate) const GPU_SHADOW_SLOT_PCF_QUALITY_SHIFT: u32 = 8;
pub(crate) const GPU_SHADOW_SLOT_PCF_QUALITY_MASK: u32 = 0b11 << GPU_SHADOW_SLOT_PCF_QUALITY_SHIFT;
pub(crate) const GPU_SHADOW_SLOT_PCF_QUALITY_LOW: u32 = 0 << GPU_SHADOW_SLOT_PCF_QUALITY_SHIFT;
pub(crate) const GPU_SHADOW_SLOT_PCF_QUALITY_MEDIUM: u32 = 1 << GPU_SHADOW_SLOT_PCF_QUALITY_SHIFT;
pub(crate) const GPU_SHADOW_SLOT_PCF_QUALITY_HIGH: u32 = 2 << GPU_SHADOW_SLOT_PCF_QUALITY_SHIFT;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuShadowSlot {
    pub(crate) view_proj: [[f32; 4]; 4],
    pub(crate) atlas_scale_bias: [f32; 4],
    pub(crate) params: [f32; 4],
}

impl GpuShadowSlot {
    pub(crate) fn disabled() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            atlas_scale_bias: [0.0, 0.0, 0.0, 0.0],
            params: [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub(crate) fn from_allocation(
        allocation: ShadowSlotAllocation,
        view_proj: Mat4,
        atlas_width: u32,
        atlas_height: u32,
        depth_bias: f32,
        normal_bias: f32,
        pcf_quality: ShadowPcfQuality,
        flags: u32,
    ) -> Self {
        let slot_texel_size = 1.0 / allocation.rect.width.max(1) as f32;
        let flags = flags | shadow_pcf_quality_flag_bits(pcf_quality) | GPU_SHADOW_SLOT_FLAG_VALID;
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            atlas_scale_bias: allocation.atlas_scale_bias(atlas_width, atlas_height),
            params: [
                depth_bias,
                normal_bias,
                slot_texel_size,
                f32::from_bits(flags),
            ],
        }
    }

    pub(crate) fn flags_bits(self) -> u32 {
        self.params[3].to_bits()
    }
}

pub(crate) const fn shadow_pcf_quality_flag_bits(quality: ShadowPcfQuality) -> u32 {
    match quality {
        ShadowPcfQuality::Low => GPU_SHADOW_SLOT_PCF_QUALITY_LOW,
        ShadowPcfQuality::Medium => GPU_SHADOW_SLOT_PCF_QUALITY_MEDIUM,
        ShadowPcfQuality::High => GPU_SHADOW_SLOT_PCF_QUALITY_HIGH,
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuShadowGlobals {
    pub(crate) cascade_splits: [f32; 4],
    pub(crate) cascade_fade_lengths: [f32; 4],
    pub(crate) atlas_params: [f32; 4],
}

impl GpuShadowGlobals {
    pub(crate) fn disabled(atlas_width: u32, atlas_height: u32) -> Self {
        Self {
            cascade_splits: [0.0; 4],
            cascade_fade_lengths: [0.0; 4],
            atlas_params: atlas_params(atlas_width, atlas_height),
        }
    }

    pub(crate) fn from_cascade_ranges(
        ranges: &[CascadeRange],
        atlas_width: u32,
        atlas_height: u32,
    ) -> Self {
        let mut globals = Self::disabled(atlas_width, atlas_height);
        for (index, range) in ranges.iter().take(MAX_SHADOW_CASCADES).enumerate() {
            globals.cascade_splits[index] = range.far;
            globals.cascade_fade_lengths[index] = range.fade_length;
        }
        globals
    }
}

fn atlas_params(atlas_width: u32, atlas_height: u32) -> [f32; 4] {
    let width = atlas_width.max(1) as f32;
    let height = atlas_height.max(1) as f32;
    [width, height, 1.0 / width, 1.0 / height]
}

#[cfg(test)]
#[path = "tests/slot.rs"]
mod tests;
