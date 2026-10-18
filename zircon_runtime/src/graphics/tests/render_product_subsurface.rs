use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::assets::{
    AlphaMode, MaterialAsset, ModelAsset, ModelPrimitiveAsset, ShaderAsset,
};
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetReference, AssetUri, MeshVertex};
use crate::core::framework::render::{
    CapturedFrame, CorePipelineKind, DisplayMode, EnvironmentExtract, GBufferChannelMask,
    PreviewEnvironmentExtract, ProjectionMode, RenderDirectionalLightSnapshot, RenderFrameExtract,
    RenderFramework, RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState,
    RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderStats, RenderViewportDescriptor, RenderWorldSnapshotHandle, ShadingModelDescriptor,
    ShadingModelId, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::{
    subsurface_render_feature_descriptor, subsurface_render_pass_executor_registrations,
    WgpuRenderFramework, SSS_RECOMBINE_EXECUTOR_ID, SSS_SCATTER_EXECUTOR_ID, SSS_SETUP_EXECUTOR_ID,
};

use crate::graphics::{
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
    RenderPassGpuExecutionContext, RenderPassGpuResourceFactory, RenderPassRecordingPolicy,
};
use crate::render_graph::RenderGraphResourceAccessKind;

const PRODUCT_SIZE: UVec2 = UVec2::new(640, 360);
const DEFERRED_PIPELINE_HANDLE: RenderPipelineHandle = RenderPipelineHandle::new(2);
const PRODUCT_IMAGE_NAME: &str = "plan18_sss_skin_sphere_deferred_burley_wgpu_20260712.png";
const PRODUCT_REPORT_NAME: &str = "plan18_sss_skin_sphere_deferred_burley_wgpu_20260712.txt";
const SSS_SHADING_MODEL_ID: ShadingModelId = ShadingModelId::new(16);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct FrameDifference {
    changed_pixel_count: u64,
    brightened_pixel_count: u64,
    darkened_pixel_count: u64,
    red_dominant_gain_count: u64,
    mean_absolute_rgb_error: f64,
    max_rgb_error: u8,
}

struct ProductRender {
    frame: CapturedFrame,
    stats: RenderStats,
    submit_cpu_micros: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProductSssMode {
    FeatureOff,
    RegisteredWithoutProfile,
    Enabled,
}

#[test]
fn render_product_sss_registered_but_without_profiles_is_exact_deferred_baseline() {
    let baseline = render_skin_sphere(ProductSssMode::FeatureOff);
    let registered = render_skin_sphere(ProductSssMode::RegisteredWithoutProfile);

    assert_eq!(baseline.frame.rgba, registered.frame.rgba);
    assert!(!registered
        .stats
        .last_graph_executed_passes
        .iter()
        .any(|pass| {
            matches!(
                pass.as_str(),
                SSS_SETUP_EXECUTOR_ID | SSS_SCATTER_EXECUTOR_ID | SSS_RECOMBINE_EXECUTOR_ID
            )
        }));
}

#[test]
fn render_product_sss_skin_sphere_executes_burley_indirect_pipeline() {
    let baseline = render_skin_sphere(ProductSssMode::FeatureOff);
    let subsurface = render_skin_sphere(ProductSssMode::Enabled);
    let difference = frame_difference(&baseline.frame, &subsurface.frame);

    for pass in [
        SSS_SETUP_EXECUTOR_ID,
        SSS_SCATTER_EXECUTOR_ID,
        SSS_RECOMBINE_EXECUTOR_ID,
    ] {
        assert!(
            subsurface
                .stats
                .last_graph_executed_passes
                .iter()
                .any(|executed| executed == pass),
            "SSS product graph did not execute {pass}: {:?}",
            subsurface.stats.last_graph_executed_passes
        );
    }
    assert!(
        difference.changed_pixel_count > 2_000,
        "skin sphere should expose broad SSS coverage: {difference:?}"
    );
    assert!(
        difference.mean_absolute_rgb_error > 0.04,
        "Burley scattering should be visibly distinct from deferred PBR: {difference:?}"
    );
    assert!(
        difference.brightened_pixel_count > 500 && difference.darkened_pixel_count > 500,
        "SSS must redistribute diffuse light instead of applying a uniform brightness change: {difference:?}"
    );
    assert!(
        difference.red_dominant_gain_count > 200,
        "skin profile must produce measurable long-radius red-channel bleed: {difference:?}"
    );
    assert_eq!(
        subsurface.stats.last_graph_compute_workload_mismatch_count, 0,
        "SSS setup and GPU-indirect scatter must both satisfy workload audit"
    );
}

#[test]
#[ignore = "exports the plan-18 SSS skin-sphere WGPU product comparison and timing evidence"]
fn export_render_product_sss_skin_sphere_png() {
    let baseline = render_skin_sphere(ProductSssMode::FeatureOff);
    let subsurface = render_skin_sphere(ProductSssMode::Enabled);
    let difference = frame_difference(&baseline.frame, &subsurface.frame);
    let output_dir = render_product_output_dir();
    fs::create_dir_all(&output_dir).expect("render product output directory should be writable");

    let image_path = output_dir.join(PRODUCT_IMAGE_NAME);
    write_side_by_side_png(&image_path, &baseline.frame, &subsurface.frame);
    let report_path = output_dir.join(PRODUCT_REPORT_NAME);
    fs::write(
        &report_path,
        product_report(&baseline, &subsurface, difference, &image_path),
    )
    .expect("SSS product report should be writable");

    assert!(image_path.is_file());
    assert!(report_path.is_file());
}

// Diagnostic copies belong to the original enabled frame, not an extra rendering pass.
const SSS_NATIVE_STAGING_LIMIT: u64 = 16 * 1024 * 1024;
type SssNativeMailbox = Arc<std::sync::Mutex<Option<SssNativeSnapshot>>>;

#[derive(Clone, Copy, Debug)]
struct SssNativePlane {
    offset: u64,
    byte_len: u64,
    row_pitch: u32,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
    format: wgpu::TextureFormat,
}

impl SssNativePlane {
    fn for_texture(texture: &wgpu::Texture, offset: u64) -> Result<Self, String> {
        let format = texture.format();
        let bytes_per_pixel = match format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => 4,
            wgpu::TextureFormat::Rgba16Float => 8,
            other => return Err(format!("unsupported actual GBuffer format {other:?}")),
        };
        let size = texture.size();
        if texture.dimension() != wgpu::TextureDimension::D2
            || texture.sample_count() != 1
            || size.depth_or_array_layers != 1
            || size.width == 0
            || size.height == 0
            || !texture.usage().contains(wgpu::TextureUsages::COPY_SRC)
        {
            return Err(format!(
                "GBuffer copy facts: format={format:?} size={size:?} samples={} usage={:?}",
                texture.sample_count(),
                texture.usage()
            ));
        }
        let unpadded = u64::from(size.width)
            .checked_mul(u64::from(bytes_per_pixel))
            .ok_or("GBuffer row overflow")?;
        let row_pitch = u32::try_from(sss_native_align(unpadded, 256)?)
            .map_err(|_| "GBuffer row exceeds u32")?;
        let byte_len = u64::from(row_pitch)
            .checked_mul(u64::from(size.height))
            .ok_or("GBuffer plane overflow")?;
        Ok(Self {
            offset,
            byte_len,
            row_pitch,
            width: size.width,
            height: size.height,
            bytes_per_pixel,
            format,
        })
    }

    fn copy(
        &self,
        texture: &wgpu::Texture,
        staging: &wgpu::Buffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: self.offset,
                    bytes_per_row: Some(self.row_pitch),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    fn alpha_bytes(&self, mapped: &[u8]) -> Result<Vec<u8>, String> {
        let start = usize::try_from(self.offset).map_err(|_| "plane offset exceeds usize")?;
        let end =
            usize::try_from(self.offset + self.byte_len).map_err(|_| "plane end exceeds usize")?;
        let source = mapped
            .get(start..end)
            .ok_or("mapped GBuffer plane truncated")?;
        let row_bytes = usize::try_from(u64::from(self.width) * u64::from(self.bytes_per_pixel))
            .map_err(|_| "row exceeds usize")?;
        let mut packed = Vec::with_capacity(row_bytes * self.height as usize);
        for row in source.chunks_exact(self.row_pitch as usize) {
            packed.extend_from_slice(&row[..row_bytes]);
        }
        match self.format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {
                Ok(packed.chunks_exact(4).map(|pixel| pixel[3]).collect())
            }
            wgpu::TextureFormat::Rgba16Float => {
                crate::core::framework::render::decode_rgba16f_texels(&packed)
                    .into_iter()
                    .map(|pixel| {
                        let alpha = pixel[3];
                        if !alpha.is_finite() {
                            return Err("non-finite GBuffer alpha".to_string());
                        }
                        Ok((alpha * 255.0).round().clamp(0.0, 255.0) as u8)
                    })
                    .collect()
            }
            _ => Err("unsupported recorded GBuffer format".to_string()),
        }
    }
}

struct SssNativeSnapshot {
    staging: wgpu::Buffer,
    waiter: Box<dyn FnOnce() -> Result<(), String> + Send>,
    planes: [SssNativePlane; 2],
    metadata_offset: u64,
    staging_bytes: u64,
    device_epoch: String,
}

struct SssNativeDiagnosticExecutor {
    inner: Arc<dyn RenderPassExecutor>,
    mailbox: SssNativeMailbox,
}

impl RenderPassExecutor for SssNativeDiagnosticExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        self.inner.execute(context)?;
        let diagnostic = context.require_gpu().and_then(sss_native_record_snapshot);
        match diagnostic {
            Ok(snapshot) => match self.mailbox.lock() {
                Ok(mut slot) if slot.is_none() => *slot = Some(snapshot),
                _ => eprintln!(
                    "SSS_NATIVE_READBACK unavailable: duplicate/poisoned per-frame mailbox"
                ),
            },
            Err(error) => eprintln!("SSS_NATIVE_READBACK unavailable: {error}"),
        }
        Ok(())
    }

    fn required_device_features(&self) -> &[zr_rhi::RenderDeviceFeature] {
        self.inner.required_device_features()
    }

    fn recording_policy(&self) -> RenderPassRecordingPolicy {
        self.inner.recording_policy()
    }
}

