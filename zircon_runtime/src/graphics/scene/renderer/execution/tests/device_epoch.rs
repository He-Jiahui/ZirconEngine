use crate::graphics::backend::RenderBackend;
use crate::rhi::{RenderQueueClass, SubmissionStatus, SubmissionTicket};

use super::super::{BufferDesc, TextureDesc, TransientResourcePool};
use crate::rhi::{BufferUsage, TextureFormat, TextureUsage};

#[test]
fn transient_resource_pool_reuses_entries_across_frames() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let desc = TextureDesc::new(
        "pooled-color",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    );
    let mut pool = TransientResourcePool::default();

    pool.begin_frame(backend.device_profile());
    let first = pool.acquire_texture(&backend.device, &desc).unwrap();
    let first_identity = first.identity();
    pool.release_texture(first);
    pool.end_frame();
    assert_eq!(pool.last_frame_report().texture_created_count, 1);
    assert_eq!(pool.last_frame_report().texture_reused_count, 0);
    assert_eq!(pool.last_frame_report().texture_pool_retained_bytes, 4_096);
    assert_eq!(
        pool.last_frame_report().texture_pool_budget_bytes,
        super::super::TRANSIENT_RESOURCE_POOL_TEXTURE_BUDGET_BYTES
    );

    pool.begin_frame(backend.device_profile());
    let second = pool.acquire_texture(&backend.device, &desc).unwrap();
    let second_identity = second.identity();
    assert_eq!(first_identity, second_identity);
    pool.release_texture(second);
    pool.end_frame();
    assert_eq!(pool.last_frame_report().texture_created_count, 0);
    assert_eq!(pool.last_frame_report().texture_reused_count, 1);
    assert_eq!(pool.last_frame_report().texture_pool_entry_count, 1);
}

#[test]
fn transient_resource_pool_discards_free_and_pending_backings_on_device_epoch_change() {
    let source_backend = RenderBackend::new_offscreen().unwrap();
    let destination_backend = RenderBackend::new_offscreen().unwrap();
    assert_ne!(
        source_backend.device_profile().device_id(),
        destination_backend.device_profile().device_id(),
        "the regression requires independently owned WGPU devices"
    );
    let texture_desc = TextureDesc::new(
        "device-epoch-texture",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    );
    let buffer_desc = BufferDesc::new(
        "device-epoch-buffer",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    );
    let ticket = SubmissionTicket::new(
        source_backend.device_profile().device_id(),
        source_backend.device_profile().generation(),
        RenderQueueClass::Graphics,
        1,
    );
    let mut pool = TransientResourcePool::default();

    pool.begin_frame(source_backend.device_profile());
    let free_texture = pool
        .acquire_texture(&source_backend.device, &texture_desc)
        .unwrap();
    let free_identity = free_texture.identity();
    let pending_texture = pool
        .acquire_texture(&source_backend.device, &texture_desc)
        .unwrap();
    let pending_identity = pending_texture.identity();
    let free_buffer = pool
        .acquire_buffer(&source_backend.device, &buffer_desc)
        .unwrap();
    let pending_buffer = pool
        .acquire_buffer(&source_backend.device, &buffer_desc)
        .unwrap();
    pool.release_texture(free_texture);
    pool.release_texture_after_submission(pending_texture, ticket);
    pool.release_buffer(free_buffer);
    pool.release_buffer_after_submission(pending_buffer, ticket);
    pool.end_frame();

    pool.begin_frame(destination_backend.device_profile());
    assert_eq!(pool.frame_report.device_epoch_discarded_texture_count, 2);
    assert_eq!(pool.frame_report.device_epoch_discarded_buffer_count, 2);
    let replacement = pool
        .acquire_texture(&destination_backend.device, &texture_desc)
        .unwrap();
    let replacement_identity = replacement.identity();
    assert_ne!(replacement_identity, free_identity);
    assert_ne!(replacement_identity, pending_identity);
    assert_eq!(pool.frame_report.texture_created_count, 1);
    assert_eq!(pool.frame_report.texture_reused_count, 0);
}

