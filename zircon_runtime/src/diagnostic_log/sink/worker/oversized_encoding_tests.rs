use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use chrono::NaiveDate;

use super::{
    encoded_pending_bytes, flush_oversized_pending, flush_pending_with_timestamp, DurableOutput,
    LogRecord, SinkMetrics, SinkOutputs, TimestampPrefix,
};
use crate::diagnostic_log::sink::append_diagnostic_log_line;
use crate::diagnostic_log::timestamp::write_log_timestamp_prefix_to;
use crate::diagnostic_log::{DiagnosticLogLevel, DiagnosticLogSinkSettings};

type Events = Arc<Mutex<Vec<(&'static str, char)>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Failure {
    None,
    Write,
    WriteZero,
    Flush,
}

#[derive(Default)]
struct OutputState {
    bytes: Vec<u8>,
    max_request: usize,
    flush_calls: usize,
    invalid_utf8_requests: usize,
    max_ascii_request: usize,
    partial_writes: usize,
}

#[derive(Clone)]
struct RecordingOutput {
    name: &'static str,
    state: Arc<Mutex<OutputState>>,
    events: Events,
    short_write: usize,
    failure: Failure,
    require_utf8: bool,
}

impl RecordingOutput {
    fn new(name: &'static str, events: &Events, short_write: usize, failure: Failure) -> Self {
        Self {
            name,
            state: Arc::new(Mutex::new(OutputState::default())),
            events: Arc::clone(events),
            short_write,
            failure,
            require_utf8: false,
        }
    }

    fn utf8_console(events: &Events, short_write: usize, failure: Failure) -> Self {
        let mut output = Self::new("console", events, short_write, failure);
        output.require_utf8 = true;
        output
    }

    fn invalid_utf8_requests(&self) -> usize {
        self.state.lock().unwrap().invalid_utf8_requests
    }

    fn max_ascii_request(&self) -> usize {
        self.state.lock().unwrap().max_ascii_request
    }

    fn partial_writes(&self) -> usize {
        self.state.lock().unwrap().partial_writes
    }

    fn bytes(&self) -> Vec<u8> {
        self.state.lock().unwrap().bytes.clone()
    }

    fn max_request(&self) -> usize {
        self.state.lock().unwrap().max_request
    }

    fn flush_calls(&self) -> usize {
        self.state.lock().unwrap().flush_calls
    }
}

impl Write for RecordingOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.events.lock().unwrap().push((self.name, 'w'));
        let mut state = self.state.lock().unwrap();
        state.max_request = state.max_request.max(bytes.len());
        if bytes.is_ascii() {
            state.max_ascii_request = state.max_ascii_request.max(bytes.len());
        }
        let utf8 = match std::str::from_utf8(bytes) {
            Ok(value) => Some(value),
            Err(_) => {
                state.invalid_utf8_requests += 1;
                if self.require_utf8 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "console rejects invalid UTF-8",
                    ));
                }
                None
            }
        };
        if state.bytes.len() >= 7 {
            match self.failure {
                Failure::Write => return Err(io::Error::other("injected partial write failure")),
                Failure::WriteZero => return Ok(0),
                Failure::None | Failure::Flush => {}
            }
        }
        let mut written = bytes.len().min(self.short_write);
        if self.require_utf8 {
            let text = utf8.expect("the console validated each request");
            while !text.is_char_boundary(written) {
                written -= 1;
            }
            if written == 0 && !text.is_empty() {
                written = text.chars().next().unwrap().len_utf8();
            }
        }
        state.partial_writes += usize::from(written < bytes.len());
        state.bytes.extend_from_slice(&bytes[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push((self.name, 'f'));
        self.state.lock().unwrap().flush_calls += 1;
        if self.failure == Failure::Flush {
            Err(io::Error::other("injected flush failure"))
        } else {
            Ok(())
        }
    }
}

impl DurableOutput for RecordingOutput {
    fn sync_data(&mut self) -> io::Result<()> {
        self.events.lock().unwrap().push((self.name, 's'));
        Ok(())
    }
}