fn sss_native_align(value: u64, alignment: u64) -> Result<u64, String> {
    value
        .checked_add(alignment - 1)
        .map(|sum| sum / alignment * alignment)
        .ok_or_else(|| "diagnostic alignment overflow".to_string())
}

fn sss_native_buffer_source(
    gpu: &RenderPassGpuExecutionContext<'_>,
    name: &str,
    relative_offset: u64,
    byte_len: u64,
) -> Result<(wgpu::Buffer, u64), String> {
    let binding = gpu.require_buffer_binding(name, RenderGraphResourceAccessKind::Read)?;
    let available = binding
        .size
        .map(|size| size.get())
        .unwrap_or_else(|| binding.buffer.size().saturating_sub(binding.offset));
    let relative_end = relative_offset
        .checked_add(byte_len)
        .ok_or("diagnostic binding overflow")?;
    let source_offset = binding
        .offset
        .checked_add(relative_offset)
        .ok_or("diagnostic offset overflow")?;
    if relative_end > available
        || source_offset
            .checked_add(byte_len)
            .is_none_or(|end| end > binding.buffer.size())
        || source_offset % 16 != 0
        || byte_len % 16 != 0
        || !binding
            .buffer
            .usage()
            .contains(wgpu::BufferUsages::COPY_SRC)
    {
        return Err(format!("invalid actual diagnostic source {name}: offset={source_offset} bytes={byte_len} available={available} usage={:?}", binding.buffer.usage()));
    }
    Ok((binding.buffer.clone(), source_offset))
}

