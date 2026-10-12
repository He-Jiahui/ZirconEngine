use crate::core::resource::ResourceId;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::types::ViewportRenderFrame;

use super::build_sprite_vertices::{append_sprite_image_vertices, visit_stage_sprites};
use super::sprite_vertex::SpriteVertex;

pub(in crate::graphics::scene::scene_renderer::sprite) struct PreparedSpriteDrawBatch {
    texture_id: ResourceId,
    vertices: Vec<SpriteVertex>,
    sprite_count: usize,
}

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
    stage: RenderPassStage,
) -> Vec<PreparedSpriteDrawBatch> {
    let mut batches = Vec::<PreparedSpriteDrawBatch>::new();
    visit_stage_sprites(frame, stage, |_sprite_index, sprite, size| {
        let texture_id = sprite.image.id();
        // 只合并尾部同贴图的连续精灵，保持 phase 遍历顺序及其透明混合先后。
        let created_batch = batches
            .last()
            .is_none_or(|current| current.texture_id != texture_id);
        if created_batch {
            batches.push(PreparedSpriteDrawBatch {
                texture_id,
                vertices: Vec::new(),
                sprite_count: 0,
            });
        }

        let appended = {
            let current = batches
                .last_mut()
                .expect("sprite batch exists before vertex projection");
            let previous_vertex_count = current.vertices.len();
            append_sprite_image_vertices(sprite, size, &mut current.vertices);
            let appended = current.vertices.len() != previous_vertex_count;
            if appended {
                current.sprite_count += 1;
            }
            appended
        };
        if created_batch && !appended {
            // 此精灵未产生任何切片时撤销空批次，避免后续创建无效 GPU pass。
            batches.pop();
        }
    });
    batches
}

pub(crate) fn prepare_sprite_queue_stats(
    frame: &ViewportRenderFrame,
    stages: impl IntoIterator<Item = RenderPassStage>,
) -> PreparedSpriteQueueStats {
    let mut stats = PreparedSpriteQueueStats::default();
    for stage in stages {
        stats.accumulate_stage(stage, &prepare_sprite_draw_batches(frame, stage));
    }
    stats
}

#[cfg(test)]
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
#[path = "tests/prepared_batches.rs"]
mod tests;
