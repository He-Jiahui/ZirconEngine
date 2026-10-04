//! Engine-owned CPAL callback and provider retirement boundary.
//!
//! CPAL exposes no callback-join receipt. The owner therefore has two explicit
//! lifetimes: the engine callback gate and a provider retirement worker which
//! owns the real `cpal::Stream` until its `Drop` has returned. The latter emits
//! its receipt only after CPAL has completed its own worker teardown. A timeout
//! retains the worker and its census for a later retry; it never detaches or
//! discards the provider owner.

#[cfg(test)]
use std::sync::{atomic::AtomicU32, OnceLock};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc, Condvar, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use kira::backend::cpal::{
    cpal::{
        self,
        traits::{DeviceTrait, HostTrait, StreamTrait},
    },
    CpalBackendSettings, Error as CpalBackendError,
};
use kira::backend::{Backend, Renderer};
use zircon_runtime::core::runtime::tasks::thread_is_join_ready;

const PROVIDER_RETIREMENT_POLL_INTERVAL: Duration = Duration::from_millis(1);
const CPAL_RETIREMENT_THREAD_NAME: &str = "zircon-sound-cpal-retire";

#[derive(Debug)]
struct CallbackGate {
    retiring: AtomicBool,
    in_flight: AtomicUsize,
    wait_lock: Mutex<()>,
    idle: Condvar,
}

impl CallbackGate {
    fn new() -> Self {
        Self {
            retiring: AtomicBool::new(false),
            in_flight: AtomicUsize::new(0),
            wait_lock: Mutex::new(()),
            idle: Condvar::new(),
        }
    }

    fn enter(self: &Arc<Self>) -> Option<CallbackLease> {
        if self.retiring.load(Ordering::Acquire) {
            return None;
        }
        self.in_flight.fetch_add(1, Ordering::AcqRel);
        if self.retiring.load(Ordering::Acquire) {
            if self.in_flight.fetch_sub(1, Ordering::AcqRel) == 1 {
                self.idle.notify_all();
            }
            return None;
        }
        Some(CallbackLease {
            gate: Arc::clone(self),
        })
    }

    fn retire_until(&self, deadline: Instant) -> bool {
        self.retiring.store(true, Ordering::Release);
        let mut wait_guard = self
            .wait_lock
            .lock()
            .expect("callback gate must not poison");
        while self.in_flight.load(Ordering::Acquire) != 0 {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                self.retiring.store(false, Ordering::Release);
                return false;
            };
            let (next, result) = self
                .idle
                .wait_timeout(wait_guard, remaining)
                .expect("callback gate must not poison");
            wait_guard = next;
            if result.timed_out() && self.in_flight.load(Ordering::Acquire) != 0 {
                self.retiring.store(false, Ordering::Release);
                return false;
            }
        }
        true
    }

    fn in_flight(&self) -> usize {
        self.in_flight.load(Ordering::Acquire)
    }
}

struct CallbackLease {
    gate: Arc<CallbackGate>,
}

impl Drop for CallbackLease {
    fn drop(&mut self) {
        if self.gate.in_flight.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.gate.idle.notify_all();
        }
    }
}

/// Census for the actual provider retirement worker. The provider ACK is
/// emitted only after the worker has dropped the real CPAL stream; the exited
/// and joined counts cover the owner thread which performed that drop.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProviderRetirementCensus {
    pub(crate) expected_worker_count: usize,
    pub(crate) provider_drop_ack_count: usize,
    pub(crate) exited_worker_count: usize,
    pub(crate) joined_worker_count: usize,
}

impl ProviderRetirementCensus {
    fn complete(self) -> bool {
        self.expected_worker_count == 1
            && self.provider_drop_ack_count == self.expected_worker_count
            && self.exited_worker_count == self.expected_worker_count
            && self.joined_worker_count == self.expected_worker_count
    }
}

#[derive(Debug, Default)]
struct ProviderRetirementState {
    provider_drop_ack_count: usize,
    exited_worker_count: usize,
    joined_worker_count: usize,
}

#[derive(Debug)]
struct ProviderRetirementShared {
    state: Mutex<ProviderRetirementState>,
    changed: Condvar,
}

