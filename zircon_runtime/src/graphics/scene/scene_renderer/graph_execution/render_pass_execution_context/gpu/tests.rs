#[test]
fn render_pass_device_epoch_is_typed_and_derived_from_materialized_resources() {
    let source = include_str!("../gpu.rs");
    let epoch = include_str!("../../render_pass_device_epoch.rs");

    assert!(epoch.contains("pub struct RenderPassDeviceEpoch"));
    assert!(epoch.contains("#[derive(Clone, Copy, Debug, PartialEq, Eq)]"));
    assert!(epoch.contains("device_id: u64"));
    assert!(epoch.contains("generation: u64"));
    assert!(epoch.contains("pub(crate) const fn raw_parts(self)"));
    assert!(source.contains("pub fn device_epoch(&self) -> Option<RenderPassDeviceEpoch>"));
    assert!(source.contains("self.resources"));
    assert!(source.contains(".device_epoch()"));
    assert!(!epoch.contains("pub device_id: u64"));
    assert!(!epoch.contains("pub generation: u64"));
}

#[test]
fn render_pass_device_epoch_equality_covers_device_and_generation() {
    let baseline = super::RenderPassDeviceEpoch::new(7, 11);

    assert_eq!(baseline, super::RenderPassDeviceEpoch::new(7, 11));
    assert_ne!(baseline, super::RenderPassDeviceEpoch::new(8, 11));
    assert_ne!(baseline, super::RenderPassDeviceEpoch::new(7, 12));
}

#[test]
fn native_gpu_handles_use_an_explicit_short_lived_capability() {
    let source = include_str!("../gpu.rs");
    let native = include_str!("native.rs");

    for field in ["encoder", "scene_bind_group", "scene_bind_group_layout"] {
        assert!(native.contains(&format!("pub {field}:")));
    }
    assert!(native.contains("device: &'a wgpu::Device"));
    assert!(!native.contains("pub device: &'a wgpu::Device"));
    assert!(native.contains("impl RenderPassGpuResourceFactory for RenderPassGpuNativeContext"));
    assert!(source.contains("impl RenderPassGpuResourceFactory for RenderPassGpuExecutionContext"));
    assert!(native.contains("fn create_buffer_init("));
    assert!(native.contains("fn create_bind_group("));
    assert!(native.contains("fn create_bind_group_layout("));
    assert!(native.contains("fn create_shader_module("));
    assert!(native.contains("fn create_pipeline_layout("));
    assert!(native.contains("fn create_compute_pipeline("));
    assert!(native.contains("fn create_render_pipeline("));
    assert!(source.contains("pub fn native_context(&mut self) -> RenderPassGpuNativeContext"));
    assert!(
        source.contains("native_resource_creates: Cell<RenderPassNativeResourceCreateMetrics>")
    );
    assert!(source.contains("take_native_resource_creates"));
    assert!(source.contains("pub(in crate::graphics::scene::scene_renderer) device:"));
    assert!(source.contains("pub(in crate::graphics::scene::scene_renderer) encoder:"));
    assert!(source.contains("pub(in crate::graphics::scene::scene_renderer) scene_bind_group:"));
    assert!(source.contains("pub(in crate::graphics::scene::scene_renderer) resources:"));
    assert!(source.contains("pub(in crate::graphics::scene::scene_renderer) plugin_outputs:"));
    let lookup = include_str!("resource_lookup.rs");
    assert!(lookup.contains(") -> Result<&'a wgpu::TextureView, String>"));
    assert!(lookup.contains(") -> Result<Option<&'a wgpu::TextureView>, String>"));
    assert!(lookup.contains(") -> Result<&'a wgpu::Buffer, String>"));
    assert!(!source.contains("    pub device: &'a wgpu::Device"));
    assert!(!source.contains("    pub encoder: &'a mut wgpu::CommandEncoder"));
    assert!(!source.contains("    pub scene_bind_group: &'a wgpu::BindGroup"));
    assert!(!source.contains("    pub resources: &'a RenderGraphExecutionResources"));
    assert!(!source.contains("    pub plugin_outputs: &'a mut RenderPluginRendererOutputs"));
}

