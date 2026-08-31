use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_90_render_facades_project_their_owned_runtime_contracts() {
    let core_root = read_runtime_src("core/mod.rs");
    let rhi_root = read_runtime_src("rhi.rs");
    let backend_root = read_runtime_src("graphics/backend/mod.rs");
    let feature_root = read_runtime_src("graphics/feature/mod.rs");
    let feature_pass_root =
        read_runtime_src("graphics/feature/render_feature_pass_descriptor/mod.rs");
    let pipeline_declarations_root = read_runtime_src("graphics/pipeline/declarations/mod.rs");
    let scene_root = read_runtime_src("graphics/scene/mod.rs");
    let frame_submission_owner = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline/frame_submission_owner.rs",
    );
    let hit_proxy_gpu_scene = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core/hit_proxy_gpu_scene.rs",
    );
    let post_process_uploads = read_runtime_src(
        "graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/prepare_scene_data_uploads.rs",
    );
    let realtime_ibl_gpu_resources =
        read_runtime_src("graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources.rs");
    let rich_text = read_runtime_src("graphics/scene/scene_renderer/ui/render/rich_text.rs");
    let ui_plan_cache = read_runtime_src("graphics/scene/scene_renderer/ui/render/plan_cache.rs");
    let viewport_pick_sources = [
        "graphics/runtime/render_framework/render_framework_state/viewport_hit_proxy_table.rs",
        "graphics/runtime/render_framework/render_framework_state/viewport_pick_frame_registry.rs",
        "graphics/runtime/render_framework/viewport_pick.rs",
    ]
    .into_iter()
    .map(read_runtime_src)
    .collect::<Vec<_>>()
    .join("\n");

    assert_contains_all(
        "runtime RHI facade",
        &rhi_root,
        &[
            "DeviceGeneration",
            "DeviceId",
            "RenderDeviceProfile",
            "SubmissionPollReceipt",
            "RenderSurfaceDescriptor",
            "SurfaceAcquireOutcome",
            "SurfaceRetryReason",
            "SurfaceSessionCreateOutcome",
            "SurfaceSessionReceipt",
        ],
    );
    assert_contains_all(
        "graphics backend facade",
        &backend_root,
        &["NeutralMvpCaptureError", "NeutralMvpRenderer"],
    );
    assert!(core_root.contains("parallel_map_indices"));
    assert!(feature_root.contains("RenderBufferSchema"));
    assert!(feature_pass_root.contains("RenderFeatureTextureViewAlias"));
    assert!(pipeline_declarations_root.contains("RenderGraphExecutionPass"));
    assert_contains_all(
        "graphics scene facade",
        &scene_root,
        &[
            "RealtimeIblCpuTimingReport",
            "MeshHitProxyTokenSource",
            "SceneHitProxySubmission",
            "SSS_PARAMS_BUFFER_SIZE_BYTES",
            "SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES",
        ],
    );
    assert!(frame_submission_owner.contains("GpuPassTimer, ViewportSurface"));
    assert_contains_all(
        "hit proxy joint-palette arena migration",
        &hit_proxy_gpu_scene,
        &[
            "create_empty_skinned_joint_palette_arena_buffer",
            "skinned_joint_palette_arena_min_binding_size",
        ],
    );
    assert!(!hit_proxy_gpu_scene.contains("skinned_joint_palette_storage_min_binding_size"));
    assert!(post_process_uploads.contains("super::super::super::super::ScenePostProcessResources"));
    assert!(realtime_ibl_gpu_resources.contains("fn slot_resource<'slots>("));
    assert!(!rich_text.contains("&& let "));
    assert!(ui_plan_cache.contains(
        "pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiPlanCache"
    ));
    assert!(!viewport_pick_sources.contains("scene::scene_renderer::"));
}

