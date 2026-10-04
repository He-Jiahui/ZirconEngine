use std::cell::Cell;
use std::sync::Arc;

use crate::core::math::UVec2;
use crate::graphics::types::ViewportRenderRegion;

use super::{BoundedResourceCache, TerminalPostProcessResourceCache};

#[test]
fn bounded_resource_cache_reuses_a_matching_resource_without_recreating_it() {
    let creates = Cell::new(0_u32);
    let mut cache = BoundedResourceCache::new(2);

    let first = cache.get_or_insert_with(64_u32, || {
        creates.set(creates.get() + 1);
        1_u32
    });
    let second = cache.get_or_insert_with(64_u32, || {
        creates.set(creates.get() + 1);
        2_u32
    });

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(creates.get(), 1);
}

#[test]
fn bounded_resource_cache_evicts_the_oldest_entry_at_its_fixed_capacity() {
    let mut cache = BoundedResourceCache::new(1);
    let first = cache.get_or_insert_with(16_u32, || 1_u32);
    let replacement = cache.get_or_insert_with(32_u32, || 2_u32);
    let rebuilt = cache.get_or_insert_with(16_u32, || 3_u32);

    assert_eq!(*replacement, 2);
    assert_eq!(*rebuilt, 3);
    assert!(!Arc::ptr_eq(&first, &rebuilt));
}

#[test]
fn bounded_resource_cache_hit_promotes_the_entry_without_reordering_slots() {
    let mut cache = BoundedResourceCache::new(2);
    let first = cache.get_or_insert_with(1_u32, || 10_u32);
    cache.get_or_insert_with(2_u32, || 20_u32);

    let warm_first = cache.get_or_insert_with(1_u32, || 11_u32);
    assert!(Arc::ptr_eq(&first, &warm_first));
    assert_eq!([cache.entries[0].key, cache.entries[1].key], [1, 2]);

    cache.get_or_insert_with(3_u32, || 30_u32);
    assert_eq!([cache.entries[0].key, cache.entries[1].key], [1, 3]);
}

#[test]
fn terminal_effect_executors_delegate_persistent_resources_to_the_cache_owner() {
    let fxaa = include_str!("../execute_fxaa/mod.rs");
    let output_transfer = include_str!("../execute_output_transfer/mod.rs");
    let smaa = include_str!("../execute_smaa/mod.rs");

    // BUG: [CR-SCENE-POST-0006] FXAA 当前使用局部终端参数；循环却要求两条路径都出现物理区域调用，FXAA 的源码匹配恒为 false。
    for (label, source) in [("FXAA", fxaa), ("output transfer", output_transfer)] {
        assert!(
            source.contains("physical_terminal_region_params_buffer(device, render_region)"),
            "{label} must resolve terminal uniforms through the persistent owner"
        );
        assert!(
            !source.contains("create_physical_terminal_region_params_buffer"),
            "{label} must not create a physical terminal uniform every frame"
        );
    }
    assert!(
        smaa.contains("smaa_stage_textures(device, viewport_size)"),
        "SMAA must resolve edge/blend backing textures through the persistent owner"
    );
    assert!(
        !smaa.contains("create_smaa_stage_texture"),
        "SMAA must not create edge/blend backing textures every frame"
    );
}

#[test]
fn scene_post_process_resources_constructs_the_terminal_cache_at_the_resources_root() {
    // BUG: [CR-SCENE-POST-0003] 资源类型已拆到子文件，此处读取仅含模块声明和重导出的根文件，后面的字段归属断言必为 false。
    let resources = include_str!("../../scene_post_process_resources/mod.rs");
    let constructor = include_str!("../construct/construct/construct.rs");
    let compact_resources = resources.split_whitespace().collect::<String>();
    let compact_constructor = constructor.split_whitespace().collect::<String>();

    assert!(
        compact_resources.contains("terminal_resource_cache:TerminalPostProcessResourceCache,"),
        "the persistent cache belongs to the post-process resource owner"
    );
    assert!(
        compact_constructor.contains(
            "usesuper::super::super::terminal_resource_cache::TerminalPostProcessResourceCache;"
        ),
        "the nested constructor must import the cache from the resources root"
    );
    assert!(
        compact_constructor
            .contains("terminal_resource_cache:TerminalPostProcessResourceCache::new(),"),
        "the post-process resource owner must initialize the cache once"
    );
}

#[test]
fn terminal_resource_cache_reuses_wgpu_backing_until_the_extent_changes() {
    let Some(device) = offscreen_test_device() else {
        eprintln!("skipping terminal resource cache GPU test: no WGPU adapter is available");
        return;
    };
    let cache = TerminalPostProcessResourceCache::new();
    let region = ViewportRenderRegion::full_target(UVec2::new(64, 48));

    let first_uniform = cache.physical_terminal_region_params_buffer(&device, region);
    let warm_uniform = cache.physical_terminal_region_params_buffer(&device, region);
    let first_stages = cache.smaa_stage_textures(&device, UVec2::new(64, 48));
    let warm_stages = cache.smaa_stage_textures(&device, UVec2::new(64, 48));
    let resized_stages = cache.smaa_stage_textures(&device, UVec2::new(96, 64));

    assert!(Arc::ptr_eq(&first_uniform, &warm_uniform));
    assert!(Arc::ptr_eq(&first_stages, &warm_stages));
    assert!(!Arc::ptr_eq(&first_stages, &resized_stages));
}

fn offscreen_test_device() -> Option<wgpu::Device> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-terminal-resource-cache-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
    .map(|(device, _queue)| device)
}
