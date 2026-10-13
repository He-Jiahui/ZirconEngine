use serde::Deserialize;

use crate::core::framework::animation::{AnimationSkeletonAsset, AnimationSkeletonBoneAsset};

#[test]
fn deserialization_rejects_parent_cycles_and_invalid_parent_indices() {
    let cycle = r#"{"name":null,"bones":[
        {"name":"a","parent_index":1,"local_translation":[0.0,0.0,0.0],"local_rotation":[0.0,0.0,0.0,1.0],"local_scale":[1.0,1.0,1.0]},
        {"name":"b","parent_index":0,"local_translation":[0.0,0.0,0.0],"local_rotation":[0.0,0.0,0.0,1.0],"local_scale":[1.0,1.0,1.0]}
    ]}"#;
    let error = AnimationSkeletonAsset::deserialize(&mut serde_json::Deserializer::from_str(cycle))
        .expect_err("cyclic skeletons must be rejected at the deserialize boundary");
    assert!(error.to_string().contains("parent cycle"));

    let invalid_parent = r#"{"name":null,"bones":[
        {"name":"orphan","parent_index":99,"local_translation":[0.0,0.0,0.0],"local_rotation":[0.0,0.0,0.0,1.0],"local_scale":[1.0,1.0,1.0]}
    ]}"#;
    let error = AnimationSkeletonAsset::deserialize(&mut serde_json::Deserializer::from_str(
        invalid_parent,
    ))
    .expect_err("out-of-range parent indices must be rejected at the deserialize boundary");
    assert!(error.to_string().contains("out-of-range parent"));
}

#[test]
fn encoding_rejects_directly_constructed_invalid_skeletons() {
    let skeleton = AnimationSkeletonAsset {
        name: None,
        bones: vec![AnimationSkeletonBoneAsset {
            name: "self".to_owned(),
            parent_index: Some(0),
            local_translation: [0.0; 3],
            local_rotation: [0.0, 0.0, 0.0, 1.0],
            local_scale: [1.0; 3],
        }],
    };
    assert!(skeleton.to_bytes().is_err());
}
