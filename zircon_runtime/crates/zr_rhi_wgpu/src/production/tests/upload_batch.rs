#[test]
fn texture_upload_batches_are_ticketed_without_per_subresource_payload_clones() {
    let buffer_batch = include_str!("../buffer_upload_batch.rs");
    let native_submission = include_str!("../device/native_submission.rs");
    let submission = include_str!("../submission.rs");
    let upload_batch = include_str!("../upload_batch.rs");
    let queued_work = include_str!("../submission/queued_work.rs");

    assert!(native_submission.contains("enqueue_native_texture_upload_batch"));
    assert!(native_submission.contains("enqueue_native_buffer_upload_batch"));
    assert!(submission.contains("BufferUpload"));
    assert!(submission.contains("TextureUploadBatch"));
    assert!(buffer_batch.contains("WgpuBufferUploadBatch"));
    assert!(buffer_batch.contains("Arc<[u8]>"));
    assert!(upload_batch.contains("Arc<[u8]>"));
    assert!(queued_work.contains("batch: WgpuBufferUploadBatch"));
    assert!(queued_work.contains("batch: WgpuTextureUploadBatch"));
    assert!(upload_batch.contains("source_range"));
    assert!(upload_batch.contains("WgpuTextureUploadBatch"));
    assert!(!native_submission.contains("queue.write_texture"));
}

#[test]
fn mixed_resource_upload_packet_is_empty_only_when_both_domains_are_empty() {
    let packet = super::super::WgpuResourceUploadBatch::from_batches(
        super::super::WgpuBufferUploadBatch::new(),
        super::super::WgpuTextureUploadBatch::new(),
    );

    assert!(packet.is_empty());
    assert_eq!(packet.buffer_upload_count(), 0);
    assert_eq!(packet.texture_upload_count(), 0);
    assert_eq!(packet.payload_byte_len(), 0);
}

#[test]
fn mixed_resource_upload_uses_one_copy_ticket_and_preserves_upload_before_draw_order() {
    let native_submission = include_str!("../device/native_submission.rs");
    let resource_batch = include_str!("../resource_upload_batch.rs");
    let submission = include_str!("../submission.rs");

    let bridge = native_submission
        .split("pub fn enqueue_native_resource_upload_batch")
        .nth(1)
        .expect("mixed resource upload bridge");
    let bridge = bridge
        .split("pub fn settle_abandoned_native_submissions")
        .next()
        .expect("bounded mixed resource bridge");
    assert_eq!(
        bridge
            .matches("begin_packet(RenderQueueClass::Copy)")
            .count(),
        1
    );
    assert!(bridge.contains("commit_resource_upload_batch(ticket, batch)"));
    assert!(!bridge.contains("enqueue_native_buffer_upload_batch"));
    assert!(!bridge.contains("enqueue_native_texture_upload_batch"));

    assert!(resource_batch.contains("buffer_uploads: WgpuBufferUploadBatch"));
    assert!(resource_batch.contains("texture_uploads: WgpuTextureUploadBatch"));
    assert!(resource_batch.contains("from_batches"));
    assert!(submission.contains("ResourceUpload"));
    assert!(submission.contains("record_resource_upload_admitted"));
    assert!(submission.contains("WgpuBufferUpload::from_owned_bytes"));
    assert!(submission.contains("WgpuTextureUpload::from_owned_bytes"));
    assert!(!submission.contains(".expect("));
    assert!(!submission.contains("unreachable!("));

    let flush = submission
        .split("pub(crate) fn flush")
        .nth(1)
        .expect("submission flush owner");
    assert!(flush.contains("for submission in submissions"));
    let resource_flush = flush
        .split("QueuedWgpuSubmission::ResourceUpload")
        .nth(1)
        .expect("mixed resource flush arm");
    let compact_resource_flush = resource_flush.split_whitespace().collect::<String>();
    let buffer_write = compact_resource_flush
        .find("self.queue.write_buffer")
        .expect("buffer writes");
    let texture_write = compact_resource_flush
        .find("self.queue.write_texture")
        .expect("texture writes");
    assert!(buffer_write < texture_write);
    assert!(resource_flush.contains("depth_or_array_layers: region.depth_or_array_layers"));
    assert!(!resource_flush.contains("depth_or_array_layers: 1"));
    let native_production = native_submission
        .split("#[cfg(test)]")
        .next()
        .expect("native submission production source");
    assert!(!native_production.contains("wgpu::Queue"));
    assert!(!native_production.contains("queue.submit"));
}
