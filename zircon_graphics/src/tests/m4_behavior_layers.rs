use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_asset::{
    AlphaMode, AssetManager, AssetReference, AssetUri, MaterialAsset, ProjectAssetManager,
    ProjectManifest, ProjectPaths,
};
use zircon_math::{Transform, UVec2, Vec3, Vec4};
use zircon_render_server::{
    RenderPipelineHandle, RenderQualityProfile, RenderServer, RenderViewportDescriptor,
    RenderViewportHandle,
};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_scene::{
    default_render_layer_mask, DisplayMode, FallbackSkyboxKind, Mobility, ProjectionMode,
    RenderBloomSettings, RenderColorGradingSettings, RenderDirectionalLightSnapshot,
    RenderFrameExtract, RenderMeshSnapshot, RenderOverlayExtract, RenderParticleSpriteSnapshot,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};

use crate::{offline_bake_frame, runtime::WgpuRenderServer, OfflineBakeSettings};

#[test]
fn bloom_quality_profile_spreads_bright_pixels_when_enabled() {
    let fixture = RenderFixture::new("graphics_m4_bloom", [1.0, 1.0, 1.0, 1.0]);
    let extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            transform: centered_quad_transform(0.35),
            model: fixture.model,
            material: fixture.material,
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: default_render_layer_mask(),
        }],
        Vec::new(),
        |extract| {
            extract.post_process.bloom = RenderBloomSettings {
                threshold: 0.55,
                intensity: 1.0,
                radius: 1.0,
            };
        },
    );

    let server = fixture.server();
    let bloom_on = fixture.render_extract(
        &server,
        extract.clone(),
        RenderQualityProfile::new("bloom-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_history_resolve(false),
    );
    let bloom_off = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("bloom-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_history_resolve(false)
            .with_bloom(false),
    );

    let bloom_ring = ring_luma(&bloom_on.rgba, fixture.viewport_size, 0.18, 0.42);
    let no_bloom_ring = ring_luma(&bloom_off.rgba, fixture.viewport_size, 0.18, 0.42);
    assert!(
        bloom_ring > no_bloom_ring + 6.0,
        "expected bloom to brighten neighboring pixels; bloom ring={bloom_ring:.2}, no-bloom ring={no_bloom_ring:.2}"
    );
}

#[test]
fn color_grading_extract_tints_scene_after_post_process() {
    let fixture = RenderFixture::new("graphics_m4_color_grading", [0.72, 0.72, 0.72, 1.0]);
    let extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            transform: fullscreen_quad_transform(),
            model: fixture.model,
            material: fixture.material,
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: default_render_layer_mask(),
        }],
        Vec::new(),
        |extract| {
            extract.post_process.color_grading = RenderColorGradingSettings {
                exposure: 1.18,
                contrast: 1.08,
                saturation: 0.92,
                gamma: 0.95,
                tint: Vec3::new(1.12, 0.78, 0.58),
            };
        },
    );

    let server = fixture.server();
    let graded = fixture.render_extract(
        &server,
        extract.clone(),
        RenderQualityProfile::new("grade-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_history_resolve(false),
    );
    let neutral = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("grade-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_history_resolve(false)
            .with_color_grading(false),
    );

    let graded_red = average_channel(&graded.rgba, 0);
    let graded_blue = average_channel(&graded.rgba, 2);
    let neutral_red = average_channel(&neutral.rgba, 0);
    let neutral_blue = average_channel(&neutral.rgba, 2);
    assert!(
        (graded_red - graded_blue) > (neutral_red - neutral_blue) + 12.0,
        "expected color grading tint to bias warm channels; graded delta={:.2}, neutral delta={:.2}",
        graded_red - graded_blue,
        neutral_red - neutral_blue
    );
}

