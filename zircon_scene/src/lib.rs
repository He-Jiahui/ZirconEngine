//! ECS worlds, persistence, and render extraction.

pub type EntityId = u64;
pub type NodeId = EntityId;

pub mod components;
mod render_extract;
pub mod semantics;
pub mod serializer;
pub mod world;

#[allow(unused_imports)]
pub(crate) use components::{
    Mobility, NodeKind, NodeRecord, Schedule, SystemStage, default_render_layer_mask,
};

pub type Scene = world::World;

#[cfg(test)]
mod tests;
