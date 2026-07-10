use std::fmt;
use std::ptr;
use std::sync::OnceLock;

use joltc_sys::{
    JPC_BroadPhaseLayerInterface, JPC_BroadPhaseLayerInterface_delete,
    JPC_BroadPhaseLayerInterface_new, JPC_FactoryInit, JPC_JobSystemThreadPool,
    JPC_JobSystemThreadPool_delete, JPC_JobSystemThreadPool_new2, JPC_ObjectLayerPairFilter,
    JPC_ObjectLayerPairFilter_delete, JPC_ObjectLayerPairFilter_new,
    JPC_ObjectVsBroadPhaseLayerFilter, JPC_ObjectVsBroadPhaseLayerFilter_delete,
    JPC_ObjectVsBroadPhaseLayerFilter_new, JPC_PhysicsSystem, JPC_PhysicsSystem_Init,
    JPC_PhysicsSystem_Update, JPC_PhysicsSystem_delete, JPC_PhysicsSystem_new,
    JPC_RegisterDefaultAllocator, JPC_RegisterTypes, JPC_TempAllocatorImpl,
    JPC_TempAllocatorImpl_delete, JPC_TempAllocatorImpl_new, JPC_MAX_PHYSICS_BARRIERS,
    JPC_MAX_PHYSICS_JOBS,
};

use crate::backend::PhysicsBackendError;

use super::layers::{
    BROAD_PHASE_LAYER_INTERFACE, OBJECT_LAYER_PAIR_FILTER, OBJECT_VS_BROAD_PHASE_FILTER,
};

const BACKEND_NAME: &str = "jolt";
const TEMP_ALLOCATOR_BYTES: u32 = 16 * 1024 * 1024;
const MAX_BODIES: u32 = 16_384;
const NUM_BODY_MUTEXES: u32 = 0;
const MAX_BODY_PAIRS: u32 = 65_536;
const MAX_CONTACT_CONSTRAINTS: u32 = 16_384;

pub(super) struct NativeWorld {
    pub(super) physics_system: *mut JPC_PhysicsSystem,
    temp_allocator: *mut JPC_TempAllocatorImpl,
    job_system: *mut JPC_JobSystemThreadPool,
    broad_phase_layer_interface: *mut JPC_BroadPhaseLayerInterface,
    object_vs_broad_phase_filter: *mut JPC_ObjectVsBroadPhaseLayerFilter,
    object_layer_pair_filter: *mut JPC_ObjectLayerPairFilter,
}

unsafe impl Send for NativeWorld {}

impl NativeWorld {
    pub(super) fn new() -> Result<Self, PhysicsBackendError> {
        initialize_jolt();
        let mut world = Self::empty();
        unsafe {
            world.temp_allocator = JPC_TempAllocatorImpl_new(TEMP_ALLOCATOR_BYTES as _);
            world.require(world.temp_allocator, "temporary allocator")?;
            world.job_system = JPC_JobSystemThreadPool_new2(
                JPC_MAX_PHYSICS_JOBS as _,
                JPC_MAX_PHYSICS_BARRIERS as _,
            );
            world.require(world.job_system, "job system")?;
            world.broad_phase_layer_interface =
                JPC_BroadPhaseLayerInterface_new(ptr::null(), BROAD_PHASE_LAYER_INTERFACE);
            world.require(
                world.broad_phase_layer_interface,
                "broad-phase layer interface",
            )?;
            world.object_vs_broad_phase_filter = JPC_ObjectVsBroadPhaseLayerFilter_new(
                ptr::null_mut(),
                OBJECT_VS_BROAD_PHASE_FILTER,
            );
            world.require(
                world.object_vs_broad_phase_filter,
                "object-vs-broad-phase filter",
            )?;
            world.object_layer_pair_filter =
                JPC_ObjectLayerPairFilter_new(ptr::null_mut(), OBJECT_LAYER_PAIR_FILTER);
            world.require(world.object_layer_pair_filter, "object layer pair filter")?;
            world.physics_system = JPC_PhysicsSystem_new();
            world.require(world.physics_system, "physics system")?;
            JPC_PhysicsSystem_Init(
                world.physics_system,
                MAX_BODIES,
                NUM_BODY_MUTEXES,
                MAX_BODY_PAIRS,
                MAX_CONTACT_CONSTRAINTS,
                world.broad_phase_layer_interface,
                world.object_vs_broad_phase_filter,
                world.object_layer_pair_filter,
            );
        }
        Ok(world)
    }

    pub(super) unsafe fn update(&mut self, dt: f32) -> u32 {
        JPC_PhysicsSystem_Update(
            self.physics_system,
            dt,
            1,
            self.temp_allocator,
            self.job_system,
        )
    }

    fn empty() -> Self {
        Self {
            physics_system: ptr::null_mut(),
            temp_allocator: ptr::null_mut(),
            job_system: ptr::null_mut(),
            broad_phase_layer_interface: ptr::null_mut(),
            object_vs_broad_phase_filter: ptr::null_mut(),
            object_layer_pair_filter: ptr::null_mut(),
        }
    }

    fn require<T>(
        &self,
        pointer: *mut T,
        resource: &'static str,
    ) -> Result<(), PhysicsBackendError> {
        if pointer.is_null() {
            Err(PhysicsBackendError::Initialization {
                backend: BACKEND_NAME,
                detail: format!("JoltC returned null for {resource}"),
            })
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for NativeWorld {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeWorld")
            .field("initialized", &!self.physics_system.is_null())
            .finish()
    }
}

impl Drop for NativeWorld {
    fn drop(&mut self) {
        unsafe {
            if !self.physics_system.is_null() {
                JPC_PhysicsSystem_delete(self.physics_system);
            }
            if !self.broad_phase_layer_interface.is_null() {
                JPC_BroadPhaseLayerInterface_delete(self.broad_phase_layer_interface);
            }
            if !self.object_vs_broad_phase_filter.is_null() {
                JPC_ObjectVsBroadPhaseLayerFilter_delete(self.object_vs_broad_phase_filter);
            }
            if !self.object_layer_pair_filter.is_null() {
                JPC_ObjectLayerPairFilter_delete(self.object_layer_pair_filter);
            }
            if !self.job_system.is_null() {
                JPC_JobSystemThreadPool_delete(self.job_system);
            }
            if !self.temp_allocator.is_null() {
                JPC_TempAllocatorImpl_delete(self.temp_allocator);
            }
        }
    }
}

fn initialize_jolt() {
    static INITIALIZED: OnceLock<()> = OnceLock::new();
    INITIALIZED.get_or_init(|| unsafe {
        JPC_RegisterDefaultAllocator();
        JPC_FactoryInit();
        JPC_RegisterTypes();
    });
}
