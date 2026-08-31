use std::io::{self, Cursor, Read};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::{
    BoundedStreamIoAdmissionError, BoundedStreamIoDrainBudget, BoundedStreamIoLane,
    BoundedStreamIoLimits, BoundedStreamIoReader, BoundedStreamIoStreamId,
};
use crate::core::runtime::tasks::{EngineTaskGraph, EngineTaskGraphOptions};

fn runtime() -> EngineTaskGraph {
    EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(8))
        .expect("test runtime should own two I/O workers")
}

fn reader(stream: BoundedStreamIoStreamId, bytes: impl Into<Vec<u8>>) -> BoundedStreamIoReader {
    BoundedStreamIoReader::new(stream, Cursor::new(bytes.into()))
}

fn wait_for_terminal(capture: &super::BoundedStreamIoCapture) {
    assert!(
        capture.wait_until_terminal(Duration::from_secs(2)),
        "fixture readers should reach terminal state"
    );
}

#[test]
fn capture_preserves_per_stream_order_identity_and_terminal_tail() {
    let runtime = runtime();
    let lane = BoundedStreamIoLane::try_new(
        &runtime,
        "test.process-output",
        BoundedStreamIoLimits::default(),
    )
    .unwrap();
    let capture = lane
        .capture(vec![
            reader(
                BoundedStreamIoStreamId::stdout(),
                b"first\r\nsecond\nstdout-tail".to_vec(),
            ),
            reader(
                BoundedStreamIoStreamId::stderr(),
                b"warning\nstderr-tail".to_vec(),
            ),
        ])
        .unwrap();

    wait_for_terminal(&capture);
    let batch = capture.drain(BoundedStreamIoDrainBudget::unlimited());
    let stdout = batch
        .records
        .iter()
        .filter(|record| record.stream == BoundedStreamIoStreamId::stdout())
        .map(|record| record.text.as_str())
        .collect::<Vec<_>>();
    let stderr = batch
        .records
        .iter()
        .filter(|record| record.stream == BoundedStreamIoStreamId::stderr())
        .map(|record| record.text.as_str())
        .collect::<Vec<_>>();

    assert_eq!(stdout, ["first", "second", "stdout-tail"]);
    assert_eq!(stderr, ["warning", "stderr-tail"]);
    assert_eq!(batch.diagnostics.completed_readers, 2);
    assert_eq!(batch.diagnostics.queued_records, 0);
    assert_eq!(batch.diagnostics.queued_bytes, 0);
}

#[test]
fn line_decoder_bounds_unterminated_input_and_reports_lossy_utf8() {
    let runtime = runtime();
    let limits = BoundedStreamIoLimits::default()
        .with_read_chunk_bytes(3)
        .with_max_line_bytes(4);
    let lane = BoundedStreamIoLane::try_new(&runtime, "test.decode", limits).unwrap();
    let capture = lane
        .capture(vec![reader(
            BoundedStreamIoStreamId::stdout(),
            vec![b'a', b'b', 0x80, b'c', b'd', b'e'],
        )])
        .unwrap();

    wait_for_terminal(&capture);
    let batch = capture.drain(BoundedStreamIoDrainBudget::unlimited());

    assert_eq!(batch.records.len(), 1);
    assert!(batch.records[0].lossy_utf8);
    assert!(batch.records[0].text.starts_with("ab"));
    assert!(batch.records[0].text.len() <= limits.max_line_bytes);
    assert_eq!(batch.records[0].truncated_bytes, 4);
    assert_eq!(batch.diagnostics.lossy_utf8_records, 1);
    assert_eq!(batch.diagnostics.truncated_records, 1);
    assert_eq!(batch.diagnostics.truncated_bytes, 4);
}

#[test]
fn queue_entry_budget_drops_new_records_without_exceeding_retained_bytes() {
    let runtime = runtime();
    let limits = BoundedStreamIoLimits::default()
        .with_queue_entry_capacity(2)
        .with_queue_byte_capacity(4 * 1024);
    let lane = BoundedStreamIoLane::try_new(&runtime, "test.queue", limits).unwrap();
    let capture = lane
        .capture(vec![reader(
            BoundedStreamIoStreamId::stdout(),
            b"one\ntwo\nthree\n".to_vec(),
        )])
        .unwrap();

    wait_for_terminal(&capture);
    let batch = capture.drain(BoundedStreamIoDrainBudget::unlimited());

    assert_eq!(
        batch
            .records
            .iter()
            .map(|record| record.text.as_str())
            .collect::<Vec<_>>(),
        ["one", "two"]
    );
    assert_eq!(batch.diagnostics.dropped_records, 1);
    assert!(batch.diagnostics.dropped_bytes > 0);
    assert!(batch.diagnostics.peak_queued_bytes <= limits.queue_byte_capacity);
    assert!(batch.diagnostics.peak_queued_records <= limits.queue_entry_capacity);
}

