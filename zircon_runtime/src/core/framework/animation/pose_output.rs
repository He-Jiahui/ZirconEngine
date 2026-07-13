use serde::{Deserialize, Serialize};

use super::{AnimationPoseBone, AnimationPoseSource};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPoseOutput {
    pub source: AnimationPoseSource,
    pub active_state: Option<String>,
    pub bones: Vec<AnimationPoseBone>,
}

impl AnimationPoseOutput {
    /// Copies a pose while retaining existing bone and bone-name allocations
    /// whenever the skeleton topology is stable.
    pub fn clone_from_reusing_storage(&mut self, source: &Self) {
        self.source = source.source;
        self.active_state.clone_from(&source.active_state);

        let shared_bones = self.bones.len().min(source.bones.len());
        for (target, source) in self.bones[..shared_bones]
            .iter_mut()
            .zip(&source.bones[..shared_bones])
        {
            target.name.clone_from(&source.name);
            target.local_transform = source.local_transform;
        }
        self.bones.truncate(source.bones.len());
        self.bones.extend_from_slice(&source.bones[shared_bones..]);
    }
}