#[test]
fn depth_prepass_binds_forward_shadow_receiver_layout_slot() {
    let source = include_str!("mesh_recording.rs");

    assert!(source.contains("record_depth_prepass_to_resources"));
    assert!(source.contains("create_forward_shadow_receiver_bind_group"));
    assert!(source.contains("bind_forward_shadow_receiver_if_needed"));
}

#[test]
fn deferred_lighting_uses_compiler_proven_buffer_binding_windows() {
    let lookup = include_str!("resource_lookup.rs");
    let deferred = include_str!("deferred.rs");
    let lighting = include_str!("../../../deferred/deferred_scene_resources/execute_lighting.rs");

    assert!(lookup.contains("require_buffer_binding_by_name"));
    assert!(lookup.contains("transient_buffer_binding_for_access(access_id)"));
    assert!(lookup.contains("offset: range.start"));
    assert!(lookup.contains("size: Some(size)"));
    assert!(deferred.contains("Self::require_buffer_binding_by_name("));
    assert!(lighting.contains("wgpu::BufferBinding<'_>"));
    assert!(lighting.contains("wgpu::BindingResource::Buffer(light_grid_params_buffer)"));
    assert!(!deferred.contains("Self::require_buffer_by_name(\n            resources,\n            resource_resolver,\n            PostProcessGraphResourceNames::LIGHT_GRID_PARAMS"));
}

#[test]
fn gpu_resolver_routes_typed_external_resources_through_access_leases() {
    let lookup = include_str!("resource_lookup.rs");
    let resolver = include_str!("../resource_resolver.rs");

    assert!(resolver.contains("exact_external_access_by_name"));
    assert!(lookup.contains("external_buffer_binding_for_access(access_id)"));
    assert!(lookup.contains("external_texture_desc_for_access(access_id)"));
    assert!(lookup.contains("optional_external_texture_view_for_access(access_id)"));
    assert!(lookup.contains("optional_external_buffer_binding_for_access(access_id)"));
}

#[test]
fn ibl_graph_storage_buffer_output_uses_exact_binding_window() {
    let dispatch = include_str!("../../../environment/ibl_bake_wgpu_dispatch.rs");
    let binding = include_str!("../../../environment/ibl_bake_wgpu_binding.rs");
    let readback = include_str!("../../../environment/ibl_bake_wgpu_readback.rs");
    let staging = include_str!(
        "../../../../../backend/render_backend/read_ibl_bake_artifact_sections/staging.rs"
    );
    let backend_readback =
        include_str!("../../../../../backend/render_backend/read_ibl_bake_artifact_sections.rs");

    assert!(dispatch.contains("gpu.require_buffer_binding("));
    assert!(dispatch.contains("StorageBufferRange"));
    assert!(binding.contains("StorageBufferRange"));
    assert!(binding.contains("offset: *offset"));
    assert!(binding.contains("size: *size"));
    assert!(readback.contains("required_graph_buffer_binding"));
    assert!(readback.contains("required_graph_texture"));
    assert!(readback.contains("graph: &CompiledRenderGraph"));
    assert!(readback.contains("with_irradiance_sh9_buffer_range"));
    assert!(readback.contains("range.start, size"));
    assert!(readback.contains("transient_buffer_binding_for_access(access_id)"));
    assert!(readback.contains("external_buffer_binding_for_access(access_id)"));
    assert!(readback.contains("transient_texture_for_access(access_id)"));
    assert!(staging.contains("copy_buffer_to_buffer(source, source_offset"));
    assert!(backend_readback.contains("buffer.buffer"));
    assert!(backend_readback.contains("buffer.offset"));
    assert!(backend_readback.contains("buffer.size"));
}

