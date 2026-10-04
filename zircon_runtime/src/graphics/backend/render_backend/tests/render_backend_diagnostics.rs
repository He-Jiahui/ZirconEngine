use super::{decode_r32_uint_texel, decode_rgba32float_texel};

#[test]
fn pick_product_texel_decoders_require_exact_little_endian_payloads() {
    assert_eq!(decode_r32_uint_texel(7_u32.to_le_bytes().to_vec()), Ok(7));

    let expected = [1.0_f32, -2.5, 0.25, 0.75];
    let bytes = expected
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
    assert_eq!(decode_rgba32float_texel(bytes), Ok(expected));
    assert!(decode_r32_uint_texel(vec![0; 3]).is_err());
    assert!(decode_rgba32float_texel(vec![0; 12]).is_err());
}

#[test]
fn product_texel_decoder_has_no_production_lane_expect() {
    let source = include_str!("../render_backend_diagnostics.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("diagnostics source must retain its test-module boundary");
    assert!(!production.contains("fixed RGBA32F lane"));
}

#[test]
fn explicit_texture_capture_uses_the_product_diagnostic_owner() {
    let diagnostics = include_str!("../render_backend_diagnostics.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let scene_target = include_str!(
        "../../../scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs"
    );
    let renderer = include_str!(
        "../../../scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline/readback.rs"
    );
    let backend_root = include_str!("../mod.rs");
    let graphics_backend_root = include_str!("../../mod.rs");
    let standalone = diagnostics
        .split("fn finish_product_diagnostic_texture_readback(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn wait_for_product_diagnostic_callback(")
                .next()
        })
        .expect("standalone product diagnostic submission owner");

    assert!(diagnostics.contains("fn read_product_diagnostic_texture_rgba8_blocking("));
    assert!(diagnostics.contains("fn read_product_diagnostic_texture_rgba16float_blocking("));
    assert!(diagnostics.contains("begin_product_diagnostic_readback_scope(frame_generation)"));
    assert!(standalone.contains("scope.submit(label)"));
    assert!(!standalone.contains("create_command_encoder"));
    assert!(!standalone.contains("submit_graphics_command_buffers_with_diagnostics("));
    assert!(diagnostics.contains("wait_for_product_diagnostic_callback("));
    assert!(diagnostics.contains("PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT"));
    assert!(diagnostics.contains("self.poll_submission_completions()?"));
    assert!(diagnostics.contains("observe_poll(poll_receipt)?"));
    assert!(diagnostics.contains("receiver.try_recv()"));
    assert!(!diagnostics.contains("receiver.recv()"));
    assert!(!diagnostics.contains("wait_indefinitely"));
    assert!(scene_target.contains("backend.read_product_diagnostic_texture_rgba8_blocking("));
    assert!(renderer.contains(".read_product_diagnostic_texture_rgba16float_blocking("));
    assert!(backend_root.contains("#[cfg(test)]\nmod read_texture_rgba;"));
    assert!(backend_root.contains("#[cfg(test)]\nmod read_texture_rgba16float_region;"));
    assert!(graphics_backend_root
        .contains("#[cfg(test)]\npub(crate) use render_backend::read_texture_rgba;"));
    assert!(graphics_backend_root.contains(
        "#[cfg(test)]\npub(crate) use render_backend::{\n    read_texture_rgba16float_cube_mip_chain, read_texture_rgba16float_region,"
    ));
}