#[test]
fn offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering() {
    let fixture = RenderFixture::new("graphics_m4_offline_bake", [0.5, 0.5, 0.5, 1.0]);
    let base_extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            transform: fullscreen_quad_transform(),
            model: fixture.model,
            material: fixture.material,
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: default_render_layer_mask(),
        }],
        vec![RenderDirectionalLightSnapshot {
            node_id: 7,
            direction: Vec3::new(-0.4, -0.4, -1.0).normalize_or_zero(),
            color: Vec3::new(1.0, 0.62, 0.28),
            intensity: 3.2,
        }],
        |_extract| {},
    );

    let bake_output = offline_bake_frame(
        &base_extract,
        &OfflineBakeSettings {
            ambient_scale: 0.24,
            reflection_probe_scale: 0.8,
            max_reflection_probes: 1,
        },
    );
    assert!(
        bake_output.baked_lighting.intensity > 0.0,
        "offline bake should produce non-zero baked lighting"
    );
    assert!(
        !bake_output.reflection_probes.is_empty(),
        "offline bake should produce at least one reflection probe"
    );

    let mut baked_extract = base_extract.clone();
    baked_extract.lighting.baked_lighting = Some(bake_output.baked_lighting);
    baked_extract.lighting.reflection_probes = bake_output.reflection_probes;

    let server = fixture.server();
    let baked_frame = fixture.render_extract(
        &server,
        baked_extract,
        RenderQualityProfile::new("baked-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_history_resolve(false),
    );
    let unbaked_frame = fixture.render_extract(
        &server,
        base_extract,
        RenderQualityProfile::new("baked-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_history_resolve(false)
            .with_baked_lighting(false)
            .with_reflection_probes(false),
    );

    let baked_red = average_channel(&baked_frame.rgba, 0);
    let unbaked_red = average_channel(&unbaked_frame.rgba, 0);
    assert!(
        baked_red > unbaked_red + 8.0,
        "expected baked lighting and probes to change the frame; baked red={baked_red:.2}, unbaked red={unbaked_red:.2}"
    );
}

#[test]
fn particle_rendering_draws_billboard_sprites_in_transparent_stage() {
    let fixture = RenderFixture::new("graphics_m4_particles", [0.1, 0.1, 0.1, 1.0]);
    let extract = fixture.frame_extract(Vec::new(), Vec::new(), |extract| {
        extract.particles.emitters = vec![42];
        extract.particles.sprites = vec![RenderParticleSpriteSnapshot {
            entity: 42,
            position: Vec3::ZERO,
            size: 0.9,
            color: Vec4::new(1.0, 0.48, 0.12, 0.8),
            intensity: 1.0,
        }];
    });

    let server = fixture.server();
    let particle_frame = fixture.render_extract(
        &server,
        extract.clone(),
        RenderQualityProfile::new("particle-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_history_resolve(false),
    );
    let no_particle_frame = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("particle-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_history_resolve(false)
            .with_particle_rendering(false),
    );

    let particle_pixels = warm_pixels(&particle_frame.rgba);
    let no_particle_pixels = warm_pixels(&no_particle_frame.rgba);
    assert!(
        particle_pixels > no_particle_pixels + 96,
        "expected particle rendering to add visible billboard pixels; particle={particle_pixels}, disabled={no_particle_pixels}"
    );
}

struct RenderFixture {
    root: PathBuf,
    asset_manager: Arc<ProjectAssetManager>,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    viewport_size: UVec2,
}

impl RenderFixture {
    fn new(label: &str, base_color: [f32; 4]) -> Self {
        let root = unique_temp_project_root(label);
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths.ensure_layout().unwrap();
        ProjectManifest::new(
            label,
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();

        write_flat_color_wgsl(
            paths.assets_root().join("shaders").join("flat_color.wgsl"),
            [base_color[0], base_color[1], base_color[2]],
        );
        write_solid_png(
            paths.assets_root().join("textures").join("white.png"),
            [255, 255, 255, 255],
        );
        write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
        write_material_with_base_color_and_texture(
            paths
                .assets_root()
                .join("materials")
                .join("flat_color.material.toml"),
            "res://shaders/flat_color.wgsl",
            base_color,
            "res://textures/white.png",
        );

        let asset_manager = Arc::new(ProjectAssetManager::default());
        asset_manager
            .open_project(root.to_string_lossy().as_ref())
            .unwrap();
        let mut project = zircon_asset::ProjectManager::open(&root).unwrap();
        project.scan_and_import().unwrap();

        let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
        let material = resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/flat_color.material.toml",
        );

        Self {
            root,
            asset_manager,
            model,
            material,
            viewport_size: UVec2::new(160, 120),
        }
    }

    fn server(&self) -> WgpuRenderServer {
        WgpuRenderServer::new(self.asset_manager.clone()).unwrap()
    }

    fn frame_extract<F>(
        &self,
        meshes: Vec<RenderMeshSnapshot>,
        lights: Vec<RenderDirectionalLightSnapshot>,
        configure: F,
    ) -> RenderFrameExtract
    where
        F: FnOnce(&mut RenderFrameExtract),
    {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            build_snapshot(meshes, lights, self.viewport_size),
        );
        configure(&mut extract);
        extract
    }

    fn render_extract(
        &self,
        server: &WgpuRenderServer,
        extract: RenderFrameExtract,
        profile: RenderQualityProfile,
    ) -> zircon_render_server::CapturedFrame {
        let viewport = server
            .create_viewport(RenderViewportDescriptor::new(self.viewport_size))
            .unwrap();
        server
            .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
            .unwrap();
        server.set_quality_profile(viewport, profile).unwrap();
        let frame = submit_extract(server, viewport, extract);
        server.destroy_viewport(viewport).unwrap();
        frame
    }
}

impl Drop for RenderFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_graphics_{label}_{unique}"))
}

