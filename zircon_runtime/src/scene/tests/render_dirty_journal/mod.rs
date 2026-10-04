//! 区分世界内的变化发布与渲染消费：稳定帧应复用同一工件，跨世界缓存必须重新绑定。

use std::sync::Arc;

use crate::core::math::{Transform, Vec3};
use crate::scene::components::LocalTransform;
use crate::scene::ecs::{Component, Mut};
use crate::scene::{NodeKind, SystemStage, World};

#[derive(Debug, PartialEq, Eq)]
struct RenderValue(u32);

impl Component for RenderValue {}

// 测试走正式提取阶段，使层级派生状态先就绪，再观察发布工件；不能直接清理脏标记替代该边界。
fn publish_render_dirty_journal(world: &mut World) {
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
}

mod publication;
mod query_mutation;
mod render_component_projection;
