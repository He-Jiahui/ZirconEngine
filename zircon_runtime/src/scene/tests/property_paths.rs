//! 路径测试跨字符串解析、World 属性路由与写入值校验；
//! 跨模块调用方据此通过中立路径读取或修改组件，并收到明确的拒绝结果。

use crate::core::framework::scene::{ComponentPropertyPath, EntityPath, ScenePropertyValue};
use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{AnimationClipMarker, ResourceHandle, ResourceId};
use crate::scene::components::{
    AnimationPlayerComponent, MeshRenderer, NodeKind, RigidBodyComponent, RigidBodyType,
};
use crate::scene::world::World;

#[path = "property_paths/read_paths.rs"]
mod read_paths;
#[path = "property_paths/runtime_mutation.rs"]
mod runtime_mutation;
#[path = "property_paths/write_validation.rs"]
mod write_validation;
