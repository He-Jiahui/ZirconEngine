use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_gpu_texture_from_asset_tests_are_child_owner() {
    let parent =
        read_runtime_src("graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs");
    let texture_format = read_runtime_src(
        "graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/texture_format.rs",
    );
    let sampler_cache = read_runtime_src("graphics/scene/resources/gpu_texture/sampler_cache.rs");
    let gpu_texture_resource =
        read_runtime_src("graphics/scene/resources/gpu_texture/gpu_texture_resource.rs");
    let fallback = read_runtime_src("graphics/scene/resources/fallback/create_fallback_texture.rs");
    let resource_streamer =
        read_runtime_src("graphics/scene/resources/resource_streamer/resource_streamer.rs");
    let resource_streamer_construction = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_construction.rs",
    );
    let resource_streamer_ensure_texture = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs",
    );
    let tests = read_runtime_src(
        "graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs",
    );

    let plan_13 = read_repo("docs/plans/zircon_runtime/render/13-texture-pipeline.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_assets_doc = read_repo("docs/zircon_runtime/asset/render-assets.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "GPU texture asset parent keeps upload, shared sampler consumption, and child test mount",
        &parent,
        &[
            "mod texture_format;",
            "pub(crate) use self::texture_format::texture_upload_support_from_device;",
            "pub(crate) fn from_asset(",
            "fn from_rgba8_asset(",
            "fn from_compressed_asset(",
            "use super::sampler_cache::{",
            "sampler_cache: Arc<TextureSamplerCache>,",
            "fn rgba8_mip_uploads(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    assert_contains_all(
        "GPU texture format owner keeps usage, capability, and compressed mappings",
        &texture_format,
        &[
            "pub(super) fn wgpu_texture_usages(",
            "pub(crate) fn texture_upload_support_from_device(",
            "pub(super) fn rgba8_wgpu_format(",
            "pub(super) fn compressed_wgpu_format(",
            "fn ktx_gl_wgpu_format(",
            "fn ktx2_vk_wgpu_format(",
            "fn astc_block_by_index(",
        ],
    );

    assert_contains_all(
        "GPU texture sampler cache owns device-lifetime sampler reuse",
        &sampler_cache,
        &[
            "pub(in crate::graphics::scene::resources) struct TextureSamplerCache",
            "pub(in crate::graphics::scene::resources) fn sampler_for_image(",
            "pub(in crate::graphics::scene::resources) fn sampler_for_image_with_anisotropy_cap(",
            "struct TextureSamplerKey",
            "samplers: Mutex<HashMap<TextureSamplerKey, Arc<wgpu::Sampler>>>",
            "sampler_descriptor_for_image_with_anisotropy_cap(descriptor, max_anisotropy)",
            "fn texture_sampler_key_distinguishes_each_address_and_filter_field()",
        ],
    );
    assert!(
        parent.contains("self.sampler_cache.sampler_for_image_with_anisotropy_cap("),
        "quality anisotropy variants must reuse the shared texture sampler cache"
    );
    assert_contains_all(
        "fallback textures consume the shared sampler cache",
        &fallback,
        &[
            "sampler_cache: Arc<TextureSamplerCache>",
            "sampler_cache.sampler_for_image(device, &descriptor)",
            "sampler_cache,",
        ],
    );
    assert_contains_all(
        "resource streamer owns the device-generation sampler cache",
        &resource_streamer,
        &["texture_sampler_cache: Arc<TextureSamplerCache>"],
    );
    assert_contains_all(
        "gpu texture resources retain shared sampler ownership",
        &gpu_texture_resource,
        &[
            "sampler: Arc<wgpu::Sampler>",
            "sampler_cache: Arc<TextureSamplerCache>",
            "self.sampler.as_ref()",
        ],
    );
    assert_contains_all(
        "resource streamer constructs its sampler cache before fallback resources",
        &resource_streamer_construction,
        &[
            "let texture_sampler_cache = Arc::new(TextureSamplerCache::new());",
            "Arc::clone(&texture_sampler_cache)",
            "texture_sampler_cache,",
        ],
    );
    assert_contains_all(
        "texture uploads retain the streamer sampler cache",
        &resource_streamer_ensure_texture,
        &[
            "GpuTextureResource::from_asset(",
            "Arc::clone(&self.texture_sampler_cache),",
        ],
    );
    for (label, source) in [
        ("asset texture uploads", parent.as_str()),
        ("fallback texture uploads", fallback.as_str()),
    ] {
        assert!(
            !source.contains("device.create_sampler("),
            "{label} must resolve samplers through TextureSamplerCache"
        );
    }

    for moved_anchor in [
        "fn rgba8_wgpu_format_uses_upload_plan_format(",
        "fn rgba8_mip_uploads_pack_levels_and_layers_in_payload_order(",
        "fn rgba8_mip_uploads_pack_layers_inside_each_mip_level(",
        "fn rgba8_material_texture_view_keeps_current_d2_binding_contract(",
        "fn wgpu_texture_usages_maps_render_image_usage_for_asset_residency(",
        "fn wgpu_texture_usages_does_not_add_upload_dst_when_not_required(",
        "fn wgpu_texture_usages_skips_storage_for_non_storage_formats(",
        "fn wgpu_texture_usages_skips_render_attachment_for_non_renderable_formats(",
        "fn sampler_descriptor_maps_texture_asset_sampler_settings(",
        "fn test_descriptor(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "GPU texture from_asset parent should delegate `{moved_anchor}` to tests.rs"
        );
        assert!(
            tests.contains(moved_anchor),
            "GPU texture from_asset test owner should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "GPU texture from_asset test owner keeps format, mip upload, usage, view, and sampler coverage",
        &tests,
        &[
            "use super::*;",
            "RGBA8_UNORM_FORMAT",
            "RGBA8_UNORM_SRGB_FORMAT",
            "Rgba8MipUpload",
            "RenderImageUsage::Storage",
            "RenderSamplerDescriptor",
        ],
    );

    for (path, source) in [
        (
            "gpu_texture/gpu_texture_resource_from_asset.rs",
            parent.as_str(),
        ),
        (
            "gpu_texture/gpu_texture_resource_from_asset/texture_format.rs",
            texture_format.as_str(),
        ),
        ("gpu_texture/sampler_cache.rs", sampler_cache.as_str()),
        (
            "gpu_texture/gpu_texture_resource_from_asset/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 13", &plan_13),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("render asset docs", &render_assets_doc),
        ("render submit docs", &render_submit_doc),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "GpuTextureResource from_asset tests owner split",
                "render_plan13_gpu_texture_from_asset_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs",
                "graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset/tests.rs",
                "runtime_15_gpu_texture_from_asset_tests_are_child_owner",
            ],
        );
    }
}