#[test]
fn live_drain_defers_records_after_count_or_byte_budget() {
    let runtime = runtime();
    let lane =
        BoundedStreamIoLane::try_new(&runtime, "test.drain", BoundedStreamIoLimits::default())
            .unwrap();
    let capture = lane
        .capture(vec![reader(
            BoundedStreamIoStreamId::stdout(),
            b"1111\n2222\n3333\n".to_vec(),
        )])
        .unwrap();
    wait_for_terminal(&capture);

    let first = capture.drain(BoundedStreamIoDrainBudget::new(
        2,
        usize::MAX,
        Duration::from_secs(1),
    ));
    assert_eq!(
        first
            .records
            .iter()
            .map(|record| record.text.as_str())
            .collect::<Vec<_>>(),
        ["1111", "2222"]
    );
    assert_eq!(first.diagnostics.queued_records, 1);

    let second = capture.drain(BoundedStreamIoDrainBudget::new(
        usize::MAX,
        1,
        Duration::from_secs(1),
    ));
    assert_eq!(second.records[0].text, "3333");
    assert_eq!(second.diagnostics.queued_records, 0);
}

#[test]
fn reader_capacity_is_reserved_for_the_whole_lane() {
    let runtime = runtime();
    let limits = BoundedStreamIoLimits::default().with_max_concurrent_readers(1);
    let lane = BoundedStreamIoLane::try_new(&runtime, "test.reader-budget", limits).unwrap();
    let (first_reader, first_writer) = controlled_reader();
    let first_capture = lane
        .capture(vec![BoundedStreamIoReader::new(
            BoundedStreamIoStreamId::stdout(),
            first_reader,
        )])
        .unwrap();

    let error = lane
        .capture(vec![reader(
            BoundedStreamIoStreamId::stderr(),
            b"must-not-start".to_vec(),
        )])
        .unwrap_err();
    assert!(matches!(
        error,
        BoundedStreamIoAdmissionError::ReaderCapacityReached {
            requested: 1,
            available: 0
        }
    ));
    assert_eq!(lane.diagnostics().active_readers, 1);
    assert_eq!(lane.diagnostics().rejected_readers, 1);

    drop(first_writer);
    wait_for_terminal(&first_capture);
    assert_eq!(lane.diagnostics().active_readers, 0);
}

#[test]
fn oversized_multi_reader_capture_aborts_before_reading_any_stream() {
    let runtime = runtime();
    let limits = BoundedStreamIoLimits::default().with_max_concurrent_readers(1);
    let lane = BoundedStreamIoLane::try_new(&runtime, "test.atomic-admission", limits).unwrap();
    let reads = Arc::new(Mutex::new(0_u32));

    let error = lane
        .capture(vec![
            BoundedStreamIoReader::new(
                BoundedStreamIoStreamId::stdout(),
                CountingReader::new(Arc::clone(&reads)),
            ),
            BoundedStreamIoReader::new(
                BoundedStreamIoStreamId::stderr(),
                CountingReader::new(Arc::clone(&reads)),
            ),
        ])
        .unwrap_err();

    assert!(matches!(
        error,
        BoundedStreamIoAdmissionError::ReaderCapacityReached {
            requested: 2,
            available: 1
        }
    ));
    assert_eq!(*reads.lock().unwrap(), 0);
}

#[test]
fn capture_never_admits_more_blocking_readers_than_physical_io_workers() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let lane = BoundedStreamIoLane::try_new(
        &runtime,
        "test.physical-reader-budget",
        BoundedStreamIoLimits::default(),
    )
    .unwrap();

    let error = lane
        .capture(vec![
            reader(BoundedStreamIoStreamId::stdout(), Vec::new()),
            reader(BoundedStreamIoStreamId::stderr(), Vec::new()),
        ])
        .unwrap_err();

    assert_eq!(lane.reader_capacity(), 1);
    assert!(matches!(
        error,
        BoundedStreamIoAdmissionError::ReaderCapacityReached {
            requested: 2,
            available: 1
        }
    ));
}

#[test]
fn cancellation_after_pipe_close_reaches_terminal_without_losing_residual_output() {
    let runtime = runtime();
    let lane =
        BoundedStreamIoLane::try_new(&runtime, "test.cancel", BoundedStreamIoLimits::default())
            .unwrap();
    let (controlled, writer) = controlled_reader();
    let capture = lane
        .capture(vec![BoundedStreamIoReader::new(
            BoundedStreamIoStreamId::stdout(),
            controlled,
        )])
        .unwrap();
    writer.send(b"residual".to_vec()).unwrap();

    capture.request_cancellation();
    drop(writer);
    wait_for_terminal(&capture);
    let batch = capture.drain(BoundedStreamIoDrainBudget::unlimited());

    assert_eq!(batch.records[0].text, "residual");
    assert_eq!(batch.diagnostics.cancelled_readers, 1);
}

