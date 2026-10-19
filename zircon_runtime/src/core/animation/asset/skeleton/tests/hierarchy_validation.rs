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

#[test]
fn hierarchy_encoding_preserves_typed_cycle_and_bounds_sources() {
    use crate::core::framework::animation::asset::{
        AnimationAssetError, AnimationSkeletonHierarchyError,
    };
    for (parent, expected) in [
        (
            0,
            AnimationSkeletonHierarchyError::ParentCycle { bone_index: 0 },
        ),
        (
            99,
            AnimationSkeletonHierarchyError::ParentOutOfBounds {
                bone_index: 0,
                parent_index: 99,
                bone_count: 1,
            },
        ),
    ] {
        let skeleton = AnimationSkeletonAsset {
            name: None,
            bones: vec![AnimationSkeletonBoneAsset {
                name: "invalid".to_owned(),
                parent_index: Some(parent),
                local_translation: [0.0; 3],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0; 3],
            }],
        };
        let error = skeleton
            .to_bytes()
            .expect_err("invalid hierarchy must not encode");
        assert!(matches!(
            &error,
            AnimationAssetError::InvalidSkeletonHierarchy { reason } if reason == &expected
        ));
        assert_eq!(
            std::error::Error::source(&error)
                .and_then(|source| source.downcast_ref::<AnimationSkeletonHierarchyError>()),
            Some(&expected)
        );
    }
}

#[test]
fn hierarchy_accepts_multiple_roots_and_parent_chains() {
    let skeleton = AnimationSkeletonAsset {
        name: None,
        bones: [None, Some(0), Some(1), None]
            .into_iter()
            .enumerate()
            .map(|(index, parent_index)| AnimationSkeletonBoneAsset {
                name: index.to_string(),
                parent_index,
                local_translation: [0.0; 3],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0; 3],
            })
            .collect(),
    };
    let bytes = skeleton.to_bytes().expect("valid forest encodes");
    assert_eq!(
        AnimationSkeletonAsset::from_bytes(&bytes).expect("valid forest decodes"),
        skeleton
    );
}