fn sss_native_record_snapshot(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
) -> Result<SssNativeSnapshot, String> {
    use crate::core::framework::render::PostProcessGraphResourceNames as Names;
    let material = gpu
        .require_texture_view(Names::GBUFFER_MATERIAL, RenderGraphResourceAccessKind::Read)?
        .texture()
        .clone();
    let normal = gpu
        .require_texture_view(Names::GBUFFER_NORMAL, RenderGraphResourceAccessKind::Read)?
        .texture()
        .clone();
    let material_plane = SssNativePlane::for_texture(&material, 0)?;
    let normal_plane =
        SssNativePlane::for_texture(&normal, sss_native_align(material_plane.byte_len, 256)?)?;
    if (material_plane.width, material_plane.height) != (normal_plane.width, normal_plane.height) {
        return Err("actual GBuffer extents differ".to_string());
    }
    let metadata_offset = sss_native_align(
        normal_plane
            .offset
            .checked_add(normal_plane.byte_len)
            .ok_or("diagnostic planes overflow")?,
        16,
    )?;
    let staging_bytes = metadata_offset
        .checked_add(64)
        .ok_or("diagnostic metadata overflow")?;
    if staging_bytes > SSS_NATIVE_STAGING_LIMIT {
        return Err(format!("diagnostic staging {staging_bytes} exceeds 16 MiB"));
    }
    let params = sss_native_buffer_source(gpu, Names::SSS_PARAMS, 0, 16)?;
    let profile7 = sss_native_buffer_source(gpu, Names::SSS_PROFILES, 7 * 32, 32)?;
    let indirect = sss_native_buffer_source(gpu, Names::SSS_INDIRECT_ARGS, 0, 16)?;
    let device_epoch = format!("{:?}", gpu.device_epoch());
    let native = gpu.native_context();
    let staging = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("sss-original-frame-native-diagnostic"),
        contents: &vec![0; staging_bytes as usize],
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    });
    material_plane.copy(&material, &staging, native.encoder);
    normal_plane.copy(&normal, &staging, native.encoder);
    native
        .encoder
        .copy_buffer_to_buffer(&params.0, params.1, &staging, metadata_offset, 16);
    native.encoder.copy_buffer_to_buffer(
        &profile7.0,
        profile7.1,
        &staging,
        metadata_offset + 16,
        32,
    );
    native.encoder.copy_buffer_to_buffer(
        &indirect.0,
        indirect.1,
        &staging,
        metadata_offset + 48,
        16,
    );
    let waiter = Box::new(native.readback_completion_waiter());
    Ok(SssNativeSnapshot {
        staging,
        waiter,
        planes: [material_plane, normal_plane],
        metadata_offset,
        staging_bytes,
        device_epoch,
    })
}

