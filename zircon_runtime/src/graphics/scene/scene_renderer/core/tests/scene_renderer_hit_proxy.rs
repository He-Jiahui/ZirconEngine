use std::sync::{Arc, Mutex};

use super::{
    decode_world_normal, record_normal_readback, record_position_readback, record_token_readback,
    validate_hit_proxy_product, SceneHitProxyCompletion, SceneHitProxyProduct,
    SceneHitProxyReadbackAccumulator,
};

#[test]
fn hit_proxy_readback_completion_is_fail_closed_without_production_expect() {
    let source = include_str!("../scene_renderer_hit_proxy.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("hit-proxy test boundary");

    assert!(!source.contains(".expect("));
    assert!(source.contains("let Some(token) = self.token.take()"));
}

#[test]
fn hit_proxy_commits_gpu_scene_only_after_diagnostics_and_terminal_submission() {
    let source = include_str!("../scene_renderer_hit_proxy.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("hit-proxy test boundary");
    let prepare_diagnostics = source
        .find("let diagnostic_frame = diagnostic_scope")
        .expect("diagnostic frame prepare");
    let enqueue_upload = source
        .find(".enqueue_copy_buffer_upload_batch(frame_buffer_uploads)?")
        .expect("GPU-scene upload enqueue");
    let terminal_submit = source
        .find(".submit_graphics_command_buffers_with_diagnostics(")
        .expect("hit-proxy terminal submission");
    let commit_upload = source
        .find("gpu_scene_upload.commit(hit_proxy_gpu_scene)")
        .expect("GPU-scene upload commit");

    assert!(prepare_diagnostics < enqueue_upload);
    assert!(enqueue_upload < terminal_submit);
    assert!(terminal_submit < commit_upload);
}

#[test]
fn hit_proxy_normal_decode_preserves_signed_unit_direction() {
    let bytes = [0x00, 0xbc, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x00];
    let normal = decode_world_normal(bytes.to_vec()).expect("RGBA16F normal");

    assert!((normal[0] + std::f32::consts::FRAC_1_SQRT_2).abs() < 0.001);
    assert_eq!(normal[1], 0.0);
    assert!((normal[2] - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.001);
}

#[test]
fn hit_proxy_product_rejects_invalid_hit_geometry_but_accepts_clear_no_hit() {
    assert!(validate_hit_proxy_product(SceneHitProxyProduct {
        token: 0,
        depth: 0.0,
        world_position: [0.0; 3],
        world_normal: [0.0; 3],
    })
    .is_ok());
    assert!(validate_hit_proxy_product(SceneHitProxyProduct {
        token: 7,
        depth: 2.0,
        world_position: [0.0; 3],
        world_normal: [0.0, 1.0, 0.0],
    })
    .is_err());
}

#[test]
fn hit_proxy_readback_aggregation_retains_early_channels_until_all_complete() {
    let observed = Arc::new(Mutex::new(None));
    let completion = SceneHitProxyCompletion::new(Box::new({
        let observed = Arc::clone(&observed);
        move |result| *observed.lock().unwrap() = Some(result)
    }));
    let accumulator = Arc::new(Mutex::new(SceneHitProxyReadbackAccumulator::new(
        completion,
    )));

    record_token_readback(&accumulator, Ok(0));
    assert!(observed.lock().unwrap().is_none());
    record_position_readback(&accumulator, Ok([0.0; 4]));
    assert!(observed.lock().unwrap().is_none());
    record_normal_readback(&accumulator, Ok(vec![0; 8]));

    let result = observed.lock().unwrap().take().expect("completion result");
    assert_eq!(result.expect("clear product").token, 0);
}
