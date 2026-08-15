use serde_json::json;
use std::path::{Path, PathBuf};

use crate::core::framework::scene::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{LocalTransform, Name, RenderLayerMask, RigidBodyComponent};
use crate::scene::ecs::{Component, Resource};
use crate::scene::{SceneError, World};

mod bundle_default_overrides;
mod bundle_lifecycle;
mod bundle_transactions;
mod bundle_width;
mod component_mutation;
mod component_registry;
mod persistent_animation;
mod persistent_entity_core;
mod persistent_lighting;
mod persistent_scene_render;
mod resource_state;
mod runtime_state;
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