#[test]
fn transient_resource_pool_waits_for_completed_submission_before_reuse() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let desc = TextureDesc::new(
        "completion-qualified-transient",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    );
    let ticket = SubmissionTicket::new(
        backend.device_profile().device_id(),
        backend.device_profile().generation(),
        RenderQueueClass::Graphics,
        1,
    );
    let mut pool = TransientResourcePool::default();

    pool.begin_frame(backend.device_profile());
    let first = pool.acquire_texture(&backend.device, &desc).unwrap();
    let first_identity = first.identity();
    pool.release_texture_after_submission(first, ticket);
    pool.end_frame();
    assert_eq!(pool.last_frame_report().pending_retire_texture_count, 1);

    pool.begin_frame(backend.device_profile());
    pool.collect_completed_submissions(|_| Ok(SubmissionStatus::Submitted));
    let second = pool.acquire_texture(&backend.device, &desc).unwrap();
    let second_identity = second.identity();
    assert_ne!(first_identity, second_identity);
    pool.release_texture(second);
    pool.end_frame();

    pool.begin_frame(backend.device_profile());
    pool.collect_completed_submissions(|_| Ok(SubmissionStatus::Completed));
    assert_eq!(
        pool.frame_report.completion_reclaimed_texture_count, 1,
        "only a completed ticket may return a backing to the reusable pool"
    );
    let third = pool.acquire_texture(&backend.device, &desc).unwrap();
    let third_identity = third.identity();
    assert_eq!(third_identity, first_identity);
}

#[test]
fn transient_resource_pool_discards_failed_submission_backings() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let desc = TextureDesc::new(
        "failed-transient-submission",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    );
    let ticket = SubmissionTicket::new(
        backend.device_profile().device_id(),
        backend.device_profile().generation(),
        RenderQueueClass::Graphics,
        2,
    );
    let mut pool = TransientResourcePool::default();

    pool.begin_frame(backend.device_profile());
    let first = pool.acquire_texture(&backend.device, &desc).unwrap();
    let first_identity = first.identity();
    pool.release_texture_after_submission(first, ticket);
    pool.end_frame();

    pool.begin_frame(backend.device_profile());
    pool.collect_completed_submissions(|_| Ok(SubmissionStatus::Failed));
    assert_eq!(pool.frame_report.completion_discarded_texture_count, 1);
    let replacement = pool.acquire_texture(&backend.device, &desc).unwrap();
    let replacement_identity = replacement.identity();
    assert_ne!(replacement_identity, first_identity);
}

#[test]
fn transient_resource_pool_discards_pending_backings_without_a_ticket() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let texture_desc = TextureDesc::new(
        "missing-ticket-texture",
        8,
        8,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    );
    let buffer_desc = BufferDesc::new("missing-ticket-buffer", 64, BufferUsage::STORAGE);
    let ticket = SubmissionTicket::new(
        backend.device_profile().device_id(),
        backend.device_profile().generation(),
        RenderQueueClass::Graphics,
        3,
    );
    let mut pool = TransientResourcePool::default();

    pool.begin_frame(backend.device_profile());
    let texture = pool
        .acquire_texture(&backend.device, &texture_desc)
        .unwrap();
    let buffer = pool.acquire_buffer(&backend.device, &buffer_desc).unwrap();
    pool.release_texture_after_submission(texture, ticket);
    pool.release_buffer_after_submission(buffer, ticket);
    pool.pending_textures[0].make_reusable();
    pool.pending_buffers[0].make_reusable();

    let mut status_query_count = 0;
    pool.collect_completed_submissions(|_| {
        status_query_count += 1;
        Ok(SubmissionStatus::Completed)
    });

    assert_eq!(status_query_count, 0);
    assert_eq!(pool.frame_report.completion_discarded_texture_count, 1);
    assert_eq!(pool.frame_report.completion_discarded_buffer_count, 1);
    assert!(pool.pending_textures.is_empty());
    assert!(pool.pending_buffers.is_empty());
}
