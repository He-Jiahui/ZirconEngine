use std::path::{Path, PathBuf};

use crate::core::framework::render::{
    RenderExtractContext, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
};
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{MeshRenderer, Mobility, NodeRecord};
use crate::scene::{NodeKind, SystemStage, World};

const LARGE_HIERARCHY_NODE_COUNT: usize = 256;

mod checked_parent_chain;
mod hierarchy_behavior;
mod hierarchy_rebuild;
mod projected_reads;
mod protected_authored_authority;
mod protected_authority;
mod runtime_freshness;
mod spawn_paths;
mod subtree_cycle_walk;
mod work_counters;

fn detached_node_record(id: u64, kind: NodeKind) -> NodeRecord {
    let mut source = World::empty();
    let entity = source
        .spawn_node(kind)
        .expect("test scene spawn should succeed");
    let mut record = source.node_record(entity).unwrap();
    record.id = id;
    record.name = format!("Imported {id}");
    record
}

fn pending_reparented_world() -> World {
    let mut world = World::new();
    let first_parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let second_parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .update_transform(
            first_parent,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(
            second_parent,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(first_parent)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world
        .set_parent_checked(child, Some(second_parent))
        .unwrap();
    world.set_active_self(second_parent, false).unwrap();
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(12.0, 0.0, 0.0)
    );
    world
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
