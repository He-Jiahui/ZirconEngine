//! World 类型化 ECS 测试共享夹具；子模块核对组件、资源、Bundle 和持久化状态的共同边界。

use serde_json::json;
use std::path::{Path, PathBuf};

use crate::core::framework::scene::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{LocalTransform, Name, RenderLayerMask, RigidBodyComponent};
use crate::scene::ecs::{Component, Resource};
use crate::scene::{SceneError, World};

#[path = "ecs_typed_api/bundle_default_overrides.rs"]
mod bundle_default_overrides;
#[path = "ecs_typed_api/bundle_lifecycle.rs"]
mod bundle_lifecycle;
#[path = "ecs_typed_api/bundle_transactions.rs"]
mod bundle_transactions;
#[path = "ecs_typed_api/bundle_width.rs"]
mod bundle_width;
#[path = "ecs_typed_api/component_mutation.rs"]
mod component_mutation;
#[path = "ecs_typed_api/component_registry.rs"]
mod component_registry;
#[path = "ecs_typed_api/persistent_animation.rs"]
mod persistent_animation;
#[path = "ecs_typed_api/persistent_entity_core.rs"]
mod persistent_entity_core;
#[path = "ecs_typed_api/persistent_lighting.rs"]
mod persistent_lighting;
#[path = "ecs_typed_api/persistent_scene_render.rs"]
mod persistent_scene_render;
#[path = "ecs_typed_api/resource_state.rs"]
mod resource_state;
#[path = "ecs_typed_api/runtime_state.rs"]
mod runtime_state;
#[path = "ecs_typed_api/source_contracts.rs"]
mod source_contracts;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Mana(u32);

impl Component for Mana {}

#[derive(Debug, PartialEq, Eq)]
struct FrameCounter(u32);

impl Resource for FrameCounter {}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