#[test]
fn runtime_90_upload_owners_are_reachable_only_through_their_module_facades() {
    let backend_root = read_runtime_src("graphics/backend/mod.rs");
    let render_root = read_runtime_src("core/framework/render/mod.rs");
    let resources_root = read_runtime_src("graphics/scene/resources/mod.rs");
    let prepared_root = read_runtime_src("graphics/scene/resources/prepared/mod.rs");
    let resource_streamer_sources = [
        "graphics/scene/resources/resource_streamer/resource_streamer_advanced_lighting.rs",
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs",
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_post_process_lut_texture.rs",
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs",
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs",
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/texture_binding.rs",
        "graphics/scene/resources/resource_streamer/resource_streamer_mip_streaming/frame_apply.rs",
    ]
    .into_iter()
    .map(read_runtime_src)
    .collect::<Vec<_>>()
    .join("\n");

    assert_contains_all(
        "system texture generation backend facade",
        &backend_root,
        &[
            "SystemTextureGenerationLease",
            "SystemTextureGenerationStartupReport",
            "SystemTexturePayloadCacheState",
        ],
    );
    assert!(render_root.contains("pub(crate) use environment::encode_rg16f_texels;"));
    assert!(prepared_root.contains("PreparedMaterialCandidateIdentity"));
    assert!(resources_root.contains("PostProcessLutTextureUploadWork"));
    assert!(!resource_streamer_sources.contains("graphics::backend::render_backend::"));
    assert!(resource_streamer_sources.contains("graphics::backend::RenderBackend"));
}

