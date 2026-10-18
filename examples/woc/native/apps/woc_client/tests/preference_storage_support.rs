use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use zircon_runtime::core::framework::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::core::framework::platform::{
    PreferenceDurabilityState, PreferenceKey, PreferenceMutationTerminal, PreferenceStorage,
    PreferenceStorageBackendKind, PreferenceStorageError, PreferenceStorageErrorKind,
    PreferenceStorageOperation, PreferenceTicketWaitResult, PreferenceWorkDeadline,
    PLATFORM_MODULE_NAME,
};
use zircon_runtime::core::manager::{platform_preference_storage_handle, resolve_manager_service};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation;
use zircon_runtime::platform::preferences::{
    PreferenceBackendWorkAuthority, PreferenceStorageBackend,
};
use zircon_runtime::platform::{self, PlatformDriver, PLATFORM_DRIVER_NAME};

const WOC_CLIENT_PREFERENCE_NAMESPACE: &str = "woc.client";

#[derive(Default)]
struct MemoryPreferenceBackend {
    values: Mutex<BTreeMap<String, Vec<u8>>>,
    read_gates: Mutex<BTreeMap<String, Arc<ReadGate>>>,
    fail_reads: AtomicBool,
    fail_writes: AtomicBool,
}

#[derive(Default)]
struct ReadGate {
    state: Mutex<ReadGateState>,
    changed: Condvar,
}

#[derive(Default)]
struct ReadGateState {
    started: bool,
    released: bool,
}