#[test]
fn post_process_cluster_binding_preserves_compiler_proven_buffer_windows() {
    let post_process = include_str!("post_process.rs");
    let computed_resources = include_str!("post_process/computed_resources.rs");
    let screen_space_reflection = include_str!("post_process/screen_space_reflection.rs");
    let execute =
        include_str!("../../../post_process/resources/execute_post_process/execute/run/execute.rs");
    let bind_group_entries = include_str!(
        "../../../post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs"
    );
    let clustered_lighting = include_str!(
        "../../../post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs"
    );
    let light_grid_pass = include_str!("../../../lighting/light_grid_pass.rs");
    let froxel = include_str!("../../../advanced_lighting/froxel/light_scatter.rs");
    let froxel_executor =
        include_str!("../../../advanced_lighting/froxel/executors/light_scatter.rs");

    assert!(post_process.contains("Self::require_buffer_binding_by_name("));
    assert!(post_process.contains("PostProcessGraphResourceNames::LIGHT_LIST"));
    assert!(computed_resources.contains("Self::require_buffer_binding_by_name("));
    assert!(computed_resources.contains("light_list_resource_name"));
    assert!(computed_resources.contains("prepare_light_grid_buffer_uploads("));
    assert!(computed_resources.contains("light_grid_params_buffer,"));
    assert!(computed_resources.contains("light_zbins_buffer,"));
    assert!(computed_resources.contains("light_tile_masks_buffer,"));
    assert_eq!(
        screen_space_reflection
            .matches("Self::require_buffer_binding_by_name(")
            .count(),
        4
    );
    assert!(execute.contains("cluster_buffer: wgpu::BufferBinding<'_>"));
    assert!(bind_group_entries.contains("cluster_buffer: wgpu::BufferBinding<'a>"));
    assert!(bind_group_entries.contains("wgpu::BindingResource::Buffer(cluster_buffer)"));
    assert!(clustered_lighting.contains("cluster_buffer: wgpu::BufferBinding<'_>"));
    assert!(clustered_lighting.contains("cluster_buffer.buffer"));
    assert!(clustered_lighting.contains("wgpu::BindingResource::Buffer(cluster_buffer)"));
    assert!(light_grid_pass.contains("BufferBinding<'_>"));
    assert!(light_grid_pass.contains("binding.offset"));
    assert!(froxel.contains("light_grid_params_buffer: wgpu::BufferBinding<'a>"));
    assert!(froxel.contains("wgpu::BindingResource::Buffer(request.light_grid_params_buffer)"));
    assert!(froxel.contains("wgpu::BindingResource::Buffer(request.light_zbins_buffer)"));
    assert!(froxel.contains("wgpu::BindingResource::Buffer(request.light_tile_masks_buffer)"));
    assert!(!froxel.contains("request.light_grid_params_buffer.as_entire_binding()"));
    assert!(froxel_executor.contains("require_buffer_binding("));
    assert!(froxel_executor.contains("get_or_try_insert_with(device_epoch"));
    assert!(!froxel_executor.contains("pipeline.as_ref().unwrap()"));
    let froxel_integrate_executor =
        include_str!("../../../advanced_lighting/froxel/executors/integrate.rs");
    let froxel_media_executor =
        include_str!("../../../advanced_lighting/froxel/executors/media_inject.rs");
    assert!(froxel_integrate_executor.contains("get_or_try_insert_with(device_epoch"));
    assert!(froxel_media_executor.contains("get_or_try_insert_with(device_epoch"));
    assert!(!froxel_integrate_executor.contains("pipeline.as_ref().unwrap()"));
    assert!(!froxel_media_executor.contains("pipeline.as_ref().unwrap()"));
    assert!(!froxel.contains("[..3].try_into().unwrap()"));
    let planar_filter_executor =
        include_str!("../../../advanced_lighting/planar_filter/executor.rs");
    assert!(planar_filter_executor.contains("get_or_try_insert_with(device_epoch"));
    assert!(!planar_filter_executor.contains("pipeline.as_ref().unwrap()"));
    let subsurface_executors =
        include_str!("../../../advanced_lighting/subsurface_pass/executors.rs");
    assert!(subsurface_executors.contains("get_or_try_insert_with("));
    assert!(!subsurface_executors.contains("pipelines.as_ref().unwrap()"));
    let subsurface_prepared =
        include_str!("../../../advanced_lighting/subsurface_pass/prepared_frame.rs");
    assert!(subsurface_prepared.contains("u32::try_from(table.profiles.len())"));
    assert!(!subsurface_prepared.contains("expect(\"SSS upload payload size must fit usize\")"));
    let subsurface_pipelines =
        include_str!("../../../advanced_lighting/subsurface_pass/pipelines.rs");
    assert!(subsurface_pipelines.contains("tile_list: wgpu::BufferBinding<'_>"));
    assert!(subsurface_pipelines.contains("indirect_args: wgpu::BufferBinding<'_>"));
    assert!(subsurface_pipelines.contains("dispatch_workgroups_indirect"));
    assert!(subsurface_pipelines.contains("indirect_args.buffer"));
    assert!(subsurface_pipelines.contains("indirect_args.offset"));
    assert!(subsurface_pipelines.contains("indirect_args.size.map(NonZeroU64::get)"));
    assert!(subsurface_pipelines.contains("wgpu::BindingResource::Buffer(buffer)"));
    assert!(!subsurface_pipelines.contains("buffer.as_entire_binding()"));
    let irradiance_resources =
        include_str!("../../../advanced_lighting/irradiance_volume/resources.rs");
    assert!(irradiance_resources.contains("-> Result<(), String>"));
    assert!(irradiance_resources.contains("WgpuBufferUpload::new("));
    assert!(
        !irradiance_resources.contains(
            "expect(\"irradiance volume params upload must reference its packed payload\")"
        )
    );
    assert!(!post_process.contains(
        "Self::require_buffer_by_name(\n            resources,\n            resource_resolver,\n            PostProcessGraphResourceNames::LIGHT_LIST"
    ));
}

