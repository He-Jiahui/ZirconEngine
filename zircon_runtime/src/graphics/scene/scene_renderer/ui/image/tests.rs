use super::*;

#[test]
fn screen_space_ui_image_vertex_buffer_uses_a_small_minimum_growth_capacity() {
    assert_eq!(image_vertex_buffer_capacity(1), 4 * 1024);
    assert_eq!(image_vertex_buffer_capacity(4 * 1024), 4 * 1024);
    assert_eq!(image_vertex_buffer_capacity(4 * 1024 + 1), 8 * 1024);
}

#[test]
fn screen_space_ui_image_vertex_buffer_reallocates_only_when_required() {
    assert!(!image_vertex_buffer_requires_reallocation(
        4 * 1024,
        4 * 1024
    ));
    assert!(image_vertex_buffer_requires_reallocation(
        4 * 1024,
        4 * 1024 + 1
    ));
}

#[test]
fn screen_space_ui_image_vertex_buffer_writes_only_for_new_payloads_or_reallocation() {
    let payload = [7; 32];
    assert!(!image_vertex_buffer_write_required(
        false,
        Some(payload),
        payload
    ));
    assert!(image_vertex_buffer_write_required(
        false,
        Some(payload),
        [8; 32]
    ));
    assert!(image_vertex_buffer_write_required(
        true,
        Some(payload),
        payload
    ));
    assert!(image_vertex_buffer_write_required(false, None, payload));
}

#[test]
fn screen_space_ui_image_releases_cpu_staging_when_no_batch_is_renderable() {
    assert!(image_cpu_staging_should_reset(0));
    assert!(!image_cpu_staging_should_reset(1));
}

#[test]
fn screen_space_ui_image_binding_cache_idle_clear_releases_bucket_storage() {
    let mut cache = ScreenSpaceUiImageBindingCache {
        next_prepare_epoch: 7,
        bindings: std::collections::HashMap::with_capacity(64),
    };

    assert!(cache.bindings.capacity() >= 64);
    cache.clear();

    assert_eq!(cache.next_prepare_epoch, 7);
    assert_eq!(cache.bindings.capacity(), 0);
}

#[test]
fn screen_space_ui_image_binding_cache_keeps_recent_idle_epochs_bounded() {
    assert!(binding_cache_epoch_is_recent(10, 8));
    assert!(binding_cache_epoch_is_recent(10, 10));
    assert!(!binding_cache_epoch_is_recent(10, 7));
}

#[test]
fn screen_space_ui_image_binding_cache_never_trims_the_active_epoch() {
    assert!(!binding_cache_entry_is_trimmable(10, 10));
    assert!(binding_cache_entry_is_trimmable(10, 9));
}

#[test]
fn screen_space_ui_image_binding_cache_epoch_wrap_does_not_keep_old_entries_forever() {
    assert!(binding_cache_epoch_is_recent(u64::MAX, u64::MAX));
    assert!(!binding_cache_epoch_is_recent(1, u64::MAX));
}

#[test]
fn screen_space_ui_image_idle_clear_releases_transient_cpu_staging() {
    let mut textures = ScreenSpaceUiImagePrepareTextureCache {
        generation: None,
        resolved_texture_ids: std::collections::HashMap::with_capacity(64),
    };
    let payload_hash = [7; 32];
    let mut vertices = ScreenSpaceUiImageVertexBuffer {
        buffer: None,
        capacity_bytes: 4 * 1024,
        payload_hash: Some(payload_hash),
        vertices: Vec::with_capacity(128),
    };

    textures.reset();
    vertices.clear_cpu_staging();

    assert_eq!(textures.resolved_texture_ids.capacity(), 0);
    assert_eq!(vertices.vertices.capacity(), 0);
    assert_eq!(vertices.capacity_bytes, 4 * 1024);
    assert_eq!(vertices.payload_hash, Some(payload_hash));
    assert!(vertices.buffer.is_none());
}

