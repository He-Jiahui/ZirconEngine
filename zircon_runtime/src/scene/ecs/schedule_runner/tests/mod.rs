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

mod support;
use support::{register_timed_external_system, run_test_stage, test_level};

mod panic_recovery;
mod typed_worker_structural;
mod worker_callback_order;
mod worker_dispatch;