impl ProviderRetirementShared {
    fn census(&self) -> ProviderRetirementCensus {
        let state = self
            .state
            .lock()
            .expect("provider retirement state must not poison");
        ProviderRetirementCensus {
            expected_worker_count: 1,
            provider_drop_ack_count: state.provider_drop_ack_count,
            exited_worker_count: state.exited_worker_count,
            joined_worker_count: state.joined_worker_count,
        }
    }

    fn provider_drop_ack(&self) {
        let mut state = self
            .state
            .lock()
            .expect("provider retirement state must not poison");
        state.provider_drop_ack_count = state.provider_drop_ack_count.saturating_add(1);
        self.changed.notify_all();
    }

    fn worker_exit(&self) {
        let mut state = self
            .state
            .lock()
            .expect("provider retirement state must not poison");
        state.exited_worker_count = state.exited_worker_count.saturating_add(1);
        self.changed.notify_all();
    }

    fn worker_joined(&self) {
        let mut state = self
            .state
            .lock()
            .expect("provider retirement state must not poison");
        state.joined_worker_count = state.joined_worker_count.saturating_add(1);
        self.changed.notify_all();
    }
}

struct ProviderWorkerExit {
    shared: Arc<ProviderRetirementShared>,
}

impl Drop for ProviderWorkerExit {
    fn drop(&mut self) {
        self.shared.worker_exit();
    }
}

/// Owns the thread that owns and drops the real CPAL stream. This wrapper is
/// intentionally retained after a deadline so a later operation can retry the
/// same provider lifetime and join authority.
struct ProviderRetirement {
    shared: Arc<ProviderRetirementShared>,
    provider_ack: Receiver<ProviderDropReceipt>,
    worker: Option<JoinHandle<()>>,
    outcome_unknown: bool,
}

struct ProviderDropReceipt;

impl ProviderRetirement {
    fn start<T: Send + 'static>(provider: T) -> Result<Self, (T, std::io::Error)> {
        let provider_slot = Arc::new(Mutex::new(Some(provider)));
        let shared = Arc::new(ProviderRetirementShared {
            state: Mutex::new(ProviderRetirementState::default()),
            changed: Condvar::new(),
        });
        let worker_slot = Arc::clone(&provider_slot);
        let worker_shared = Arc::clone(&shared);
        let (provider_ack, provider_ack_sender) = sync_channel(1);
        let worker = match thread::Builder::new()
            .name(CPAL_RETIREMENT_THREAD_NAME.to_string())
            .spawn(move || {
                let _exit = ProviderWorkerExit {
                    shared: Arc::clone(&worker_shared),
                };
                let provider = worker_slot
                    .lock()
                    .expect("provider stream slot must not poison")
                    .take();
                // CPAL's stream drop is the provider termination operation. The
                // receipt below is not sent until that operation returns.
                drop(provider);
                // This channel receipt is sent after the real provider Drop
                // returns. The owner converts it into the ACK census entry.
                let _ = provider_ack_sender.send(ProviderDropReceipt);
            }) {
            Ok(worker) => worker,
            Err(error) => {
                let provider = provider_slot
                    .lock()
                    .expect("provider stream slot must not poison")
                    .take()
                    .expect("failed provider worker retains the stream");
                return Err((provider, error));
            }
        };
        Ok(Self {
            shared,
            provider_ack,
            worker: Some(worker),
            outcome_unknown: false,
        })
    }

    fn census(&self) -> ProviderRetirementCensus {
        self.shared.census()
    }

    fn finish_until(
        &mut self,
        deadline: Instant,
    ) -> Result<ProviderRetirementCensus, CpalQuiescenceError> {
        if self.outcome_unknown {
            return Err(CpalQuiescenceError::unknown(self.census()));
        }
        if self.worker.is_none() {
            self.outcome_unknown = true;
            return Err(CpalQuiescenceError::unknown(self.census()));
        }
        loop {
            if self.shared.census().provider_drop_ack_count == 0 {
                if let Ok(_receipt) = self.provider_ack.try_recv() {
                    self.shared.provider_drop_ack();
                }
            }
            let census = self.census();
            let join_ready = self
                .worker
                .as_ref()
                .is_some_and(|worker| thread_is_join_ready(worker));
            if join_ready && census.provider_drop_ack_count != census.expected_worker_count {
                let worker = self
                    .worker
                    .take()
                    .expect("provider retirement retains join authority");
                let _ = worker.join();
                self.shared.worker_joined();
                if self.provider_ack.try_recv().is_ok() {
                    self.shared.provider_drop_ack();
                    let completed = self.census();
                    debug_assert!(completed.complete());
                    return Ok(completed);
                }
                self.outcome_unknown = true;
                return Err(CpalQuiescenceError::unknown(self.census()));
            }
            if census.provider_drop_ack_count == census.expected_worker_count && join_ready {
                let worker = self
                    .worker
                    .take()
                    .expect("provider retirement retains join authority");
                if worker.join().is_err() {
                    self.outcome_unknown = true;
                    return Err(CpalQuiescenceError::unknown(census));
                }
                self.shared.worker_joined();
                let completed = self.census();
                debug_assert!(completed.complete());
                return Ok(completed);
            }
            if Instant::now() >= deadline {
                return Err(CpalQuiescenceError::provider(census));
            }
            let state = self
                .shared
                .state
                .lock()
                .expect("provider retirement state must not poison");
            let remaining = deadline.saturating_duration_since(Instant::now());
            let _ = self
                .shared
                .changed
                .wait_timeout(state, remaining.min(PROVIDER_RETIREMENT_POLL_INTERVAL))
                .expect("provider retirement state must not poison");
        }
    }
}