fn sss_native_wrap_scatter(
    executors: &mut [RenderPassExecutorRegistration],
    mailbox: &SssNativeMailbox,
) {
    for registration in executors {
        if registration.executor_id.as_str() == SSS_SCATTER_EXECUTOR_ID {
            registration.executor = Arc::new(SssNativeDiagnosticExecutor {
                inner: Arc::clone(&registration.executor),
                mailbox: Arc::clone(mailbox),
            });
        }
    }
}

fn sss_native_report(mailbox: &SssNativeMailbox, generation: u64) {
    let snapshot = match mailbox.lock() {
        Ok(mut slot) => slot.take(),
        Err(_) => {
            eprintln!("SSS_NATIVE_READBACK unavailable: poisoned mailbox");
            return;
        }
    };
    let Some(snapshot) = snapshot else {
        eprintln!(
            "SSS_NATIVE_READBACK unavailable: no executed scatter snapshot generation={generation}"
        );
        return;
    };
    let SssNativeSnapshot {
        staging,
        waiter,
        planes,
        metadata_offset,
        staging_bytes,
        device_epoch,
    } = snapshot;
    // Mapping is requested only after the original submission/capture, never while encoding.
    let (sender, receiver) = std::sync::mpsc::channel();
    staging
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    let terminal = waiter().and_then(|_| {
        receiver
            .try_recv()
            .map_err(|error| format!("native map callback not terminal: {error}"))?
            .map_err(|error| format!("native map failed: {error}"))
    });
    if let Err(error) = terminal {
        staging.unmap();
        eprintln!("SSS_NATIVE_READBACK unavailable: {error}");
        return;
    }
    let mapped = staging.slice(..).get_mapped_range();
    let report = (|| -> Result<(), String> {
        let material = planes[0].alpha_bytes(&mapped)?;
        let normal = planes[1].alpha_bytes(&mapped)?;
        if material.len() != normal.len() {
            return Err("decoded alpha counts differ".to_string());
        }
        let mut histogram = std::collections::BTreeMap::<(u8, u8), u64>::new();
        for (model, profile) in material.into_iter().zip(normal) {
            *histogram.entry((model & 0x7f, profile)).or_default() += 1;
        }
        let base = metadata_offset as usize;
        let metadata = mapped
            .get(base..base + 64)
            .ok_or("mapped metadata truncated")?;
        let words: Vec<u32> = metadata
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
            .collect();
        let profile7: Vec<f32> = words[4..12]
            .iter()
            .map(|word| f32::from_bits(*word))
            .collect();
        eprintln!("SSS_NATIVE_READBACK generation={generation} epoch={device_epoch} staging_bytes={staging_bytes} planes={planes:?} params_wh_count_mask={:?} profile7_radius_scale_falloff_id={profile7:?} indirect_xyz={:?} model_profile_pixels={histogram:?}", &words[..4], &words[12..15]);
        Ok(())
    })();
    drop(mapped);
    staging.unmap();
    if let Err(error) = report {
        eprintln!("SSS_NATIVE_READBACK unavailable: {error}");
    }
}