#[test]
fn screen_space_ui_image_resolution_cache_reuses_one_management_generation() {
    let generation = Arc::new(ResourceManagementGeneration::default());
    let requested = ResourceId::from_stable_label("res://ui/stable.svg");
    let resolved = ResourceId::from_stable_label("asset://ui/stable.svg");
    let mut cache = ScreenSpaceUiImagePrepareTextureCache::default();

    assert!(cache.begin_prepare(Some(Arc::clone(&generation))));
    cache.resolved_texture_ids.insert(requested, Some(resolved));
    assert!(!cache.begin_prepare(Some(generation)));

    assert_eq!(
        cache.resolved_texture_ids.get(&requested),
        Some(&Some(resolved))
    );
}

#[test]
fn screen_space_ui_image_resolution_cache_invalidates_a_new_management_generation() {
    let requested = ResourceId::from_stable_label("res://ui/reloaded.svg");
    let mut cache = ScreenSpaceUiImagePrepareTextureCache::default();

    assert!(cache.begin_prepare(Some(Arc::new(ResourceManagementGeneration::default()))));
    cache.resolved_texture_ids.insert(requested, None);
    assert!(cache.begin_prepare(Some(Arc::new(ResourceManagementGeneration::default()))));

    assert!(cache.resolved_texture_ids.is_empty());
}

#[test]
fn screen_space_ui_image_resolution_cache_does_not_retain_missing_authority() {
    let requested = ResourceId::from_stable_label("res://ui/pending.svg");
    let mut cache = ScreenSpaceUiImagePrepareTextureCache::default();

    cache.resolved_texture_ids.insert(requested, None);
    assert!(cache.begin_prepare(None));
    assert!(cache.resolved_texture_ids.is_empty());

    cache.resolved_texture_ids.insert(requested, None);
    assert!(cache.begin_prepare(None));
    assert!(cache.resolved_texture_ids.is_empty());
}

#[test]
fn screen_space_ui_image_segment_cache_reuses_exact_plan_and_viewport() {
    let plan = Arc::new(PlannedScreenSpaceUi::default());
    let viewport_size = UVec2::new(1280, 720);
    let current = Arc::downgrade(&plan);

    assert!(screen_space_ui_image_segment_plan_reused(
        Some(&current),
        viewport_size,
        &plan,
        viewport_size,
    ));
}

#[test]
fn screen_space_ui_image_segment_cache_invalidates_plan_or_viewport_change() {
    let plan = Arc::new(PlannedScreenSpaceUi::default());
    let replacement = Arc::new(PlannedScreenSpaceUi::default());
    let current = Arc::downgrade(&plan);

    assert!(!screen_space_ui_image_segment_plan_reused(
        Some(&current),
        UVec2::new(1280, 720),
        &replacement,
        UVec2::new(1280, 720),
    ));
    assert!(!screen_space_ui_image_segment_plan_reused(
        Some(&current),
        UVec2::new(1280, 720),
        &plan,
        UVec2::new(1920, 1080),
    ));
}

#[test]
fn screen_space_ui_image_segment_geometry_deduplicates_texture_dependencies() {
    let viewport = UiFrame::new(0.0, 0.0, 1280.0, 720.0);
    let texture = ResourceId::from_stable_label("res://ui/shared.svg");
    let batches = [
        ScreenSpaceUiImageBatch {
            texture,
            frame: UiFrame::new(10.0, 20.0, 30.0, 40.0),
            clip_frame: None,
            tint: [1.0; 4],
        },
        ScreenSpaceUiImageBatch {
            texture,
            frame: UiFrame::new(50.0, 60.0, 70.0, 80.0),
            clip_frame: None,
            tint: [1.0; 4],
        },
    ];
    let mut dependency_indices = HashMap::new();
    let mut dependencies = Vec::new();
    let mut vertices = Vec::new();

    let images: Vec<_> = batches
        .iter()
        .filter_map(|batch| {
            ScreenSpaceUiImageSystem::prepare_batch_geometry(
                viewport,
                batch,
                &mut dependency_indices,
                &mut dependencies,
                &mut vertices,
            )
        })
        .collect();

    assert_eq!(images.len(), 2);
    assert_eq!(dependencies.len(), 1);
    assert!(images.iter().all(|image| image.dependency_index == 0));
    assert_eq!(vertices.len(), 12);
}

