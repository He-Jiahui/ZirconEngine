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
fn shader_graph_reports_missing_add_operands() {
    let asset = ShaderGraphAsset {
        name: "missing-add-inputs".to_string(),
        target: ShaderGraphTarget::PostProcess,
        nodes: vec![
            ShaderGraphNode::Add {
                id: "sum".to_string(),
                left: "missing".to_string(),
                right: "missing".to_string(),
            },
            ShaderGraphNode::ColorOutput {
                input: "sum".to_string(),
            },
        ],
    };

    let report = compile_shader_graph_to_wgsl(&asset);

    assert_eq!(report.diagnostics.len(), 2, "{:?}", report.diagnostics);
    assert!(report
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.contains("missing node `missing`")));
    assert!(report
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.contains("node `sum`")));
}

#[test]
fn shader_graph_reports_missing_multiply_operand() {
    let report = compile_shader_graph_to_wgsl(&ShaderGraphAsset {
        name: "missing-multiply-input".to_string(),
        target: ShaderGraphTarget::Material,
        nodes: vec![
            ShaderGraphNode::ConstantFloat {
                id: "known".to_string(),
                value: 0.5,
            },
            ShaderGraphNode::Multiply {
                id: "product".to_string(),
                left: "known".to_string(),
                right: "absent".to_string(),
            },
            ShaderGraphNode::ColorOutput {
                input: "product".to_string(),
            },
        ],
    });

    assert_eq!(report.diagnostics.len(), 1, "{:?}", report.diagnostics);
    assert!(report.diagnostics[0].contains("node `product`"));
    assert!(report.diagnostics[0].contains("missing node `absent`"));
}

#[test]
fn shader_graph_reports_missing_output_inputs() {
    let color = compile_shader_graph_to_wgsl(&ShaderGraphAsset {
        name: "missing-color-output-input".to_string(),
        target: ShaderGraphTarget::PostProcess,
        nodes: vec![ShaderGraphNode::ColorOutput {
            input: "unknown_color".to_string(),
        }],
    });
    assert_eq!(color.diagnostics.len(), 1, "{:?}", color.diagnostics);
    assert!(color.diagnostics[0].contains("node `ColorOutput`"));
    assert!(color.diagnostics[0].contains("missing node `unknown_color`"));

    let material = compile_shader_graph_to_wgsl(&ShaderGraphAsset {
        name: "missing-material-output-inputs".to_string(),
        target: ShaderGraphTarget::Material,
        nodes: vec![ShaderGraphNode::MaterialOutput {
            base_color: "unknown_color".to_string(),
            roughness: "unknown_roughness".to_string(),
        }],
    });
    assert_eq!(material.diagnostics.len(), 2, "{:?}", material.diagnostics);
    assert!(material
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("missing node `unknown_color`")));
    assert!(material
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("missing node `unknown_roughness`")));
}

#[test]
fn shader_graph_reports_references_used_before_declaration() {
    let report = compile_shader_graph_to_wgsl(&ShaderGraphAsset {
        name: "forward-reference".to_string(),
        target: ShaderGraphTarget::PostProcess,
        nodes: vec![
            ShaderGraphNode::Add {
                id: "sum".to_string(),
                left: "left".to_string(),
                right: "right".to_string(),
            },
            ShaderGraphNode::ConstantFloat {
                id: "left".to_string(),
                value: 1.0,
            },
            ShaderGraphNode::ConstantFloat {
                id: "right".to_string(),
                value: 2.0,
            },
            ShaderGraphNode::ColorOutput {
                input: "sum".to_string(),
            },
        ],
    });

    assert_eq!(report.diagnostics.len(), 2, "{:?}", report.diagnostics);
    assert!(report
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.contains("before it is declared")));
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