fn render_skin_sphere(mode: ProductSssMode) -> ProductRender {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let model = register_skin_sphere_model(&asset_manager);
    let registered_sss = mode != ProductSssMode::FeatureOff;
    let material = register_skin_material(
        &asset_manager,
        registered_sss,
        mode == ProductSssMode::Enabled,
    );
    let render_features = registered_sss
        .then(subsurface_render_feature_descriptor)
        .into_iter()
        .collect::<Vec<_>>();
    let mut executors = if registered_sss {
        subsurface_render_pass_executor_registrations()
    } else {
        Vec::new()
    };
    let native_mailbox = Arc::new(std::sync::Mutex::new(None));
    if mode == ProductSssMode::Enabled {
        sss_native_wrap_scatter(&mut executors, &native_mailbox);
    }
    let framework =
        WgpuRenderFramework::new_for_test_with_plugin_render_extensions_and_shading_models(
            asset_manager,
            render_features,
            executors,
            Vec::new(),
            Vec::new(),
            registered_sss
                .then(sss_shading_model_descriptor)
                .into_iter(),
            Vec::new(),
            Vec::new(),
        )
        .expect("WGPU framework should initialize for the SSS product scene");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(PRODUCT_SIZE))
        .expect("SSS product viewport should be created");
    framework
        .set_pipeline_asset(viewport, DEFERRED_PIPELINE_HANDLE)
        .expect("SSS product must use the deferred pipeline");
    framework
        .set_quality_profile(viewport, product_quality_profile())
        .expect("SSS product quality profile should be accepted");

    let start = Instant::now();
    framework
        .submit_frame_extract(viewport, skin_sphere_extract(model, material))
        .expect("SSS product frame should submit");
    let submit_cpu_micros = start.elapsed().as_micros();
    let frame = framework
        .capture_frame(viewport)
        .expect("SSS product capture should succeed")
        .expect("SSS product viewport should expose a frame");
    if mode == ProductSssMode::Enabled {
        sss_native_report(&native_mailbox, frame.generation);
    } else {
        eprintln!(
            "SSS_NATIVE_READBACK absent: mode={mode:?} generation={}",
            frame.generation
        );
    }
    let stats = framework
        .query_stats()
        .expect("SSS product stats should exist");
    framework
        .destroy_viewport(viewport)
        .expect("SSS product viewport should be destroyed");

    ProductRender {
        frame,
        stats,
        submit_cpu_micros,
    }
}

