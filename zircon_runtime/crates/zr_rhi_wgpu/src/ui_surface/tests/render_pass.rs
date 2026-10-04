use zr_rhi::{UiSurfaceDrawList, UiSurfaceRect};

use super::{
    damage_scissor, draw_bounds_intersect_scissor, next_ui_vertex_buffer_capacity,
    ui_vertex_buffer_upload_action, vertex_buffer_needs_reallocation, UiVertexBufferUploadAction,
    WgpuUiDrawBufferCache, WgpuUiRecordedDrawStats,
};

#[test]
fn recorded_draw_stats_count_only_submitted_work_and_unique_layers() {
    let mut stats = WgpuUiRecordedDrawStats::default();

    stats.record_draw(2, 3, 18, 3, 0);
    stats.record_draw(2, 1, 0, 0, 6);
    stats.record_draw(5, 2, 0, 0, 0);

    assert_eq!(stats.draw_calls, 3);
    assert_eq!(stats.visible_draw_item_count, 6);
    assert_eq!(stats.solid_vertex_count, 18);
    assert_eq!(stats.solid_instance_count, 3);
    assert_eq!(stats.image_vertex_count, 6);
    assert_eq!(stats.batch_layer_count, 2);
}

#[test]
fn draw_buffer_cache_key_allows_a_versioned_damage_projection() {
    let versioned = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 9);
    let damaged = UiSurfaceDrawList::with_generation(
        (64, 32),
        Some(UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0)),
        Vec::new(),
        9,
    );
    let legacy = UiSurfaceDrawList::new((64, 32), None, Vec::new());

    assert!(WgpuUiDrawBufferCache::cache_key(&versioned).is_some());
    assert!(WgpuUiDrawBufferCache::cache_key(&damaged).is_some());
    assert_eq!(WgpuUiDrawBufferCache::cache_key(&legacy), None);
}

#[test]
fn draw_buffer_cache_key_ignores_target_only_resize() {
    let mut draw_list = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 9);
    let original = WgpuUiDrawBufferCache::cache_key(&draw_list);

    draw_list.retarget_surface_size_preserving_projection((32, 16));

    assert_eq!(WgpuUiDrawBufferCache::cache_key(&draw_list), original);
}

#[test]
fn damage_scissor_clamps_to_the_surface_and_rejects_empty_regions() {
    assert_eq!(
        damage_scissor(Some(UiSurfaceRect::new(-2.0, 4.2, 9.0, 9.0)), (10, 10)),
        Some((0, 4, 7, 6))
    );
    assert_eq!(
        damage_scissor(Some(UiSurfaceRect::new(12.0, 0.0, 1.0, 1.0)), (10, 10)),
        None
    );
    assert_eq!(damage_scissor(None, (10, 10)), Some((0, 0, 10, 10)));
}

#[test]
fn draw_batch_scissor_culls_only_disjoint_batches() {
    let scissor = (8, 4, 12, 10);

    assert!(draw_bounds_intersect_scissor(
        UiSurfaceRect::new(2.0, 2.0, 10.0, 8.0),
        scissor
    ));
    assert!(!draw_bounds_intersect_scissor(
        UiSurfaceRect::new(24.0, 4.0, 8.0, 8.0),
        scissor
    ));
}

#[test]
fn persistent_vertex_buffers_reuse_capacity_before_growing() {
    assert!(vertex_buffer_needs_reallocation(None, 64));
    assert!(!vertex_buffer_needs_reallocation(Some(256), 64));
    assert!(vertex_buffer_needs_reallocation(Some(256), 257));
    assert_eq!(next_ui_vertex_buffer_capacity(1), 256);
    assert_eq!(next_ui_vertex_buffer_capacity(256), 256);
    assert_eq!(next_ui_vertex_buffer_capacity(257), 512);
}

#[test]
fn persistent_vertex_buffers_retain_empty_categories_until_they_are_reused() {
    assert_eq!(
        ui_vertex_buffer_upload_action(Some(256), 0),
        UiVertexBufferUploadAction::RetainExisting
    );
    assert_eq!(
        ui_vertex_buffer_upload_action(Some(256), 64),
        UiVertexBufferUploadAction::ReuseExisting
    );
    assert_eq!(
        ui_vertex_buffer_upload_action(Some(256), 257),
        UiVertexBufferUploadAction::Allocate
    );
    assert_eq!(
        ui_vertex_buffer_upload_action(None, 64),
        UiVertexBufferUploadAction::Allocate
    );
}
