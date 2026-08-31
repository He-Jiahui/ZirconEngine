use super::*;

const GLTF_ZERO_ROUGHNESS_MATERIAL_URI: &str =
    "res://materials/explicit_zero_roughness.gltf#Material0";
const GLTF_ZERO_ROUGHNESS_OUTPUT_NAME: &str =
    "runtime_shader_pbr_realtime_ibl_external_gltf_zero_roughness_mirror_20260824.png";
const GLTF_ZERO_ROUGHNESS_GPU_TIMING_REPORT_NAME: &str =
    "runtime_shader_pbr_realtime_ibl_external_gltf_zero_roughness_mirror_20260824_gpu_timing.txt";
const GLTF_ZERO_ROUGHNESS_CPU_TIMING_REPORT_NAME: &str =
    "runtime_shader_pbr_realtime_ibl_external_gltf_zero_roughness_mirror_20260824_cpu_timing.txt";
const RENDERDOC_CAPTURE_GLTF_ZERO_ROUGHNESS_ENV: &str =
    "ZR_RENDERDOC_CAPTURE_GLTF_ZERO_ROUGHNESS_FINAL_SH9";

#[test]
#[ignore = "manual WGPU product acceptance for Shader 06 external glTF roughnessFactor=0"]
fn export_realtime_ibl_external_gltf_zero_roughness_mirror_png() {
    let root = super::unique_temp_project_root("shader_pbr_realtime_ibl_gltf_zero_roughness");
    prepare_external_gltf_zero_roughness_project(
        &root,
        super::SinglePbrSphereCameraView::front(super::ProjectionMode::Perspective),
    );
    let scene_uri = super::AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml")
        .expect("external glTF mirror scene URI");
    let asset_manager = std::sync::Arc::new(super::ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .expect("open external glTF mirror project");
    let mut project =
        super::ProjectManager::open(&root).expect("open external glTF mirror manager");
    let imported = project
        .scan_and_import()
        .expect("import external glTF mirror project");
    let material_record = imported
        .iter()
        .find(|record| record.primary_locator().to_string() == GLTF_ZERO_ROUGHNESS_MATERIAL_URI)
        .expect("external glTF material must be imported");
    assert_eq!(
        material_record.state,
        super::ResourceState::Ready,
        "external glTF material import must publish a ready artifact: {:#?}",
        material_record.diagnostics
    );
    let material = match project
        .load_artifact(
            &super::AssetUri::parse(GLTF_ZERO_ROUGHNESS_MATERIAL_URI).expect("material URI"),
        )
        .expect("load imported external glTF material")
    {
        zircon_runtime::asset::ImportedAsset::Material(material) => material,
        other => {
            panic!("external glTF material URI must resolve a material artifact, got {other:?}")
        }
    };
    assert_eq!(
        material.shader,
        zircon_runtime::asset::assets::default_pbr_shader_reference(),
        "external glTF material must resolve the canonical compound default-PBR shader asset"
    );
    assert_eq!(material.metallic, 1.0);
    assert_eq!(
        material.roughness, 0.0,
        "the external glTF factor must remain zero until the GPU material roughness floor"
    );
    let world = zircon_runtime::scene::world::World::load_scene_from_uri(&project, &scene_uri)
        .expect("load external glTF mirror scene");
    let environment = super::directional_procedural_environment();
    let asset_runtime = super::support::ProjectAssetTestRuntime::new(asset_manager);
    let framework =
        super::WgpuRenderFramework::new(asset_runtime.access(), asset_runtime.worker_pool())
            .expect("create external glTF mirror framework");
    let viewport = framework
        .create_viewport(
            super::RenderViewportDescriptor::new(super::MULTI_VIEW_OUTPUT_SIZE)
                .with_label("shader06.realtime-ibl-external-gltf-zero-roughness"),
        )
        .expect("create external glTF mirror viewport");
    let cpu_timing_output =
        super::shader_test_output_dir().join(GLTF_ZERO_ROUGHNESS_CPU_TIMING_REPORT_NAME);
    assert!(cpu_timing_output.starts_with(super::shader_test_output_dir()));
    super::cpu_profile_capture::clear_current_cpu_timing_sidecar(&cpu_timing_output);
    let cpu_timing_feature_enabled = super::RealtimeIblCpuProfileCapture::feature_enabled();
    let mut cpu_timing_capture = super::RealtimeIblCpuProfileCapture::begin();
    let mut snapshot = world.build_viewport_render_packet(&super::SceneViewportExtractRequest {
        settings: super::ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: Some(super::realtime_mirror_camera_descriptor(
            super::SinglePbrSphereCameraView::front(super::ProjectionMode::Perspective),
            super::MULTI_VIEW_OUTPUT_SIZE,
        )),
        viewport_size: Some(super::MULTI_VIEW_OUTPUT_SIZE),
        virtual_geometry_debug: None,
    });
    snapshot.environment = environment;
    snapshot.preview = super::PreviewEnvironmentExtract::from_environment(
        &snapshot.environment,
        true,
        super::Vec4::ZERO,
    );
    snapshot.overlays = super::RenderOverlayExtract::default();

    let request_renderdoc_capture =
        std::env::var(RENDERDOC_CAPTURE_GLTF_ZERO_ROUGHNESS_ENV).is_ok_and(|value| value == "1");
    for slice_index in 0..super::REALTIME_GENERATION_TICKET_FRAME_COUNT {
        let capture_this_slice = request_renderdoc_capture
            && slice_index + 1 == super::REALTIME_GENERATION_TICKET_FRAME_COUNT;
        if capture_this_slice {
            framework
                .request_graphics_debugger_capture(viewport)
                .expect("request RenderDoc capture for external glTF final SH9 slice");
        }
        super::submit_compiled_realtime_ibl_frame(&framework, viewport, snapshot.clone());
        if capture_this_slice {
            let capture_status = framework
                .query_graphics_debugger_status()
                .expect("query external glTF final SH9 RenderDoc capture status");
            assert!(
                !capture_status.capture_pending,
                "external glTF final SH9 RenderDoc capture must complete in its requested frame"
            );
            assert_eq!(
                capture_status.last_error, None,
                "external glTF final SH9 RenderDoc capture must complete without a debugger error"
            );
        }
    }

    let frame = super::capture_compiled_viewport_frame(&framework, viewport);
    assert!(
        framework.realtime_ibl_gpu_timing_supported(),
        "external glTF product fixture must expose compiled realtime IBL timestamp queries"
    );
    let gpu_timings = framework
        .take_realtime_ibl_gpu_timing_reports()
        .expect("drain external glTF realtime IBL GPU timing reports");
    super::assert_realtime_gpu_timings(&gpu_timings, 1);
    super::assert_realtime_capture_and_source_mip_binding_metrics(&gpu_timings, 1);
    let cpu_timings = if cpu_timing_capture.has_owned_capture() {
        cpu_timing_capture.stop();
        framework
            .take_realtime_ibl_cpu_timing_reports()
            .expect("drain external glTF realtime IBL CPU timing reports")
    } else if !cpu_timing_feature_enabled {
        framework
            .take_realtime_ibl_cpu_timing_reports()
            .expect("drain disabled external glTF realtime IBL CPU timing reports")
    } else {
        Vec::new()
    };
    if cpu_timing_capture.has_owned_capture() {
        super::assert_realtime_cpu_timings(&cpu_timings, 1);
    } else if !cpu_timing_feature_enabled {
        assert!(
            cpu_timings.is_empty(),
            "external glTF CPU timing reports require the profiling feature capture"
        );
    }
    super::assert_realtime_mirror_view(&frame, "external glTF roughnessFactor=0 mirror");
    super::assert_directional_procedural_mirror_highlight(
        &frame,
        "external glTF roughnessFactor=0 mirror",
    );
    let gpu_timing_output =
        super::shader_test_output_dir().join(GLTF_ZERO_ROUGHNESS_GPU_TIMING_REPORT_NAME);
    std::fs::write(&gpu_timing_output, super::gpu_timing_report(&gpu_timings))
        .expect("write external glTF realtime IBL GPU timing report");
    if cpu_timing_capture.has_owned_capture() {
        std::fs::write(&cpu_timing_output, super::cpu_timing_report(&cpu_timings))
            .expect("write external glTF realtime IBL CPU timing report");
    }
    let output = super::shader_test_output_dir().join(GLTF_ZERO_ROUGHNESS_OUTPUT_NAME);
    super::save_viewport_frame_png(&frame, &output);
    assert!(output.starts_with(super::shader_test_output_dir()));
    assert!(gpu_timing_output.starts_with(super::shader_test_output_dir()));
    let _ = std::fs::remove_dir_all(root);
}

fn prepare_external_gltf_zero_roughness_project(
    root: &std::path::Path,
    camera_view: super::SinglePbrSphereCameraView,
) -> super::ProjectPaths {
    let paths = super::ProjectPaths::from_root(root).expect("external glTF project paths");
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .expect("create external glTF project layout");
    let scene_uri = super::AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml")
        .expect("external glTF scene URI");
    super::ProjectManifest::new("GraphicsPbrRealtimeIblExternalGltf", scene_uri, 1)
        .save(paths.manifest_path())
        .expect("save external glTF project manifest");
    let asset_root =
        paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    super::write_uv_sphere_model(
        asset_root
            .join("models")
            .join("single_pbr_sphere.model.toml"),
        "res://models/single_pbr_sphere.model.toml",
        48,
        96,
    );
    write_project_default_pbr_shader(&asset_root);
    write_external_gltf_zero_roughness_material(
        asset_root
            .join("materials")
            .join("explicit_zero_roughness.gltf"),
    );
    super::scene_fixtures::write_single_pbr_sphere_scene_with_camera_view_and_material(
        asset_root
            .join("scenes")
            .join("single_pbr_sphere.scene.toml"),
        camera_view,
        GLTF_ZERO_ROUGHNESS_MATERIAL_URI,
    );
    paths
}

fn write_project_default_pbr_shader(asset_root: &std::path::Path) {
    let shader_uri = super::AssetUri::parse(zircon_runtime::asset::assets::DEFAULT_PBR_SHADER_URI)
        .expect("external glTF default PBR shader URI");
    let mut meta = zircon_runtime::asset::project::AssetMetaDocument::new(
        zircon_runtime::asset::AssetUuid::from_stable_label(
            "shader06/external-gltf-zero-roughness/default-pbr",
        ),
        shader_uri,
        zircon_runtime::asset::AssetKind::Shader,
    );
    meta.unit = zircon_runtime::asset::project::AssetSourceUnit::Compound;
    meta.save(asset_root.join("shaders").join("default_pbr.zmeta"))
        .expect("write external glTF default PBR shader metadata");
    let package_directory = asset_root.join("shaders").join("default_pbr");
    std::fs::create_dir_all(&package_directory)
        .expect("create external glTF default PBR shader package");
    std::fs::write(
        package_directory.join("default_pbr.zshader"),
        r#"kind = "surface"
version = 2
name = "Shader06 External glTF Default PBR"
shading_model = "standard_pbr"
wgsl_files = ["default_pbr.wgsl"]

[[properties]]
name = "base_color"
kind = "vec4"
default = [1.0, 1.0, 1.0, 1.0]

[[properties]]
name = "metallic"
kind = "float"
default = 0.0

[[properties]]
name = "roughness"
kind = "float"
default = 1.0

[[properties]]
name = "emissive"
kind = "vec3"
default = [0.0, 0.0, 0.0]

[[texture_slots]]
name = "base_color"
kind = "texture_2d"
default = "white"

[[texture_slots]]
name = "metallic_roughness"
kind = "texture_2d"
default = "white"

[[texture_slots]]
name = "occlusion"
kind = "texture_2d"
default = "white"

[[texture_slots]]
name = "emissive"
kind = "texture_2d"
default = "black"
"#,
    )
    .expect("write external glTF default PBR shader descriptor");
    std::fs::write(
        package_directory.join("default_pbr.wgsl"),
        r#"fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let base_sample = zr_sample_base_color(input.uv0);
    let metallic_roughness = zr_sample_metallic_roughness(input.uv0).rgb;
    let occlusion_sample = zr_sample_occlusion(input.uv0).r;
    let emissive_sample = zr_sample_emissive(input.uv0).rgb;

    var surface = zr_surface_from_base_color(
        zr_mat_base_color() * base_sample * input.tint * input.color,
    );
    surface.normal_ws = zr_normalize_or_zero(input.normal_ws);
    surface.metallic = clamp(zr_mat_metallic() * metallic_roughness.b, 0.0, 1.0);
    surface.roughness = clamp(zr_mat_roughness() * metallic_roughness.g, 0.001, 1.0);
    surface.occlusion = clamp(occlusion_sample, 0.0, 1.0);
    surface.emissive = max(zr_mat_emissive(), vec3<f32>(0.0)) * emissive_sample;
    surface.shading_model_id = 2u;
    return surface;
}
"#,
    )
    .expect("write external glTF default PBR shader source");
}

