use std::sync::Arc;

use zr_rhi::{UiSurfaceDrawList, UiSurfaceRect};

use super::batching::{BatchDrawPlan, DrawOp};
use super::geometry::UI_QUAD_VERTEX_COUNT;
use super::image_cache::WgpuUiImageCache;
use super::text::WgpuUiTextRenderer;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WgpuUiDrawBufferStats {
    pub(super) vertex_buffer_create_count: u64,
    pub(super) vertex_upload_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WgpuUiRecordedDrawStats {
    pub(super) draw_calls: u64,
    pub(super) render_pass_count: u64,
    pub(super) visible_draw_item_count: u64,
    pub(super) solid_vertex_count: u64,
    pub(super) solid_instance_count: u64,
    pub(super) image_vertex_count: u64,
    pub(super) batch_layer_count: u64,
    last_layer_index: Option<u32>,
}

impl WgpuUiRecordedDrawStats {
    fn record_draw(
        &mut self,
        layer_index: u32,
        item_count: u32,
        solid_vertex_count: u32,
        solid_instance_count: u32,
        image_vertex_count: u32,
    ) {
        self.draw_calls = self.draw_calls.saturating_add(1);
        self.visible_draw_item_count = self
            .visible_draw_item_count
            .saturating_add(u64::from(item_count));
        self.solid_vertex_count = self
            .solid_vertex_count
            .saturating_add(u64::from(solid_vertex_count));
        self.solid_instance_count = self
            .solid_instance_count
            .saturating_add(u64::from(solid_instance_count));
        self.image_vertex_count = self
            .image_vertex_count
            .saturating_add(u64::from(image_vertex_count));
        if self.last_layer_index != Some(layer_index) {
            self.batch_layer_count = self.batch_layer_count.saturating_add(1);
            self.last_layer_index = Some(layer_index);
        }
    }
}

#[derive(Clone)]
pub(super) struct WgpuUiDrawBuffers {
    solid: Option<UiVertexBuffer>,
    solid_instance: Option<UiVertexBuffer>,
    image: Option<UiVertexBuffer>,
}

#[derive(Clone)]
struct UiVertexBuffer {
    buffer: Arc<wgpu::Buffer>,
    capacity: u64,
}

impl WgpuUiDrawBuffers {
    fn upload(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        previous: Option<&Self>,
        draw_plan: &BatchDrawPlan,
    ) -> (Self, WgpuUiDrawBufferStats) {
        let (solid, solid_created) = upload_ui_vertex_buffer(
            device,
            queue,
            previous.and_then(|buffers| buffers.solid.as_ref()),
            bytemuck::cast_slice(draw_plan.solid_vertices.as_slice()),
            "zircon-ui-solid-vertices",
        );
        let (image, image_created) = upload_ui_vertex_buffer(
            device,
            queue,
            previous.and_then(|buffers| buffers.image.as_ref()),
            bytemuck::cast_slice(draw_plan.image_vertices.as_slice()),
            "zircon-ui-image-vertices",
        );
        let (solid_instance, solid_instance_created) = upload_ui_vertex_buffer(
            device,
            queue,
            previous.and_then(|buffers| buffers.solid_instance.as_ref()),
            bytemuck::cast_slice(draw_plan.solid_instances.as_slice()),
            "zircon-ui-solid-instances",
        );
        let solid_upload_bytes = std::mem::size_of_val(draw_plan.solid_vertices.as_slice()) as u64;
        let solid_instance_upload_bytes =
            std::mem::size_of_val(draw_plan.solid_instances.as_slice()) as u64;
        let image_upload_bytes = std::mem::size_of_val(draw_plan.image_vertices.as_slice()) as u64;
        (
            Self {
                solid,
                solid_instance,
                image,
            },
            WgpuUiDrawBufferStats {
                vertex_buffer_create_count: u64::from(solid_created as u8)
                    .saturating_add(u64::from(solid_instance_created as u8))
                    .saturating_add(u64::from(image_created as u8)),
                vertex_upload_bytes: solid_upload_bytes
                    .saturating_add(solid_instance_upload_bytes)
                    .saturating_add(image_upload_bytes),
            },
        )
    }
}

const MIN_UI_VERTEX_BUFFER_BYTES: u64 = 256;

fn upload_ui_vertex_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    existing: Option<&UiVertexBuffer>,
    contents: &[u8],
    label: &'static str,
) -> (Option<UiVertexBuffer>, bool) {
    let required_bytes = contents.len() as u64;
    let action =
        ui_vertex_buffer_upload_action(existing.map(|buffer| buffer.capacity), required_bytes);
    let allocate = || {
        let capacity = next_ui_vertex_buffer_capacity(required_bytes);
        UiVertexBuffer {
            buffer: Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })),
            capacity,
        }
    };
    let (buffer, reallocated) = match action {
        UiVertexBufferUploadAction::RetainExisting => {
            return (existing.cloned(), false);
        }
        UiVertexBufferUploadAction::ReuseExisting => match existing {
            Some(existing) => (existing.clone(), false),
            None => (allocate(), true),
        },
        UiVertexBufferUploadAction::Allocate => (allocate(), true),
    };
    queue.write_buffer(&buffer.buffer, 0, contents);
    (Some(buffer), reallocated)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiVertexBufferUploadAction {
    RetainExisting,
    ReuseExisting,
    Allocate,
}

