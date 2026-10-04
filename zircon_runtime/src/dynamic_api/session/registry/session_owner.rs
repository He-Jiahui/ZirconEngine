use std::any::Any;
use std::cell::RefCell;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex, TryLockError};
use std::time::Instant;

use crossbeam_channel::{bounded, Receiver, SendTimeoutError, Sender};

pub(super) mod abi_status;
pub(super) mod runtime;

#[cfg(all(test, windows))]
#[path = "session_owner/tests/receipt_publication.rs"]
pub(in crate::dynamic_api::session) mod receipt_publication;

use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

const COMMAND_CAPACITY: usize = 64;
const RECEIPT_PENDING: u8 = 0;
const RECEIPT_SUCCEEDED: u8 = 1;
const RECEIPT_RETRYABLE: u8 = 2;
const RECEIPT_PANICKED: u8 = 3;

thread_local! {
    static ACTIVE_SESSION_OWNERS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

type OwnerAction<'a, S> = Box<dyn FnOnce(&mut S) + Send + 'a>;
type OwnerCleanup<S> = Box<dyn FnOnce(&mut S) -> bool + Send>;

enum OwnerCommand<S> {
    Action(OwnerAction<'static, S>),
    Scoped(ScopedAction<S>),
    Shutdown(OwnerCleanup<S>, Arc<CleanupReceipt>),
}

struct ScopedAction<S> {
    callback: *mut (),
    invoke: unsafe fn(*mut (), &mut S),
    completed: Sender<()>,
}

impl<S> ScopedAction<S> {
    fn new<A>(callback: &mut Option<A>, completed: Sender<()>) -> Self
    where
        A: FnOnce(&mut S) + Send,
    {
        Self {
            callback: (callback as *mut Option<A>).cast::<()>(),
            invoke: invoke_scoped::<S, A>,
            completed,
        }
    }
}

// The callback pointer is only dereferenced by the owner thread, while the
// caller waits for `completed` before reclaiming the stack callback.
unsafe impl<S> Send for ScopedAction<S> {}

impl<S> Drop for ScopedAction<S> {
    fn drop(&mut self) {
        let _ = self.completed.send(());
    }
}

unsafe fn invoke_scoped<S, A>(callback: *mut (), state: &mut S)
where
    A: FnOnce(&mut S) + Send,
{
    let callback = unsafe { &mut *callback.cast::<Option<A>>() };
    callback
        .take()
        .expect("scoped owner callback must be invoked at most once")(state);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OwnerDispatchError {
    Closing,
    Reentrant,
    Terminated,
    AdmissionIncomplete,
}

struct CleanupReceipt {
    state: AtomicU8,
    wait: Mutex<()>,
    changed: Condvar,
    #[cfg(all(test, windows))]
    publication: Option<receipt_publication::Publication>,
}

impl CleanupReceipt {
    fn new() -> Self {
        Self {
            state: AtomicU8::new(RECEIPT_PENDING),
            wait: Mutex::new(()),
            changed: Condvar::new(),
            #[cfg(all(test, windows))]
            publication: receipt_publication::current(),
        }
    }

    fn publish(&self, state: u8) {
        let _wait = self
            .wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        #[cfg(all(test, windows))]
        receipt_publication::publisher_locked(self, state);
        self.state.store(state, Ordering::Release);
        self.changed.notify_all();
        #[cfg(all(test, windows))]
        receipt_publication::published(self);
    }

    fn wait_until(&self, deadline: Instant) -> u8 {
        #[cfg(all(test, windows))]
        receipt_publication::before_wait(self, deadline);
        let wait = match self.wait.try_lock() {
            Ok(wait) => wait,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            #[cfg(all(test, windows))]
            Err(TryLockError::WouldBlock) => {
                receipt_publication::actual_would_block(self, deadline);
                return RECEIPT_PENDING;
            }
            #[cfg(not(all(test, windows)))]
            Err(TryLockError::WouldBlock) => return RECEIPT_PENDING,
        };
        let _ = self
            .changed
            .wait_timeout_while(
                wait,
                deadline.saturating_duration_since(Instant::now()),
                |_| self.state.load(Ordering::Acquire) == RECEIPT_PENDING,
            )
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.state.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::dynamic_api::session) enum OwnerShutdownReceipt {
    Pending,
    Retryable,
    Poisoned,
    Panicked,
    Joined,
}

/// State is created, accessed and destroyed in one persistent managed task.
/// Only owned commands cross threads; S deliberately has no Send bound.
pub(super) struct SessionOwner<S> {
    pool: TaskPool,
    terminal: Arc<AtomicBool>,
    identity: Arc<()>,
    sender: Sender<OwnerCommand<S>>,
    shutdown: Mutex<Option<Arc<CleanupReceipt>>>,
    joined: AtomicBool,
    _code_owner: Arc<dyn Any + Send + Sync>,
}

impl<S> Drop for SessionOwner<S> {
    fn drop(&mut self) {
        if !self.joined.load(Ordering::Acquire) {
            // DLL owners must retain this controller after an incomplete receipt.
            eprintln!("fatal dynamic session owner dropped before task termination");
            std::process::abort();
        }
    }
}

impl<S: 'static> SessionOwner<S> {
    pub(super) fn create(
        factory: impl FnOnce() -> S + Send + 'static,
        code_owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, String> {
        let pool = TaskPool::try_new(
            TaskPoolDescriptor::io()
                .with_worker_threads(1)
                .with_thread_name("zircon-session-owner"),
        )
        .map_err(|error| error.to_string())?;
        let (sender, receiver) = bounded(COMMAND_CAPACITY);
        let terminal = Arc::new(AtomicBool::new(false));
        let task_terminal = Arc::clone(&terminal);
        let identity = Arc::new(());
        let task_identity = Arc::clone(&identity);
        #[cfg(all(test, windows))]
        let publication_schedule = receipt_publication::bind_owner(Arc::as_ptr(&identity) as usize);
        pool.spawn(move || {
            let _owner_thread = OwnerThreadGuard::enter(&task_identity);
            #[cfg(all(test, windows))]
            let _publication_thread = receipt_publication::worker_enter(&publication_schedule);
            let _ = catch_unwind(AssertUnwindSafe(|| run_owner(factory, receiver)));
            task_terminal.store(true, Ordering::Release);
            #[cfg(all(test, windows))]
            receipt_publication::terminal_published(
                &publication_schedule,
                task_terminal.load(Ordering::Acquire),
            );
        });
        Ok(Self {
            pool,
            terminal,
            identity,
            sender,
            shutdown: Mutex::new(None),
            joined: AtomicBool::new(false),
            _code_owner: code_owner,
        })
    }

    /// Runs a borrowing callback on the owner and does not return until the callback
    /// has completed or has been dropped by a terminated owner. The erased lifetime
    /// is sound because every exit waits for the command channel to release it.
    pub(super) fn dispatch_scoped<R: Send>(
        &self,
        action: impl FnOnce(&mut S) -> R + Send,
    ) -> Result<R, OwnerDispatchError> {
        if self.is_current_thread() {
            return Err(OwnerDispatchError::Reentrant);
        }
        let shutdown = self
            .shutdown
            .try_lock()
            .map_err(|_| OwnerDispatchError::Closing)?;
        if shutdown.is_some() {
            return Err(OwnerDispatchError::Closing);
        }
        let result = Arc::new(Mutex::new(None));
        let result_for_owner = Arc::clone(&result);
        let (completed_tx, completed_rx) = bounded(1);
        let mut callback = Some(move |state: &mut S| {
            let outcome = catch_unwind(AssertUnwindSafe(|| action(state)));
            *result_for_owner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(outcome);
        });
        self.sender
            .send(OwnerCommand::Scoped(ScopedAction::new(
                &mut callback,
                completed_tx,
            )))
            .map_err(|_| OwnerDispatchError::Terminated)?;
        drop(shutdown);
        completed_rx
            .recv()
            .map_err(|_| OwnerDispatchError::Terminated)?;
        let outcome = result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        match outcome.ok_or(OwnerDispatchError::Terminated)? {
            Ok(result) => Ok(result),
            Err(payload) => resume_unwind(payload),
        }
    }

    /// Admission is bounded; the caller owns the result receiver even after timeout.
    pub(super) fn dispatch<R: Send + 'static>(
        &self,
        deadline: Instant,
        action: impl FnOnce(&mut S) -> R + Send + 'static,
    ) -> Result<Receiver<R>, OwnerDispatchError> {
        if self.is_current_thread() {
            return Err(OwnerDispatchError::Reentrant);
        }
        let shutdown = self
            .shutdown
            .try_lock()
            .map_err(|_| OwnerDispatchError::Closing)?;
        if shutdown.is_some() {
            return Err(OwnerDispatchError::Closing);
        }
        let (result_tx, result_rx) = bounded(1);
        self.sender
            .send_timeout(
                OwnerCommand::Action(Box::new(move |state| {
                    let result = action(state);
                    let _ = result_tx.send(result);
                })),
                deadline.saturating_duration_since(Instant::now()),
            )
            .map_err(|error| match error {
                SendTimeoutError::Timeout(_) => OwnerDispatchError::AdmissionIncomplete,
                SendTimeoutError::Disconnected(_) => OwnerDispatchError::Terminated,
            })?;
        Ok(result_rx)
    }

    /// A pending cleanup is never invoked twice, including across expired waits.
    pub(super) fn shutdown_until(
        &self,
        deadline: Instant,
        cleanup: impl FnOnce(&mut S) -> bool + Send + 'static,
    ) -> OwnerShutdownReceipt {
        self.shutdown_until_inner(deadline, cleanup, false)
    }

    /// A cleanup panic poisons normal shutdown. Only an explicit recovery callback
    /// may repair that state; a normal retry never reinvokes the panicked callback.
    pub(super) fn recover_poisoned_until(
        &self,
        deadline: Instant,
        recovery: impl FnOnce(&mut S) -> bool + Send + 'static,
    ) -> Result<OwnerShutdownReceipt, &'static str> {
        {
            let shutdown = self.shutdown.try_lock().map_err(|_| "owner busy")?;
            if !shutdown
                .as_ref()
                .is_some_and(|receipt| receipt.state.load(Ordering::Acquire) == RECEIPT_PANICKED)
            {
                return Err("owner is not poisoned");
            }
        }
        Ok(self.shutdown_until_inner(deadline, recovery, true))
    }

    fn shutdown_until_inner(
        &self,
        deadline: Instant,
        cleanup: impl FnOnce(&mut S) -> bool + Send + 'static,
        recover_poisoned: bool,
    ) -> OwnerShutdownReceipt {
        if self.task_is_terminal() {
            if !self.pool.shutdown_until(deadline) {
                return OwnerShutdownReceipt::Pending;
            }
            self.joined.store(true, Ordering::Release);
            return match self.shutdown.try_lock() {
                Ok(shutdown)
                    if shutdown.as_ref().is_some_and(|receipt| {
                        receipt.state.load(Ordering::Acquire) == RECEIPT_SUCCEEDED
                    }) =>
                {
                    OwnerShutdownReceipt::Joined
                }
                _ => OwnerShutdownReceipt::Panicked,
            };
        }
        let receipt = {
            let mut shutdown = match self.shutdown.try_lock() {
                Ok(shutdown) => shutdown,
                Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
                Err(TryLockError::WouldBlock) => return OwnerShutdownReceipt::Pending,
            };
            let retry = match shutdown
                .as_ref()
                .map(|receipt| receipt.state.load(Ordering::Acquire))
            {
                None | Some(RECEIPT_RETRYABLE) => !recover_poisoned,
                Some(RECEIPT_PANICKED) => recover_poisoned,
                _ => false,
            };
            if retry {
                let receipt = Arc::new(CleanupReceipt::new());
                #[cfg(all(test, windows))]
                receipt_publication::prepare_receipt(
                    &receipt,
                    Arc::as_ptr(&self.identity) as usize,
                    deadline,
                );
                if self
                    .sender
                    .try_send(OwnerCommand::Shutdown(
                        Box::new(cleanup),
                        Arc::clone(&receipt),
                    ))
                    .is_err()
                {
                    receipt.publish(RECEIPT_RETRYABLE);
                    *shutdown = Some(receipt);
                    return OwnerShutdownReceipt::Pending;
                }
                *shutdown = Some(receipt);
                #[cfg(all(test, windows))]
                receipt_publication::admitted(shutdown.as_ref().expect("actual admitted receipt"));
            }
            Arc::clone(shutdown.as_ref().expect("admitted shutdown receipt"))
        };
        match receipt.wait_until(deadline) {
            RECEIPT_RETRYABLE => return OwnerShutdownReceipt::Retryable,
            RECEIPT_PANICKED => return OwnerShutdownReceipt::Poisoned,
            RECEIPT_SUCCEEDED => {
                return if self.pool.shutdown_until(deadline) {
                    self.joined.store(true, Ordering::Release);
                    OwnerShutdownReceipt::Joined
                } else {
                    OwnerShutdownReceipt::Pending
                };
            }
            _ => return OwnerShutdownReceipt::Pending,
        }
    }

    pub(super) fn task_is_terminal(&self) -> bool {
        self.terminal.load(Ordering::Acquire)
    }

    fn is_current_thread(&self) -> bool {
        let identity = Arc::as_ptr(&self.identity) as usize;
        ACTIVE_SESSION_OWNERS.with(|owners| owners.borrow().contains(&identity))
    }

    pub(super) fn shutdown_receipt(&self) -> Option<OwnerShutdownReceipt> {
        let shutdown = match self.shutdown.try_lock() {
            Ok(shutdown) => shutdown,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return Some(OwnerShutdownReceipt::Pending),
        };
        shutdown
            .as_ref()
            .map(|receipt| match receipt.state.load(Ordering::Acquire) {
                RECEIPT_RETRYABLE => OwnerShutdownReceipt::Retryable,
                RECEIPT_PANICKED => OwnerShutdownReceipt::Poisoned,
                RECEIPT_SUCCEEDED if self.joined.load(Ordering::Acquire) => {
                    OwnerShutdownReceipt::Joined
                }
                _ => OwnerShutdownReceipt::Pending,
            })
    }
}

struct OwnerThreadGuard {
    identity: usize,
}

impl OwnerThreadGuard {
    fn enter(identity: &Arc<()>) -> Self {
        let identity = Arc::as_ptr(identity) as usize;
        ACTIVE_SESSION_OWNERS.with(|owners| owners.borrow_mut().push(identity));
        Self { identity }
    }
}

impl Drop for OwnerThreadGuard {
    fn drop(&mut self) {
        ACTIVE_SESSION_OWNERS.with(|owners| {
            let popped = owners.borrow_mut().pop();
            debug_assert_eq!(popped, Some(self.identity));
        });
    }
}

fn run_owner<S>(factory: impl FnOnce() -> S, receiver: Receiver<OwnerCommand<S>>) {
    let mut state = factory();
    while let Ok(command) = receiver.recv() {
        match command {
            OwnerCommand::Action(action) => {
                // The outer owner boundary reports terminal panic after state is
                // destroyed here; no further command may access unwound state.
                action(&mut state);
            }
            OwnerCommand::Scoped(scoped) => {
                unsafe { (scoped.invoke)(scoped.callback, &mut state) };
            }
            OwnerCommand::Shutdown(cleanup, receipt) => {
                match catch_unwind(AssertUnwindSafe(|| cleanup(&mut state))) {
                    Ok(true) => {
                        drop(state);
                        receipt.publish(RECEIPT_SUCCEEDED);
                        return;
                    }
                    Ok(false) => receipt.publish(RECEIPT_RETRYABLE),
                    Err(_) => receipt.publish(RECEIPT_PANICKED),
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "session_owner/tests/cases.rs"]
mod tests;
