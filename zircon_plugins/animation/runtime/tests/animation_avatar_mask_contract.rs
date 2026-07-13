use zircon_plugin_animation_runtime::{AvatarMaskAsset, MaskWeights, SkeletonTargetTable};
use zircon_runtime::core::framework::animation::{
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};

#[test]
fn subtree_weight_inherits_and_overrides() {
    let table = SkeletonTargetTable::compile(&skeleton()).unwrap();
    let asset = AvatarMaskAsset::from_toml(
        r#"
id = "upper_body"
default_weight = 0.0

[[rules]]
target = "Root/Spine"
weight = 1.0

[[rules]]
target = "Root/Spine/Chest/Hand"
weight = 0.35
"#,
    )
    .unwrap();
    let weights = MaskWeights::compile(&asset, &table).unwrap();

    assert_eq!(weights.as_slice(), &[0.0, 1.0, 1.0, 0.35]);
}

#[test]
fn mask_boundary_gradient_values() {
    let table = SkeletonTargetTable::compile(&skeleton()).unwrap();
    let asset = AvatarMaskAsset::from_toml(
        r#"
id = "upper_body_gradient"
default_weight = 0.0

[[rules]]
target = "Root/Spine"
weight = 1.0
boundary_weights = [0.2, 0.6, 1.0]
"#,
    )
    .unwrap();
    let weights = MaskWeights::compile(&asset, &table).unwrap();

    assert_eq!(weights.as_slice(), &[0.0, 0.2, 0.6, 1.0]);
}

fn skeleton() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("Avatar".into()),
        bones: vec![
            bone("Root", None),
            bone("Spine", Some(0)),
            bone("Chest", Some(1)),
            bone("Hand", Some(2)),
        ],
    }
}

fn bone(name: &str, parent_index: Option<u32>) -> AnimationSkeletonBoneAsset {
    AnimationSkeletonBoneAsset {
        name: name.into(),
        parent_index,
        local_translation: [0.0; 3],
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0; 3],
    }
}