#[test]
fn core_advanced_pipeline_caches_are_device_epoch_qualified() {
    let cache = include_str!("../../render_pass_device_epoch_cache.rs");
    let froxel_integrate = include_str!("../../../advanced_lighting/froxel/executors/integrate.rs");
    let froxel_scatter =
        include_str!("../../../advanced_lighting/froxel/executors/light_scatter.rs");
    let froxel_media = include_str!("../../../advanced_lighting/froxel/executors/media_inject.rs");
    let oit = include_str!("../../../advanced_lighting/oit_buffers/executors.rs");
    let planar = include_str!("../../../advanced_lighting/planar_filter/executor.rs");
    let subsurface = include_str!("../../../advanced_lighting/subsurface_pass/executors.rs");
    let assert_epoch_before = |source: &str, owner: &str, first_work: &str| {
        let owner_start = source
            .find(owner)
            .unwrap_or_else(|| panic!("missing executor owner `{owner}`"));
        let owner_source = &source[owner_start..];
        let epoch_gate = owner_source
            .find("gpu.device_epoch()")
            .unwrap_or_else(|| panic!("`{owner}` must require a device epoch"));
        let work = owner_source
            .find(first_work)
            .unwrap_or_else(|| panic!("`{owner}` must contain `{first_work}`"));
        assert!(
            epoch_gate < work,
            "`{owner}` must gate work on device epoch"
        );
    };

    assert!(cache.contains("struct RenderPassDeviceEpochCacheEntry<K, V>"));
    assert!(cache.contains("device_epoch: RenderPassDeviceEpoch"));
    assert!(cache.contains("key: K"));
    assert!(cache.contains("entry.device_epoch == device_epoch && entry.key == key"));
    let release = cache
        .find("drop(self.entry.take())")
        .expect("cache replacement must release the old entry");
    let create = cache
        .find("let value = create()?")
        .expect("cache replacement must construct a new value");
    let publish = cache
        .find("self.entry.insert(RenderPassDeviceEpochCacheEntry")
        .expect("cache replacement must publish the new entry");
    assert!(release < create && create < publish);

    for source in [froxel_integrate, froxel_scatter, froxel_media, planar] {
        assert!(source.contains("RenderPassDeviceEpochCache<"));
        assert_eq!(source.matches("gpu.device_epoch()").count(), 1);
        assert!(source.contains("get_or_try_insert_with(device_epoch"));
        assert!(!source.contains("Mutex<Option<"));
    }
    assert_epoch_before(
        froxel_integrate,
        "impl RenderPassExecutor for VolumetricIntegrateExecutor",
        "gpu.frame_extract()",
    );
    assert_epoch_before(
        froxel_scatter,
        "impl RenderPassExecutor for VolumetricLightScatterExecutor",
        "gpu.frame_extract()",
    );
    assert_epoch_before(
        froxel_media,
        "impl RenderPassExecutor for VolumetricMediaInjectExecutor",
        "gpu.frame_extract()",
    );
    assert_epoch_before(
        planar,
        "impl RenderPassExecutor for PlanarReflectionFilterExecutor",
        "gpu.frame_extract()",
    );
    assert_eq!(oit.matches("RenderPassDeviceEpochCache<").count(), 2);
    assert_eq!(oit.matches("gpu.device_epoch()").count(), 2);
    assert_eq!(oit.matches("get_or_try_insert_with(").count(), 2);
    assert!(!oit.contains("Mutex<Option<"));
    let fragment_admission = oit
        .find("pipeline_cache.get_or_try_insert_with(device_epoch, depth_format")
        .expect("OIT fragment cache must be admitted for the current epoch");
    let fragment_clear = oit
        .find("gpu.encoder.clear_buffer(")
        .expect("OIT fragment pass must clear its counter buffer");
    assert!(fragment_admission < fragment_clear);
    assert_epoch_before(
        oit,
        "impl RenderPassExecutor for OitFragmentStoreExecutor",
        ".require_buffer_binding(",
    );
    assert_epoch_before(
        oit,
        "impl RenderPassExecutor for OitResolveExecutor",
        ".frame_extract()",
    );

    assert_eq!(subsurface.matches("RenderPassDeviceEpochCache<").count(), 1);
    assert_eq!(subsurface.matches("gpu.device_epoch()").count(), 3);
    assert_eq!(subsurface.matches("get_or_try_insert_with(").count(), 1);
    assert!(!subsurface.contains("Mutex<Option<"));
    assert_epoch_before(
        subsurface,
        "impl RenderPassExecutor for SetupExecutor",
        "texture(gpu",
    );
    assert_epoch_before(
        subsurface,
        "impl RenderPassExecutor for ScatterExecutor",
        "texture(gpu",
    );
    assert_epoch_before(
        subsurface,
        "impl RenderPassExecutor for RecombineExecutor",
        "texture(gpu",
    );
}