#[test]
fn screen_space_ui_image_texture_dependency_uses_exact_gpu_identity() {
    let current = Arc::new(7_u8);
    let same = Arc::clone(&current);
    let equal_but_replaced = Arc::new(7_u8);

    assert!(screen_space_ui_image_texture_dependency_is_current(
        Some(&current),
        &same,
    ));
    assert!(!screen_space_ui_image_texture_dependency_is_current(
        Some(&current),
        &equal_but_replaced,
    ));
    assert!(!screen_space_ui_image_texture_dependency_is_current(
        None, &same,
    ));
}

#[test]
fn screen_space_ui_image_prepared_draw_uses_a_compact_dependency_index() {
    let source = include_str!("../image.rs");
    let prepared_definition = source
        .split_once("pub(super) struct PreparedScreenSpaceUiImage {")
        .and_then(|(_, remainder)| remainder.split_once("#[derive"))
        .map(|(definition, _)| definition)
        .expect("prepared UI image definition must remain present");
    let prepare_batch = source
        .split_once("fn prepare_batch_geometry(")
        .and_then(|(_, remainder)| remainder.split_once("fn refresh_segment_dependencies"))
        .map(|(definition, _)| definition)
        .expect("UI image prepare path must remain present");
    let handle = ScreenSpaceUiImageBindingHandle(17);

    assert_eq!(handle, ScreenSpaceUiImageBindingHandle(17));
    assert_eq!(
        std::mem::size_of::<ScreenSpaceUiImageBindingHandle>(),
        std::mem::size_of::<usize>()
    );
    assert!(prepared_definition.contains("dependency_index"));
    assert!(!prepared_definition.contains("Arc<"));
    assert!(!prepare_batch.contains("Arc::clone"));
}

#[test]
fn screen_space_ui_image_clip_outside_viewport_skips_the_batch() {
    let image_frame = UiFrame::new(8.0, 12.0, 40.0, 30.0);
    let viewport = UiFrame::new(0.0, 0.0, 100.0, 80.0);
    let clip_outside_viewport = UiFrame::new(120.0, 0.0, 20.0, 20.0);

    assert!(image_batch_scissor(image_frame, viewport, Some(clip_outside_viewport)).is_none());
}

#[test]
fn screen_space_ui_image_without_clip_uses_the_full_viewport_scissor() {
    let image_frame = UiFrame::new(8.0, 12.0, 40.0, 30.0);
    let viewport = UiFrame::new(0.0, 0.0, 100.0, 80.0);

    let scissor =
        image_batch_scissor(image_frame, viewport, None).expect("viewport scissor should exist");
    assert_eq!(
        (scissor.x, scissor.y, scissor.width, scissor.height),
        (0, 0, 100, 80)
    );
}

#[test]
fn screen_space_ui_image_clip_intersects_the_viewport_before_scissoring() {
    let image_frame = UiFrame::new(60.0, 40.0, 60.0, 60.0);
    let viewport = UiFrame::new(0.0, 0.0, 100.0, 80.0);
    let partially_visible_clip = UiFrame::new(80.0, 60.0, 40.0, 30.0);

    let scissor = image_batch_scissor(image_frame, viewport, Some(partially_visible_clip))
        .expect("intersecting clip should produce a scissor");
    assert_eq!(
        (scissor.x, scissor.y, scissor.width, scissor.height),
        (80, 60, 20, 20)
    );
}

#[test]
fn screen_space_ui_image_clip_missing_the_image_frame_skips_the_batch() {
    let image_frame = UiFrame::new(8.0, 12.0, 40.0, 30.0);
    let viewport = UiFrame::new(0.0, 0.0, 100.0, 80.0);
    let clip_outside_image_frame = UiFrame::new(60.0, 0.0, 20.0, 20.0);

    assert!(image_batch_scissor(image_frame, viewport, Some(clip_outside_image_frame)).is_none());
}
