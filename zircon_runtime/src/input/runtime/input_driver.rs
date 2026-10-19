use std::fmt;
use std::mem::size_of;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::core::framework::input::{
    InputBatchLimits, InputBatchRejection, InputBatchSize, InputEvent, InputManager,
    DEFAULT_INPUT_EVENT_RECORDING_CAPACITY,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputIngressLimits {
    pub events_per_frame: usize,
    pub owned_bytes_per_frame: usize,
    pub max_receive_age: Duration,
}

impl Default for InputIngressLimits {
    fn default() -> Self {
        Self {
            events_per_frame: DEFAULT_INPUT_EVENT_RECORDING_CAPACITY as usize,
            owned_bytes_per_frame: 4 * 1024 * 1024,
            max_receive_age: Duration::from_secs(1),
        }
    }
}

/// This clock starts at Runtime receipt, not at the host or device timestamp.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputIngressPhase {
    #[default]
    Unattached,
    Receiving,
    Gap,
    Detached,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputIngressRejection {
    InvalidLimits,
    AlreadyAttached,
    GenerationExhausted,
    AttachmentMismatch,
    Detached,
    Gap,
    ReceiveClockAhead,
    ReceiveExpired,
    Batch(InputBatchRejection),
    FrameCapacity {
        reason: InputBatchRejection,
        reset_completed: bool,
    },
}

impl fmt::Display for InputIngressRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for InputIngressRejection {}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputIngressAdmission {
    Accepted(InputBatchSize),
    Rejected(InputIngressRejection),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputIngressHealth {
    pub generation: Option<NonZeroU64>,
    pub phase: InputIngressPhase,
    pub frame_events: usize,
    pub frame_owned_bytes: usize,
    pub accepted_batches: u64,
    pub rejected_batches: u64,
    pub gap_resets: u64,
    pub teardown_reset_pending: bool,
    pub last_receive_age: Option<Duration>,
    pub last_rejection: Option<InputIngressRejection>,
}

#[derive(Debug)]
struct AttachmentIdentity {
    generation: NonZeroU64,
}

/// Exact session capability. Equal generation numbers from another Driver are not interchangeable.
#[derive(Clone, Debug)]
pub struct InputAttachment {
    identity: Arc<AttachmentIdentity>,
}

impl InputAttachment {
    pub fn generation(&self) -> NonZeroU64 {
        self.identity.generation
    }
}

#[derive(Debug, Default)]
struct IngressState {
    generation: u64,
    attachment: Option<Arc<AttachmentIdentity>>,
    phase: InputIngressPhase,
    frame_events: usize,
    frame_owned_bytes: usize,
    accepted_batches: u64,
    rejected_batches: u64,
    gap_resets: u64,
    reset_pending: bool,
    last_receive: Option<Instant>,
    last_rejection: Option<InputIngressRejection>,
}

/// Admission and lifecycle owner for the registered Runtime session ingress.
/// DefaultInputManager alone owns physical state. This Driver has no polling thread or event queue.
#[derive(Debug)]
pub struct InputDriver {
    limits: InputIngressLimits,
    operation: Mutex<()>,
    state: Mutex<IngressState>,
}

impl Default for InputDriver {
    fn default() -> Self {
        Self {
            limits: InputIngressLimits::default(),
            operation: Mutex::new(()),
            state: Mutex::new(IngressState::default()),
        }
    }
}

impl InputDriver {
    pub fn with_limits(limits: InputIngressLimits) -> Result<Self, InputIngressRejection> {
        if limits.events_per_frame == 0
            || limits.owned_bytes_per_frame == 0
            || limits.max_receive_age.is_zero()
        {
            return Err(InputIngressRejection::InvalidLimits);
        }
        Ok(Self {
            limits,
            ..Self::default()
        })
    }

    fn lock_state(&self) -> MutexGuard<'_, IngressState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_operation(&self) -> MutexGuard<'_, ()> {
        self.operation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Claims once for a Session. Neither a later event nor a frame boundary reattaches a closed/gap source.
    pub fn claim_attachment(&self) -> Result<InputAttachment, InputIngressRejection> {
        let _operation = self.lock_operation();
        let mut state = self.lock_state();
        if state.attachment.is_some()
            && (state.phase != InputIngressPhase::Detached || state.reset_pending)
        {
            return Err(InputIngressRejection::AlreadyAttached);
        }
        let generation = state
            .generation
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or(InputIngressRejection::GenerationExhausted)?;
        let identity = Arc::new(AttachmentIdentity { generation });
        state.generation = generation.get();
        state.attachment = Some(Arc::clone(&identity));
        state.phase = InputIngressPhase::Unattached;
        state.frame_events = 0;
        state.frame_owned_bytes = 0;
        state.last_receive = None;
        state.last_rejection = None;
        Ok(InputAttachment { identity })
    }

    fn matches(state: &IngressState, attachment: &InputAttachment) -> bool {
        state
            .attachment
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &attachment.identity))
    }

    pub fn health(&self) -> InputIngressHealth {
        let state = self.lock_state();
        InputIngressHealth {
            generation: state
                .attachment
                .as_ref()
                .map(|identity| identity.generation),
            phase: state.phase,
            frame_events: state.frame_events,
            frame_owned_bytes: state.frame_owned_bytes,
            accepted_batches: state.accepted_batches,
            rejected_batches: state.rejected_batches,
            gap_resets: state.gap_resets,
            teardown_reset_pending: state.reset_pending,
            last_receive_age: state.last_receive.map(|received| received.elapsed()),
            last_rejection: state.last_rejection,
        }
    }

    pub fn submit_batch(
        &self,
        manager: &dyn InputManager,
        attachment: &InputAttachment,
        events: Vec<InputEvent>,
        received_at: Instant,
    ) -> InputIngressAdmission {
        let _operation = self.lock_operation();
        let mut state = self.lock_state();
        let rejection = if !Self::matches(&state, attachment) {
            Some(InputIngressRejection::AttachmentMismatch)
        } else {
            match state.phase {
                InputIngressPhase::Gap => Some(InputIngressRejection::Gap),
                InputIngressPhase::Detached => Some(InputIngressRejection::Detached),
                _ => match Instant::now().checked_duration_since(received_at) {
                    None => Some(InputIngressRejection::ReceiveClockAhead),
                    Some(age) if age > self.limits.max_receive_age => {
                        Some(InputIngressRejection::ReceiveExpired)
                    }
                    _ => None,
                },
            }
        };
        if let Some(reason) = rejection {
            state.rejected_batches = state.rejected_batches.saturating_add(1);
            state.last_rejection = Some(reason);
            return InputIngressAdmission::Rejected(reason);
        }
        let remaining = InputBatchLimits::new(
            self.limits
                .events_per_frame
                .saturating_sub(state.frame_events),
            self.limits
                .owned_bytes_per_frame
                .saturating_sub(state.frame_owned_bytes),
        );
        drop(state);
        // Health queries never need the operation lock; recorder notifications
        // can inspect health while the canonical manager commits the batch.
        match manager.submit_batch_with_admission(events, remaining) {
            Ok(size) => {
                let mut state = self.lock_state();
                state.phase = InputIngressPhase::Receiving;
                state.frame_events += size.events;
                state.frame_owned_bytes += size.owned_bytes;
                state.accepted_batches = state.accepted_batches.saturating_add(1);
                state.last_receive = Some(received_at);
                InputIngressAdmission::Accepted(size)
            }
            Err(reason) if reason.is_capacity() => {
                {
                    let mut state = self.lock_state();
                    state.phase = InputIngressPhase::Gap;
                    state.reset_pending = true;
                }
                // This separately identified control batch has reserved capacity.
                // A rejected data batch itself never partially updates the manager.
                let reset_completed = Self::reset_physical(manager).is_ok();
                let reason = InputIngressRejection::FrameCapacity {
                    reason,
                    reset_completed,
                };
                let mut state = self.lock_state();
                state.reset_pending = !reset_completed;
                state.gap_resets = state.gap_resets.saturating_add(u64::from(reset_completed));
                state.rejected_batches = state.rejected_batches.saturating_add(1);
                state.last_rejection = Some(reason);
                InputIngressAdmission::Rejected(reason)
            }
            Err(reason) => {
                let reason = InputIngressRejection::Batch(reason);
                let mut state = self.lock_state();
                state.rejected_batches = state.rejected_batches.saturating_add(1);
                state.last_rejection = Some(reason);
                InputIngressAdmission::Rejected(reason)
            }
        }
    }

    fn reset_physical(manager: &dyn InputManager) -> Result<InputBatchSize, InputBatchRejection> {
        manager.submit_batch_with_admission(
            vec![InputEvent::FocusLost],
            InputBatchLimits::new(1, size_of::<InputEvent>()),
        )
    }

    pub fn begin_frame(
        &self,
        manager: &dyn InputManager,
        attachment: &InputAttachment,
    ) -> Result<(), InputIngressRejection> {
        let _operation = self.lock_operation();
        {
            let state = self.lock_state();
            if !Self::matches(&state, attachment) {
                return Err(InputIngressRejection::AttachmentMismatch);
            }
            if state.phase == InputIngressPhase::Detached {
                return Err(InputIngressRejection::Detached);
            }
        }
        manager.begin_frame();
        let mut state = self.lock_state();
        state.frame_events = 0;
        state.frame_owned_bytes = 0;
        Ok(())
    }

    /// Idempotent only for this exact capability. Old cleanup cannot detach a successor.
    /// A failed reset remains pending and can be retried by the owning Session.
    pub fn detach(
        &self,
        manager: &dyn InputManager,
        attachment: &InputAttachment,
    ) -> Result<(), InputIngressRejection> {
        let _operation = self.lock_operation();
        let reset_required = {
            let mut state = self.lock_state();
            if !Self::matches(&state, attachment) {
                return Err(InputIngressRejection::AttachmentMismatch);
            }
            let reset_required = state.reset_pending || state.phase == InputIngressPhase::Receiving;
            state.phase = InputIngressPhase::Detached;
            state.reset_pending = reset_required;
            reset_required
        };
        if reset_required {
            let reset = Self::reset_physical(manager);
            let mut state = self.lock_state();
            state.reset_pending = reset.is_err();
            if let Err(reason) = reset {
                let reason = InputIngressRejection::Batch(reason);
                state.last_rejection = Some(reason);
                return Err(reason);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "input_driver/tests/cases.rs"]
mod tests;
