use crate::{
    run_cpu, NnDataType, NnGemmAttrs, NnModelAsset, NnOp, NnOpAttrs, NnOpCode, NnPool2dAttrs,
    NnTensorDesc, NnTensorKind, NN_WEIGHT_ALIGNMENT,
};

#[test]
fn nn_cpu_reference_executes_gemm_then_relu() {
    let mut weights = Vec::new();
    for value in [1.0_f32, 0.0, 0.0, 1.0, 1.0, 1.0] {
        weights.extend_from_slice(&value.to_le_bytes());
    }
    weights.resize(NN_WEIGHT_ALIGNMENT as usize, 0);
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 2, [1, 1, 2, 3]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 2, [1, 1, 3, 2]),
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
            NnOp::new(NnOpCode::Relu, vec![2], vec![3], NnOpAttrs::None),
        ],
        weights,
    };

    let outputs = run_cpu(&model, &[(0, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])])
        .expect("the reference interpreter should execute supported ops");

    assert_eq!(outputs, vec![vec![4.0, 5.0, 10.0, 11.0]]);
}

#[test]
fn nn_cpu_reference_executes_nchw_conv2d() {
    let mut weights = Vec::new();
    for _ in 0..9 {
        weights.extend_from_slice(&1.0_f32.to_le_bytes());
    }
    weights.resize(NN_WEIGHT_ALIGNMENT as usize, 0);
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 4, [1, 1, 3, 3]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 4, [1, 1, 3, 3]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, [1, 1, 1, 1]),
        ],
        ops: vec![NnOp::new(
            NnOpCode::Conv2d,
            vec![0, 1],
            vec![2],
            NnOpAttrs::Conv2d(crate::NnConv2dAttrs::default()),
        )],
        weights,
    };

    let outputs = run_cpu(
        &model,
        &[(0, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])],
    )
    .expect("the reference interpreter should execute Conv2d");

    assert_eq!(outputs, vec![vec![45.0]]);
}

#[test]
fn nn_cpu_reference_executes_nchw_max_and_average_pooling() {
    let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let pool = NnPool2dAttrs {
        kernel: [2, 2],
        stride: [1, 1],
        padding: [0, 0, 0, 0],
    };

    for (code, expected) in [
        (NnOpCode::MaxPool2d, vec![5.0, 6.0, 8.0, 9.0]),
        (NnOpCode::AvgPool2d, vec![3.0, 4.0, 6.0, 7.0]),
    ] {
        let model = NnModelAsset {
            tensors: vec![
                NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 4, [1, 1, 3, 3]),
                NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, [1, 1, 2, 2]),
            ],
            ops: vec![NnOp::new(code, vec![0], vec![1], NnOpAttrs::Pool2d(pool))],
            weights: Vec::new(),
        };

        assert_eq!(run_cpu(&model, &[(0, &input)]), Ok(vec![expected]));
    }
}

#[test]
fn nn_cpu_reference_executes_integer_nearest_nchw_upsample() {
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 4, [1, 1, 2, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, [1, 1, 4, 4]),
        ],
        ops: vec![NnOp::new(
            NnOpCode::Upsample2d,
            vec![0],
            vec![1],
            NnOpAttrs::Upsample2d { scale: [2, 2] },
        )],
        weights: Vec::new(),
    };

    assert_eq!(
        run_cpu(&model, &[(0, &[1.0, 2.0, 3.0, 4.0])]),
        Ok(vec![vec![
            1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 3.0, 3.0, 4.0, 4.0,
        ]])
    );
}

#[test]
fn nn_cpu_reference_executes_channelwise_batch_normalization() {
    let mut weights = vec![0; NN_WEIGHT_ALIGNMENT as usize * 4];
    for (offset, values) in [
        (0, [2.0_f32, 0.5]),
        (NN_WEIGHT_ALIGNMENT as usize, [0.0, 1.0]),
        (NN_WEIGHT_ALIGNMENT as usize * 2, [1.0, 2.0]),
        (NN_WEIGHT_ALIGNMENT as usize * 3, [1.0, 4.0]),
    ] {
        for (index, value) in values.into_iter().enumerate() {
            weights[offset + index * 4..offset + (index + 1) * 4]
                .copy_from_slice(&value.to_le_bytes());
        }
    }
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 4, [1, 2, 1, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(0),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(NN_WEIGHT_ALIGNMENT),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(NN_WEIGHT_ALIGNMENT * 2),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(NN_WEIGHT_ALIGNMENT * 3),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, [1, 2, 1, 2]),
        ],
        ops: vec![NnOp::new(
            NnOpCode::BatchNorm,
            vec![0, 1, 2, 3, 4],
            vec![5],
            NnOpAttrs::BatchNorm { epsilon: 0.0 },
        )],
        weights,
    };

    assert_eq!(
        run_cpu(&model, &[(0, &[1.0, 3.0, 2.0, 4.0])]),
        Ok(vec![vec![0.0, 4.0, 1.0, 1.5]])
    );
}

#[test]
fn nn_cpu_reference_executes_last_dimension_layer_normalization() {
    let mut weights = vec![0; NN_WEIGHT_ALIGNMENT as usize * 2];
    for (offset, values) in [
        (0, [2.0_f32, 0.5]),
        (NN_WEIGHT_ALIGNMENT as usize, [0.0, 1.0]),
    ] {
        for (index, value) in values.into_iter().enumerate() {
            weights[offset + index * 4..offset + (index + 1) * 4]
                .copy_from_slice(&value.to_le_bytes());
        }
    }
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 2, [1, 1, 2, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(0),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 2])
                .with_weight_offset(NN_WEIGHT_ALIGNMENT),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 2, [1, 1, 2, 2]),
        ],
        ops: vec![NnOp::new(
            NnOpCode::LayerNorm,
            vec![0, 1, 2],
            vec![3],
            NnOpAttrs::LayerNorm { epsilon: 0.0 },
        )],
        weights,
    };

    assert_eq!(
        run_cpu(&model, &[(0, &[1.0, 3.0, 2.0, 4.0])]),
        Ok(vec![vec![-2.0, 1.5, -2.0, 1.5]])
    );
}