#[test]
fn exposure_and_color_lut_consumers_preserve_compiler_buffer_windows() {
    let computed_resources = include_str!("post_process/computed_resources.rs");
    let exposure =
        include_str!("../../../post_process/resources/execute_exposure/execute_exposure.rs");
    let color_lut = include_str!("../../../post_process/resources/execute_color_lut_bake/mod.rs");
    let bind_group_entries = include_str!(
        "../../../post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs"
    );
    let effects = include_str!("post_process/effects.rs");

    assert!(computed_resources.contains("Self::require_buffer_binding_by_name("));
    assert!(computed_resources.contains("Self::optional_buffer_binding_by_name("));
    assert!(computed_resources.contains("default_exposure_buffer_binding()"));
    assert!(exposure.contains("histogram_buffer: wgpu::BufferBinding<'_>"));
    assert!(exposure.contains("previous_exposure_buffer: wgpu::BufferBinding<'_>"));
    assert!(exposure.contains("current_exposure_buffer: wgpu::BufferBinding<'_>"));
    assert!(exposure.contains("histogram_buffer.size.map"));
    assert!(exposure.contains("BindingResource::Buffer(histogram_buffer)"));
    assert!(color_lut.contains("exposure_buffer: wgpu::BufferBinding<'_>"));
    assert!(color_lut.contains("BindingResource::Buffer(exposure_buffer)"));
    assert!(bind_group_entries.contains("exposure_buffer: wgpu::BufferBinding<'a>"));
    assert!(bind_group_entries.contains("BindingResource::Buffer(exposure_buffer)"));
    assert_eq!(
        effects
            .matches("Self::optional_buffer_binding_by_name(")
            .count(),
        4
    );
    assert!(!exposure.contains("histogram_buffer.as_entire_binding()"));
    assert!(!exposure.contains("previous_exposure_buffer.as_entire_binding()"));
    assert!(!exposure.contains("current_exposure_buffer.as_entire_binding()"));
    assert!(!color_lut.contains("exposure_buffer.as_entire_binding()"));
}