fn skin_sphere_extract(
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderFrameExtract {
    let environment = EnvironmentExtract::procedural_default();
    let mut camera = ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.25, 4.6), Vec3::ZERO, Vec3::Y),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 48.0_f32.to_radians(),
        z_near: 0.1,
        z_far: 40.0,
        core_pipeline: CorePipelineKind::Core3d,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(PRODUCT_SIZE);
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: vec![RenderMeshSnapshot {
                node_id: 18_600,
                stable_instance_key: 18_600 << 16,
                transform_revision: 1,
                transform: Transform {
                    scale: Vec3::splat(1.35),
                    ..Transform::default()
                },
                model,
                mesh: None,
                material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::ONE,
                mobility: Mobility::Dynamic,
                static_state: RenderMeshStaticState::default(),
                common: crate::core::framework::render::RendererCommon {
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                    ..Default::default()
                },
            }],
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 18_601,
                light_id: 18_601,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                direction: Vec3::new(-0.65, -0.35, -0.68).normalize_or_zero(),
                color: Vec3::new(1.0, 0.86, 0.76),
                intensity: 3.2,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: None,
            }],
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: crate::core::framework::render::RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..Default::default()
        },
        environment: environment.clone(),
        preview: PreviewEnvironmentExtract::from_environment(
            &environment,
            true,
            Vec4::new(0.018, 0.022, 0.03, 1.0),
        ),
        virtual_geometry_debug: None,
    };
    RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(18_602), snapshot)
        .with_viewport_size(PRODUCT_SIZE)
}

fn register_skin_material(
    asset_manager: &ProjectAssetManager,
    subsurface: bool,
    with_embedded_profile: bool,
) -> ResourceHandle<MaterialMarker> {
    let uri = AssetUri::parse("res://materials/plan18-skin-sss.zmaterial").unwrap();
    let id = ResourceId::from_locator(&uri);
    let builtin_shader_uri = AssetUri::parse("builtin://shader/pbr.wgsl").unwrap();
    let shader_reference = if subsurface {
        // The builtin shader carries a small fallback surface. A project shader must
        // use the standard material surface to read the same roughness and profile uniforms.
        let mut shader = asset_manager
            .load_shader_asset_by_uri(&builtin_shader_uri)
            .expect("SSS fixture should load the builtin PBR surface shader");
        let shader_uri = AssetUri::parse("res://shaders/plan18-skin-subsurface.zshader").unwrap();
        let shader_id = ResourceId::from_locator(&shader_uri);
        let surface =
            crate::graphics::shader::template::standard_material_surface_source_for_features(
                crate::core::framework::render::ShaderFeatureBits::default(),
                0.0,
            );
        let surface_source = surface.source.replacen(
            &format!("fn {}(", surface.entry_point),
            "fn zr_material_surface(",
            1,
        );
        shader.source = surface_source.clone();
        shader.wgsl_source = surface_source;
        shader.uri = shader_uri.clone();
        shader.shading_model = Some("custom:subsurface".to_string());
        asset_manager
            .assets::<ShaderAsset>()
            .insert(
                ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri.clone())
                    .with_source_hash("plan18-skin-subsurface-shader-v2"),
                shader,
            )
            .expect("SSS surface shader should register");
        AssetReference::from_locator(shader_uri)
    } else {
        AssetReference::from_locator(builtin_shader_uri)
    };
    let mut material = MaterialAsset {
        name: Some("Plan18 Skin SSS".to_string()),
        shader: shader_reference,
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [0.95, 0.42, 0.31, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 0.58,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0; 3],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    if subsurface {
        material.property_values.insert(
            "lighting_model".to_string(),
            toml::Value::String("custom:subsurface".to_string()),
        );
        material
            .property_values
            .insert("subsurface_profile".to_string(), toml::Value::Integer(7));
    }
    if with_embedded_profile {
        material.property_values.insert(
            "subsurface_scatter_radius".to_string(),
            toml::Value::Array(
                vec![8.0, 3.4, 1.5]
                    .into_iter()
                    .map(toml::Value::Float)
                    .collect(),
            ),
        );
        material.property_values.insert(
            "subsurface_falloff".to_string(),
            toml::Value::Array(
                vec![1.0, 0.48, 0.3]
                    .into_iter()
                    .map(toml::Value::Float)
                    .collect(),
            ),
        );
        material.property_values.insert(
            "subsurface_world_unit_scale".to_string(),
            toml::Value::Float(0.04),
        );
    }
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, uri)
                .with_source_hash("plan18-skin-sss-v1"),
            material,
        )
        .expect("SSS skin material should register");
    ResourceHandle::new(id)
}