#[test]
fn runtime_90_system_textures_are_lazy_device_generation_resources() {
    let owner =
        read_runtime_src("graphics/backend/render_backend/system_texture_generation_owner.rs");
    let resources = read_runtime_src(
        "graphics/backend/render_backend/system_texture_generation_owner/resources.rs",
    );
    let payloads = read_runtime_src(
        "graphics/backend/render_backend/system_texture_generation_owner/payloads.rs",
    );
    let backend = read_runtime_src("graphics/backend/render_backend/render_backend.rs");
    let backend_new =
        read_runtime_src("graphics/backend/render_backend/render_backend_new_offscreen.rs");
    let renderer =
        read_runtime_src("graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs");
    let startup_report =
        read_runtime_src("graphics/scene/scene_renderer/core/scene_renderer/startup_report.rs");
    let renderer_new = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs",
    );
    let brdf = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs",
    );
    let cubemap = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs",
    );

    let owner_production = owner.split("#[cfg(test)]").next().unwrap_or_default();
    let owner_new = owner_production
        .split("pub(crate) fn new")
        .nth(1)
        .and_then(|source| source.split("pub(crate) fn acquire").next())
        .expect("system texture cold owner constructor");
    let owner_acquire = owner_production
        .split("pub(crate) fn acquire")
        .nth(1)
        .and_then(|source| source.split("impl SystemTextureGenerationLease").next())
        .expect("system texture generation acquire path");
    assert_eq!(owner_new.matches("create_texture(").count(), 0);
    assert_eq!(owner_acquire.matches("uploads.push(").count(), 0);
    assert_eq!(resources.matches("push_upload(").count(), 7);
    assert_eq!(resources.matches("push_solid_upload(").count(), 6);
    assert!(resources.contains("SYSTEM_TEXTURE_UPLOAD_COUNT: usize = 10"));
    assert!(resources.contains("SYSTEM_TEXTURE_UPLOAD_BYTES: u64 = 16_768"));
    assert_eq!(
        owner_acquire
            .matches("enqueue_native_texture_upload_batch(uploads)")
            .count(),
        1
    );
    assert_eq!(owner_acquire.matches("flush_submissions()?").count(), 1);
    assert!(
        owner_acquire.find("flush_submissions()?").unwrap()
            < owner_acquire.find("*published = Some").unwrap(),
        "system textures must publish only after the device-owned upload flush succeeds"
    );
    assert_contains_all(
        "system texture generation identity and contiguous cubemap upload",
        owner_acquire,
        &[
            "profile.device_id() != self.device_id",
            "profile.generation() != self.generation",
            "if let Some(lease) = published.as_ref()",
            "payload_cache_state: SystemTexturePayloadCacheState::Reused",
            "native_submission_count: 0",
            "texture_upload_count: 0",
        ],
    );
    assert!(resources.contains("with_depth_or_array_layers(BLACK_CUBE_FACE_COUNT)"));
    assert!(!owner_production.contains("queue.write_texture"));
    assert!(!resources.contains("queue.write_texture"));
    assert!(owner_production.contains("OnceLock<Arc<[u8]>>"));
    assert!(payloads.contains("OnceLock<Arc<[u8]>>"));
    assert!(!owner_production.contains("OnceLock<Vec<u8>>"));
    assert!(!payloads.contains("OnceLock<Vec<u8>>"));

    let backend_owner = backend
        .split("pub(crate) struct RenderBackend")
        .nth(1)
        .and_then(|source| source.split("impl RenderBackend").next())
        .expect("render backend owner declaration");
    assert!(
        backend_owner.find("system_textures:").unwrap()
            < backend_owner.find("render_device:").unwrap(),
        "native system resources must drop before their device-generation owner"
    );
    assert_contains_all(
        "lazy owner wiring and scene acquisition",
        &format!("{backend_new}\n{renderer_new}"),
        &[
            "SystemTextureGenerationOwner::new(",
            "backend.acquire_system_texture_lease()?",
            "system_texture_initialization",
        ],
    );

    let renderer_owner = renderer
        .split("pub struct SceneRenderer")
        .nth(1)
        .and_then(|source| source.split("pub struct SceneRendererGpuPassTiming").next())
        .expect("scene renderer owner declaration");
    let renderer_backend = renderer_owner.find("backend: RenderBackend").unwrap();
    for resource_owner in [
        "core: SceneRendererCore",
        "streamer: ResourceStreamer",
        "advanced_plugin_outputs:",
    ] {
        assert!(renderer_owner.find(resource_owner).unwrap() < renderer_backend);
    }
    assert!(renderer.lines().count() <= 800);
    assert!(startup_report.lines().count() <= 800);

    let brdf_production = brdf.split("#[cfg(test)]").next().unwrap_or_default();
    let cubemap_production = cubemap.split("#[cfg(test)]").next().unwrap_or_default();
    let fallback = cubemap_production
        .split("fn fallback(")
        .nth(1)
        .and_then(|source| source.split("fn texture_layout_entry").next())
        .expect("scene environment fallback construction");
    assert_eq!(brdf_production.matches("create_texture(").count(), 0);
    assert_eq!(fallback.matches("create_texture(").count(), 0);
    assert_eq!(fallback.matches("create_sampler(").count(), 0);
    assert_eq!(
        format!("{brdf_production}\n{fallback}")
            .matches("queue.write_texture(")
            .count(),
        0,
        "scene environment wrappers must not bypass the backend generation owner"
    );
}