#[test]
fn forward_mesh_consumers_preserve_compiler_proven_light_grid_windows() {
    let lookup = include_str!("resource_lookup.rs");
    let mesh_recording = include_str!("mesh_recording.rs");
    let oit = include_str!("oit.rs");
    let receiver = include_str!("../../../mesh/mesh_pipeline_cache/forward_shadow_receiver.rs");
    let base_scene = include_str!("../../../overlay/passes/base_scene_pass.rs");

    assert!(lookup.contains("optional_buffer_binding_by_name"));
    assert_eq!(
        mesh_recording
            .matches("Self::optional_buffer_binding_by_name(")
            .count(),
        3
    );
    assert_eq!(
        oit.matches("Self::optional_buffer_binding_by_name(")
            .count(),
        3
    );
    assert!(receiver.contains("light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>"));
    assert!(receiver.contains("wgpu::BindingResource::Buffer(light_grid_params_buffer"));
    assert!(receiver.contains("offset: 0"));
    assert!(base_scene.contains("light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>"));
    let base_production = base_scene
        .split_once("#[cfg(test)]")
        .map_or(base_scene, |(head, _)| head);
    assert!(
        !base_production.contains("generic Base commands require a forward receiver bind group")
    );
}

#[test]
fn oit_buffers_preserve_compiler_proven_binding_windows() {
    let executors = include_str!("../../../advanced_lighting/oit_buffers/executors.rs");
    let fragment_store =
        include_str!("../../../advanced_lighting/oit_buffers/fragment_store_pipeline.rs");
    let resolve = include_str!("../../../advanced_lighting/oit_buffers/resolve_pipeline.rs");
    let oit_context = include_str!("oit.rs");

    assert_eq!(executors.matches("require_buffer_binding(").count(), 4);
    assert!(executors.contains("counts.buffer"));
    assert!(executors.contains("counts.size.map(std::num::NonZeroU64::get)"));
    assert!(executors.contains("binding.size.map_or_else"));
    assert!(executors.contains("binding.buffer.size().saturating_sub(binding.offset)"));
    let production = executors
        .split_once("#[cfg(test)]")
        .map_or(executors, |(head, _)| head);
    assert!(!production.contains("pipeline.as_ref().unwrap()"));
    assert!(fragment_store.contains("oit_layout"));
    assert!(resolve.contains("layers: wgpu::BufferBinding<'_>"));
    assert!(resolve.contains("wgpu::BindingResource::Buffer(layers)"));
    assert!(resolve.contains("wgpu::BindingResource::Buffer(counts)"));
    assert!(oit_context.contains("layers: &wgpu::BufferBinding<'_>"));
    assert!(oit_context.contains("wgpu::BindingResource::Buffer(layers.clone())"));
    assert!(oit_context.contains("wgpu::BindingResource::Buffer(counts.clone())"));
}

