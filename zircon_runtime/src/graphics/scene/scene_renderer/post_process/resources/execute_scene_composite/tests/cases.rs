use crate::core::framework::render::PostProcessGraphResourceNames;

#[test]
fn scene_composite_pass_writes_dedicated_intermediate_resource() {
    assert_eq!(
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        "postprocess.scene-composited"
    );
}

#[test]
fn scene_composite_params_are_returned_as_pre_submit_uploads() {
    let source = include_str!("../mod.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("scene-composite source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(!production.contains("create_post_process_params_buffer"));
    assert!(production.contains("post_process_params_upload("));
    assert!(production.contains("WgpuBufferUploadBatch"));
}
