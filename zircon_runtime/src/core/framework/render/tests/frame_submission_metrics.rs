use super::RenderFrameSubmissionMetrics;

#[test]
fn frame_submission_metrics_keep_logical_and_physical_counts_distinct() {
    let metrics = RenderFrameSubmissionMetrics::new(3, 3, 2, 1, 2, 4, 5, 4096);

    assert_eq!(metrics.admitted_logical_packet_count(), 3);
    assert_eq!(metrics.flushed_logical_ticket_count(), 3);
    assert_eq!(metrics.physical_backend_submission_count(), 2);
    assert_eq!(metrics.buffer_upload_batch_count(), 1);
    assert_eq!(metrics.texture_upload_batch_count(), 2);
    assert_eq!(metrics.buffer_write_count(), 4);
    assert_eq!(metrics.texture_write_count(), 5);
    assert_eq!(metrics.upload_payload_bytes(), 4096);
}
