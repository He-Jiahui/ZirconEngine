use crate::{
    NnConv2dAttrs, NnDataType, NnGemmAttrs, NnGraphExecutor, NnGraphIo, NnModelAsset, NnOp,
    NnOpAttrs, NnOpCode, NnPool2dAttrs, NnTensorDesc, NnTensorKind,
};
use zircon_runtime::graphics::ComputeShaderSource;
use zircon_runtime::render_graph::ComputeBindingKind;

#[test]
fn nn_graph_executor_emits_compute_descriptors_and_folds_view_ops() {
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 2, [1, 1, 2, 3]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 2, [1, 1, 3, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Intermediate, 2, [1, 1, 2, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Intermediate, 2, [1, 1, 2, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 2, [1, 1, 2, 2]),
        ],
        ops: vec![
            NnOp::new(
                NnOpCode::Gemm,
                vec![0, 1],
                vec![2],
                NnOpAttrs::Gemm(NnGemmAttrs::default()),
            ),
            NnOp::new(NnOpCode::Reshape, vec![2], vec![3], NnOpAttrs::None),
            NnOp::new(NnOpCode::Relu, vec![3], vec![4], NnOpAttrs::None),
        ],
        weights: vec![0; 256],
    };
    let io = NnGraphIo::new("nn.weights")
        .with_input(0, "scene-tensor")
        .with_output(4, "post-output");

    let plan = NnGraphExecutor::default()
        .build_plan(&model, &io)
        .expect("supported neural graph should plan compute descriptors");
    let passes = plan.iter().map(|pass| &pass.descriptor).collect::<Vec<_>>();

    assert_eq!(passes.len(), 2);
    assert_eq!(plan[0].parameter_bytes.len(), 32);
    assert_eq!(plan[1].parameter_bytes.len(), 16);
    assert_eq!(
        passes[0].bindings[0].kind,
        ComputeBindingKind::UniformBuffer
    );
    assert_eq!(passes[0].bindings[1].resource, "scene-tensor");
    assert_eq!(passes[0].bindings[2].resource, "nn.weights");
    assert_eq!(passes[0].bindings[2].buffer_offset, Some(0));
    assert_eq!(passes[1].bindings[1].resource, "nn.tensor.2");
    assert_eq!(passes[1].bindings[2].resource, "post-output");
    assert!(matches!(
        &passes[0].shader,
        ComputeShaderSource::InlineWgsl { label, .. } if label.contains("nn.gemm")
    ));
}

#[test]
fn nn_graph_executor_plans_nchw_conv2d_with_fixed_uniform_layout() {
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 4, [1, 1, 4, 4]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 4, [1, 1, 3, 3]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, [1, 1, 2, 2]),
        ],
        ops: vec![NnOp::new(
            NnOpCode::Conv2d,
            vec![0, 1],
            vec![2],
            NnOpAttrs::Conv2d(NnConv2dAttrs::default()),
        )],
        weights: vec![0; 256],
    };
    let io = NnGraphIo::new("nn.weights")
        .with_input(0, "scene-tensor")
        .with_output(2, "post-output");

    let plan = NnGraphExecutor::default()
        .build_plan(&model, &io)
        .expect("a supported Conv2d should produce a generic compute pass");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].parameter_bytes.len(), 80);
    assert_eq!(plan[0].descriptor.workgroup_size, [8, 8, 1]);
    assert_eq!(plan[0].descriptor.bindings[2].resource, "nn.weights");
    assert_eq!(plan[0].descriptor.bindings[2].buffer_offset, Some(0));
    assert!(matches!(
        &plan[0].descriptor.shader,
        ComputeShaderSource::InlineWgsl { label, source }
            if label.contains("nn.conv2d") && source.contains("struct ConvParams")
    ));
}