fn register_skin_sphere_model(asset_manager: &ProjectAssetManager) -> ResourceHandle<ModelMarker> {
    let uri = AssetUri::parse("res://models/plan18-skin-sphere.zmodel").unwrap();
    let id = ResourceId::from_locator(&uri);
    let (vertices, indices) = uv_sphere_geometry(48, 72);
    let model = ModelAsset {
        uri: uri.clone(),
        primitives: vec![ModelPrimitiveAsset {
            vertices,
            indices,
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    };
    asset_manager
        .assets::<ModelAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Model, uri)
                .with_source_hash("plan18-skin-sphere-v1"),
            model,
        )
        .expect("SSS skin sphere model should register");
    ResourceHandle::new(id)
}

fn uv_sphere_geometry(rings: usize, segments: usize) -> (Vec<MeshVertex>, Vec<u32>) {
    let rings = rings.max(3);
    let segments = segments.max(6);
    let mut vertices = Vec::with_capacity((rings + 1) * (segments + 1));
    for ring in 0..=rings {
        let theta = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = theta.cos();
        let radius = theta.sin();
        for segment in 0..=segments {
            let phi = std::f32::consts::TAU * segment as f32 / segments as f32;
            let normal = Vec3::new(radius * phi.cos(), y, radius * phi.sin());
            vertices.push(MeshVertex::new(
                normal,
                normal,
                Vec2::new(segment as f32 / segments as f32, ring as f32 / rings as f32),
            ));
        }
    }
    let mut indices = Vec::with_capacity(rings * segments * 6);
    for ring in 0..rings {
        for segment in 0..segments {
            let a = (ring * (segments + 1) + segment) as u32;
            let b = a + 1;
            let c = a + segments as u32 + 1;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    (vertices, indices)
}

fn product_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("advanced-lighting-sss-product")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
}

fn sss_shading_model_descriptor() -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        SSS_SHADING_MODEL_ID,
        "custom:subsurface",
        "zr_shading_standard_pbr.wgsl",
        "zr_gbuffer_encode_subsurface.wgsl",
        "zr_shade_deferred_subsurface.wgsl",
        GBufferChannelMask::standard_lit(),
    )
}

