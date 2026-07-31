use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use zircon_runtime_interface::{ZrRuntimeWakeSinkV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

#[derive(Clone)]
pub(in crate::dynamic_api::session) struct RuntimeWakeRegistration {
    shared: Arc<RuntimeWakeShared>,
}

struct RuntimeWakeShared {
    callback: Option<unsafe extern "C" fn(u64)>,
    token: u64,
    lifecycle: Mutex<RuntimeWakeLifecycle>,
    callbacks_drained: Condvar,
}

#[derive(Debug)]
struct RuntimeWakeLifecycle {
    enabled: bool,
    in_flight_callbacks: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::dynamic_api::session) enum InvalidRuntimeWakeSink {
    UnsupportedVersion,
    InvalidPair,
}

impl RuntimeWakeRegistration {
    pub(super) fn disabled() -> Self {
        Self::from_abi(ZrRuntimeWakeSinkV1::disabled())
            .expect("the interface disabled wake sink must remain valid")
    }

    pub(in crate::dynamic_api::session) fn from_abi(
        sink: ZrRuntimeWakeSinkV1,
    ) -> Result<Self, InvalidRuntimeWakeSink> {
        if sink.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(InvalidRuntimeWakeSink::UnsupportedVersion);
        }
        if !matches!((sink.token, sink.wake), (0, None) | (1.., Some(_))) {
            return Err(InvalidRuntimeWakeSink::InvalidPair);
        }
        Ok(Self {
            shared: Arc::new(RuntimeWakeShared {
                callback: sink.wake,
                token: sink.token,
                lifecycle: Mutex::new(RuntimeWakeLifecycle {
                    enabled: true,
                    in_flight_callbacks: 0,
                }),
                callbacks_drained: Condvar::new(),
            }),
        })
    }

    /// Invokes the copied ABI callback without holding the lifecycle or session lock.
    pub(in crate::dynamic_api::session) fn wake(&self) -> bool {
        let Some(callback) = self.shared.callback else {
            return false;
        };
        {
            let mut lifecycle = self.lock_lifecycle();
            if !lifecycle.enabled {
                return false;
            }
            lifecycle.in_flight_callbacks += 1;
        }
        let _callback_guard = WakeCallbackGuard {
            shared: Arc::clone(&self.shared),
        };
        unsafe { callback(self.shared.token) };
        true
    }

    pub(super) fn disable_new_entries(&self) {
        self.lock_lifecycle().enabled = false;
    }

    pub(super) fn wait_for_callbacks(&self) {
        let lifecycle = self.lock_lifecycle();
        drop(
            self.shared
                .callbacks_drained
                .wait_while(lifecycle, |state| state.in_flight_callbacks != 0)
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
    }

    fn lock_lifecycle(&self) -> MutexGuard<'_, RuntimeWakeLifecycle> {
        self.shared
            .lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct WakeCallbackGuard {
    shared: Arc<RuntimeWakeShared>,
}

impl Drop for WakeCallbackGuard {
    fn drop(&mut self) {
        let mut lifecycle = self
            .shared
            .lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        lifecycle.in_flight_callbacks -= 1;
        if lifecycle.in_flight_callbacks == 0 {
            self.shared.callbacks_drained.notify_all();
        }
    }
}