enum PendingProviderOwner {
    Stream(cpal::Stream),
    Retirement(ProviderRetirement),
}

// A backend destructor cannot return a timeout. Keep the actual stream or its
// join handle in this explicit owner until a later owner operation reaps it.
static PENDING_PROVIDER_OWNERS: std::sync::OnceLock<Mutex<Vec<PendingProviderOwner>>> =
    std::sync::OnceLock::new();

fn retain_pending_provider_owner(owner: PendingProviderOwner) {
    PENDING_PROVIDER_OWNERS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .expect("pending provider owner must not poison")
        .push(owner);
}

fn reap_pending_provider_owners(deadline: Instant) {
    let owners = PENDING_PROVIDER_OWNERS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .expect("pending provider owner must not poison")
        .drain(..)
        .collect::<Vec<_>>();
    let mut retained = Vec::new();
    for owner in owners {
        match owner {
            PendingProviderOwner::Stream(stream) => match ProviderRetirement::start(stream) {
                Ok(mut retirement) => {
                    if retirement.finish_until(deadline).is_err() {
                        retained.push(PendingProviderOwner::Retirement(retirement));
                    }
                }
                Err((stream, _)) => retained.push(PendingProviderOwner::Stream(stream)),
            },
            PendingProviderOwner::Retirement(mut retirement) => {
                if retirement.finish_until(deadline).is_err() {
                    retained.push(PendingProviderOwner::Retirement(retirement));
                }
            }
        }
    }
    PENDING_PROVIDER_OWNERS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .expect("pending provider owner must not poison")
        .extend(retained);
}

#[derive(Debug)]
pub(crate) struct CpalQuiescenceError {
    phase: &'static str,
    census: ProviderRetirementCensus,
    in_flight_callbacks: usize,
    outcome_unknown: bool,
}

/// The provider stream has entered irreversible close and its terminal result
/// is not known before the caller's deadline. The owning backend keeps the
/// pending worker so the same lifetime can be retried.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetiringOutcomeUnknown {
    pub(crate) census: ProviderRetirementCensus,
}

impl CpalQuiescenceError {
    fn callback(in_flight: usize) -> Self {
        Self {
            phase: "callback",
            census: ProviderRetirementCensus {
                expected_worker_count: 0,
                provider_drop_ack_count: 0,
                exited_worker_count: 0,
                joined_worker_count: 0,
            },
            in_flight_callbacks: in_flight,
            outcome_unknown: false,
        }
    }

    fn provider(census: ProviderRetirementCensus) -> Self {
        Self {
            phase: "provider",
            census,
            in_flight_callbacks: 0,
            // CPAL exposes no provider-drop receipt. Until the retained
            // worker is joined, the caller cannot claim the stream's terminal
            // outcome even though the same worker remains retryable.
            outcome_unknown: true,
        }
    }

