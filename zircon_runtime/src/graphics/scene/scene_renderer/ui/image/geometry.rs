use std::sync::{Arc, Weak};

use zircon_runtime_interface::ui::layout::UiFrame;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use crate::core::math::UVec2;

use super::{
    ScreenSpaceUiImageVertex, ScreenSpaceUiImageVertexBuffer,
    SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_IDLE_EPOCHS,
    SCREEN_SPACE_UI_IMAGE_MIN_VERTEX_BUFFER_CAPACITY_BYTES,
};

pub(super) fn screen_space_ui_image_segment_plan_reused(
    current: Option<&Weak<super::super::render::PlannedScreenSpaceUi>>,
    current_viewport_size: UVec2,
    next: &Arc<super::super::render::PlannedScreenSpaceUi>,
    next_viewport_size: UVec2,
) -> bool {
    current_viewport_size == next_viewport_size
        && current.is_some_and(|current| std::ptr::eq(current.as_ptr(), Arc::as_ptr(next)))
}

pub(super) fn screen_space_ui_image_texture_dependency_is_current<T>(
    current: Option<&Arc<T>>,
    next: &Arc<T>,
) -> bool {
    current.is_some_and(|current| Arc::ptr_eq(current, next))
}

pub(super) fn image_batch_scissor(
    frame: UiFrame,
    viewport: UiFrame,
    clip_frame: Option<UiFrame>,
) -> Option<super::super::render::ScreenSpaceUiScissor> {
    super::super::render::clipped_scissor(
        frame,
        clip_frame,
        viewport,
        super::super::render::frame_to_scissor(viewport)?,
    )
}

pub(super) fn write_screen_space_ui_image_vertex_buffer(
    device: &wgpu::Device,
    image_vertices: &mut ScreenSpaceUiImageVertexBuffer,
    uploads: &mut WgpuBufferUploadBatch,
    force_full_upload: bool,
) {
    if image_vertices.vertices.is_empty() {
        return;
    }

    let vertex_bytes = bytemuck::cast_slice(image_vertices.vertices.as_slice());
    let required_byte_len = vertex_bytes.len();
    let requires_reallocation = image_vertices.buffer.is_none()
        || image_vertex_buffer_requires_reallocation(
            image_vertices.capacity_bytes,
            required_byte_len,
        );
    if requires_reallocation {
        let capacity_bytes = image_vertex_buffer_capacity(required_byte_len);
        image_vertices.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-screen-space-ui-image-vertices"),
            size: capacity_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        image_vertices.capacity_bytes = capacity_bytes;
    }
    let payload_hash = *blake3::hash(vertex_bytes).as_bytes();
    let write_required = image_vertex_buffer_write_required(
        requires_reallocation || force_full_upload,
        image_vertices.payload_hash,
        payload_hash,
    );
    if write_required {
        if let Some(vertex_buffer) = image_vertices.buffer.as_ref() {
            uploads.push(WgpuBufferUpload::from_bytes(
                vertex_buffer.clone(),
                0,
                vertex_bytes,
            ));
            image_vertices.payload_hash = Some(payload_hash);
        }
    }
}

pub(super) const fn image_cpu_staging_should_reset(image_count: usize) -> bool {
    image_count == 0
}

pub(super) fn binding_cache_epoch_is_recent(current_epoch: u64, last_prepare_epoch: u64) -> bool {
    current_epoch >= last_prepare_epoch
        && current_epoch - last_prepare_epoch <= SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_IDLE_EPOCHS
}

pub(super) fn binding_cache_entry_is_trimmable(
    current_epoch: u64,
    last_prepare_epoch: u64,
) -> bool {
    last_prepare_epoch != current_epoch
}

pub(super) fn image_vertex_buffer_capacity(required_byte_len: usize) -> u64 {
    let required_byte_len =
        (required_byte_len as u64).max(SCREEN_SPACE_UI_IMAGE_MIN_VERTEX_BUFFER_CAPACITY_BYTES);
    required_byte_len
        .checked_next_power_of_two()
        .unwrap_or(required_byte_len)
}

pub(super) fn image_vertex_buffer_requires_reallocation(
    capacity_bytes: u64,
    required_byte_len: usize,
) -> bool {
    capacity_bytes < required_byte_len as u64
}

pub(super) fn image_vertex_buffer_write_required(
    requires_reallocation: bool,
    current_payload_hash: Option<[u8; 32]>,
    next_payload_hash: [u8; 32],
) -> bool {
    requires_reallocation || current_payload_hash != Some(next_payload_hash)
}

pub(super) fn image_vertices(
    frame: UiFrame,
    viewport: UiFrame,
    tint: [f32; 4],
) -> [ScreenSpaceUiImageVertex; 6] {
    let x0 = (frame.x / viewport.width.max(1.0)) * 2.0 - 1.0;
    let x1 = (frame.right() / viewport.width.max(1.0)) * 2.0 - 1.0;
    let y0 = 1.0 - (frame.y / viewport.height.max(1.0)) * 2.0;
    let y1 = 1.0 - (frame.bottom() / viewport.height.max(1.0)) * 2.0;
    [
        ScreenSpaceUiImageVertex {
            position: [x0, y0],
            uv: [0.0, 0.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x1, y0],
            uv: [1.0, 0.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x1, y1],
            uv: [1.0, 1.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x0, y0],
            uv: [0.0, 0.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x1, y1],
            uv: [1.0, 1.0],
            tint,
        },
        ScreenSpaceUiImageVertex {
            position: [x0, y1],
            uv: [0.0, 1.0],
            tint,
        },
    ]
}
