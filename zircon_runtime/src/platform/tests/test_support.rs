use std::ops::Deref;
use std::sync::Arc;

use crate::core::runtime::{TaskPool, TaskPoolDescriptor};
use crate::platform::preferences::PreferenceStorageBackend;

use super::{PlatformDriver, PlatformManager};

pub(crate) struct TestPlatformDriver {
    driver: Arc<PlatformDriver>,
    _io_pool: TaskPool,
}

impl TestPlatformDriver {
    pub(crate) fn shared(&self) -> Arc<PlatformDriver> {
        Arc::clone(&self.driver)
    }
}

impl Deref for TestPlatformDriver {
    type Target = PlatformDriver;

    fn deref(&self) -> &Self::Target {
        self.driver.as_ref()
    }
}

pub(crate) struct TestPlatformManager {
    manager: PlatformManager,
    _io_pool: TaskPool,
}

impl Deref for TestPlatformManager {
    type Target = PlatformManager;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

pub(crate) fn platform_driver() -> TestPlatformDriver {
    let io_pool = test_io_pool();
    let driver = Arc::new(PlatformDriver::with_io_task_pool(io_pool.clone()));
    TestPlatformDriver {
        driver,
        _io_pool: io_pool,
    }
}

pub(crate) fn platform_manager() -> TestPlatformManager {
    let io_pool = test_io_pool();
    let driver = Arc::new(PlatformDriver::with_io_task_pool(io_pool.clone()));
    TestPlatformManager {
        manager: PlatformManager::new(driver),
        _io_pool: io_pool,
    }
}

pub(crate) fn platform_manager_with_backend(
    backend: Arc<dyn PreferenceStorageBackend>,
) -> TestPlatformManager {
    let io_pool = test_io_pool();
    let driver = Arc::new(
        PlatformDriver::with_preference_storage_backend(io_pool.clone(), backend)
            .expect("test preference storage backend installs"),
    );
    TestPlatformManager {
        manager: PlatformManager::new(driver),
        _io_pool: io_pool,
    }
}

fn test_io_pool() -> TaskPool {
    TaskPool::new(
        TaskPoolDescriptor::io()
            .with_worker_threads(1)
            .with_thread_name("platform-test-io"),
    )
}
