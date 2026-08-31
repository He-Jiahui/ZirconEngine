use std::ops::Deref;

use zircon_runtime::core::runtime::{EngineTaskGraph, EngineTaskGraphOptions};

use crate::DefaultNavigationManager;

const TEST_NAVIGATION_WORKER_THREADS: usize = 2;

#[derive(Clone)]
pub(crate) struct TestNavigationManager {
    manager: DefaultNavigationManager,
    _task_graph: EngineTaskGraph,
}

impl Deref for TestNavigationManager {
    type Target = DefaultNavigationManager;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

pub(crate) fn navigation_manager() -> TestNavigationManager {
    let task_graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(
        TEST_NAVIGATION_WORKER_THREADS,
    ))
    .expect("navigation test task graph");
    let manager = DefaultNavigationManager::new(task_graph.worker_pool().clone());
    TestNavigationManager {
        manager,
        _task_graph: task_graph,
    }
}
