use super::*;
use crate::core::framework::render::{ShaderVariantPrewarmSource, GEOMETRY_SOURCE_ID_STATIC_MESH};
use crate::core::resource::{ResourceKind, ResourceLocator, ResourceRecord};

use super::super::super::prewarm_manifest::pipeline_key_from_prewarm_request;

#[test]
fn runtime_prewarm_reuses_draw_publication_and_invalidates_only_changed_dependencies() {
    let backend =
        RenderBackend::new_offscreen().expect("prewarm acceptance requires an offscreen adapter");
    let (system_textures, _) = backend.acquire_system_texture_lease().unwrap();
    let RenderBackend { device, queue, .. } = backend;
    let assets = Arc::new(ProjectAssetManager::default());
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer =
        ResourceStreamer::new_for_test(assets.clone(), &device, &queue, &texture_layout);
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let manifest = builtin_fallback_shader_prewarm_manifest();
    let mut request = manifest
        .variants
        .iter()
        .find(|request| {
            request.key.pass_type == ShaderPassType::Forward
                && request.key.geometry_source == GEOMETRY_SOURCE_ID_STATIC_MESH
        })
        .cloned()
        .expect("builtin forward prewarm request");
    let source = manifest.source_for(&request).unwrap().clone();
    let resources = assets.resource_manager();
    let dependency_locator = ResourceLocator::parse("res://prewarm/dependency.asset").unwrap();
    let dependency = ResourceRecord::new(
        ResourceId::from_locator(&dependency_locator),
        ResourceKind::Data,
        dependency_locator,
    )
    .with_source_hash("v1");
    resources.register_ready(dependency.clone(), ()).unwrap();
    let shader = assets
        .load_shader_asset_snapshot(request.key.material_shader)
        .unwrap();
    let mut record = resources
        .registry()
        .get(request.key.material_shader)
        .cloned()
        .unwrap();
    record.dependency_ids.push(dependency.id);
    resources.register_ready(record, (*shader).clone()).unwrap();
    request.key.material_revision = resources
        .registry()
        .get(request.key.material_shader)
        .unwrap()
        .revision;
    let manifest = ShaderVariantPrewarmManifest::new(vec![source], vec![request.clone()]);

    let first = cache.prewarm_manifest(&device, &mut streamer, &manifest);
    assert_eq!(first.failed_count(), 0, "{:?}", first.failures());
    assert_eq!(first.ready_count(), 1);
    assert_eq!(first.cache_hit_count(), 0);
    let (_, revision, identity, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))
        .unwrap();
    let mut draw_key = pipeline_key_from_prewarm_request(&request).unwrap();
    draw_key.shader_revision = revision;
    draw_key.shader_dependency_identity = Some(identity);
    let first_variant = cache.resolve_variant_for_geometry(
        MeshPassPipelineKind::Base,
        &draw_key,
        request.key.geometry_source,
        request.key.quality,
    );
    let created = cache
        .shader_variant_miss_report()
        .render_pipeline_creation_count;
    assert!(ensure_base_pipeline_after_shader_validation(
        &mut cache,
        &device,
        &streamer,
        first_variant
    )
    .is_ready());
    assert_eq!(
        cache
            .shader_variant_miss_report()
            .render_pipeline_creation_count,
        created
    );

    let unrelated = ResourceLocator::parse("res://prewarm/unrelated.asset").unwrap();
    resources
        .register_ready(
            ResourceRecord::new(
                ResourceId::from_locator(&unrelated),
                ResourceKind::Data,
                unrelated,
            ),
            (),
        )
        .unwrap();
    let unchanged = cache.prewarm_manifest(&device, &mut streamer, &manifest);
    assert_eq!(unchanged.cache_hit_count(), 1);
    assert_eq!(
        cache
            .shader_variant_miss_report()
            .render_pipeline_creation_count,
        created
    );

    resources
        .register_ready(dependency.with_source_hash("v2"), ())
        .unwrap();
    let changed = cache.prewarm_manifest(&device, &mut streamer, &manifest);
    assert_eq!(changed.failed_count(), 0, "{:?}", changed.failures());
    assert_eq!(changed.ready_count(), 1);
    assert_eq!(changed.cache_hit_count(), 0);
    let (_, changed_revision, changed_identity, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))
        .unwrap();
    assert_eq!(changed_revision, revision);
    draw_key.shader_dependency_identity = Some(changed_identity);
    let next_variant = cache.resolve_variant_for_geometry(
        MeshPassPipelineKind::Base,
        &draw_key,
        request.key.geometry_source,
        request.key.quality,
    );
    assert_ne!(first_variant, next_variant);
    let recreated = cache
        .shader_variant_miss_report()
        .render_pipeline_creation_count;
    assert_eq!(recreated, created + 1);
    assert!(ensure_base_pipeline_after_shader_validation(
        &mut cache,
        &device,
        &streamer,
        next_variant
    )
    .is_ready());
    assert_eq!(
        cache
            .shader_variant_miss_report()
            .render_pipeline_creation_count,
        recreated
    );

    let mut stale = manifest.clone();
    stale.variants[0].key.material_revision += 1;
    assert_eq!(
        cache
            .prewarm_manifest(&device, &mut streamer, &stale)
            .failed_count(),
        1
    );
    let original = &manifest.sources[0];
    let foreign_source = ShaderVariantPrewarmSource::new(
        "different content",
        format!("{}\n// different publication", original.wgsl_source),
        original.include_content_hashes.clone(),
        original.template_revision.clone(),
        original.naga_version.clone(),
        original.wgpu_version.clone(),
    );
    let mut foreign_request = request;
    foreign_request.source_id = foreign_source.id.clone();
    let foreign_manifest =
        ShaderVariantPrewarmManifest::new(vec![foreign_source], vec![foreign_request]);
    assert_eq!(
        cache
            .prewarm_manifest(&device, &mut streamer, &foreign_manifest)
            .failed_count(),
        1
    );
    assert_eq!(
        cache
            .shader_variant_miss_report()
            .render_pipeline_creation_count,
        recreated
    );
}
