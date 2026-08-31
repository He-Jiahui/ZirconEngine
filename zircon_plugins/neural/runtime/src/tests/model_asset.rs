use crate::{
    NnDataType, NnGemmAttrs, NnModelAsset, NnModelFormatError, NnOp, NnOpAttrs, NnOpCode,
    NnTensorDesc, NnTensorKind, NN_WEIGHT_ALIGNMENT,
};

fn sample_model() -> NnModelAsset {
    NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 2, [1, 1, 2, 3]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 2, [1, 1, 3, 2])
                .with_weight_offset(0),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 2, [1, 1, 2, 2]),
        ],
        ops: vec![NnOp::new(
            NnOpCode::Gemm,
            vec![0, 1],
            vec![2],
            NnOpAttrs::Gemm(NnGemmAttrs::default()),
        )],
        weights: vec![0; NN_WEIGHT_ALIGNMENT as usize],
    }
}

#[test]
fn nn_model_asset_binary_roundtrip() {
    let model = sample_model();
    let bytes = model.to_znn_bytes().expect("sample model should serialize");

    let weight_blob_offset = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    assert_eq!(weight_blob_offset % NN_WEIGHT_ALIGNMENT, 0);

    let decoded = NnModelAsset::from_znn_bytes(&bytes).expect("serialized model should load");
    assert_eq!(decoded, model);
}

#[test]
fn nn_model_asset_rejects_bad_magic_or_version() {
    let bytes = sample_model()
        .to_znn_bytes()
        .expect("sample model should serialize");

    let mut bad_magic = bytes.clone();
    bad_magic[0] = b'X';
    assert!(NnModelAsset::from_znn_bytes(&bad_magic).is_err());

    let mut bad_version = bytes;
    bad_version[4..8].copy_from_slice(&2_u32.to_le_bytes());
    assert!(NnModelAsset::from_znn_bytes(&bad_version).is_err());
}

#[test]
fn nn_model_asset_rejects_unaligned_weight_tensor_offset() {
    let mut model = sample_model();
    model.tensors[1].weight_offset = 4;

    assert!(model.validate().is_err());
}

#[test]
fn nn_model_asset_rejects_weight_tensor_past_the_blob_end() {
    let mut model = sample_model();
    model.weights.clear();

    assert!(model.validate().is_err());
}

#[test]
fn nn_model_asset_rejects_declared_op_count_beyond_table_capacity_before_allocation() {
    let mut bytes = sample_model()
        .to_znn_bytes()
        .expect("sample model should serialize");
    bytes[16..20].copy_from_slice(&u32::MAX.to_le_bytes());

    let error = NnModelAsset::from_znn_bytes(&bytes)
        .expect_err("untrusted op count must be admitted before allocation");

    assert!(matches!(
        error,
        NnModelFormatError::ResourceLimitExceeded {
            resource: "op_count",
            actual,
            limit,
        } if actual == u64::from(u32::MAX) && limit == 1_048_576
    ));
}

#[test]
fn nn_model_asset_rejects_weight_blob_over_budget_before_copy() {
    let mut bytes = sample_model()
        .to_znn_bytes()
        .expect("sample model should serialize");
    let declared_weight_bytes = 1_u64 << 40;
    bytes[32..40].copy_from_slice(&declared_weight_bytes.to_le_bytes());

    let error = NnModelAsset::from_znn_bytes(&bytes)
        .expect_err("untrusted weight length must be admitted before copying");

    assert!(matches!(
        error,
        NnModelFormatError::ResourceLimitExceeded {
            resource: "weight_blob_bytes",
            actual,
            limit,
        } if actual == declared_weight_bytes && limit == 512 * 1024 * 1024
    ));
}

#[test]
fn nn_model_asset_rejects_op_count_that_cannot_fit_the_encoded_table() {
    let mut bytes = sample_model()
        .to_znn_bytes()
        .expect("sample model should serialize");
    let op_table_size = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
    let impossible_count = op_table_size / 8 + 1;
    bytes[16..20].copy_from_slice(&impossible_count.to_le_bytes());

    let error = NnModelAsset::from_znn_bytes(&bytes)
        .expect_err("op count must be proved by the minimum encoded record size");

    assert!(matches!(
        error,
        NnModelFormatError::DeclaredCountExceedsTableCapacity {
            resource: "op_count",
            declared,
            maximum,
        } if declared == impossible_count as usize && maximum == (op_table_size / 8) as usize
    ));
}