fn ui_vertex_buffer_upload_action(
    existing_capacity: Option<u64>,
    required_bytes: u64,
) -> UiVertexBufferUploadAction {
    if required_bytes == 0 {
        UiVertexBufferUploadAction::RetainExisting
    } else if vertex_buffer_needs_reallocation(existing_capacity, required_bytes) {
        UiVertexBufferUploadAction::Allocate
    } else {
        UiVertexBufferUploadAction::ReuseExisting
    }
}

fn vertex_buffer_needs_reallocation(existing_capacity: Option<u64>, required_bytes: u64) -> bool {
    required_bytes > 0 && existing_capacity.is_none_or(|capacity| capacity < required_bytes)
}

fn next_ui_vertex_buffer_capacity(required_bytes: u64) -> u64 {
    required_bytes
        .max(MIN_UI_VERTEX_BUFFER_BYTES)
        .checked_next_power_of_two()
        .unwrap_or(required_bytes)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiDrawBufferCacheKey {
    generation: u64,
    projection_size: (u32, u32),
}

#[derive(Default)]
pub(super) struct WgpuUiDrawBufferCache {
    key: Option<UiDrawBufferCacheKey>,
    buffers: Option<WgpuUiDrawBuffers>,
}

pub(super) struct ResolvedWgpuUiDrawBuffers {
    pub(super) buffers: WgpuUiDrawBuffers,
    pub(super) stats: WgpuUiDrawBufferStats,
}

impl WgpuUiDrawBufferCache {
    pub(super) fn resolve(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        draw_list: &UiSurfaceDrawList,
        draw_plan: &BatchDrawPlan,
    ) -> ResolvedWgpuUiDrawBuffers {
        let cache_key = Self::cache_key(draw_list);
        if let Some(key) = cache_key {
            if self.key == Some(key) {
                if let Some(buffers) = &self.buffers {
                    return ResolvedWgpuUiDrawBuffers {
                        buffers: buffers.clone(),
                        stats: WgpuUiDrawBufferStats::default(),
                    };
                }
            }
        }

        // Keep the last allocation even for unversioned and damage inputs: only a matching full
        // generation may skip uploads, but every input can reuse a sufficiently large buffer.
        let (buffers, stats) =
            WgpuUiDrawBuffers::upload(device, queue, self.buffers.as_ref(), draw_plan);
        self.key = cache_key;
        self.buffers = Some(buffers.clone());
        ResolvedWgpuUiDrawBuffers { buffers, stats }
    }

    fn cache_key(draw_list: &UiSurfaceDrawList) -> Option<UiDrawBufferCacheKey> {
        draw_list
            .generation()
            .map(|generation| UiDrawBufferCacheKey {
                generation,
                projection_size: draw_list.projection_size(),
            })
    }
}

#[derive(Clone, Copy)]
pub(super) enum TargetLoad {
    ClearBlack,
    Load,
}

impl TargetLoad {
    fn load_op(self) -> wgpu::LoadOp<wgpu::Color> {
        match self {
            Self::ClearBlack => wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            Self::Load => wgpu::LoadOp::Load,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record_draw_ops_to_view(
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    initial_load: TargetLoad,
    surface_size: (u32, u32),
    projection_size: (u32, u32),
    damage: Option<UiSurfaceRect>,
    draw_ops: &[DrawOp],
    buffers: &WgpuUiDrawBuffers,
    solid_pipeline: &wgpu::RenderPipeline,
    solid_instance_pipeline: &wgpu::RenderPipeline,
    image_pipeline: &wgpu::RenderPipeline,
    image_cache: &WgpuUiImageCache,
    text: &mut WgpuUiTextRenderer,
) -> WgpuUiRecordedDrawStats {
    let Some(scissor) = damage_scissor(damage, surface_size) else {
        return WgpuUiRecordedDrawStats::default();
    };
    if draw_ops.is_empty() {
        let mut pass = begin_ui_surface_pass(encoder, target_view, initial_load);
        set_surface_viewport(&mut pass, projection_size);
        pass.set_scissor_rect(scissor.0, scissor.1, scissor.2, scissor.3);
        return WgpuUiRecordedDrawStats {
            render_pass_count: 1,
            ..WgpuUiRecordedDrawStats::default()
        };
    }

    let mut stats = WgpuUiRecordedDrawStats::default();
    let mut first_pass = true;
    let mut op_index = 0;
    while op_index < draw_ops.len() {
        let load = if first_pass {
            initial_load
        } else {
            TargetLoad::Load
        };
        if let DrawOp::Text(draw) = &draw_ops[op_index] {
            if !draw_bounds_intersect_scissor(draw.bounds, scissor) {
                op_index += 1;
                continue;
            }
            let mut pass = begin_ui_surface_pass(encoder, target_view, load);
            stats.render_pass_count = stats.render_pass_count.saturating_add(1);
            set_surface_viewport(&mut pass, projection_size);
            pass.set_scissor_rect(scissor.0, scissor.1, scissor.2, scissor.3);
            if text.render_batch(draw.batch_index, &mut pass) {
                stats.record_draw(draw.layer_index, draw.command_indices.len() as u32, 0, 0, 0);
            }
            first_pass = false;
            op_index += 1;
            continue;
        }

        let run_end = non_text_run_end(draw_ops, op_index);
        if !draw_ops[op_index..run_end].iter().any(|op| match op {
            DrawOp::Solid(draw) => draw_bounds_intersect_scissor(draw.bounds, scissor),
            DrawOp::Image(draw) => draw_bounds_intersect_scissor(draw.bounds, scissor),
            DrawOp::Text(_) => unreachable!("non-text draw run cannot contain text"),
        }) {
            op_index = run_end;
            continue;
        }
        let mut pass = begin_ui_surface_pass(encoder, target_view, load);
        stats.render_pass_count = stats.render_pass_count.saturating_add(1);
        set_surface_viewport(&mut pass, projection_size);
        pass.set_scissor_rect(scissor.0, scissor.1, scissor.2, scissor.3);
        for run_index in op_index..run_end {
            match &draw_ops[run_index] {
                DrawOp::Solid(draw) => {
                    if !draw_bounds_intersect_scissor(draw.bounds, scissor) {
                        continue;
                    }
                    let vertex_count = draw.vertex_end.saturating_sub(draw.vertex_start);
                    let instance_count = draw.instance_end.saturating_sub(draw.instance_start);
                    if instance_count > 0 {
                        let Some(buffer) = buffers.solid_instance.as_ref() else {
                            continue;
                        };
                        pass.set_pipeline(solid_instance_pipeline);
                        pass.set_vertex_buffer(0, buffer.buffer.slice(..));
                        pass.draw(
                            0..UI_QUAD_VERTEX_COUNT,
                            draw.instance_start..draw.instance_end,
                        );
                    } else {
                        let Some(buffer) = buffers.solid.as_ref() else {
                            continue;
                        };
                        pass.set_pipeline(solid_pipeline);
                        pass.set_vertex_buffer(0, buffer.buffer.slice(..));
                        pass.draw(draw.vertex_start..draw.vertex_end, 0..1);
                    }
                    stats.record_draw(
                        draw.layer_index,
                        draw.item_count,
                        vertex_count
                            .saturating_add(instance_count.saturating_mul(UI_QUAD_VERTEX_COUNT)),
                        instance_count,
                        0,
                    );
                }
                DrawOp::Image(draw) => {
                    if !draw_bounds_intersect_scissor(draw.bounds, scissor) {
                        continue;
                    }
                    let Some(buffer) = buffers.image.as_ref() else {
                        continue;
                    };
                    pass.set_pipeline(image_pipeline);
                    pass.set_vertex_buffer(0, buffer.buffer.slice(..));
                    if let Some(resource) =
                        image_cache.get(draw.resource_key.as_str(), draw.resource_generation)
                    {
                        pass.set_bind_group(0, &resource.bind_group, &[]);
                        pass.draw(draw.vertex_start..draw.vertex_end, 0..1);
                        stats.record_draw(
                            draw.layer_index,
                            draw.item_count,
                            0,
                            0,
                            draw.vertex_end.saturating_sub(draw.vertex_start),
                        );
                    }
                }
                DrawOp::Text(_) => unreachable!("non-text draw run cannot contain text"),
            }
        }
        first_pass = false;
        op_index = run_end;
    }
    stats
}

fn non_text_run_end(draw_ops: &[DrawOp], start: usize) -> usize {
    debug_assert!(!matches!(draw_ops.get(start), Some(DrawOp::Text(_))));
    draw_ops[start..]
        .iter()
        .position(|op| matches!(op, DrawOp::Text(_)))
        .map_or(draw_ops.len(), |offset| start + offset)
}

fn begin_ui_surface_pass<'encoder>(
    encoder: &'encoder mut wgpu::CommandEncoder,
    target_view: &'encoder wgpu::TextureView,
    load: TargetLoad,
) -> wgpu::RenderPass<'encoder> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("zircon-ui-surface-pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: load.load_op(),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        // The presenter wraps the complete retained-UI submission in encoder timestamps so one
        // sample covers all material/text passes without requiring inside-pass query support.
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

fn set_surface_viewport(pass: &mut wgpu::RenderPass<'_>, surface_size: (u32, u32)) {
    pass.set_viewport(
        0.0,
        0.0,
        surface_size.0 as f32,
        surface_size.1 as f32,
        0.0,
        1.0,
    );
}

fn damage_scissor(
    damage: Option<UiSurfaceRect>,
    surface_size: (u32, u32),
) -> Option<(u32, u32, u32, u32)> {
    let surface_width = surface_size.0.max(1) as f32;
    let surface_height = surface_size.1.max(1) as f32;
    let Some(damage) = damage else {
        return Some((0, 0, surface_width as u32, surface_height as u32));
    };
    if !damage.x.is_finite()
        || !damage.y.is_finite()
        || !damage.width.is_finite()
        || !damage.height.is_finite()
    {
        return None;
    }
    let left = damage.x.max(0.0).min(surface_width).floor();
    let top = damage.y.max(0.0).min(surface_height).floor();
    let right = (damage.x + damage.width).max(0.0).min(surface_width).ceil();
    let bottom = (damage.y + damage.height)
        .max(0.0)
        .min(surface_height)
        .ceil();
    (right > left && bottom > top).then(|| {
        (
            left as u32,
            top as u32,
            (right - left) as u32,
            (bottom - top) as u32,
        )
    })
}

fn draw_bounds_intersect_scissor(bounds: UiSurfaceRect, scissor: (u32, u32, u32, u32)) -> bool {
    if !bounds.x.is_finite()
        || !bounds.y.is_finite()
        || !bounds.width.is_finite()
        || !bounds.height.is_finite()
    {
        return false;
    }
    let right = bounds.x + bounds.width;
    let bottom = bounds.y + bounds.height;
    let scissor_right = scissor.0.saturating_add(scissor.2) as f32;
    let scissor_bottom = scissor.1.saturating_add(scissor.3) as f32;
    bounds.x < scissor_right
        && right > scissor.0 as f32
        && bounds.y < scissor_bottom
        && bottom > scissor.1 as f32
}

#[cfg(test)]
mod tests {
    use zr_rhi::{UiSurfaceDrawList, UiSurfaceRect};

    use super::{
        damage_scissor, draw_bounds_intersect_scissor, next_ui_vertex_buffer_capacity,
        ui_vertex_buffer_upload_action, vertex_buffer_needs_reallocation,
        UiVertexBufferUploadAction, WgpuUiDrawBufferCache, WgpuUiRecordedDrawStats,
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
}
