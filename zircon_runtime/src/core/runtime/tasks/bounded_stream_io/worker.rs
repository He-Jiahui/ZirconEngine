use std::io::ErrorKind;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use super::super::TaskCancellationToken;
use super::decoder::BoundedLineDecoder;
use super::lane::ReaderPermit;
use super::model::{BoundedStreamIoFailure, BoundedStreamIoLimits, BoundedStreamIoReader};
use super::state::{CaptureState, ReaderOutcome};

pub(super) struct ReaderStartGate {
    state: Mutex<ReaderStartState>,
    ready: Condvar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReaderStartState {
    Pending,
    Started,
    Aborted,
}

impl ReaderStartGate {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ReaderStartState::Pending),
            ready: Condvar::new(),
        }
    }

    pub fn start(&self) {
        *self.lock() = ReaderStartState::Started;
        self.ready.notify_all();
    }

    pub fn abort(&self) {
        *self.lock() = ReaderStartState::Aborted;
        self.ready.notify_all();
    }

    fn wait(&self) -> bool {
        let state = self.lock();
        let state = self
            .ready
            .wait_while(state, |state| *state == ReaderStartState::Pending)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *state == ReaderStartState::Started
    }

    fn lock(&self) -> MutexGuard<'_, ReaderStartState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(super) fn run_reader(
    mut input: BoundedStreamIoReader,
    state: Arc<CaptureState>,
    limits: BoundedStreamIoLimits,
    gate: Arc<ReaderStartGate>,
    _token: TaskCancellationToken,
    _permit: ReaderPermit,
) {
    let mut terminal = ReaderTerminalGuard::new(Arc::clone(&state), input.stream.clone());
    if !gate.wait() {
        terminal.complete(ReaderOutcome::Cancelled);
        return;
    }
    if !state.accepts_records() {
        terminal.complete(ReaderOutcome::Cancelled);
        return;
    }

    let mut decoder = BoundedLineDecoder::new(limits.max_line_bytes);
    let mut buffer = vec![0_u8; limits.read_chunk_bytes];
    loop {
        match input.reader.read(&mut buffer) {
            Ok(0) => {
                if let Some(line) = decoder.finish() {
                    state.enqueue(&input.stream, line, limits);
                }
                let outcome = if state.is_cancellation_requested() {
                    ReaderOutcome::Cancelled
                } else {
                    ReaderOutcome::Completed
                };
                terminal.complete(outcome);
                return;
            }
            Ok(read) => {
                if !decoder.push(&buffer[..read], |line| {
                    state.enqueue(&input.stream, line, limits)
                }) {
                    terminal.complete(ReaderOutcome::Cancelled);
                    return;
                }
                if !state.accepts_records() {
                    terminal.complete(ReaderOutcome::Cancelled);
                    return;
                }
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => {
                if let Some(line) = decoder.finish() {
                    state.enqueue(&input.stream, line, limits);
                }
                if state.is_cancellation_requested() {
                    terminal.complete(ReaderOutcome::Cancelled);
                } else {
                    terminal.complete(ReaderOutcome::Failed(BoundedStreamIoFailure {
                        stream: input.stream.clone(),
                        message: error.to_string(),
                    }));
                }
                return;
            }
        }
    }
}

struct ReaderTerminalGuard {
    state: Arc<CaptureState>,
    stream: super::model::BoundedStreamIoStreamId,
    outcome: Option<ReaderOutcome>,
}

impl ReaderTerminalGuard {
    fn new(state: Arc<CaptureState>, stream: super::model::BoundedStreamIoStreamId) -> Self {
        Self {
            state,
            stream,
            outcome: None,
        }
    }

    fn complete(&mut self, outcome: ReaderOutcome) {
        self.outcome = Some(outcome);
    }
}

impl Drop for ReaderTerminalGuard {
    fn drop(&mut self) {
        let outcome = self.outcome.take().unwrap_or_else(|| {
            ReaderOutcome::Failed(BoundedStreamIoFailure {
                stream: self.stream.clone(),
                message: "bounded stream reader panicked".to_owned(),
            })
        });
        self.state.finish_reader(outcome);
    }
}