fn prefix_for_year(year: i32) -> TimestampPrefix {
    let timestamp = NaiveDate::from_ymd_opt(year, 2, 1)
        .unwrap()
        .and_hms_opt(3, 4, 5)
        .unwrap();
    let mut prefix = TimestampPrefix::default();
    write_log_timestamp_prefix_to(&mut prefix, timestamp).unwrap();
    let expected = format!("[{}] ", timestamp.format("%Y-%m-%d-%H-%M-%S"));
    assert_eq!(prefix.as_bytes(), expected.as_bytes());
    assert!(prefix.as_bytes().len() <= 29);
    prefix
}

fn expected_bytes(pending: &[LogRecord], prefix: &[u8]) -> Vec<u8> {
    let mut expected = Vec::new();
    for record in pending {
        expected.extend_from_slice(prefix);
        append_diagnostic_log_line(
            &mut expected,
            record.level,
            record.scope(),
            record.message(),
        );
    }
    expected
}

fn assert_console_before_file(events: &Events) {
    let events = events.lock().unwrap();
    let first_file = events
        .iter()
        .position(|(name, _)| *name == "file")
        .expect("the file sink is attempted");
    assert!(first_file != 0);
    assert!(events[..first_file]
        .iter()
        .all(|(name, _)| *name == "console"));
    assert!(events[first_file..].iter().all(|(name, _)| *name == "file"));
}

#[test]
fn oversized_first_record_preserves_full_utf8_scope_message_and_fifo() {
    let events = Events::default();
    let captured = RecordingOutput::new("file", &events, 257, Failure::None);
    let mut outputs = SinkOutputs {
        file: Some(Box::new(captured.clone())),
        console_enabled: false,
        failed: false,
    };
    let scope = "pane] scope\n界🙂".repeat(128);
    let message = format!("{}\nλ🙂 literal\\n\r", "x".repeat(1024 * 1024));
    let mut pending = vec![
        LogRecord::new(DiagnosticLogLevel::Log, &scope, &message),
        LogRecord::new(DiagnosticLogLevel::Warn, "second", "λ\n🙂"),
        LogRecord::new(DiagnosticLogLevel::Error, "third", "last\\n"),
    ];
    let prefix = prefix_for_year(2026);
    let expected = expected_bytes(&pending, prefix.as_bytes());
    let metrics = SinkMetrics::new();
    let mut buffer = Vec::with_capacity(64);
    let limit = DiagnosticLogSinkSettings::default().max_batch_bytes;
    let mut timestamp_calls = 0;
    flush_pending_with_timestamp(
        &mut outputs,
        &mut pending,
        &metrics,
        &mut buffer,
        limit,
        || {
            timestamp_calls += 1;
            prefix
        },
    );
    assert_eq!(timestamp_calls, 1);
    assert_eq!(captured.bytes(), expected);
    assert_eq!(captured.max_request(), limit);
    assert_eq!(captured.flush_calls(), 1);
    assert!(pending.is_empty());
    assert_eq!(buffer.capacity(), 0);
    assert!(!outputs.failed);
    let snapshot = metrics.snapshot(0);
    assert_eq!(snapshot.flush_batches, 1);
    assert_eq!(snapshot.written_records, 3);
    assert_eq!(snapshot.written_bytes, expected.len() as u64);
    assert_eq!(snapshot.output_errors, 0);
}

