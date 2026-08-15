use crate::{
    NnDataType, NnGemmAttrs, NnModelAsset, NnOp, NnOpAttrs, NnOpCode, NnTensorDesc, NnTensorKind,
    NN_WEIGHT_ALIGNMENT,
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
