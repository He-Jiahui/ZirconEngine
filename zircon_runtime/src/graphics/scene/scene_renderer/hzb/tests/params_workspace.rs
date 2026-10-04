use super::*;

#[test]
fn hzb_params_prepare_is_retryable_until_post_admission_commit() {
    let source = include_str!("../params_workspace.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("HZB params workspace source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(!production.contains("queue: &wgpu::Queue"));
    assert!(production.contains("committed_args_count: Option<u32>"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(production.contains("HzbOcclusionParamsCommit"));
    assert!(production.contains("fn commit("));
    let prepare = production.find("fn prepare(").expect("prepare method");
    let commit = production.find("fn commit(").expect("commit method");
    assert!(!production[prepare..commit].contains("committed_args_count ="));
}
