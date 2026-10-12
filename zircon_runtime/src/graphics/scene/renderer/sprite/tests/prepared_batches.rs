use crate::core::math::{Vec2, Vec3, Vec4};

use super::*;

#[test]
fn sprite_batching_preserves_order_and_only_merges_adjacent_matching_textures() {
    let texture_a = ResourceId::from_stable_label("builtin://test/sprite-a");
    let texture_b = ResourceId::from_stable_label("builtin://test/sprite-b");

    let batches = batch_sprite_draw_items([
        (0, texture_a, test_vertices()),
        (1, texture_a, test_vertices()),
        (2, texture_b, test_vertices()),
        (3, texture_a, test_vertices()),
    ]);

    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].texture_id(), texture_a);
    assert_eq!(batches[0].sprite_count(), 2);
    assert_eq!(batches[0].vertices().len(), 12);
    assert_eq!(batches[1].texture_id(), texture_b);
    assert_eq!(batches[1].sprite_count(), 1);
    assert_eq!(batches[1].vertices().len(), 6);
    assert_eq!(batches[2].texture_id(), texture_a);
    assert_eq!(batches[2].sprite_count(), 1);
    assert_eq!(batches[2].vertices().len(), 6);
}

#[test]
fn sprite_batching_skips_empty_vertex_items() {
    let texture_a = ResourceId::from_stable_label("builtin://test/sprite-a");

    let batches =
        batch_sprite_draw_items([(0, texture_a, Vec::new()), (1, texture_a, test_vertices())]);

    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].sprite_count(), 1);
    assert_eq!(batches[0].vertices().len(), 6);
}

#[test]
fn sprite_queue_stats_count_stage_batches_sprites_and_vertices() {
    let texture_a = ResourceId::from_stable_label("builtin://test/sprite-a");
    let texture_b = ResourceId::from_stable_label("builtin://test/sprite-b");
    let opaque_batches = batch_sprite_draw_items([
        (0, texture_a, test_vertices()),
        (1, texture_a, test_vertices()),
        (2, texture_b, test_vertices()),
    ]);
    let transparent_batches = batch_sprite_draw_items([(3, texture_b, test_vertices())]);
    let mut stats = PreparedSpriteQueueStats::default();

    stats.accumulate_stage(RenderPassStage::Opaque2d, &opaque_batches);
    stats.accumulate_stage(RenderPassStage::Transparent2d, &transparent_batches);

    assert_eq!(
        stats,
        PreparedSpriteQueueStats {
            draw_batch_count: 3,
            sprite_count: 4,
            image_slice_count: 4,
            expanded_image_slice_count: 0,
            vertex_count: 24,
            opaque_draw_batch_count: 2,
            alpha_mask_draw_batch_count: 0,
            transparent_draw_batch_count: 1,
        }
    );
}

#[test]
fn sprite_queue_stats_report_generated_image_slices_separately_from_sprites() {
    let texture_a = ResourceId::from_stable_label("builtin://test/sprite-a");
    let expanded_batches = batch_sprite_draw_items([(0, texture_a, repeated_vertices(3))]);
    let mut stats = PreparedSpriteQueueStats::default();

    stats.accumulate_stage(RenderPassStage::Transparent2d, &expanded_batches);

    assert_eq!(stats.sprite_count, 1);
    assert_eq!(stats.image_slice_count, 3);
    assert_eq!(stats.expanded_image_slice_count, 2);
    assert_eq!(stats.vertex_count, 18);
}

fn test_vertices() -> Vec<SpriteVertex> {
    vec![
        SpriteVertex::new(Vec3::ZERO, Vec2::ZERO, Vec4::ONE),
        SpriteVertex::new(Vec3::X, Vec2::X, Vec4::ONE),
        SpriteVertex::new(Vec3::Y, Vec2::Y, Vec4::ONE),
        SpriteVertex::new(Vec3::Y, Vec2::Y, Vec4::ONE),
        SpriteVertex::new(Vec3::X, Vec2::X, Vec4::ONE),
        SpriteVertex::new(Vec3::ONE, Vec2::ONE, Vec4::ONE),
    ]
}

fn repeated_vertices(image_slice_count: usize) -> Vec<SpriteVertex> {
    let mut vertices = Vec::new();
    for _ in 0..image_slice_count {
        vertices.extend(test_vertices());
    }
    vertices
}