impl ReadGate {
    fn wait_for_release(&self) {
        let mut state = lock(&self.state);
        state.started = true;
        self.changed.notify_all();
        while !state.released {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn wait_until_started(&self, deadline: Instant) {
        let mut state = lock(&self.state);
        while !state.started {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(
                !remaining.is_zero(),
                "preference backend read did not start"
            );
            let (next, timeout) = self
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next;
            assert!(
                !timeout.timed_out() || state.started,
                "preference backend read did not start"
            );
        }
    }

    fn release(&self) {
        let mut state = lock(&self.state);
        state.released = true;
        self.changed.notify_all();
    }
}

impl PreferenceStorageBackend for MemoryPreferenceBackend {
    fn backend_kind(&self) -> PreferenceStorageBackendKind {
        PreferenceStorageBackendKind::HostProvided
    }

    fn open_read(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<Option<Box<dyn Read + Send>>, PreferenceStorageError> {
        let read_gate = lock(&self.read_gates).get(key.key()).cloned();
        if let Some(read_gate) = read_gate {
            read_gate.wait_for_release();
        }
        if self.fail_reads.load(Ordering::Relaxed) {
            return Err(injected_error(PreferenceStorageOperation::Read));
        }
        Ok(lock(&self.values)
            .get(key.key())
            .cloned()
            .map(|value| Box::new(Cursor::new(value)) as Box<dyn Read + Send>))
    }

    fn write(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
        value: &[u8],
    ) -> Result<(), PreferenceStorageError> {
        if self.fail_writes.load(Ordering::Relaxed) {
            return Err(injected_error(PreferenceStorageOperation::Write));
        }
        lock(&self.values).insert(key.key().to_owned(), value.to_vec());
        Ok(())
    }

    fn remove(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<(), PreferenceStorageError> {
        lock(&self.values).remove(key.key());
        Ok(())
    }

    fn flush(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), PreferenceStorageError> {
        if self.fail_writes.load(Ordering::Relaxed) {
            Err(injected_error(PreferenceStorageOperation::Flush))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct MemoryPreferenceStorage {
    _runtime: Arc<CoreRuntime>,
    backend: Arc<MemoryPreferenceBackend>,
    storage: Arc<dyn PreferenceStorage>,
}

impl Default for MemoryPreferenceStorage {
    fn default() -> Self {
        let runtime = Arc::new(CoreRuntime::new());
        runtime
            .register_module(foundation::module_descriptor())
            .expect("register foundation module");
        runtime
            .register_module(platform::module_descriptor())
            .expect("register platform module");
        runtime
            .activate_module(FOUNDATION_MODULE_NAME)
            .expect("activate foundation module");
        runtime
            .activate_module(PLATFORM_MODULE_NAME)
            .expect("activate platform module");

        let backend = Arc::new(MemoryPreferenceBackend::default());
        runtime
            .resolve_driver::<PlatformDriver>(PLATFORM_DRIVER_NAME)
            .expect("resolve platform driver")
            .install_preference_storage_backend(backend.clone())
            .expect("install memory preference backend");
        let storage = resolve_manager_service(
            &runtime.handle(),
            platform_preference_storage_handle(&runtime.handle())
                .expect("resolve preference manager handle"),
        )
        .expect("resolve preference manager service");

        Self {
            _runtime: runtime,
            backend,
            storage,
        }
    }
}

impl AsRef<dyn PreferenceStorage> for MemoryPreferenceStorage {
    fn as_ref(&self) -> &(dyn PreferenceStorage + 'static) {
        self.storage.as_ref()
    }
}

impl MemoryPreferenceStorage {
    pub fn block_read(&self, key: &str) {
        let previous = lock(&self.backend.read_gates).insert(key.to_owned(), Arc::default());
        assert!(previous.is_none(), "preference read gate already installed");
    }

    pub fn wait_until_read_started(&self, key: &str) {
        self.read_gate(key)
            .wait_until_started(Instant::now() + Duration::from_secs(2));
    }

    pub fn release_read(&self, key: &str) {
        self.read_gate(key).release();
    }

    pub fn seed_persisted(&self, key: &str, value: &str) {
        lock(&self.backend.values).insert(key.to_owned(), value.as_bytes().to_vec());
    }

    pub fn wait_until_loaded(&self, key: &str) {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let snapshot = self
                .storage
                .snapshot(&preference_key(key))
                .expect("load preference snapshot");
            if snapshot.value().is_some()
                || snapshot.durability() != PreferenceDurabilityState::Pending
            {
                return;
            }
            assert!(Instant::now() < deadline, "preference load timed out");
            std::thread::yield_now();
        }
    }

    pub fn insert(&self, key: &str, value: &str) {
        let submission = self
            .storage
            .submit_write(
                preference_key(key),
                Arc::from(value.as_bytes()),
                PreferenceWorkDeadline::none(),
            )
            .expect("seed preference write");
        assert!(matches!(
            submission
                .ticket()
                .wait_until(Instant::now() + Duration::from_secs(2)),
            PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
        ));
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let snapshot = self.storage.snapshot(&preference_key(key)).ok()?;
        std::str::from_utf8(snapshot.value()?)
            .ok()
            .map(str::to_owned)
    }

    pub fn persisted(&self, key: &str) -> Option<String> {
        lock(&self.backend.values)
            .get(key)
            .and_then(|value| std::str::from_utf8(value).ok())
            .map(str::to_owned)
    }

    pub fn set_fail_reads(&self, fail: bool) {
        self.backend.fail_reads.store(fail, Ordering::Relaxed);
    }

    pub fn set_fail_writes(&self, fail: bool) {
        self.backend.fail_writes.store(fail, Ordering::Relaxed);
    }

    pub fn set_unavailable(&self, unavailable: bool) {
        self.set_fail_reads(unavailable);
        self.set_fail_writes(unavailable);
    }

    fn read_gate(&self, key: &str) -> Arc<ReadGate> {
        lock(&self.backend.read_gates)
            .get(key)
            .cloned()
            .expect("preference read gate")
    }
}

fn preference_key(key: &str) -> PreferenceKey {
    PreferenceKey::new(WOC_CLIENT_PREFERENCE_NAMESPACE, key).expect("valid WOC preference key")
}

fn injected_error(operation: PreferenceStorageOperation) -> PreferenceStorageError {
    PreferenceStorageError::new(
        PreferenceStorageErrorKind::Unavailable,
        operation,
        "woc_test_memory",
        "injected unavailable preference backend",
    )
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
