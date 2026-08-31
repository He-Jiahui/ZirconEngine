use std::sync::Arc;

use crate::core::math::{Transform, Vec3};
use crate::scene::components::LocalTransform;
use crate::scene::ecs::{Component, Mut};
use crate::scene::{NodeKind, SystemStage, World};

#[derive(Debug, PartialEq, Eq)]
struct RenderValue(u32);

impl Component for RenderValue {}

fn publish_render_dirty_journal(world: &mut World) {
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
}

mod publication;
mod query_mutation;
mod render_component_projection;
