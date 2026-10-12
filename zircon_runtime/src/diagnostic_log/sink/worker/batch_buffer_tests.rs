use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use super::{flush_pending, sync_outputs, DurableOutput, LogRecord, SinkMetrics, SinkOutputs};
use crate::diagnostic_log::DiagnosticLogLevel;

#[derive(Clone, Default)]
struct CapturedOutput(Arc<Mutex<Vec<u8>>>);

impl CapturedOutput {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl Write for CapturedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl DurableOutput for CapturedOutput {
    fn sync_data(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct RejectedOutput;

impl Write for RejectedOutput {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other(
            "injected encoding buffer delivery failure",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl DurableOutput for RejectedOutput {
    fn sync_data(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn record(level: DiagnosticLogLevel, message: &str) -> LogRecord {
    LogRecord::new(level, "buffer", message)
}

fn without_timestamps(output: &CapturedOutput) -> Vec<String> {
    output
        .text()
        .lines()
        .map(|line| {
            line.split_once("] ")
                .expect("real timestamp header")
                .1
                .to_owned()
        })
        .collect()
}

#[test]
fn reused_encoding_buffer_preserves_independent_batches_and_delivery_metrics() {
    let captured = CapturedOutput::default();
    let mut outputs = SinkOutputs {
        file: Some(Box::new(captured.clone())),
        console_enabled: false,
        failed: false,
    };
    let metrics = SinkMetrics::new();
    let mut buffer = Vec::new();
    let mut pending = vec![
        record(DiagnosticLogLevel::Log, "first\n第二"),
        record(DiagnosticLogLevel::Warn, r"literal \n"),
    ];
    flush_pending(&mut outputs, &mut pending, &metrics, &mut buffer, 1024);
    assert!(pending.is_empty());
    assert!(buffer.is_empty());
    assert!(buffer.capacity() > 0 && buffer.capacity() <= 1024);
    let allocation = buffer.as_ptr();
    let capacity = buffer.capacity();
    pending.push(record(DiagnosticLogLevel::Error, "third"));
    flush_pending(&mut outputs, &mut pending, &metrics, &mut buffer, 1024);
    assert_eq!(buffer.as_ptr(), allocation);
    assert_eq!(buffer.capacity(), capacity);
    assert!(buffer.is_empty());
    assert_eq!(
        without_timestamps(&captured),
        [
            "[log] [buffer] first\\n第二",
            r"[warn] [buffer] literal \n",
            "[error] [buffer] third",
        ]
    );
    let written = captured.text();
    let snapshot = metrics.snapshot(0);
    assert_eq!(snapshot.flush_batches, 2);
    assert_eq!(snapshot.written_records, 3);
    assert_eq!(snapshot.written_bytes, written.len() as u64);
    assert_eq!(snapshot.output_errors, 0);
    flush_pending(&mut outputs, &mut pending, &metrics, &mut buffer, 1024);
    assert_eq!(metrics.snapshot(0), snapshot);
    assert_eq!(captured.text(), written);
    assert!(sync_outputs(&mut outputs, &metrics).is_ok());
}

#[test]
fn oversized_success_and_failed_delivery_release_encoding_storage_without_truncation() {
    let captured = CapturedOutput::default();
    let mut outputs = SinkOutputs {
        file: Some(Box::new(captured.clone())),
        console_enabled: false,
        failed: false,
    };
    let metrics = SinkMetrics::new();
    let mut buffer = Vec::new();
    let message = "λ".repeat(256);
    let mut pending = vec![record(DiagnosticLogLevel::Warn, &message)];
    flush_pending(&mut outputs, &mut pending, &metrics, &mut buffer, 128);
    assert!(pending.is_empty());
    assert_eq!(buffer.capacity(), 0);
    assert_eq!(
        without_timestamps(&captured),
        [format!("[warn] [buffer] {message}")]
    );
    assert_eq!(metrics.snapshot(0).written_records, 1);
    assert_eq!(
        metrics.snapshot(0).written_bytes,
        captured.text().len() as u64
    );
    pending.push(record(DiagnosticLogLevel::Log, "after oversize"));
    flush_pending(&mut outputs, &mut pending, &metrics, &mut buffer, 128);
    assert_eq!(without_timestamps(&captured).len(), 2);
    assert_eq!(metrics.snapshot(0).written_records, 2);
    assert!(buffer.capacity() <= 128);
    assert!(sync_outputs(&mut outputs, &metrics).is_ok());

    let mut failed = SinkOutputs {
        file: Some(Box::new(RejectedOutput)),
        console_enabled: false,
        failed: false,
    };
    let failed_metrics = SinkMetrics::new();
    pending.push(record(DiagnosticLogLevel::Error, &message));
    flush_pending(&mut failed, &mut pending, &failed_metrics, &mut buffer, 128);
    assert!(pending.is_empty());
    assert_eq!(buffer.capacity(), 0);
    let snapshot = failed_metrics.snapshot(0);
    assert_eq!(snapshot.flush_batches, 1);
    assert_eq!(snapshot.written_records, 0);
    assert_eq!(snapshot.written_bytes, 0);
    assert_eq!(snapshot.output_errors, 1);
    assert!(failed.failed);
    assert!(sync_outputs(&mut failed, &failed_metrics).is_err());
}
