#[test]
fn native_submission_bridge_reuses_the_device_owner_without_queue_escape_hatches() {
    let source = include_str!("../native_submission.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production native submission source");

    assert!(!production.contains("wgpu::Queue"));
    assert!(!production.contains(".poll("));
    assert!(!production.contains(".flush("));
    assert!(!production.contains("queue.submit"));
    assert!(production.contains("self.submissions.begin_packet(RenderQueueClass::Copy)?"));
    assert!(production.contains("commit_buffer_upload_batch(ticket, batch)"));
    assert!(production.contains("commit_texture_upload_batch(ticket, batch)"));
    assert!(production.contains("commit_resource_upload_batch(ticket, batch)"));
    assert!(production.contains("settle_abandoned_submissions(tickets)?"));
}