#[test]
fn runtime_90_material_fallbacks_project_the_system_texture_generation() {
    let owner =
        read_runtime_src("graphics/backend/render_backend/system_texture_generation_owner.rs");
    let resources = read_runtime_src(
        "graphics/backend/render_backend/system_texture_generation_owner/resources.rs",
    );
    let fallback = read_runtime_src("graphics/scene/resources/fallback/create_fallback_texture.rs");
    let streamer = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_construction.rs",
    );
    let sampler_cache = read_runtime_src("graphics/scene/resources/gpu_texture/sampler_cache.rs");
    let renderer = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs",
    );

    assert_contains_all(
        "system textures publish material-compatible white and normal views",
        &format!("{owner}\n{resources}"),
        &[
            "white_rgba8_srgb_view()",
            "normal_rgba8_texture()",
            "normal_rgba8_view()",
            "wgpu::TextureFormat::Rgba8UnormSrgb",
        ],
    );
    assert_contains_all(
        "resource streamer material fallbacks are binding projections, not upload owners",
        &format!("{fallback}\n{streamer}\n{sampler_cache}\n{renderer}"),
        &[
            "SystemTextureGenerationLease",
            "create_fallback_texture_from_system",
            "create_fallback_normal_texture_from_system",
            "new_with_linear_clamp_sampler",
            "&resource_streamer_system_textures",
        ],
    );
    assert!(!fallback.contains("queue.write_texture("));
    let production_constructor = streamer
        .split("pub(crate) fn new_with_plugin_shading_models_and_shader_modules")
        .nth(1)
        .and_then(|source| source.split("fn new_with_shading_model_registry").next())
        .expect("production resource streamer constructor");
    assert!(!production_constructor.contains("queue: &wgpu::Queue"));
}

#[test]
fn runtime_90_neutral_fallback_family_has_one_generation_owner() {
    let resources = read_runtime_src(
        "graphics/backend/render_backend/system_texture_generation_owner/resources.rs",
    );
    let transmission = read_runtime_src(
        "graphics/scene/scene_renderer/advanced_lighting/transmission/resources.rs",
    );
    let irradiance = read_runtime_src(
        "graphics/scene/scene_renderer/advanced_lighting/irradiance_volume/resources.rs",
    );
    let lightmap =
        read_runtime_src("graphics/scene/scene_renderer/environment/lightmap_binding.rs");
    let post_process = read_runtime_src(
        "graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs",
    );
    let post_process_fallbacks = read_runtime_src(
        "graphics/scene/scene_renderer/post_process/resources/construct/create_fallback_texture_views/create.rs",
    );
    let post_process_black = read_runtime_src(
        "graphics/scene/scene_renderer/post_process/resources/construct/create_fallback_texture_views/black_texture_view.rs",
    );
    let post_process_white = read_runtime_src(
        "graphics/scene/scene_renderer/post_process/resources/construct/create_fallback_texture_views/white_texture_view.rs",
    );
    let post_process_hzb = read_runtime_src(
        "graphics/scene/scene_renderer/post_process/resources/construct/create_fallback_texture_views/hzb_source_texture_view.rs",
    );
    let post_process_effect = read_runtime_src(
        "graphics/scene/scene_renderer/post_process/resources/construct/create_fallback_texture_views/effect_lut_texture_view.rs",
    );
    let mesh =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs");

    assert_eq!(
        resources
            .matches("let black_rgba16float_texture = create_texture(")
            .count(),
        1
    );
    assert_contains_all(
        "one RGBA16F fallback texture publishes typed 2D and array views",
        &resources,
        &[
            "let black_rgba16float_view =",
            "let black_rgba16float_array_view =",
            "dimension: Some(wgpu::TextureViewDimension::D2Array)",
        ],
    );
    let post_process_fallback_family = format!(
        "{post_process_fallbacks}\n{post_process_black}\n{post_process_white}\n{post_process_hzb}\n{post_process_effect}"
    );
    assert_contains_all(
        "neutral fallback consumers project the generation lease",
        &format!("{transmission}\n{irradiance}\n{lightmap}\n{post_process_fallback_family}"),
        &[
            "SystemTextureGenerationLease",
            "black_rgba8_view()",
            "irradiance_volume_black_view()",
            "black_rgba16float_array_view()",
            "effect_lut_3d_view()",
        ],
    );
    for (label, consumer) in [
        ("transmission", transmission.as_str()),
        ("irradiance volume", irradiance.as_str()),
        ("lightmap", lightmap.as_str()),
        (
            "post-process fallbacks",
            post_process_fallback_family.as_str(),
        ),
    ] {
        assert_eq!(
            consumer.matches("queue.write_texture").count(),
            0,
            "{label} must not bypass the generation owner",
        );
    }
    assert!(!post_process.contains("queue: &wgpu::Queue"));
    assert!(mesh.contains("LightCookieAtlasResources::new(device, queue)"));
    assert_eq!(mesh.matches("new(device, queue)").count(), 1);
    assert!(resources.lines().count() <= 800);
}