#[test]
fn oversized_mirrors_preserve_console_then_file_single_prefix_and_short_writes() {
    let rejected_events = Events::default();
    let mut rejecting_console = RecordingOutput::utf8_console(&rejected_events, 2, Failure::None);
    assert_eq!(
        rejecting_console.write(&[0xf0]).unwrap_err().kind(),
        io::ErrorKind::InvalidData,
    );
    assert_eq!(rejecting_console.invalid_utf8_requests(), 1);
    assert!(rejecting_console.bytes().is_empty());

    for limit in [1usize, 2, 3, 4, 5, 7, 17] {
        for console_short_write in [1, 2, 3] {
            let events = Events::default();
            let mut console =
                RecordingOutput::utf8_console(&events, console_short_write, Failure::None);
            let mut file = RecordingOutput::new("file", &events, 3, Failure::None);
            let prefix = prefix_for_year(-10_000);
            let mut pending = vec![
                LogRecord::new(
                    DiagnosticLogLevel::Log,
                    "界\nscope🙂λ",
                    &"λ界🙂\n".repeat(50),
                ),
                LogRecord::new(DiagnosticLogLevel::Error, "tail]", "\nlast\n"),
            ];
            let expected = expected_bytes(&pending, prefix.as_bytes());
            let encoded_bytes = encoded_pending_bytes(&pending, prefix.as_bytes()).unwrap();
            assert_eq!(encoded_bytes, expected.len());
            let metrics = SinkMetrics::new();
            let mut buffer = Vec::with_capacity(4096);
            assert!(!flush_oversized_pending(
                Some(&mut console),
                Some(&mut file),
                &mut pending,
                (prefix.as_bytes(), encoded_bytes),
                limit,
                &metrics,
                &mut buffer,
            ));
            assert_console_before_file(&events);
            assert_eq!(console.bytes(), expected);
            assert_eq!(file.bytes(), expected);
            assert_eq!(console.invalid_utf8_requests(), 0);
            assert!(console.max_request() <= limit.max(4));
            assert!(console.max_ascii_request() <= limit);
            assert!(file.max_request() <= limit);
            if limit < 4 {
                assert_eq!(console.max_request(), 4);
                assert!(file.invalid_utf8_requests() != 0);
            }
            if limit > 3 {
                assert!(console.partial_writes() != 0);
                assert!(file.partial_writes() != 0);
            }
            assert_eq!(console.flush_calls(), 1);
            assert_eq!(file.flush_calls(), 1);
            assert!(pending.is_empty());
            assert_eq!(buffer.capacity(), 0);
            let snapshot = metrics.snapshot(0);
            assert_eq!(snapshot.flush_batches, 1);
            assert_eq!(snapshot.written_records, 2);
            assert_eq!(snapshot.written_bytes, expected.len() as u64);
            assert_eq!(snapshot.output_errors, 0);
        }
    }
}

#[test]
fn oversized_partial_write_zero_and_flush_failures_attempt_next_sink_and_release_storage() {
    for (console_failure, file_failure) in [
        (Failure::Write, Failure::None),
        (Failure::None, Failure::Write),
        (Failure::WriteZero, Failure::None),
        (Failure::None, Failure::WriteZero),
        (Failure::Flush, Failure::None),
        (Failure::None, Failure::Flush),
        (Failure::Write, Failure::Flush),
        (Failure::Flush, Failure::WriteZero),
    ] {
        let events = Events::default();
        let mut console = RecordingOutput::utf8_console(&events, 3, console_failure);
        let mut file = RecordingOutput::new("file", &events, 2, file_failure);
        let prefix = prefix_for_year(10_000);
        let mut pending = vec![
            LogRecord::new(DiagnosticLogLevel::Warn, "scope", &"λ\n🙂".repeat(40)),
            LogRecord::new(DiagnosticLogLevel::Error, "tail", "last"),
        ];
        let expected = expected_bytes(&pending, prefix.as_bytes());
        let encoded_bytes = encoded_pending_bytes(&pending, prefix.as_bytes()).unwrap();
        let metrics = SinkMetrics::new();
        let mut buffer = Vec::with_capacity(4096);
        let limit = 13;
        assert!(flush_oversized_pending(
            Some(&mut console),
            Some(&mut file),
            &mut pending,
            (prefix.as_bytes(), encoded_bytes),
            limit,
            &metrics,
            &mut buffer,
        ));
        assert_console_before_file(&events);
        assert_eq!(console.invalid_utf8_requests(), 0);
        assert!(console.max_ascii_request() <= limit);
        for (output, failure) in [(&console, console_failure), (&file, file_failure)] {
            let bytes = output.bytes();
            assert!(output.max_request() <= limit);
            assert_eq!(bytes.as_slice(), &expected[..bytes.len()]);
            if matches!(failure, Failure::None | Failure::Flush) {
                assert_eq!(bytes, expected);
                assert_eq!(output.flush_calls(), 1);
            } else {
                assert!(bytes.len() < expected.len());
                assert_eq!(output.flush_calls(), 0);
            }
        }
        let snapshot = metrics.snapshot(0);
        let failures = usize::from(console_failure != Failure::None)
            + usize::from(file_failure != Failure::None);
        assert_eq!(snapshot.output_errors, failures as u64);
        assert_eq!(snapshot.written_records, 0);
        assert_eq!(snapshot.written_bytes, 0);
        assert_eq!(snapshot.flush_batches, 1);
        assert!(pending.is_empty());
        assert_eq!(buffer.capacity(), 0);
    }

    for failure in [Failure::Write, Failure::WriteZero, Failure::Flush] {
        let events = Events::default();
        let captured = RecordingOutput::new("file", &events, 2, failure);
        let mut outputs = SinkOutputs {
            file: Some(Box::new(captured)),
            console_enabled: false,
            failed: false,
        };
        let mut pending = vec![LogRecord::new(
            DiagnosticLogLevel::Error,
            "scope",
            &"x".repeat(128),
        )];
        let metrics = SinkMetrics::new();
        let mut buffer = Vec::with_capacity(4096);
        flush_pending_with_timestamp(
            &mut outputs,
            &mut pending,
            &metrics,
            &mut buffer,
            13,
            || prefix_for_year(-1),
        );
        assert!(outputs.failed);
        assert!(pending.is_empty());
        assert_eq!(buffer.capacity(), 0);
        assert_eq!(metrics.snapshot(0).output_errors, 1);
        assert_eq!(metrics.snapshot(0).written_records, 0);
    }
}

