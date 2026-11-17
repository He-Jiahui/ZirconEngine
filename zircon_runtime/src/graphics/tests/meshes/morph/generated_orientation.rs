//! Read the real deferred GBuffer; neither light brightness nor a substitute surface shader is an oracle.
use std::{path::PathBuf, sync::Mutex, time::SystemTime};

use crate::asset::pipeline::manager::AssetManager;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::{
    asset_kind_for_imported_asset, AssetImportOutcome, AssetImporter, AssetReference,
    ImportedAsset, MESH_ATTRIBUTE_TANGENT,
};
use crate::core::framework::render::{
    DisplayMode, PostProcessGraphResourceNames as Names, RenderCaptureSource, RenderViewportHandle,
};
use crate::core::resource::ResourceMutationBatch;
use crate::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorRegistration, RenderPassGpuResourceFactory,
    RenderPassStage, RenderPipelineAsset,
};
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind::Read};

use super::*;

const EXPAND_GLTF: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/asset/tests/fixtures/gltf/mirrored_morph_endpoints/expand.gltf"
);
const ROOT_URI: &str = "res://models/mirrored-morph-expand.gltf";
const SIZE: UVec2 = UVec2::new(128, 128);
const OBSERVER: &str = "test.morph-gbuffer-observer";
const PLANE_BYTES: u64 = 4 * SIZE.x as u64 * SIZE.y as u64;
// The runtime may pool GBuffer allocations without COPY_SRC, but both planes
// are shader-readable. This observer only transfers the existing pixels; it
// does not replace or re-execute the production material or vertex shader.
const OBSERVE_WGSL: &str = r#"
@group(0) @binding(0) var albedo: texture_2d<f32>;
@group(0) @binding(1) var normal: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> pixels: array<u32>;

@compute @workgroup_size(8, 8)
fn observe(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(normal);
    if (any(id.xy >= size)) { return; }
    let texel = vec2<i32>(id.xy);
    let index = id.y * size.x + id.x;
    pixels[index] = pack4x8unorm(textureLoad(albedo, texel, 0));
    pixels[size.x * size.y + index] = pack4x8unorm(textureLoad(normal, texel, 0));
}
"#;
type Mailbox = Arc<Mutex<Option<NativeGBuffer>>>;

struct NativeGBuffer {
    staging: wgpu::Buffer,
    waiter: Box<dyn FnOnce() -> Result<(), String> + Send>,
}
struct GBuffer {
    albedo: Vec<[u8; 4]>,
    normal: Vec<[u8; 4]>,
}
struct GBufferObserver(Mailbox);

impl RenderPassExecutor for GBufferObserver {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        let mut slot = self
            .0
            .lock()
            .map_err(|_| "MORPH_GBUFFER_READBACK poisoned mailbox")?;
        if slot.is_some() {
            return Err("MORPH_GBUFFER_READBACK duplicate observer execution".into());
        }
        let gpu = context.require_gpu()?;
        let mut views = Vec::with_capacity(2);
        for name in [Names::GBUFFER_ALBEDO, Names::GBUFFER_NORMAL] {
            let view = gpu.require_texture_view(name, Read)?.clone();
            let texture = view.texture();
            let format_ok = if name == Names::GBUFFER_NORMAL {
                texture.format() == wgpu::TextureFormat::Rgba8Unorm
            } else {
                matches!(
                    texture.format(),
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb
                )
            };
            let size = texture.size();
            eprintln!(
                "MORPH_GBUFFER_SOURCE name={name} format={:?} size={size:?} samples={} usage={:?}",
                texture.format(),
                texture.sample_count(),
                texture.usage()
            );
            if !format_ok
                || texture.sample_count() != 1
                || texture.dimension() != wgpu::TextureDimension::D2
                || size.width != SIZE.x
                || size.height != SIZE.y
                || size.depth_or_array_layers != 1
                || !texture
                    .usage()
                    .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            {
                return Err(format!(
                    "MORPH_GBUFFER_READBACK unsupported physical source {name}"
                ));
            }
            views.push(view);
        }
        let native = gpu.native_context();
        let bytes = vec![0; (2 * PLANE_BYTES) as usize];
        let observed = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("morph-original-gbuffer-pixels"),
            contents: &bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let staging = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("morph-original-gbuffer-readback"),
            contents: &bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        });
        let shader = native.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("morph-gbuffer-pixel-transfer"),
            source: wgpu::ShaderSource::Wgsl(OBSERVE_WGSL.into()),
        });
        let pipeline = native.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("morph-gbuffer-pixel-transfer"),
            layout: None,
            module: &shader,
            entry_point: Some("observe"),
            compilation_options: Default::default(),
            cache: None,
        });
        let bindings = native.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("morph-gbuffer-pixel-transfer"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&views[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&views[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: observed.as_entire_binding(),
                },
            ],
        });
        {
            let mut pass = native
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("morph-gbuffer-pixel-transfer"),
                    timestamp_writes: None,
                });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bindings, &[]);
            pass.dispatch_workgroups(SIZE.x.div_ceil(8), SIZE.y.div_ceil(8), 1);
        }
        native
            .encoder
            .copy_buffer_to_buffer(&observed, 0, &staging, 0, 2 * PLANE_BYTES);
        *slot = Some(NativeGBuffer {
            staging,
            waiter: Box::new(native.readback_completion_waiter()),
        });
        Ok(()) // Serial by default; copy belongs to this product frame, not a diagnostic redraw.
    }
}

