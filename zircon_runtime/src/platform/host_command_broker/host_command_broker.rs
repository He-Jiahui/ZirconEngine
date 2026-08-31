use std::collections::{HashMap, VecDeque};
use std::num::NonZeroUsize;
use std::time::Instant;

use crate::core::framework::window::{
    WindowCommand, WindowCommandAccepted, WindowCommandHeader, WindowCommandId,
    WindowCommandReceipt, WindowCommandTerminal, WindowEffectiveSnapshot, WindowId,
    WindowRequestedGeneration, WindowRequestedState, WindowStateSnapshot,
};

use super::{
    HostCommandAdmissionError, HostCommandBrokerError, HostCommandDispatch, HostCommandExecution,
    WindowCommandFailure,
};

/// Platform-thread command authority. It only coordinates desired-state
/// delivery and receipts; WindowStateRegistry remains the owner of the
/// snapshots passed into every terminal transition.
pub(crate) struct HostCommandBroker {
    maximum_outstanding: usize,
    last_request_id: u64,
    pending: VecDeque<HostCommandExecution>,
    in_flight: HashMap<WindowCommandId, HostCommandExecution>,
    terminal: HashMap<
        WindowCommandId,
        WindowCommandReceipt<WindowEffectiveSnapshot, WindowCommandFailure>,
    >,
}

impl HostCommandBroker {
    pub(crate) fn new(maximum_outstanding: NonZeroUsize) -> Self {
        Self {
            maximum_outstanding: maximum_outstanding.get(),
            last_request_id: 0,
            pending: VecDeque::new(),
            in_flight: HashMap::new(),
            terminal: HashMap::new(),
        }
    }

    /// Admits one desired state against a current, generation-qualified state
    /// snapshot. A command already at its deadline is accepted only long
    /// enough to publish the same exact canceled receipt as any later expiry.
    pub(crate) fn submit(
        &mut self,
        target: WindowId,
        deadline: Instant,
        desired: WindowRequestedState,
        current: &WindowStateSnapshot,
        accepted_at: Instant,
    ) -> Result<WindowCommandAccepted, HostCommandBrokerError> {
        let header = self.prepare_submission(target, deadline, current)?;
        Ok(self.commit_submission(
            header,
            desired,
            current.requested().generation(),
            current,
            accepted_at,
        ))
    }

    /// Reserves broker capacity before publishing requested state. The closure
    /// runs only after the deadline gate while the driver still owns the
    /// registry/state/broker transaction; a publication failure therefore
    /// cannot enqueue a command or consume a request identifier.
    pub(crate) fn submit_after_requested_state<E>(
        &mut self,
        target: WindowId,
        deadline: Instant,
        desired: WindowRequestedState,
        current: &WindowStateSnapshot,
        accepted_at: Instant,
        publish_requested_state: impl FnOnce() -> Result<(), E>,
    ) -> Result<WindowCommandAccepted, HostCommandAdmissionError<E>> {
        let header = self
            .prepare_submission(target, deadline, current)
            .map_err(HostCommandAdmissionError::Broker)?;
        if deadline <= accepted_at {
            return Ok(self.commit_submission(
                header,
                desired,
                current.requested().generation(),
                current,
                accepted_at,
            ));
        }
        let requested_generation =
            current
                .requested()
                .generation()
                .next()
                .ok_or(HostCommandAdmissionError::Broker(
                    HostCommandBrokerError::RequestedGenerationExhausted {
                        window: target,
                        current: current.requested().generation(),
                    },
                ))?;
        publish_requested_state().map_err(HostCommandAdmissionError::RequestedState)?;
        Ok(self.commit_submission(header, desired, requested_generation, current, accepted_at))
    }

    pub(crate) fn next_target(&self) -> Option<WindowId> {
        self.pending.front().map(HostCommandExecution::target)
    }

