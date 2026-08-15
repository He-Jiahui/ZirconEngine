use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::scene::components::{Hierarchy, Name};
use crate::scene::ecs::{
    Command, CommandQueue, Component, DeferredCommandOperation, DeferredCommandTarget,
    DeferredSystemKey, LifecycleEventKind, Resource, StorageType, WorkerCommandBuffer,
};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct SparseMarker;

impl Component for SparseMarker {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[derive(Debug, PartialEq, Eq)]
struct NestedDeferredResource;

impl Resource for NestedDeferredResource {}

#[derive(Debug, PartialEq, Eq)]
struct DeferredBarrierObservation(usize);

impl Resource for DeferredBarrierObservation {}

#[derive(Debug, PartialEq, Eq)]
struct MergedCommandOrder(Vec<u8>);

impl Resource for MergedCommandOrder {}

struct DropProbe {
    drops: Arc<AtomicUsize>,
}

impl Command for DropProbe {
    fn apply(self, _world: &mut World) {}
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

struct SmallNoop(u8);

impl Command for SmallNoop {
    fn apply(self, _world: &mut World) {
        let _ = self.0;
    }
}

struct LargeNoop([u8; 256]);

impl Command for LargeNoop {
    fn apply(self, _world: &mut World) {
        let _ = self.0;
    }
}

struct MaxInlineNoop([u8; 192]);

impl Command for MaxInlineNoop {
    fn apply(self, _world: &mut World) {
        let _ = self.0;
    }
}

#[repr(align(64))]
struct CacheAlignedNoop {
    calls: Arc<AtomicUsize>,
}

impl Command for CacheAlignedNoop {
    fn apply(self, _world: &mut World) {
        self.calls.fetch_add(1, Ordering::SeqCst);
    }
}

#[repr(align(128))]
struct OverAlignedNoop;

impl Command for OverAlignedNoop {
    fn apply(self, _world: &mut World) {}
}

struct EnqueueThenPanic;

impl Command for EnqueueThenPanic {
    fn apply(self, world: &mut World) {
        world.commands().queue_fn(|world| {
            world.insert_resource(NestedDeferredResource);
        });
        panic!("deferred command panic probe");
    }
}

mod packed_arena;
mod structural_batches;