#[test]
fn dropping_capture_is_nonblocking_but_runtime_keeps_blocked_reader_accounted() {
    let runtime = runtime();
    let lane =
        BoundedStreamIoLane::try_new(&runtime, "test.drop", BoundedStreamIoLimits::default())
            .unwrap();
    let (controlled, writer) = controlled_reader();
    let capture = lane
        .capture(vec![BoundedStreamIoReader::new(
            BoundedStreamIoStreamId::stdout(),
            controlled,
        )])
        .unwrap();
    assert!(wait_until(Duration::from_secs(1), || {
        lane.scope_census().running == 1
    }));

    let started = Instant::now();
    drop(capture);
    assert!(started.elapsed() < Duration::from_millis(50));
    assert_eq!(lane.scope_census().running, 1);

    drop(writer);
    runtime
        .shutdown(Duration::from_secs(2))
        .expect("closing the producer pipe should release the accounted I/O task");
}

#[test]
fn read_failures_are_typed_and_bounded_to_the_source_stream() {
    let runtime = runtime();
    let lane = BoundedStreamIoLane::try_new(
        &runtime,
        "test.read-error",
        BoundedStreamIoLimits::default(),
    )
    .unwrap();
    let capture = lane
        .capture(vec![BoundedStreamIoReader::new(
            BoundedStreamIoStreamId::stderr(),
            FailingReader,
        )])
        .unwrap();

    wait_for_terminal(&capture);
    let failures = capture.failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].stream, BoundedStreamIoStreamId::stderr());
    assert!(failures[0].message.contains("fixture read failure"));
    assert_eq!(capture.diagnostics().failed_readers, 1);
}

#[test]
fn interrupted_reads_are_retried_without_terminal_failure() {
    let runtime = runtime();
    let lane = BoundedStreamIoLane::try_new(
        &runtime,
        "test.interrupted-read",
        BoundedStreamIoLimits::default(),
    )
    .unwrap();
    let capture = lane
        .capture(vec![BoundedStreamIoReader::new(
            BoundedStreamIoStreamId::stdout(),
            InterruptedOnceReader {
                interrupted: false,
                bytes: Cursor::new(b"after-interrupt\n".to_vec()),
            },
        )])
        .unwrap();

    wait_for_terminal(&capture);
    let batch = capture.drain(BoundedStreamIoDrainBudget::unlimited());
    assert_eq!(batch.records[0].text, "after-interrupt");
    assert_eq!(batch.diagnostics.completed_readers, 1);
    assert_eq!(batch.diagnostics.failed_readers, 0);
}

#[test]
fn runtime_stream_owner_does_not_create_private_reader_threads() {
    let source = [
        include_str!("capture.rs"),
        include_str!("lane.rs"),
        include_str!("worker.rs"),
    ]
    .concat();
    let forbidden_spawn = ["thread", "::", "spawn"].concat();

    assert!(!source.contains(&forbidden_spawn));
    assert!(source.contains("TaskPoolKind::Io"));
    assert!(source.contains("TaskGraphScope"));
}

fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let started = Instant::now();
    while started.elapsed() < timeout {
        if condition() {
            return true;
        }
        std::thread::yield_now();
    }
    condition()
}

struct ControlledReader {
    receiver: Receiver<Vec<u8>>,
    pending: Cursor<Vec<u8>>,
}

fn controlled_reader() -> (ControlledReader, Sender<Vec<u8>>) {
    let (sender, receiver) = mpsc::channel();
    (
        ControlledReader {
            receiver,
            pending: Cursor::new(Vec::new()),
        },
        sender,
    )
}

impl Read for ControlledReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        loop {
            let read = self.pending.read(output)?;
            if read > 0 {
                return Ok(read);
            }
            match self.receiver.recv() {
                Ok(bytes) => self.pending = Cursor::new(bytes),
                Err(_) => return Ok(0),
            }
        }
    }
}

struct CountingReader {
    reads: Arc<Mutex<u32>>,
}

impl CountingReader {
    fn new(reads: Arc<Mutex<u32>>) -> Self {
        Self { reads }
    }
}

impl Read for CountingReader {
    fn read(&mut self, _output: &mut [u8]) -> io::Result<usize> {
        *self.reads.lock().unwrap() += 1;
        Ok(0)
    }
}

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _output: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("fixture read failure"))
    }
}

struct InterruptedOnceReader {
    interrupted: bool,
    bytes: Cursor<Vec<u8>>,
}

impl Read for InterruptedOnceReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if !self.interrupted {
            self.interrupted = true;
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        self.bytes.read(output)
    }
}