#[test]
fn disabled_forward_volumetric_params_buffer_is_cache_owned() {
    let cache_source = include_str!("../../../mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs");
    let construct_source = include_str!("../../../mesh/mesh_pipeline_cache/construct.rs");
    let binding_source =
        include_str!("../../../mesh/mesh_pipeline_cache/forward_shadow_receiver.rs");
    let cache_field = cache_source
        .lines()
        .zip(cache_source.lines().skip(1))
        .any(|(field, ty)| {
            field
                .trim_end()
                .ends_with("forward_volumetric_disabled_params_buffer:")
                && ty.trim() == "wgpu::Buffer,"
        });

    assert!(cache_field);
    assert!(construct_source.contains("let forward_volumetric_disabled_params_buffer ="));
    assert!(construct_source.contains("forward_volumetric_disabled_params_buffer,"));
    assert!(binding_source.contains("&self.forward_volumetric_disabled_params_buffer"));
    assert!(!binding_source.contains("create_disabled_params_buffer("));
}

#[test]
fn taa_reactive_mask_uses_one_clear_and_draw_pass_only_when_commands_exist() {
    let mesh_recording = include_str!("mesh_recording.rs");
    let reactive_mask_start = mesh_recording
        .find("fn record_taa_reactive_mask_mesh_to_resource")
        .expect("reactive-mask mesh recording must exist");
    let reactive_mask_end = mesh_recording[reactive_mask_start..]
        .find("\nfn mesh_stage_attachment_ops")
        .map(|offset| reactive_mask_start + offset)
        .expect("reactive-mask mesh recording must end before mesh-stage helpers");
    let reactive_mask_recording = &mesh_recording[reactive_mask_start..reactive_mask_end];
    let empty_stream = reactive_mask_recording
        .find("if stream.is_empty()")
        .expect("reactive-mask recording must skip empty streams");
    let begin_pass = reactive_mask_recording
        .find("begin_render_pass")
        .expect("non-empty reactive-mask streams must record one pass");

    assert!(empty_stream < begin_pass);
    assert!(reactive_mask_recording.contains("RenderGraphAttachmentOps::clear_store()"));
    assert!(reactive_mask_recording.contains("drop(pass);"));
    assert!(reactive_mask_recording.contains("record_taa_reactive_mask_encoding"));

    let resource_binding = include_str!(
        "../../../core/scene_renderer_core_render_compiled_scene/render/bind_taa_reactive_mask_graph_resource.rs"
    );
    assert!(resource_binding.contains("taa_reactive_mask_stream().is_empty()"));
    assert!(resource_binding.contains("black_texture_view()"));
    assert!(
        !mesh_recording.contains("record_taa_reactive_mask_clear_to_resource"),
        "the mesh writer owns the single clear-and-draw pass"
    );
}

#[test]
fn physical_output_region_is_reserved_for_present_targets_only() {
    assert!(super::writes_physical_output_resource(
        crate::core::framework::render::PostProcessGraphResourceNames::FINAL_COLOR
    ));
    assert!(super::writes_physical_output_resource(
        crate::core::framework::render::PostProcessGraphResourceNames::VIEWPORT_OUTPUT
    ));
    assert!(!super::writes_physical_output_resource(
        crate::core::framework::render::PostProcessGraphResourceNames::FINAL_COMPOSITED
    ));
    assert!(!super::writes_physical_output_resource(
        crate::core::framework::render::PostProcessGraphResourceNames::COLOR_GRADED
    ));
}

#[test]
fn preview_sky_renderer_context_is_fallible() {
    let source = include_str!("surface.rs");

    assert!(source.contains(
        "preview sky graph executor for pass `{pass_name}` requires preview sky renderer context"
    ));
    assert!(
        !source.contains("preview sky renderer context was checked before resource resolution")
    );
}