fn observer_feature() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "test.morph-gbuffer-observation",
        Vec::new(),
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            OBSERVER,
            QueueLane::Graphics,
        )
        .with_side_effects()
        .read_texture_from(Names::GBUFFER_ALBEDO, "gbuffer-mesh")
        .read_texture_from(Names::GBUFFER_NORMAL, "gbuffer-mesh")],
    )
}

#[test]
fn render_product_imported_generated_morph_frame_preserves_gbuffer_normal_orientation() {
    let root_uri = AssetUri::parse(ROOT_URI).unwrap();
    let importer = AssetImporter::default();
    let source = PathBuf::from(EXPAND_GLTF);
    assert_eq!(
        importer.descriptor_for_source(&source).unwrap().id,
        "zircon.builtin.model.gltf"
    );
    let mut outcome = importer
        .import_with_settings(&source, &root_uri, Default::default())
        .expect("collapsed base must import and expand through GPU morphing");
    let mesh_uri = AssetUri::parse(&format!("{ROOT_URI}#Mesh0/Primitive0")).unwrap();
    let imported = outcome
        .entries
        .iter()
        .find_map(|entry| match &entry.asset {
            ImportedAsset::Mesh(mesh) if entry.locator == mesh_uri => Some(mesh.clone()),
            _ => None,
        })
        .expect("real imported Mesh0/Primitive0");
    let manager = Arc::new(ProjectAssetManager::default());
    open_isolated_project(&manager);
    // The empty project lacks res://shaders/default_pbr. Only bind the real builtin
    // standard PBR template; imported mesh/deltas/material values/normal texture stay intact.
    let pbr = AssetUri::parse("builtin://shader/pbr.wgsl").unwrap();
    manager.load_shader_asset_by_uri(&pbr).unwrap();
    for entry in &mut outcome.entries {
        if let ImportedAsset::Material(material) = &mut entry.asset {
            let old = std::mem::replace(
                &mut material.shader,
                AssetReference::from_locator(pbr.clone()),
            );
            for dependency in &mut entry.dependencies {
                if *dependency == old.locator {
                    *dependency = pbr.clone();
                }
            }
        }
    }
    register_imported_payloads(&manager, outcome);
    let mailbox = Mailbox::default();
    // Register the executor, but add its GBuffer-dependent feature only to Deferred,
    // not the constructor's default Forward pipeline. Device/backend policy is unchanged.
    let framework = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::clone(&manager),
        Vec::new(),
        [RenderPassExecutorRegistration::new_executor(
            OBSERVER,
            Arc::new(GBufferObserver(Arc::clone(&mailbox))),
        )],
        Vec::new(),
    )
    .expect("native WGPU framework; the runner pins DX12 hardware, with no skip/fallback here");
    let mut pipeline =
        RenderPipelineAsset::default_deferred().with_plugin_render_features([observer_feature()]);
    pipeline.handle = RenderPipelineHandle::new(14_141);
    let pipeline = framework.register_pipeline_asset(pipeline).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(SIZE))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            morph_parity_quality_profile().with_pipeline_asset(pipeline),
        )
        .unwrap();
    let material_id =
        ResourceId::from_locator(&AssetUri::parse(&format!("{ROOT_URI}#Material0")).unwrap());
    let model_id = ResourceId::from_locator(&root_uri);
    let capture = |mesh_id, world, label| {
        capture_gbuffer(
            &framework,
            viewport,
            &mailbox,
            material_id,
            model_id,
            mesh_id,
            world,
            label,
        )
    };
    // Actual product result precedes any sign authoring or CPU w assertion.
    let actual = capture(
        ResourceId::from_locator(&mesh_uri),
        14_101,
        "imported-target1",
    );
    // T=+Y, N=+Z require w=-1 for B=dP/dv=+X. Controls differ ONLY in w;
    // neither recomputes xyz/deltas nor CPU-bakes positions.
    let positive = capture(
        register_signed_control(&manager, &imported, -1.0, "positive-v"),
        14_102,
        "w=-1",
    );
    let negative = capture(
        register_signed_control(&manager, &imported, 1.0, "negative-v"),
        14_103,
        "w=+1",
    );
    framework.destroy_viewport(viewport).unwrap();
    assert_normal_orientation(&actual, &positive, &negative);
}