#[test]
fn tiny_and_exact_byte_limits_reuse_one_prefix_and_preserve_variable_years() {
    for year in [-10_000, -1, 0, 9_999, 10_000] {
        let prefix = prefix_for_year(year);
        let record = LogRecord::new(DiagnosticLogLevel::Error, "界\n] scope", "λ\n🙂\n");
        let expected = expected_bytes(&[record], prefix.as_bytes());
        for limit in [
            1,
            prefix.as_bytes().len() - 1,
            expected.len() - 1,
            expected.len(),
            expected.len() + 1,
        ] {
            let events = Events::default();
            let captured = RecordingOutput::new("file", &events, usize::MAX, Failure::None);
            let mut outputs = SinkOutputs {
                file: Some(Box::new(captured.clone())),
                console_enabled: false,
                failed: false,
            };
            let mut pending = vec![LogRecord::new(
                DiagnosticLogLevel::Error,
                "界\n] scope",
                "λ\n🙂\n",
            )];
            assert_eq!(
                encoded_pending_bytes(&pending, prefix.as_bytes()).unwrap(),
                expected.len(),
            );
            let metrics = SinkMetrics::new();
            let mut buffer = Vec::with_capacity(8);
            let mut timestamp_calls = 0;
            flush_pending_with_timestamp(
                &mut outputs,
                &mut pending,
                &metrics,
                &mut buffer,
                limit,
                || {
                    timestamp_calls += 1;
                    prefix_for_year(year)
                },
            );
            assert_eq!(timestamp_calls, 1);
            assert_eq!(captured.bytes(), expected);
            assert!(captured.max_request() <= limit);
            assert!(pending.is_empty());
            assert!(!outputs.failed);
            assert!(buffer.is_empty());
            if expected.len() > limit {
                assert_eq!(buffer.capacity(), 0);
            } else {
                assert!(buffer.capacity() >= expected.len() && buffer.capacity() <= limit);
                assert_eq!(captured.max_request(), expected.len());
            }
            let snapshot = metrics.snapshot(0);
            assert_eq!(snapshot.written_records, 1);
            assert_eq!(snapshot.written_bytes, expected.len() as u64);
            assert_eq!(snapshot.flush_batches, 1);
            assert_eq!(snapshot.output_errors, 0);
        }
    }
}

