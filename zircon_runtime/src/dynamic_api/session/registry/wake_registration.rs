use std::cell::RefCell;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use zircon_runtime_interface::{ZrRuntimeWakeSinkV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use crate::core::framework::channel::ChannelWakeCallback;

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

thread_local! {
    static ACTIVE_RUNTIME_WAKE_CALLBACKS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

fn callback_stack_contains(callbacks: &[usize], identity: usize) -> bool {
    callbacks
        .iter()
        .rev()
        .any(|callback_identity| *callback_identity == identity)
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
        let _callback_thread_guard = WakeCallbackThreadGuard::enter(&self.shared);
        unsafe { callback(self.shared.token) };
        true
    }

    pub(in crate::dynamic_api::session) fn channel_wake(&self) -> ChannelWakeCallback {
        let registration = self.clone();
        Arc::new(move || {
            let _ = registration.wake();
        })
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

    pub(super) fn callback_active_on_current_thread(&self) -> bool {
        let identity = Arc::as_ptr(&self.shared) as usize;
        ACTIVE_RUNTIME_WAKE_CALLBACKS
            .with(|callbacks| callback_stack_contains(&callbacks.borrow(), identity))
    }

    fn lock_lifecycle(&self) -> MutexGuard<'_, RuntimeWakeLifecycle> {
        self.shared
            .lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct WakeCallbackThreadGuard {
    identity: usize,
}

impl WakeCallbackThreadGuard {
    fn enter(shared: &Arc<RuntimeWakeShared>) -> Self {
        let identity = Arc::as_ptr(shared) as usize;
        ACTIVE_RUNTIME_WAKE_CALLBACKS.with(|callbacks| callbacks.borrow_mut().push(identity));
        Self { identity }
    }
}

impl Drop for WakeCallbackThreadGuard {
    fn drop(&mut self) {
        ACTIVE_RUNTIME_WAKE_CALLBACKS.with(|callbacks| {
            let popped = callbacks.borrow_mut().pop();
            debug_assert_eq!(popped, Some(self.identity));
        });
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

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::callback_stack_contains;

    const CALLBACK_DEPTH: usize = 512;
    const LOOKUPS_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fv_runtime478_callback_stack_lookup_preserves_nested_membership() {
        let callbacks = [11, 23, 37, 41];

        assert!(callback_stack_contains(&callbacks, 41));
        assert!(callback_stack_contains(&callbacks, 23));
        assert!(!callback_stack_contains(&callbacks, 43));
        assert!(!callback_stack_contains(&[], 11));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fv_runtime478_reverse_callback_stack_lookup_benchmark() {
        let callbacks = (0..CALLBACK_DEPTH).collect::<Vec<_>>();
        let active_identity = CALLBACK_DEPTH - 1;
        for _ in 0..4 {
            black_box(measure_lookups(&callbacks, active_identity, false));
            black_box(measure_lookups(&callbacks, active_identity, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_lookups(&callbacks, active_identity, false));
                optimized_samples.push(measure_lookups(&callbacks, active_identity, true));
            } else {
                optimized_samples.push(measure_lookups(&callbacks, active_identity, true));
                legacy_samples.push(measure_lookups(&callbacks, active_identity, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME478_REVERSE_CALLBACK_STACK_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} callback_depth={CALLBACK_DEPTH} lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_comparisons_per_top_hit={CALLBACK_DEPTH} optimized_comparisons_per_top_hit=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=75",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 25 / 100);
    }

    fn measure_lookups(callbacks: &[usize], identity: usize, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..LOOKUPS_PER_SAMPLE {
            let found = if optimized {
                callback_stack_contains(black_box(callbacks), black_box(identity))
            } else {
                black_box(callbacks).contains(&black_box(identity))
            };
            black_box(found);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
