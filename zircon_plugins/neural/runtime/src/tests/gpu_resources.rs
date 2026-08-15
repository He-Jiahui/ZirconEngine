use crate::{
    NnDataType, NnModelAsset, NnTensorDesc, NnTensorKind, NnTensorLayout, NnWeightUploadPlan,
    NN_WEIGHT_ALIGNMENT,
};

#[test]
fn nn_tensor_layout_uses_contiguous_nchw_storage() {
    let layout = NnTensorLayout::from_descriptor(&NnTensorDesc::new(
        NnDataType::F32,
        NnTensorKind::Intermediate,
        4,
        [1, 3, 4, 5],
    ))
    .expect("valid tensor descriptor should have a GPU layout");

    assert_eq!(layout.element_count, 60);
    assert_eq!(layout.storage_size_bytes, 240);
}

#[test]
fn nn_weight_upload_plan_keeps_model_offsets_and_blob() {
    let model = NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 1])
                .with_weight_offset(0),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Weight, 1, [1, 1, 1, 1])
                .with_weight_offset(NN_WEIGHT_ALIGNMENT),
        ],
        ops: Vec::new(),
        weights: vec![0; (NN_WEIGHT_ALIGNMENT * 2) as usize],
    };

    let upload = NnWeightUploadPlan::from_model(&model, "nn.weights")
        .expect("valid model weights should create an upload plan");

    assert_eq!(upload.resource_name, "nn.weights");
    assert_eq!(upload.offset_for_tensor(1), Some(NN_WEIGHT_ALIGNMENT));
    assert_eq!(upload.bytes.len(), model.weights.len());
}