    fn provider_start(census: ProviderRetirementCensus) -> Self {
        Self {
            phase: "provider-start",
            census,
            in_flight_callbacks: 0,
            // The actual Stream remains installed when worker creation fails;
            // this is retryable setup failure rather than irreversible close.
            outcome_unknown: false,
        }
    }

    fn unknown(census: ProviderRetirementCensus) -> Self {
        Self {
            phase: "provider-outcome-unknown",
            census,
            in_flight_callbacks: 0,
            outcome_unknown: true,
        }
    }

    pub(crate) fn census(&self) -> ProviderRetirementCensus {
        self.census
    }

    pub(crate) fn is_provider_phase(&self) -> bool {
        matches!(self.phase, "provider" | "provider-outcome-unknown")
    }

    pub(crate) fn retiring_outcome_unknown(&self) -> RetiringOutcomeUnknown {
        debug_assert!(self.outcome_unknown);
        RetiringOutcomeUnknown {
            census: self.census,
        }
    }
}

impl std::fmt::Display for CpalQuiescenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "CPAL {} retirement deadline elapsed (in_flight_callbacks={}, expected={}, ack={}, exited={}, joined={})",
            self.phase,
            self.in_flight_callbacks,
            self.census.expected_worker_count,
            self.census.provider_drop_ack_count,
            self.census.exited_worker_count,
            self.census.joined_worker_count,
        )
    }
}

pub(crate) struct OwnerCpalBackend {
    device: Option<cpal::Device>,
    config: cpal::StreamConfig,
    stream: Option<cpal::Stream>,
    pending_retirement: Option<ProviderRetirement>,
    gate: Arc<CallbackGate>,
}

#[cfg(test)]
static CALLBACK_ENTER_SIGNAL: OnceLock<Mutex<Option<SyncSender<()>>>> = OnceLock::new();
#[cfg(test)]
static CALLBACK_NONZERO_SIGNAL: OnceLock<Mutex<Option<SyncSender<()>>>> = OnceLock::new();
#[cfg(test)]
static CALLBACK_RETIRED_SIGNAL: OnceLock<Mutex<Option<SyncSender<()>>>> = OnceLock::new();
#[cfg(test)]
static CALLBACK_PEAK_BITS: AtomicU32 = AtomicU32::new(0);

#[cfg(test)]
fn signal_once(slot: &OnceLock<Mutex<Option<SyncSender<()>>>>) {
    if let Some(sender) = slot.get_or_init(|| Mutex::new(None)).lock().unwrap().take() {
        let _ = sender.send(());
    }
}

#[cfg(test)]
fn record_peak(data: &[f32]) {
    let peak = data.iter().copied().map(f32::abs).fold(0.0, f32::max);
    let mut current = CALLBACK_PEAK_BITS.load(Ordering::Acquire);
    loop {
        if peak <= f32::from_bits(current) {
            return;
        }
        match CALLBACK_PEAK_BITS.compare_exchange_weak(
            current,
            peak.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return,
            Err(next) => current = next,
        }
    }
}

impl Backend for OwnerCpalBackend {
    type Settings = CpalBackendSettings;
    type Error = CpalBackendError;

    fn setup(
        settings: Self::Settings,
        _internal_buffer_size: usize,
    ) -> Result<(Self, u32), Self::Error> {
        let host = cpal::default_host();
        let device = settings
            .device
            .or_else(|| host.default_output_device())
            .ok_or(CpalBackendError::NoDefaultOutputDevice)?;
        let config = settings
            .config
            .unwrap_or(device.default_output_config()?.config());
        let sample_rate = config.sample_rate;
        Ok((
            Self {
                device: Some(device),
                config,
                stream: None,
                pending_retirement: None,
                gate: Arc::new(CallbackGate::new()),
            },
            sample_rate,
        ))
    }