fn build_snapshot(
    meshes: Vec<RenderMeshSnapshot>,
    lights: Vec<RenderDirectionalLightSnapshot>,
    viewport_size: UVec2,
) -> RenderSceneSnapshot {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes,
            lights,
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        preview: zircon_scene::PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
    }
}

fn fullscreen_quad_transform() -> Transform {
    Transform {
        scale: Vec3::new(1.8, 1.8, 1.0),
        ..Transform::default()
    }
}

fn centered_quad_transform(scale: f32) -> Transform {
    Transform {
        scale: Vec3::new(scale, scale, 1.0),
        ..Transform::default()
    }
}

fn submit_extract(
    server: &WgpuRenderServer,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> zircon_render_server::CapturedFrame {
    server.submit_frame_extract(viewport, extract).unwrap();
    server
        .capture_frame(viewport)
        .unwrap()
        .expect("frame should be available after submission")
}

fn ring_luma(rgba: &[u8], viewport_size: UVec2, inner_radius: f32, outer_radius: f32) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    let center_x = viewport_size.x as f32 * 0.5;
    let center_y = viewport_size.y as f32 * 0.5;
    let normalizer = viewport_size.x.min(viewport_size.y) as f32 * 0.5;
    for y in 0..viewport_size.y as usize {
        for x in 0..viewport_size.x as usize {
            let dx = x as f32 + 0.5 - center_x;
            let dy = y as f32 + 0.5 - center_y;
            let radius = (dx * dx + dy * dy).sqrt() / normalizer.max(1.0);
            if radius < inner_radius || radius > outer_radius {
                continue;
            }
            let index = (y * viewport_size.x as usize + x) * 4;
            let pixel = &rgba[index..index + 4];
            total += 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}

fn warm_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| {
            pixel[3] == 255 && pixel[0] > 28 && pixel[0] > pixel[1] && pixel[1] > pixel[2]
        })
        .count()
}

fn average_channel(rgba: &[u8], channel: usize) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }
    let total = rgba
        .chunks_exact(4)
        .map(|pixel| pixel[channel] as f32)
        .sum::<f32>();
    total / (rgba.len() as f32 / 4.0)
}

fn write_flat_color_wgsl(path: PathBuf, color: [f32; 3]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        format!(
            r#"
struct SceneUniform {{
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    ambient_color: vec4<f32>,
}};
struct ModelUniform {{
    model: mat4x4<f32>,
    tint: vec4<f32>,
}};
@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model_data: ModelUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;

struct VertexInput {{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}};

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {{
    var output: VertexOutput;
    let world = model_data.model * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    return output;
}}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
    let alpha = textureSample(albedo_tex, albedo_sampler, input.uv).a;
    return vec4<f32>({:.6}, {:.6}, {:.6}, alpha) * model_data.tint;
}}
"#,
            color[0], color[1], color[2]
        ),
    )
    .unwrap();
}

fn write_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba(rgba))
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
}

fn write_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "\
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 1.0 1.0 0.0
v -1.0 1.0 0.0
vt 0.0 1.0
vt 1.0 1.0
vt 1.0 0.0
vt 0.0 0.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 1/1/1 3/3/1 4/4/1
",
    )
    .unwrap();
}

fn write_material_with_base_color_and_texture(
    path: PathBuf,
    shader_uri: &str,
    base_color: [f32; 4],
    base_color_texture: &str,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("FlatColor".to_string()),
        shader: asset_reference(shader_uri),
        base_color,
        base_color_texture: Some(asset_reference(base_color_texture)),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn resource_handle<T>(asset_manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        asset_manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing resource id for {uri}")),
    )
}
