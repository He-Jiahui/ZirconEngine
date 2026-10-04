use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

#[cfg(feature = "platform-winit")]
use winit::event_loop::EventLoopProxy;
use zircon_runtime_interface::ZrRuntimeWakeSinkV1;

static NEXT_WAKE_TOKEN: AtomicU64 = AtomicU64::new(1);
static WAKE_REGISTRY: OnceLock<Mutex<HashMap<u64, Arc<RuntimeWakeTarget>>>> = OnceLock::new();

enum RuntimeWakeTarget {
    #[cfg(feature = "platform-winit")]
    Windowed(EventLoopProxy),
    Callback(Box<dyn Fn() + Send + Sync>),
}

impl RuntimeWakeTarget {
    fn wake_up(&self) {
        match self {
            #[cfg(feature = "platform-winit")]
            Self::Windowed(proxy) => proxy.wake_up(),
            Self::Callback(callback) => callback(),
        }
    }
}

pub(in crate::entry) struct RuntimeWakeRegistration {
    token: u64,
    proxy: Arc<RuntimeWakeTarget>,
}

impl RuntimeWakeRegistration {
    #[cfg(feature = "platform-winit")]
    pub(in crate::entry) fn register(proxy: EventLoopProxy) -> Self {
        Self::register_target(RuntimeWakeTarget::Windowed(proxy))
    }

    pub(in crate::entry) fn register_callback(callback: impl Fn() + Send + Sync + 'static) -> Self {
        Self::register_target(RuntimeWakeTarget::Callback(Box::new(callback)))
    }

    fn register_target(target: RuntimeWakeTarget) -> Self {
        let proxy = Arc::new(target);
        loop {
            let token = NEXT_WAKE_TOKEN.fetch_add(1, Ordering::Relaxed);
            if token == 0 {
                continue;
            }
            let mut registry = lock_registry();
            if let Entry::Vacant(entry) = registry.entry(token) {
                entry.insert(proxy.clone());
                return Self { token, proxy };
            }
        }
    }

    pub(super) fn sink(&self) -> ZrRuntimeWakeSinkV1 {
        ZrRuntimeWakeSinkV1::new(self.token, runtime_wake_trampoline)
    }

    pub(super) fn wake(&self) {
        if self.token != 0 {
            self.proxy.wake_up();
        }
    }

    pub(super) fn unregister(&mut self) {
        if self.token == 0 {
            return;
        }
        lock_registry().remove(&self.token);
        self.token = 0;
    }
}

impl Drop for RuntimeWakeRegistration {
    fn drop(&mut self) {
        self.unregister();
    }
}

unsafe extern "C" fn runtime_wake_trampoline(token: u64) {
    let _ = catch_unwind(AssertUnwindSafe(|| wake_token(token)));
}

fn wake_token(token: u64) {
    let proxy = lock_registry().get(&token).cloned();
    if let Some(proxy) = proxy {
        proxy.wake_up();
    }
}

fn lock_registry() -> MutexGuard<'static, HashMap<u64, Arc<RuntimeWakeTarget>>> {
    WAKE_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(all(test, feature = "platform-winit"))]
#[path = "tests/wake_registry.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/wake_registry_callback_tests.rs"]
mod callback_tests;