fn frame_difference(baseline: &CapturedFrame, subsurface: &CapturedFrame) -> FrameDifference {
    assert_eq!(baseline.width, subsurface.width);
    assert_eq!(baseline.height, subsurface.height);
    let mut difference = FrameDifference::default();
    let mut absolute_error_sum = 0_u64;
    for (baseline_pixel, sss_pixel) in baseline
        .rgba
        .chunks_exact(4)
        .zip(subsurface.rgba.chunks_exact(4))
    {
        let mut changed = false;
        for channel in 0..3 {
            let error = baseline_pixel[channel].abs_diff(sss_pixel[channel]);
            changed |= error != 0;
            absolute_error_sum = absolute_error_sum.saturating_add(u64::from(error));
            difference.max_rgb_error = difference.max_rgb_error.max(error);
        }
        if changed {
            difference.changed_pixel_count += 1;
        }
        let baseline_rgb = u32::from(baseline_pixel[0])
            + u32::from(baseline_pixel[1])
            + u32::from(baseline_pixel[2]);
        let sss_rgb = u32::from(sss_pixel[0]) + u32::from(sss_pixel[1]) + u32::from(sss_pixel[2]);
        if sss_rgb > baseline_rgb + 2 {
            difference.brightened_pixel_count += 1;
        } else if baseline_rgb > sss_rgb + 2 {
            difference.darkened_pixel_count += 1;
        }
        let red_gain = i16::from(sss_pixel[0]) - i16::from(baseline_pixel[0]);
        let blue_gain = i16::from(sss_pixel[2]) - i16::from(baseline_pixel[2]);
        if red_gain > 1 && red_gain > blue_gain + 1 {
            difference.red_dominant_gain_count += 1;
        }
    }
    let sample_count = u64::from(baseline.width) * u64::from(baseline.height) * 3;
    difference.mean_absolute_rgb_error = absolute_error_sum as f64 / sample_count.max(1) as f64;
    difference
}

fn write_side_by_side_png(path: &Path, baseline: &CapturedFrame, subsurface: &CapturedFrame) {
    const SEPARATOR_WIDTH: u32 = 6;
    let output_width = baseline.width * 2 + SEPARATOR_WIDTH;
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(output_width, baseline.height, |x, y| {
        if x < baseline.width {
            captured_pixel(baseline, x, y)
        } else if x < baseline.width + SEPARATOR_WIDTH {
            Rgba([235, 239, 245, 255])
        } else {
            captured_pixel(subsurface, x - baseline.width - SEPARATOR_WIDTH, y)
        }
    });
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("SSS side-by-side PNG should be writable");
}

fn captured_pixel(frame: &CapturedFrame, x: u32, y: u32) -> Rgba<u8> {
    let index = ((y * frame.width + x) * 4) as usize;
    Rgba([
        frame.rgba[index],
        frame.rgba[index + 1],
        frame.rgba[index + 2],
        frame.rgba[index + 3],
    ])
}

fn product_report(
    baseline: &ProductRender,
    subsurface: &ProductRender,
    difference: FrameDifference,
    image_path: &Path,
) -> String {
    format!(
        concat!(
            "Plan 18 AF-M4 SSS WGPU product evidence\n",
            "image={}\n",
            "viewport={}x{}\n",
            "shading_model_id={}\n",
            "burley_sample_count=64\n",
            "changed_pixel_count={}\n",
            "brightened_pixel_count={}\n",
            "darkened_pixel_count={}\n",
            "red_dominant_gain_count={}\n",
            "mean_absolute_rgb_error={:.4}\n",
            "max_rgb_error={}\n",
            "baseline_submit_cpu_micros={}\n",
            "sss_submit_cpu_micros={}\n",
            "sss_executed_passes={}\n",
        ),
        image_path.display(),
        PRODUCT_SIZE.x,
        PRODUCT_SIZE.y,
        SSS_SHADING_MODEL_ID.value(),
        difference.changed_pixel_count,
        difference.brightened_pixel_count,
        difference.darkened_pixel_count,
        difference.red_dominant_gain_count,
        difference.mean_absolute_rgb_error,
        difference.max_rgb_error,
        baseline.submit_cpu_micros,
        subsurface.submit_cpu_micros,
        subsurface.stats.last_graph_executed_passes.join(","),
    )
}

fn render_product_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime should have repository parent")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}