fn open_isolated_project(manager: &ProjectAssetManager) {
    let unique = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_morph_orientation_{}_{unique}",
        std::process::id()
    ));
    assert!(root.is_absolute(), "runner TEMP must be absolute");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "GeneratedMorphOrientation",
        AssetUri::parse("res://scenes/unused.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .expect("activate TEMP project before renderer/cache construction");
    assert_eq!(
        manager.current_project_manager().unwrap().paths().root(),
        paths.root()
    );
    eprintln!(
        "MORPH_ORIENTATION_PROJECT active_root={}",
        paths.root().display()
    );
}

fn register_imported_payloads(manager: &ProjectAssetManager, outcome: AssetImportOutcome) {
    let mut batch = ResourceMutationBatch::new();
    for entry in outcome.entries {
        let record = ResourceRecord::new(
            ResourceId::from_locator(&entry.locator),
            asset_kind_for_imported_asset(&entry.asset),
            entry.locator,
        )
        .with_source_hash("mirrored-morph-expand-imported-v1")
        .with_dependency_ids(
            entry
                .dependencies
                .iter()
                .map(ResourceId::from_locator)
                .collect(),
        );
        batch = batch.upsert_imported_erased(record, entry.asset.into_resource_data());
    }
    manager
        .resource_manager()
        .commit(batch)
        .expect("publish real imported payloads and dependencies");
}

fn register_signed_control(
    manager: &ProjectAssetManager,
    imported: &MeshAsset,
    sign: f32,
    label: &str,
) -> ResourceId {
    let uri = AssetUri::parse(&format!("res://meshes/morph-orientation-{label}.zmesh")).unwrap();
    let mut mesh = imported.clone();
    mesh.uri = uri.clone();
    let Some(MeshAttributeValues::Float32x4(tangents)) =
        mesh.attributes.get_mut(MESH_ATTRIBUTE_TANGENT)
    else {
        panic!("imported generated tangent stream required for the sign-only control");
    };
    for tangent in tangents {
        tangent[3] = sign;
    }
    let id = ResourceId::from_locator(&uri);
    manager
        .assets::<MeshAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Mesh, uri)
                .with_source_hash(format!("mirrored-morph-expand-{label}-v1")),
            mesh,
        )
        .unwrap();
    id
}

#[allow(clippy::too_many_arguments)]
fn capture_gbuffer(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    mailbox: &Mailbox,
    material_id: ResourceId,
    model_id: ResourceId,
    mesh_id: ResourceId,
    world: u64,
    label: &str,
) -> GBuffer {
    let mut extract = direct_morph_parity_extract(material_id, mesh_id, world, true);
    let mut mesh = direct_morph_parity_mesh_snapshot(material_id, mesh_id, true);
    mesh.model = ResourceHandle::<ModelMarker>::new(model_id);
    mesh.transform = Transform {
        translation: Vec3::new(-1.6, -1.6, 0.0),
        scale: Vec3::splat(3.2),
        ..Transform::default()
    };
    *extract.geometry = GeometryExtract::from_meshes(extract.view.core_pipeline, vec![mesh]);
    extract.debug.overlays.display_mode = DisplayMode::Shaded;
    extract.post_process.display_mode = DisplayMode::Shaded;
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("completed product frame");
    let stats = framework.query_stats().unwrap();
    eprintln!("MORPH_GBUFFER_DRAW label={label} generation={} gpu_morph={} prepared_draws={} material_ready={} fallback={} pipeline_pending={} executed={:?} shader_report={:?}", frame.generation,
        stats.last_mesh_gpu_morphed_source_draw_count, stats.last_mesh_draw_count,
        stats.last_material_ready_count, stats.last_material_fallback_count,
        stats.last_pipeline_async_pending_count, stats.last_graph_executed_executor_ids, stats.last_shader_variant_miss_report);
    let NativeGBuffer { staging, waiter } = mailbox
        .lock()
        .unwrap()
        .take()
        .expect("MORPH_GBUFFER_READBACK observer did not execute");
    let (sender, receiver) = std::sync::mpsc::channel();
    staging
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    waiter().expect("MORPH_GBUFFER_READBACK completion wait failed");
    receiver
        .try_recv()
        .expect("MORPH_GBUFFER_READBACK map not terminal")
        .expect("MORPH_GBUFFER_READBACK map failed");
    let mapped = staging.slice(..).get_mapped_range();
    assert_eq!(mapped.len(), (2 * PLANE_BYTES) as usize);
    let mut albedo: Vec<[u8; 4]> = mapped
        .chunks_exact(4)
        .map(|pixel| pixel.try_into().unwrap())
        .collect();
    let normal = albedo.split_off((SIZE.x * SIZE.y) as usize);
    drop(mapped);
    staging.unmap();
    // Prepared counters alone are not proof of rasterization; coverage is checked separately.
    assert_eq!(
        stats.last_mesh_gpu_morphed_source_draw_count, 1,
        "{label}: actual GPU morph source"
    );
    assert_eq!(stats.last_mesh_draw_count, 1);
    assert_eq!(stats.last_material_ready_count, 1);
    assert_eq!(stats.last_material_fallback_count, 0);
    assert_eq!(stats.last_material_validation_error_count, 0);
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );
    for executor in ["deferred.gbuffer", OBSERVER] {
        assert!(
            stats
                .last_graph_executed_executor_ids
                .iter()
                .any(|id| id == executor),
            "{label}: missing {executor}"
        );
    }
    GBuffer { albedo, normal }
}