#[test]
fn runtime_90_array_texture_copy_contract_reaches_wgpu_submission() {
    let region = read_repo("zircon_runtime/crates/zr_rhi/src/texture_copy/region.rs");
    let encoder = read_repo(
        "zircon_runtime/crates/zr_rhi_wgpu/src/production/command_encoder/texture_copy.rs",
    );
    let submission = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/production/submission.rs");

    assert_contains_all(
        "neutral array-layer copy contract",
        &region,
        &[
            "pub depth_or_array_layers: u32",
            "default = \"default_depth_or_array_layers\"",
            "pub fn with_depth_or_array_layers",
        ],
    );
    assert_eq!(
        submission
            .matches("depth_or_array_layers: region.depth_or_array_layers")
            .count(),
        2
    );
    assert_eq!(
        submission.matches("depth_or_array_layers: 1").count(),
        0,
        "WGPU upload submission must preserve the neutral array-layer extent"
    );
    assert_eq!(encoder.matches("wgpu::Extent3d {").count(), 3);
    assert_eq!(encoder.matches("depth_or_array_layers: 1").count(), 0);
    assert!(
        encoder.matches(".depth_or_array_layers,").count() >= 3,
        "every WGPU texture-copy extent must preserve the neutral depth"
    );
}

#[test]
fn runtime_90_reflection_probe_uploads_share_the_frame_resource_transaction() {
    let resources =
        read_runtime_src("graphics/scene/scene_renderer/environment/probe_buffer/resources.rs");
    let upload =
        read_runtime_src("graphics/scene/scene_renderer/environment/probe_buffer/upload.rs");
    let write_scene_uniform = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs",
    );
    let compiled_render = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs",
    );
    let direct_render = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs",
    );
    let compiled_submit = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs",
    );

    let append = upload
        .split("pub(super) fn append_probe_pmrem_texture_uploads")
        .nth(1)
        .and_then(|source| source.split("fn rgba16f_cube_mip_chain_len").next())
        .expect("probe PMREM upload batch builder");
    assert!(append.contains("let payload: Arc<[u8]> = Arc::from(bytes);"));
    assert_eq!(append.matches("for mip_level in").count(), 1);
    assert!(!append.contains("for face in"));
    assert_eq!(append.matches("WgpuTextureUpload::new(").count(), 1);
    assert!(append.contains("with_depth_or_array_layers(REFLECTION_PROBE_FACE_COUNT)"));
    assert!(!upload.contains("queue.write_texture"));

    assert_contains_all(
        "probe slot publication remains provisional until scene submission",
        &resources,
        &[
            "let prepare_epoch = self.begin_prepare_epoch();",
            "self.slots.available(cubemap, revision, prepare_epoch)",
            "self.pending_uploads.push(PendingReflectionProbeUpload",
            "self.discard_pending_uploads();",
            "pub(in crate::graphics::scene::scene_renderer) fn commit_pending_uploads",
        ],
    );
    assert!(!resources.contains("self.slots.clone()"));
    assert_contains_all(
        "probe uploads join both frame resource packets",
        &format!("{write_scene_uniform}\n{compiled_render}\n{direct_render}"),
        &[
            "frame_texture_uploads: &mut WgpuTextureUploadBatch",
            "&mut frame_texture_uploads,",
            "WgpuResourceUploadBatch::from_batches(",
        ],
    );
    let submit = compiled_submit
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("compiled scene submission");
    let commit = compiled_submit
        .find(".reflection_probes\n            .commit_pending_uploads()")
        .expect("compiled probe slot commit");
    assert!(submit < commit);
}
