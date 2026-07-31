use std::io::{self, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use super::super::worker::DurableOutput;

#[derive(Clone, Default)]
pub(super) struct SharedOutput(Arc<Mutex<Vec<u8>>>);

impl SharedOutput {
    pub(super) fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl Write for SharedOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl DurableOutput for SharedOutput {
    fn sync_data(&mut self) -> io::Result<()> {
        self.flush()
    }
}

#[derive(Clone, Default)]
pub(super) struct BlockingOutput {
    bytes: SharedOutput,
    state: Arc<(Mutex<BlockingOutputState>, Condvar)>,
}

#[derive(Default)]
struct BlockingOutputState {
    blocked: bool,
    released: bool,
}

impl BlockingOutput {
    pub(super) fn wait_until_blocked(&self) {
        let (state, changed) = &*self.state;
        let mut state = state.lock().unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !state.blocked {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let (next_state, wait) = changed.wait_timeout(state, remaining).unwrap();
            state = next_state;
            assert!(
                !wait.timed_out(),
                "sink worker did not enter the blocking output"
            );
        }
    }

    pub(super) fn release(&self) {
        let (state, changed) = &*self.state;
        let mut state = state.lock().unwrap();
        state.released = true;
        changed.notify_all();
    }

    pub(super) fn text(&self) -> String {
        self.bytes.text()
    }
}

impl Write for BlockingOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let (state, changed) = &*self.state;
        let mut state = state.lock().unwrap();
        state.blocked = true;
        changed.notify_all();
        while !state.released {
            state = changed.wait(state).unwrap();
        }
        drop(state);
        self.bytes.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.bytes.flush()
    }
}

impl DurableOutput for BlockingOutput {
    fn sync_data(&mut self) -> io::Result<()> {
        self.flush()
    }
}

pub(super) struct FailingOutput;

impl Write for FailingOutput {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected write failure"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected flush failure"))
    }
}

impl DurableOutput for FailingOutput {
    fn sync_data(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected sync failure"))
    }
}

#[derive(Default)]
pub(super) struct SyncFailingOutput(Vec<u8>);

impl Write for SyncFailingOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl DurableOutput for SyncFailingOutput {
    fn sync_data(&mut self) -> io::Result<()> {
        Err(io::Error::other("injected sync failure"))
    }
}
