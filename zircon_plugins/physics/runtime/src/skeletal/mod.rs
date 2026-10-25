#[path = "codec.rs"]
mod codec;
#[path = "profile.rs"]
mod profile;
#[path = "runtime.rs"]
mod runtime;

pub use codec::RagdollProfileEncodeError;
pub use profile::{
    RagdollBoneProfile, RagdollProfile, RagdollProfileError, RagdollSpawn, RagdollSpawnError,
};
pub(crate) use runtime::{drive_ragdoll_bodies_from_animation, write_simulated_pose_feed};
pub use runtime::{RagdollMode, RagdollRuntime};

#[cfg(test)]
#[path = "tests/authoring.rs"]
mod authoring_tests;
#[cfg(test)]
#[path = "tests/codec.rs"]
mod codec_tests;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
