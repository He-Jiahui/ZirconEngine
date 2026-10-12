use super::*;
use crate::core::framework::render::{GpuLightType, ShadowPcfQuality, ShadowResolutionTier};
use crate::graphics::scene::scene_renderer::shadow::atlas::{ShadowAtlasRect, ShadowSlotKey};
use std::mem::{offset_of, size_of};

fn allocation() -> ShadowSlotAllocation {
    ShadowSlotAllocation {
        key: ShadowSlotKey::new(GpuLightType::Point, 42, 0),
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