fn write_external_gltf_zero_roughness_material(path: std::path::PathBuf) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create external glTF material directory");
    }
    std::fs::write(
        path,
        r#"{
  "asset": { "version": "2.0" },
  "materials": [
    {
      "name": "External Zero Roughness Mirror",
      "pbrMetallicRoughness": {
        "baseColorFactor": [1.0, 1.0, 1.0, 1.0],
        "metallicFactor": 1.0,
        "roughnessFactor": 0.0
      }
    }
  ]
}"#,
    )
    .expect("write external glTF zero roughness material");
}

#[test]
fn external_gltf_zero_roughness_export_persists_timestamp_evidence_below_shader_root() {
    let source = include_str!("gltf_zero_roughness.rs");

    assert!(source.contains("GLTF_ZERO_ROUGHNESS_GPU_TIMING_REPORT_NAME"));
    assert!(source.contains("GLTF_ZERO_ROUGHNESS_CPU_TIMING_REPORT_NAME"));
    assert!(source.contains("super::gpu_timing_report(&gpu_timings)"));
    assert!(source.contains("super::cpu_timing_report(&cpu_timings)"));
    assert!(source.contains("super::cpu_profile_capture::clear_current_cpu_timing_sidecar"));
    assert!(source.contains("write external glTF realtime IBL GPU timing report"));
    assert!(source.contains("write external glTF realtime IBL CPU timing report"));
    assert!(source.contains("gpu_timing_output.starts_with(super::shader_test_output_dir())"));
}
