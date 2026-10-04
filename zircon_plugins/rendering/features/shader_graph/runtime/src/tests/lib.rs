use super::*;

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
