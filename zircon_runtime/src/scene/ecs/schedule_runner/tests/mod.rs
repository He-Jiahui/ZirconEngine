use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

use crate::core::framework::scene::WorldHandle;
use crate::core::{CoreRuntime, JobScheduler, TaskPool, TaskPoolDescriptor};
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::components::Name;
use crate::scene::ecs::{
    CommandsParam, Component, DeferredCommandOperation, DeferredCommandTarget, LifecycleEventKind,
    Resource, SceneSystemThreadAffinity, SystemParamAccess,
};
use crate::scene::{EntityId, LevelMetadata, World};

use super::*;

#[path = "support.rs"]
mod support;
use support::{register_timed_external_system, run_test_stage, test_level};

#[path = "admission.rs"]
mod admission;
#[path = "owner_replacement.rs"]
mod owner_replacement;
#[path = "panic_recovery.rs"]
mod panic_recovery;
#[path = "typed_worker_structural.rs"]
mod typed_worker_structural;
#[path = "worker_callback_order.rs"]
mod worker_callback_order;
#[path = "worker_dispatch.rs"]
mod worker_dispatch;