    /// Moves the next FIFO command to the platform thread. The caller obtains
    /// the snapshot while it still owns the state-registry transaction, so a
    /// stale target cannot be dispatched against a different window slot.
    pub(crate) fn dispatch_next(
        &mut self,
        now: Instant,
        current: &WindowStateSnapshot,
    ) -> Result<Option<HostCommandDispatch>, HostCommandBrokerError> {
        // The current host bridge owns one platform-thread execution lane.
        // Serial completion preserves the correspondence between native call
        // order and effective-state generations without a second scheduler.
        if !self.in_flight.is_empty() {
            return Ok(None);
        }
        let Some(header) = self.pending.front().map(HostCommandExecution::header) else {
            return Ok(None);
        };
        self.validate_snapshot_target(header.target(), current)?;
        if self.in_flight.contains_key(&header.request_id()) {
            return Err(HostCommandBrokerError::DuplicateInFlightRequest {
                request_id: header.request_id(),
            });
        }
        if self.terminal.contains_key(&header.request_id()) {
            return Err(HostCommandBrokerError::DuplicateTerminalReceipt {
                request_id: header.request_id(),
            });
        }
        if header.deadline() <= now {
            self.reserve_terminal_receipts(1)?;
        }
        let Some(execution) = self.pending.pop_front() else {
            return Ok(None);
        };

        if execution.command().deadline() <= now {
            let receipt =
                self.receipt(execution.header(), current, WindowCommandTerminal::Canceled);
            self.insert_terminal(receipt.clone());
            return Ok(Some(HostCommandDispatch::Terminal(receipt)));
        }

        let previous = self
            .in_flight
            .insert(execution.request_id(), execution.clone());
        debug_assert!(previous.is_none());
        Ok(Some(HostCommandDispatch::Execute(execution)))
    }

    /// Publishes the exact state snapshot observed after the platform thread
    /// resolves one native command. Repeating the completion cannot overwrite
    /// or replace its terminal result.
    pub(crate) fn complete(
        &mut self,
        request_id: WindowCommandId,
        current: &WindowStateSnapshot,
        terminal: WindowCommandTerminal<WindowCommandFailure>,
    ) -> Result<
        WindowCommandReceipt<WindowEffectiveSnapshot, WindowCommandFailure>,
        HostCommandBrokerError,
    > {
        let execution = self
            .in_flight
            .get(&request_id)
            .cloned()
            .ok_or(HostCommandBrokerError::UnknownInFlightRequest { request_id })?;
        let header = execution.header();
        self.validate_snapshot_target(header.target(), current)?;
        if self.terminal.contains_key(&request_id) {
            return Err(HostCommandBrokerError::DuplicateTerminalReceipt { request_id });
        }
        self.reserve_terminal_receipts(1)?;

        let receipt = self.receipt(header, current, terminal);
        let removed = self.in_flight.remove(&request_id);
        debug_assert_eq!(removed, Some(execution));
        self.insert_terminal(receipt.clone());
        Ok(receipt)
    }

    /// Cancels every command for a closing window only after the platform host
    /// has quiesced native execution. This preserves the ordering required by
    /// native-window and surface-lease teardown: stop execution, terminalize,
    /// then release the window generation.
    pub(crate) fn cancel_window_after_quiesce(
        &mut self,
        target: WindowId,
        current: &WindowStateSnapshot,
    ) -> Result<usize, HostCommandBrokerError> {
        self.validate_snapshot_target(target, current)?;
        let cancellation_count = self
            .pending
            .iter()
            .filter(|execution| execution.target() == target)
            .count()
            .saturating_add(
                self.in_flight
                    .values()
                    .filter(|execution| execution.target() == target)
                    .count(),
            );
        if cancellation_count == 0 {
            return Ok(0);
        }
        if self.terminal.len().saturating_add(cancellation_count) > self.maximum_outstanding {
            return Err(HostCommandBrokerError::OutstandingLimitReached {
                limit: self.maximum_outstanding,
            });
        }
        self.reserve_terminal_receipts(cancellation_count)?;

        let mut headers = Vec::new();
        headers
            .try_reserve(cancellation_count)
            .map_err(|_| HostCommandBrokerError::AllocationFailed)?;
        headers.extend(
            self.pending
                .iter()
                .filter(|execution| execution.target() == target)
                .map(HostCommandExecution::header),
        );
        headers.extend(
            self.in_flight
                .values()
                .filter(|execution| execution.target() == target)
                .map(HostCommandExecution::header),
        );
        headers.sort_by_key(|header| header.request_id().raw());
        if let Some(header) = headers
            .iter()
            .find(|header| self.terminal.contains_key(&header.request_id()))
        {
            return Err(HostCommandBrokerError::DuplicateTerminalReceipt {
                request_id: header.request_id(),
            });
        }

        self.pending
            .retain(|execution| execution.target() != target);
        for header in headers {
            let removed = self.in_flight.remove(&header.request_id());
            debug_assert!(
                removed.is_none() || removed.is_some_and(|entry| entry.header() == header)
            );
            self.insert_terminal(self.receipt(header, current, WindowCommandTerminal::Canceled));
        }
        Ok(cancellation_count)
    }

    pub(crate) fn take_terminal_receipt(
        &mut self,
        request_id: WindowCommandId,
    ) -> Option<WindowCommandReceipt<WindowEffectiveSnapshot, WindowCommandFailure>> {
        self.terminal.remove(&request_id)
    }