#[test]
fn nn_graph_executor_plans_pool_and_integer_upsample_passes() {
    let pool = NnPool2dAttrs {
        kernel: [2, 2],
        stride: [1, 1],
        padding: [0, 0, 0, 0],
    };
    for (code, input_shape, output_shape, attrs, parameter_bytes, shader_marker) in [
        (
            NnOpCode::MaxPool2d,
            [1, 1, 3, 3],
            [1, 1, 2, 2],
            NnOpAttrs::Pool2d(pool),
            64,
            "PoolParams",
        ),
        (
            NnOpCode::AvgPool2d,
            [1, 1, 3, 3],
            [1, 1, 2, 2],
            NnOpAttrs::Pool2d(pool),
            64,
            "PoolParams",
        ),
        (
            NnOpCode::Upsample2d,
            [1, 1, 2, 2],
            [1, 1, 4, 4],
            NnOpAttrs::Upsample2d { scale: [2, 2] },
            48,
            "UpsampleParams",
        ),
    ] {
        let model = NnModelAsset {
            tensors: vec![
                NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 4, input_shape),
                NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, output_shape),
            ],
            ops: vec![NnOp::new(code, vec![0], vec![1], attrs)],
            weights: Vec::new(),
        };
        let io = NnGraphIo::new("nn.weights")
            .with_input(0, "scene-tensor")
            .with_output(1, "post-output");

        let plan = NnGraphExecutor::default()
            .build_plan(&model, &io)
            .expect("V1 pooling and integer upsample ops should plan compute passes");

        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].parameter_bytes.len(), parameter_bytes);
        assert_eq!(plan[0].descriptor.workgroup_size, [8, 8, 1]);
        assert!(matches!(
            &plan[0].descriptor.shader,
            ComputeShaderSource::InlineWgsl { source, .. } if source.contains(shader_marker)
        ));
    }
}

#[test]
fn nn_graph_executor_plans_batch_normalization_with_channel_parameters() {
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 4, [1, 2, 1, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(0),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(256),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(512),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(768),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, [1, 2, 1, 2]),
        ],
        ops: vec![NnOp::new(
            NnOpCode::BatchNorm,
            vec![0, 1, 2, 3, 4],
            vec![5],
            NnOpAttrs::BatchNorm { epsilon: 1.0e-5 },
        )],
        weights: vec![0; 1024],
    };
    let io = NnGraphIo::new("nn.weights")
        .with_input(0, "scene-tensor")
        .with_output(5, "post-output");

    let plan = NnGraphExecutor::default()
        .build_plan(&model, &io)
        .expect("BatchNorm should plan a generic compute pass");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].parameter_bytes.len(), 32);
    assert_eq!(plan[0].descriptor.workgroup_size, [64, 1, 1]);
    assert_eq!(plan[0].descriptor.bindings.len(), 7);
    assert!(matches!(
        &plan[0].descriptor.shader,
        ComputeShaderSource::InlineWgsl { source, .. }
            if source.contains("struct BatchNormParams")
    ));
}

#[test]
fn nn_graph_executor_plans_last_dimension_layer_normalization() {
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 2, [1, 1, 2, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(0),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(256),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 2, [1, 1, 2, 2]),
        ],
        ops: vec![NnOp::new(
            NnOpCode::LayerNorm,
            vec![0, 1, 2],
            vec![3],
            NnOpAttrs::LayerNorm { epsilon: 1.0e-5 },
        )],
        weights: vec![0; 512],
    };
    let io = NnGraphIo::new("nn.weights")
        .with_input(0, "scene-tensor")
        .with_output(3, "post-output");

    let plan = NnGraphExecutor::default()
        .build_plan(&model, &io)
        .expect("last-dimension LayerNorm should plan a generic compute pass");

    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].parameter_bytes.len(), 32);
    assert_eq!(plan[0].descriptor.workgroup_size, [64, 1, 1]);
    assert_eq!(plan[0].descriptor.bindings.len(), 5);
    assert!(matches!(
        &plan[0].descriptor.shader,
        ComputeShaderSource::InlineWgsl { source, .. }
            if source.contains("struct LayerNormParams")
    ));
}