    fn start(&mut self, mut renderer: Renderer) -> Result<(), Self::Error> {
        let Some(device) = self.device.take() else {
            return Err(CpalBackendError::NoDefaultOutputDevice);
        };
        let config = self.config;
        let channels = config.channels;
        let gate = Arc::clone(&self.gate);
        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], _| {
                data.fill(0.0);
                let Some(_lease) = gate.enter() else { return };
                #[cfg(test)]
                signal_once(&CALLBACK_ENTER_SIGNAL);
                renderer.on_start_processing();
                renderer.process(data, channels);
                #[cfg(test)]
                {
                    record_peak(data);
                    if f32::from_bits(CALLBACK_PEAK_BITS.load(Ordering::Acquire)) > 0.01 {
                        signal_once(&CALLBACK_NONZERO_SIGNAL);
                    }
                }
            },
            // The engine-owned gate closes one callback generation. CPAL
            // device-error reporting/restart remains an explicit hot-plug gate
            // until the provider can expose an owner-visible restart receipt.
            |_error| {},
            None,
        )?;
        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }
}

impl OwnerCpalBackend {
    pub(crate) fn quiesce(&mut self, timeout: Duration) -> Result<(), CpalQuiescenceError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        reap_pending_provider_owners(deadline);

        if !self.gate.retire_until(deadline) {
            return Err(CpalQuiescenceError::callback(self.gate.in_flight()));
        }

        if let Some(mut pending) = self.pending_retirement.take() {
            match pending.finish_until(deadline) {
                Ok(_) => {
                    #[cfg(test)]
                    signal_once(&CALLBACK_RETIRED_SIGNAL);
                    return Ok(());
                }
                Err(error) => {
                    self.pending_retirement = Some(pending);
                    return Err(error);
                }
            }
        }

        let Some(stream) = self.stream.take() else {
            #[cfg(test)]
            signal_once(&CALLBACK_RETIRED_SIGNAL);
            return Ok(());
        };
        let mut pending = match ProviderRetirement::start(stream) {
            Ok(pending) => pending,
            Err((stream, _error)) => {
                self.stream = Some(stream);
                self.gate.retiring.store(false, Ordering::Release);
                return Err(CpalQuiescenceError::provider_start(
                    ProviderRetirementCensus::default(),
                ));
            }
        };
        match pending.finish_until(deadline) {
            Ok(_) => {
                #[cfg(test)]
                signal_once(&CALLBACK_RETIRED_SIGNAL);
                Ok(())
            }
            Err(error) => {
                self.pending_retirement = Some(pending);
                Err(error)
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn install_callback_enter_signal(sender: SyncSender<()>) {
        *CALLBACK_ENTER_SIGNAL
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(sender);
    }

    #[cfg(test)]
    pub(crate) fn install_callback_nonzero_signal(sender: SyncSender<()>) {
        *CALLBACK_NONZERO_SIGNAL
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(sender);
    }

    #[cfg(test)]
    pub(crate) fn install_callback_retired_signal(sender: SyncSender<()>) {
        *CALLBACK_RETIRED_SIGNAL
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(sender);
    }

    #[cfg(test)]
    pub(crate) fn reset_callback_telemetry() {
        CALLBACK_PEAK_BITS.store(0, Ordering::Release);
        *CALLBACK_ENTER_SIGNAL
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = None;
        *CALLBACK_NONZERO_SIGNAL
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = None;
        *CALLBACK_RETIRED_SIGNAL
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = None;
    }

    #[cfg(test)]
    pub(crate) fn callback_peak_for_test() -> f32 {
        f32::from_bits(CALLBACK_PEAK_BITS.load(Ordering::Acquire))
    }
}

impl Drop for OwnerCpalBackend {
    fn drop(&mut self) {
        // Destruction cannot return a timeout. Retain the real stream or the
        // real provider worker in the explicit pending-owner registry so this
        // path never waits forever and never detaches a provider lifetime.
        self.gate.retiring.store(true, Ordering::Release);
        if let Some(pending) = self.pending_retirement.take() {
            retain_pending_provider_owner(PendingProviderOwner::Retirement(pending));
        }
        if let Some(stream) = self.stream.take() {
            match ProviderRetirement::start(stream) {
                Ok(pending) => {
                    retain_pending_provider_owner(PendingProviderOwner::Retirement(pending))
                }
                Err((stream, _error)) => {
                    retain_pending_provider_owner(PendingProviderOwner::Stream(stream))
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/cpal_owner.rs"]
mod tests;
