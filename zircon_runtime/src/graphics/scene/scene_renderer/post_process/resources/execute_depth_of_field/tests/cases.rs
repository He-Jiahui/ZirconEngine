use crate::core::framework::render::PostProcessGraphResourceNames;

#[test]
fn depth_of_field_pass_writes_dedicated_intermediate_resource() {
    assert_eq!(
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        "postprocess.depth-of-fielded"
    );
}

#[test]
fn depth_of_field_params_are_returned_as_pre_submit_uploads() {
    let source = include_str!("../mod.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("depth-of-field source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(!production.contains("create_post_process_params_buffer"));
    assert!(production.contains("post_process_params_upload("));
    assert!(production.contains("WgpuBufferUploadBatch"));
}