    /// Returns the immutable execution identity while completion owns the
    /// driver transaction. No internal map reference escapes this broker.
    pub(crate) fn in_flight_execution(
        &self,
        request_id: WindowCommandId,
    ) -> Result<HostCommandExecution, HostCommandBrokerError> {
        self.in_flight
            .get(&request_id)
            .cloned()
            .ok_or(HostCommandBrokerError::UnknownInFlightRequest { request_id })
    }

    pub(crate) fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn in_flight_len(&self) -> usize {
        self.in_flight.len()
    }

    pub(crate) fn terminal_len(&self) -> usize {
        self.terminal.len()
    }

    fn outstanding_len(&self) -> usize {
        self.pending
            .len()
            .saturating_add(self.in_flight.len())
            .saturating_add(self.terminal.len())
    }

    fn reserve_submission_state(&mut self) -> Result<(), HostCommandBrokerError> {
        self.pending
            .try_reserve(1)
            .map_err(|_| HostCommandBrokerError::AllocationFailed)?;
        self.in_flight
            .try_reserve(1)
            .map_err(|_| HostCommandBrokerError::AllocationFailed)?;
        self.terminal
            .try_reserve(1)
            .map_err(|_| HostCommandBrokerError::AllocationFailed)
    }

    /// Terminal entries accumulate after commands leave pending/in-flight, so
    /// submission-time reservation cannot cover a later batch cancellation.
    fn reserve_terminal_receipts(
        &mut self,
        additional: usize,
    ) -> Result<(), HostCommandBrokerError> {
        self.terminal
            .try_reserve(additional)
            .map_err(|_| HostCommandBrokerError::AllocationFailed)
    }

    fn prepare_submission(
        &mut self,
        target: WindowId,
        deadline: Instant,
        current: &WindowStateSnapshot,
    ) -> Result<WindowCommandHeader, HostCommandBrokerError> {
        self.validate_snapshot_target(target, current)?;
        if self.outstanding_len() >= self.maximum_outstanding {
            return Err(HostCommandBrokerError::OutstandingLimitReached {
                limit: self.maximum_outstanding,
            });
        }
        self.reserve_submission_state()?;
        let request_id = self.next_request_id_candidate()?;
        Ok(WindowCommandHeader::new(target, request_id, deadline))
    }

    fn commit_submission(
        &mut self,
        header: WindowCommandHeader,
        desired: WindowRequestedState,
        requested_generation: WindowRequestedGeneration,
        current: &WindowStateSnapshot,
        accepted_at: Instant,
    ) -> WindowCommandAccepted {
        debug_assert_eq!(
            header.request_id().raw(),
            self.last_request_id.saturating_add(1)
        );
        self.last_request_id = header.request_id().raw();
        let accepted = WindowCommandAccepted::new(header, accepted_at);
        if header.deadline() <= accepted_at {
            self.insert_terminal(self.receipt(header, current, WindowCommandTerminal::Canceled));
        } else {
            self.pending.push_back(HostCommandExecution::new(
                WindowCommand::new(header, desired),
                requested_generation,
            ));
        }
        accepted
    }

    fn next_request_id_candidate(&self) -> Result<WindowCommandId, HostCommandBrokerError> {
        let next = self
            .last_request_id
            .checked_add(1)
            .ok_or(HostCommandBrokerError::RequestIdExhausted)?;
        let request_id =
            WindowCommandId::new(next).ok_or(HostCommandBrokerError::RequestIdExhausted)?;
        Ok(request_id)
    }

    fn validate_snapshot_target(
        &self,
        expected: WindowId,
        current: &WindowStateSnapshot,
    ) -> Result<(), HostCommandBrokerError> {
        if current.window() != expected {
            return Err(HostCommandBrokerError::SnapshotTargetMismatch {
                expected,
                actual: current.window(),
            });
        }
        Ok(())
    }

    fn receipt(
        &self,
        header: WindowCommandHeader,
        current: &WindowStateSnapshot,
        terminal: WindowCommandTerminal<WindowCommandFailure>,
    ) -> WindowCommandReceipt<WindowEffectiveSnapshot, WindowCommandFailure> {
        WindowCommandReceipt::new(
            header,
            current.observed().generation(),
            current.effective().clone(),
            terminal,
        )
    }

    fn insert_terminal(
        &mut self,
        receipt: WindowCommandReceipt<WindowEffectiveSnapshot, WindowCommandFailure>,
    ) {
        let request_id = receipt.header().request_id();
        let previous = self.terminal.insert(request_id, receipt);
        debug_assert!(previous.is_none());
    }
}
