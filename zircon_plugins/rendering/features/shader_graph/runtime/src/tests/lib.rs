use super::*;

#[path = "../../../../../test_support/frame_smoke.rs"]
mod frame_smoke;

#[test]
fn shader_graph_compiles_minimal_color_output_to_wgsl() {
    let asset = ShaderGraphAsset {
        name: "flat".to_string(),
        target: ShaderGraphTarget::PostProcess,
        nodes: vec![
            ShaderGraphNode::ConstantColor {
                id: "color".to_string(),
                value: [0.2, 0.3, 0.4, 1.0],
            },
            ShaderGraphNode::ColorOutput {
                input: "color".to_string(),
            },
        ],
    };

    let report = compile_shader_graph_to_wgsl(&asset);

    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert!(report.wgsl.contains("fn zircon_post_process_graph"));
    assert!(report.wgsl.contains("return color;"));
}

#[test]
fn shader_graph_feature_is_opt_in() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(report.extensions.render_features()[0].name, FEATURE_NAME);
}

#[test]
fn shader_graph_feature_executes_a_real_product_frame() {
    let report = compile_shader_graph_to_wgsl(&ShaderGraphAsset {
        name: "product-frame-smoke".to_string(),
        target: ShaderGraphTarget::PostProcess,
        nodes: vec![
            ShaderGraphNode::ConstantColor {
                id: "color".to_string(),
                value: [0.25, 0.5, 0.75, 1.0],
            },
            ShaderGraphNode::ColorOutput {
                input: "color".to_string(),
            },
        ],
    });
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert!(report.wgsl.contains("fn zircon_post_process_graph"));

    let (frame, stats) = frame_smoke::render_nonempty_product_frame(
        [render_feature_descriptor()],
        [render_pass_executor_registration()],
    );
    assert_eq!((frame.width, frame.height), (96, 72));
    assert!(stats
        .last_graph_executed_executor_ids
        .iter()
        .any(|executor| executor == EXECUTOR_ID));
}
