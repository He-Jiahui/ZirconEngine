use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::Transform;

use super::{blend_layer_pose, AnimationStateMachineLayerError};
use crate::PoseLayerBlendMode;

#[test]
fn layer_pose_bone_name_mismatch_is_typed() {
    let mut base = pose("Hand");
    let layer = pose("Foot");

    let error =
        blend_layer_pose(&mut base, &layer, 1.0, PoseLayerBlendMode::Override, None).unwrap_err();

    assert_eq!(
        error,
        AnimationStateMachineLayerError::BoneNameMismatch {
            index: 0,
            base: "Hand".to_string(),
            layer: "Foot".to_string(),
        }
    );
}

fn pose(name: &str) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::StateMachine,
        active_state: Some("State".to_string()),
        bones: vec![AnimationPoseBone {
            name: name.to_string(),
            local_transform: Transform::default(),
        }],
    }
}
