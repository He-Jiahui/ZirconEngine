use crate::core::resource::ResourceId;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::types::ViewportRenderFrame;

use super::build_sprite_vertices::build_sprite_vertices;
use super::sprite_vertex::SpriteVertex;

pub(in crate::graphics::scene::scene_renderer::sprite) struct PreparedSpriteDrawBatch {
    texture_id: ResourceId,
    vertices: Vec<SpriteVertex>,
    sprite_count: usize,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PreparedSpriteQueueStats {
    pub(crate) draw_batch_count: usize,
    pub(crate) sprite_count: usize,
    pub(crate) image_slice_count: usize,
    pub(crate) expanded_image_slice_count: usize,
    pub(crate) vertex_count: usize,
    pub(crate) opaque_draw_batch_count: usize,
    pub(crate) alpha_mask_draw_batch_count: usize,
    pub(crate) transparent_draw_batch_count: usize,
}

impl PreparedSpriteDrawBatch {
    pub(in crate::graphics::scene::scene_renderer::sprite) fn texture_id(&self) -> ResourceId {
        self.texture_id
    }

    pub(in crate::graphics::scene::scene_renderer::sprite) fn vertices(&self) -> &[SpriteVertex] {
        &self.vertices
    }

    fn sprite_count(&self) -> usize {
        self.sprite_count
    }

    fn image_slice_count(&self) -> usize {
        self.vertices.len() / SPRITE_IMAGE_SLICE_VERTEX_COUNT
    }
}

const SPRITE_IMAGE_SLICE_VERTEX_COUNT: usize = 6;

pub(in crate::graphics::scene::scene_renderer::sprite) fn prepare_sprite_draw_batches(
    frame: &ViewportRenderFrame,
    sprite_vertices: Vec<(usize, Vec<SpriteVertex>)>,
) -> Vec<PreparedSpriteDrawBatch> {
    batch_sprite_draw_items(
        sprite_vertices
            .into_iter()
            .filter_map(|(sprite_index, vertices)| {
                frame
                    .sprites()
                    .get(sprite_index)
                    .map(|sprite| (sprite_index, sprite.image.id(), vertices))
            }),
    )
}

pub(crate) fn prepare_sprite_queue_stats(
    frame: &ViewportRenderFrame,
    stages: impl IntoIterator<Item = RenderPassStage>,
) -> PreparedSpriteQueueStats {
    let mut stats = PreparedSpriteQueueStats::default();
    for stage in stages {
        stats.accumulate_stage(
            stage,
            &prepare_sprite_draw_batches(frame, build_sprite_vertices(frame, stage)),
        );
    }
    stats
}

fn batch_sprite_draw_items(
    items: impl IntoIterator<Item = (usize, ResourceId, Vec<SpriteVertex>)>,
) -> Vec<PreparedSpriteDrawBatch> {
    let mut batches = Vec::<PreparedSpriteDrawBatch>::new();
    for (_sprite_index, texture_id, vertices) in items {
        if vertices.is_empty() {
            continue;
        }
        if let Some(current) = batches.last_mut() {
            if current.texture_id == texture_id {
                current.vertices.extend(vertices);
                current.sprite_count += 1;
                continue;
            }
        }
        batches.push(PreparedSpriteDrawBatch {
            texture_id,
            vertices,
            sprite_count: 1,
        });
    }
    batches
}

impl PreparedSpriteQueueStats {
    fn accumulate_stage(&mut self, stage: RenderPassStage, batches: &[PreparedSpriteDrawBatch]) {
        let draw_batch_count = batches.len();
        self.draw_batch_count += draw_batch_count;
        self.sprite_count += batches
            .iter()
            .map(PreparedSpriteDrawBatch::sprite_count)
            .sum::<usize>();
        self.image_slice_count += batches
            .iter()
            .map(PreparedSpriteDrawBatch::image_slice_count)
            .sum::<usize>();
        self.expanded_image_slice_count = self.image_slice_count.saturating_sub(self.sprite_count);
        self.vertex_count += batches
            .iter()
            .map(|batch| batch.vertices().len())
            .sum::<usize>();
        match stage {
            RenderPassStage::Opaque2d => self.opaque_draw_batch_count += draw_batch_count,
            RenderPassStage::AlphaMask2d => {
                self.alpha_mask_draw_batch_count += draw_batch_count;
            }
            RenderPassStage::Transparent2d => {
                self.transparent_draw_batch_count += draw_batch_count;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
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
}