#[test]
fn oversized_console_guard_releases_before_file_delivery_on_success_and_failure() {
    use std::sync::atomic::{AtomicBool, Ordering};

    struct GuardedOutput {
        output: RecordingOutput,
        console_guard: bool,
        console_released: Arc<AtomicBool>,
    }

    impl Write for GuardedOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if !self.console_guard {
                assert!(self.console_released.load(Ordering::Acquire));
            }
            self.output.write(bytes)
        }

        fn flush(&mut self) -> io::Result<()> {
            if !self.console_guard {
                assert!(self.console_released.load(Ordering::Acquire));
            }
            self.output.flush()
        }
    }

    impl Drop for GuardedOutput {
        fn drop(&mut self) {
            if self.console_guard {
                self.console_released.store(true, Ordering::Release);
                self.output.events.lock().unwrap().push(("console", 'd'));
            }
        }
    }

    for failure in [
        Failure::None,
        Failure::Write,
        Failure::WriteZero,
        Failure::Flush,
    ] {
        let events = Events::default();
        let console = RecordingOutput::utf8_console(&events, 2, failure);
        let file = RecordingOutput::new("file", &events, 3, Failure::None);
        let console_released = Arc::new(AtomicBool::new(false));
        let console_guard = GuardedOutput {
            output: console.clone(),
            console_guard: true,
            console_released: Arc::clone(&console_released),
        };
        let mut file_after_console = GuardedOutput {
            output: file.clone(),
            console_guard: false,
            console_released: Arc::clone(&console_released),
        };
        let prefix = prefix_for_year(2026);
        let mut pending = vec![
            LogRecord::new(DiagnosticLogLevel::Log, "界scope🙂", &"λ🙂\n".repeat(40)),
            LogRecord::new(DiagnosticLogLevel::Error, "tail", "last"),
        ];
        let expected = expected_bytes(&pending, prefix.as_bytes());
        let encoded_bytes = encoded_pending_bytes(&pending, prefix.as_bytes()).unwrap();
        let metrics = SinkMetrics::new();
        let mut buffer = Vec::with_capacity(4096);
        let failed = flush_oversized_pending(
            Some(console_guard),
            Some(&mut file_after_console),
            &mut pending,
            (prefix.as_bytes(), encoded_bytes),
            3,
            &metrics,
            &mut buffer,
        );
        assert_eq!(failed, failure != Failure::None);
        assert!(console_released.load(Ordering::Acquire));
        assert_console_before_file(&events);
        {
            let events = events.lock().unwrap();
            assert_eq!(
                events
                    .iter()
                    .filter(|&&event| event == ("console", 'd'))
                    .count(),
                1
            );
            let released = events
                .iter()
                .position(|&event| event == ("console", 'd'))
                .unwrap();
            let first_file = events.iter().position(|&(name, _)| name == "file").unwrap();
            assert!(released < first_file);
        }
        assert_eq!(file.bytes(), expected);
        assert_eq!(file.flush_calls(), 1);
        assert_eq!(console.invalid_utf8_requests(), 0);
        assert!(console.max_request() <= 4);
        assert!(file.max_request() <= 3);
        if matches!(failure, Failure::None | Failure::Flush) {
            assert_eq!(console.bytes(), expected);
        } else {
            let bytes = console.bytes();
            assert!(bytes.len() < expected.len());
            assert_eq!(bytes.as_slice(), &expected[..bytes.len()]);
        }
        assert!(pending.is_empty());
        assert_eq!(buffer.capacity(), 0);
        let snapshot = metrics.snapshot(0);
        assert_eq!(snapshot.output_errors, u64::from(failure != Failure::None));
        assert_eq!(
            snapshot.written_records,
            if failure == Failure::None { 2 } else { 0 }
        );
        assert_eq!(
            snapshot.written_bytes,
            if failure == Failure::None {
                expected.len() as u64
            } else {
                0
            }
        );
        assert_eq!(snapshot.flush_batches, 1);
    }
}
