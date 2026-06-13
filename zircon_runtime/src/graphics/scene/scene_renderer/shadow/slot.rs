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
mod tests {
    use super::*;
    use crate::core::framework::render::{ShadowPcfQuality, ShadowResolutionTier};
    use crate::graphics::scene::scene_renderer::shadow::atlas::{ShadowAtlasRect, ShadowSlotKey};
    use std::mem::{offset_of, size_of};

    fn allocation() -> ShadowSlotAllocation {
        ShadowSlotAllocation {
            key: ShadowSlotKey::new(42, 0),
            rect: ShadowAtlasRect::new(256, 512, 1024, 1024),
            requested_tier: ShadowResolutionTier::T1024,
            allocated_tier: ShadowResolutionTier::T1024,
            priority: 1.0,
            reused_previous: false,
        }
    }

    #[test]
    fn render_shadow_slot_layout_matches_plan_05_std430_contract() {
        assert_eq!(size_of::<GpuShadowSlot>(), GPU_SHADOW_SLOT_STRIDE);
        assert_eq!(offset_of!(GpuShadowSlot, view_proj), 0);
        assert_eq!(offset_of!(GpuShadowSlot, atlas_scale_bias), 64);
        assert_eq!(offset_of!(GpuShadowSlot, params), 80);
    }

    #[test]
    fn render_shadow_slot_disabled_has_no_valid_flag() {
        let slot = GpuShadowSlot::disabled();

        assert_eq!(slot.view_proj, Mat4::IDENTITY.to_cols_array_2d());
        assert_eq!(slot.flags_bits() & GPU_SHADOW_SLOT_FLAG_VALID, 0);
    }

    #[test]
    fn render_shadow_slot_from_allocation_writes_atlas_slice_and_flags() {
        let slot = GpuShadowSlot::from_allocation(
            allocation(),
            Mat4::IDENTITY,
            4096,
            4096,
            0.003,
            0.01,
            ShadowPcfQuality::High,
            GPU_SHADOW_SLOT_FLAG_DIRECTIONAL_CASCADE,
        );

        assert_eq!(slot.atlas_scale_bias, [0.25, 0.25, 0.0625, 0.125]);
        assert_eq!(slot.params[0], 0.003);
        assert_eq!(slot.params[1], 0.01);
        assert_eq!(slot.params[2], 1.0 / 1024.0);
        assert_ne!(slot.flags_bits() & GPU_SHADOW_SLOT_FLAG_VALID, 0);
        assert_ne!(
            slot.flags_bits() & GPU_SHADOW_SLOT_FLAG_DIRECTIONAL_CASCADE,
            0
        );
        assert_eq!(
            slot.flags_bits() & GPU_SHADOW_SLOT_PCF_QUALITY_MASK,
            GPU_SHADOW_SLOT_PCF_QUALITY_HIGH
        );
    }

    #[test]
    fn render_shadow_slot_encodes_pcf_quality_in_flags() {
        assert_eq!(
            shadow_pcf_quality_flag_bits(ShadowPcfQuality::Low),
            GPU_SHADOW_SLOT_PCF_QUALITY_LOW
        );
        assert_eq!(
            shadow_pcf_quality_flag_bits(ShadowPcfQuality::Medium),
            GPU_SHADOW_SLOT_PCF_QUALITY_MEDIUM
        );
        assert_eq!(
            shadow_pcf_quality_flag_bits(ShadowPcfQuality::High),
            GPU_SHADOW_SLOT_PCF_QUALITY_HIGH
        );
    }

    #[test]
    fn render_shadow_globals_layout_and_atlas_params_are_stable() {
        assert_eq!(size_of::<GpuShadowGlobals>(), GPU_SHADOW_GLOBALS_STRIDE);
        assert_eq!(offset_of!(GpuShadowGlobals, cascade_splits), 0);
        assert_eq!(offset_of!(GpuShadowGlobals, cascade_fade_lengths), 16);
        assert_eq!(offset_of!(GpuShadowGlobals, atlas_params), 32);

        let globals = GpuShadowGlobals::from_cascade_ranges(
            &[
                CascadeRange {
                    index: 0,
                    near: 0.1,
                    far: 10.0,
                    fade_start: 9.0,
                    fade_length: 1.0,
                },
                CascadeRange {
                    index: 1,
                    near: 10.0,
                    far: 40.0,
                    fade_start: 37.0,
                    fade_length: 3.0,
                },
            ],
            4096,
            2048,
        );

        assert_eq!(globals.cascade_splits, [10.0, 40.0, 0.0, 0.0]);
        assert_eq!(globals.cascade_fade_lengths, [1.0, 3.0, 0.0, 0.0]);
        assert_eq!(
            globals.atlas_params,
            [4096.0, 2048.0, 1.0 / 4096.0, 1.0 / 2048.0]
        );
    }
}
