use std::fmt;
use std::sync::Arc;

use super::super::state::CoreRuntimeInner;
use super::super::tasks::{JobScheduler, TaskPool, TaskPoolKind, TaskPoolReport, TaskPools};
use super::super::weak::CoreWeak;

#[derive(Clone)]
pub struct CoreHandle {
    pub(crate) inner: Arc<CoreRuntimeInner>,
}

impl CoreHandle {
    pub fn downgrade(&self) -> CoreWeak {
        CoreWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn scheduler(&self) -> &JobScheduler {
        &self.inner.scheduler
    }

    pub fn task_pools(&self) -> &TaskPools {
        &self.inner.task_pools
    }

    pub fn task_pool(&self, kind: TaskPoolKind) -> &TaskPool {
        self.inner.task_pools.get(kind)
    }

    pub fn task_pool_report(&self) -> TaskPoolReport {
        self.inner.task_pools.report()
    }
}

impl fmt::Debug for CoreHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreHandle").finish()
    }
}
