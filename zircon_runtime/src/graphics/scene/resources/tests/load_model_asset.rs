use std::sync::Arc;

use super::load_model_asset_with_cache;
use crate::asset::{
    AssetUri, MeshAsset, MeshVertex, ModelAsset, ModelPrimitiveAsset, ProjectAssetManager,
    VirtualGeometryAsset,
};
use crate::core::math::{Vec2, Vec3};
use crate::core::resource::{
    MeshMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::ResourceStreamer;

const ENSURE_MODEL_SOURCE: &str = include_str!("../resource_streamer/ensure_model.rs");

#[test]
fn ensure_model_consumes_the_shared_asset_without_rewrapping_it() {
    let body = ENSURE_MODEL_SOURCE
        .split("pub(crate) fn ensure_model(")
        .nth(1)
        .expect("model preparation entry must remain present");
    let compact = body
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    assert!(compact.contains("self.load_model_asset(id)"));
    assert_eq!(compact.matches(".load_model_asset(id)").count(), 1);
    assert!(!compact.contains("Arc::new(asset_manager"));
}

#[test]
fn current_prepared_model_asset_short_circuits_fallback_loading() {
    let cached = Arc::new(cooked_model_asset("res://models/cached.model.toml"));

    let loaded = load_model_asset_with_cache(Some((&cached, 7)), Some(7), || {
        panic!("current prepared asset should short-circuit fallback loading")
    });

    assert!(
        Arc::ptr_eq(
            loaded.as_ref().expect("current prepared asset must load"),
            &cached,
        ),
        "current prepared model asset must be borrowed through the existing Arc"
    );
    let loaded_again = load_model_asset_with_cache(Some((&cached, 7)), Some(7), || {
        panic!("repeated current prepared hit should not invoke fallback loading")
    });
    assert!(Arc::ptr_eq(
        loaded_again
            .as_ref()
            .expect("repeated current prepared asset must load"),
        &cached,
    ));
}

#[test]
fn composite_geometry_revision_does_not_invalidate_source_asset_cache() {
    let cached = Arc::new(cooked_model_asset("res://models/external-mesh.model.toml"));
    let composite_geometry_revision = 0xfeed_beef_u64;
    let source_revision = 7;
    assert_ne!(composite_geometry_revision, source_revision);

    let loaded = load_model_asset_with_cache(
        Some((&cached, source_revision)),
        Some(source_revision),
        || panic!("composite geometry revision must not force source asset reload"),
    );

    assert!(Arc::ptr_eq(
        loaded.as_ref().expect("current prepared asset must load"),
        &cached,
    ));
}

#[test]
fn stale_prepared_model_asset_falls_back_to_latest_asset_load() {
    let cached = Arc::new(cooked_model_asset("res://models/cached.model.toml"));
    let fresh = plain_model_asset("res://models/fresh.model.toml");

    let loaded = load_model_asset_with_cache(Some((&cached, 7)), Some(8), || {
        Some(Arc::new(fresh.clone()))
    });

    let loaded = loaded.expect("stale model asset must use fallback");
    assert!(!Arc::ptr_eq(&loaded, &cached));
    assert_eq!(loaded.as_ref(), &fresh);
}

#[test]
fn model_cpu_admission_survives_real_gpu_stage_and_ensure_upload() {
    let (backend, assets, mut streamer) = admission_upload_streamer();
    for (name, staged) in [("staged", true), ("ensured", false)] {
        let uri = AssetUri::parse(&format!("res://models/admission-{name}.model.toml")).unwrap();
        let id = ResourceId::from_locator(&uri);
        let model = ModelAsset {
            uri: uri.clone(),
            primitives: vec![admission_upload_primitive()],
        };
        assets
            .assets::<ModelAsset>()
            .insert(ResourceRecord::new(id, ResourceKind::Model, uri), model)
            .expect("publish model through the resource owner");
        let handle = ResourceHandle::<ModelMarker>::new(id);
        let admitted = streamer
            .admit_model_for_virtual_geometry(handle)
            .expect("CPU model admission");
        let asset = Arc::clone(&streamer.models[&id].asset);
        assert!(streamer.models[&id].resource.is_none());
        if staged {
            let projection = assets.resource_manager().projection_snapshot();
            let prepared = streamer
                .stage_model(&backend.device, id, &projection)
                .expect("stage actual GPU model")
                .expect("CPU-only model requires upload");
            assert!(streamer.models[&id].resource.is_none());
            assert!(Arc::ptr_eq(&admitted, &prepared.admission));
            assert!(Arc::ptr_eq(&asset, &prepared.asset));
            let gpu = prepared.resource.as_ref().expect("actual GPU model");
            assert_eq!(gpu.meshes[0].index_count, 3);
            assert!(Arc::ptr_eq(
                &admitted,
                gpu.meshes[0]
                    .source_admission()
                    .expect("GPU provenance")
                    .source()
            ));
            streamer.models.insert(id, prepared);
            assert!(streamer
                .stage_model(&backend.device, id, &projection)
                .expect("repeat stage")
                .is_none());
        } else {
            streamer
                .ensure_model(&backend.device, handle)
                .expect("ensure actual GPU model");
        }
        let prepared = &streamer.models[&id];
        assert!(Arc::ptr_eq(&admitted, &prepared.admission));
        assert!(Arc::ptr_eq(&asset, &prepared.asset));
        let gpu = Arc::clone(prepared.resource.as_ref().expect("uploaded GPU model"));
        assert_eq!(gpu.meshes[0].index_count, 3);
        assert!(Arc::ptr_eq(
            &admitted,
            gpu.meshes[0]
                .source_admission()
                .expect("GPU provenance")
                .source()
        ));
        streamer
            .ensure_model(&backend.device, handle)
            .expect("repeat ensure");
        assert!(Arc::ptr_eq(
            &gpu,
            streamer.models[&id].resource.as_ref().unwrap()
        ));
    }
}

#[test]
fn mesh_cpu_admission_survives_real_gpu_stage_upload() {
    let (backend, assets, mut streamer) = admission_upload_streamer();
    let uri = AssetUri::parse("res://meshes/admission-stage.zmesh").unwrap();
    let id = ResourceId::from_locator(&uri);
    let mesh = MeshAsset::from_model_primitive(uri.clone(), &admission_upload_primitive());
    assets
        .assets::<MeshAsset>()
        .insert(ResourceRecord::new(id, ResourceKind::Mesh, uri), mesh)
        .expect("publish mesh through the resource owner");
    let handle = ResourceHandle::<MeshMarker>::new(id);
    let admitted = streamer
        .admit_mesh_for_virtual_geometry(handle)
        .expect("CPU mesh admission");
    let asset = Arc::clone(&streamer.meshes[&id].asset);
    assert!(streamer.meshes[&id].resource.is_none());
    let projection = assets.resource_manager().projection_snapshot();
    let prepared = streamer
        .stage_mesh(&backend.device, id, &projection)
        .expect("stage actual GPU mesh")
        .expect("CPU-only mesh requires upload");
    assert!(streamer.meshes[&id].resource.is_none());
    assert!(Arc::ptr_eq(&admitted, &prepared.admission));
    assert!(Arc::ptr_eq(&asset, &prepared.asset));
    let gpu = Arc::clone(prepared.resource.as_ref().expect("uploaded GPU mesh"));
    assert_eq!(gpu.index_count, 3);
    assert!(Arc::ptr_eq(
        &admitted,
        gpu.source_admission().expect("GPU provenance").source()
    ));
    streamer.meshes.insert(id, prepared);
    assert!(streamer
        .stage_mesh(&backend.device, id, &projection)
        .expect("repeat stage")
        .is_none());
    streamer
        .ensure_mesh(&backend.device, handle)
        .expect("repeat ensure");
    assert!(Arc::ptr_eq(
        &gpu,
        streamer.meshes[&id].resource.as_ref().unwrap()
    ));
}

fn admission_upload_primitive() -> ModelPrimitiveAsset {
    ModelPrimitiveAsset {
        vertices: vec![
            MeshVertex::new(Vec3::ZERO, Vec3::Z, Vec2::ZERO),
            MeshVertex::new(Vec3::X, Vec3::Z, Vec2::X),
            MeshVertex::new(Vec3::Y, Vec3::Z, Vec2::Y),
        ],
        indices: vec![0, 1, 2],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    }
}

fn admission_upload_streamer() -> (RenderBackend, Arc<ProjectAssetManager>, ResourceStreamer) {
    let backend =
        RenderBackend::new_offscreen().expect("admission upload requires a real GPU device");
    let layout = backend
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("source-admission-upload-test-texture-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
    let assets = Arc::new(ProjectAssetManager::default());
    let streamer = ResourceStreamer::new_for_test(
        Arc::clone(&assets),
        &backend.device,
        &backend.queue,
        &layout,
    );
    (backend, assets, streamer)
}

#[test]
#[ignore = "current-source realistic-payload pointer-reuse profile"]
fn shader06_current_model_asset_hits_reuse_realistic_payload_storage() {
    const INSTANCE_COUNT: usize = 16_384;
    const VERTEX_COUNT: usize = 262_144;
    const INDEX_COUNT: usize = 786_432;

    let mut model = plain_model_asset("res://models/current-source-profile.model.toml");
    model.primitives[0].vertices =
        vec![MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO); VERTEX_COUNT];
    model.primitives[0].indices = (0..INDEX_COUNT)
        .map(|index| (index % VERTEX_COUNT) as u32)
        .collect();
    let cached = Arc::new(model);
    let vertex_pointer = cached.primitives[0].vertices.as_ptr();
    let index_pointer = cached.primitives[0].indices.as_ptr();
    let payload_bytes = std::mem::size_of_val(cached.primitives[0].vertices.as_slice())
        + std::mem::size_of_val(cached.primitives[0].indices.as_slice());

    for _ in 0..INSTANCE_COUNT {
        let loaded = load_model_asset_with_cache(Some((&cached, 11)), Some(11), || {
            panic!("current prepared model must not deep-load the realistic payload")
        })
        .expect("current prepared model must remain available");
        assert!(Arc::ptr_eq(&loaded, &cached));
        assert_eq!(loaded.primitives[0].vertices.as_ptr(), vertex_pointer);
        assert_eq!(loaded.primitives[0].indices.as_ptr(), index_pointer);
    }

    println!(
        "PERF_RESULT shader06_model_asset_arc_reuse instances={INSTANCE_COUNT} vertices={VERTEX_COUNT} indices={INDEX_COUNT} payload_bytes={payload_bytes} pointer_reuses={INSTANCE_COUNT} fallback_loads=0"
    );
}

fn cooked_model_asset(uri: &str) -> ModelAsset {
    ModelAsset {
        uri: AssetUri::parse(uri).unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: Some(VirtualGeometryAsset {
                root_page_table: vec![1],
                ..VirtualGeometryAsset::default()
            }),
        }],
    }
}

fn plain_model_asset(uri: &str) -> ModelAsset {
    ModelAsset {
        uri: AssetUri::parse(uri).unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    }
}