fn assert_normal_orientation(actual: &GBuffer, positive: &GBuffer, negative: &GBuffer) {
    // Normal clears to BLACK with alpha=1: it is NOT a coverage channel. Albedo
    // clears to TRANSPARENT and the opaque production surface writes alpha=1.
    let covered = |image: &GBuffer| image.albedo.iter().filter(|p| p[3] >= 240).count();
    let mut sufficient = true;
    for (label, image) in [
        ("imported-target1", actual),
        ("w=-1", positive),
        ("w=+1", negative),
    ] {
        let count = covered(image);
        let red: Vec<_> = image
            .normal
            .iter()
            .zip(&image.albedo)
            .filter(|(_, a)| a[3] >= 240)
            .map(|(n, _)| n[0])
            .collect();
        let status = if count == 0 {
            "NO_COVERAGE"
        } else if count < 64 {
            "INSUFFICIENT_COVERAGE"
        } else {
            "COVERED"
        };
        eprintln!("MORPH_GBUFFER_COVERAGE label={label} status={status} pixels={count} normal_r_min={:?} normal_r_max={:?}", red.iter().min(), red.iter().max());
        sufficient &= count >= 64;
    }
    assert!(sufficient, "MORPH_GBUFFER_NO_OR_INSUFFICIENT_COVERAGE: no direction verdict; prepared draw stats do not establish rasterization");
    let mask: Vec<_> = (0..positive.albedo.len())
        .filter(|&i| positive.albedo[i][3] >= 240 && negative.albedo[i][3] >= 240)
        .collect();
    assert!(
        mask.len() >= 64,
        "MORPH_GBUFFER_CONTROL_COVERAGE_MISMATCH: no direction verdict"
    );
    let actual_coverage = mask.iter().filter(|&&i| actual.albedo[i][3] >= 240).count();
    assert!(
        actual_coverage * 100 >= mask.len() * 95,
        "MORPH_GBUFFER_ACTUAL_COVERAGE_MISMATCH: no direction verdict"
    );
    // Production encode_gbuffer stores n*0.5+0.5 in linear RGBA8. GL normal
    // (128,204) decodes to (0.00392,0.6,~0.8): T=+Y/B=+X => (204,128,~230).
    // A w-only reversal gives (51,128,~230). No illumination or display transfer.
    let matches = |image: &GBuffer, expected: [u8; 3]| {
        mask.iter()
            .filter(|&&i| {
                image.normal[i][..3]
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| a.abs_diff(b) <= 6)
            })
            .count()
    };
    let positive_ok = matches(positive, [204, 128, 230]);
    let negative_ok = matches(negative, [51, 128, 230]);
    let actual_positive = matches(actual, [204, 128, 230]);
    let actual_negative = matches(actual, [51, 128, 230]);
    eprintln!("MORPH_GBUFFER_NORMAL mask={} control_positive={positive_ok} control_negative={negative_ok} actual_positive={actual_positive} actual_negative={actual_negative}", mask.len());
    assert!(positive_ok * 100 >= mask.len() * 95 && negative_ok * 100 >= mask.len() * 95,
        "MORPH_GBUFFER_CONTROLS_NOT_ORIENTED: actual production normal path has not established sign discrimination; no imported-w verdict");
    if actual_positive * 100 < mask.len() * 95 {
        assert!(actual_negative * 100 < mask.len() * 95,
            "MORPH_GBUFFER_OLD_WRONG_W: imported target1 matches w=+1 / B=-X, not authored w=-1 / dP/dv=+X");
        panic!("MORPH_GBUFFER_ACTUAL_NORMAL_MISMATCH: covered but matches neither signed control; do not attribute this to w alone");
    }
    eprintln!("MORPH_GBUFFER_ORIENTATION_OK: actual target1 agrees with authored B=+X");
}
