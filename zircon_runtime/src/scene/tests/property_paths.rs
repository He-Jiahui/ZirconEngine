use crate::core::framework::scene::{ComponentPropertyPath, EntityPath, ScenePropertyValue};
use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{AnimationClipMarker, ResourceHandle, ResourceId};
use crate::scene::components::{
    AnimationPlayerComponent, MeshRenderer, NodeKind, RigidBodyComponent, RigidBodyType,
};
use crate::scene::world::World;

mod read_paths;
mod runtime_mutation;
mod write_validation;
