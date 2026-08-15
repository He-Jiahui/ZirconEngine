use std::io::Read;
use std::mem;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError, TrySendError};

const PLAY_OUTPUT_QUEUE_CAPACITY: usize = 1_024;
const PLAY_OUTPUT_QUEUE_BYTE_CAPACITY: usize = 4 * 1024 * 1024;
const PLAY_OUTPUT_MAX_LINE_BYTES: usize = 64 * 1024;
const PLAY_OUTPUT_DRAIN_LIMIT: usize = 64;
const PLAY_OUTPUT_DRAIN_BYTE_LIMIT: usize = 256 * 1024;
const PLAY_OUTPUT_DRAIN_TIME_BUDGET: Duration = Duration::from_millis(2);
const PLAY_OUTPUT_READ_CHUNK_BYTES: usize = 8 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayOutputStream {
    Stdout,
    Stderr,
}

impl PlayOutputStream {
    const fn label(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

#[derive(Debug)]
struct PlayOutputLine {
    stream: PlayOutputStream,
    text: String,
    truncated_bytes: u64,
    queued_bytes: usize,
    captured_at: Instant,
}

#[derive(Default)]
struct PlayOutputCounters {
    dropped_lines: AtomicU64,
    dropped_bytes: AtomicU64,
    truncated_lines: AtomicU64,
    truncated_bytes: AtomicU64,
}

struct OutputByteBudget {
    limit: usize,
    used: AtomicUsize,
}

impl OutputByteBudget {
    const fn new(limit: usize) -> Self {
        Self {
            limit,
            used: AtomicUsize::new(0),
        }
    }

    fn try_reserve(&self, bytes: usize) -> bool {
        let mut used = self.used.load(Ordering::Acquire);
        loop {
            let Some(next) = used.checked_add(bytes) else {
                return false;
            };
            if next > self.limit {
                return false;
            }
            match self
                .used
                .compare_exchange_weak(used, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return true,
                Err(current) => used = current,
            }
        }
    }

    fn release(&self, bytes: usize) {
        let previous = self.used.fetch_sub(bytes, Ordering::AcqRel);
        debug_assert!(previous >= bytes, "output byte budget underflow");
    }

    fn used(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }
}

struct BoundedLineDecoder {
    bytes: Vec<u8>,
    max_bytes: usize,
    truncated_bytes: u64,
}

struct DecodedOutputLine {
    text: String,
    truncated_bytes: u64,
}

impl BoundedLineDecoder {
    fn new(max_bytes: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(max_bytes),
            max_bytes,
            truncated_bytes: 0,
        }
    }

    fn push(&mut self, input: &[u8]) -> Vec<DecodedOutputLine> {
        let mut lines = Vec::new();
        for byte in input {
            if *byte == b'\n' {
                lines.push(self.finish_line());
            } else if self.bytes.len() < self.max_bytes {
                self.bytes.push(*byte);
            } else {
                self.truncated_bytes = self.truncated_bytes.saturating_add(1);
            }
        }
        lines
    }

    fn finish(&mut self) -> Option<DecodedOutputLine> {
        (!self.bytes.is_empty() || self.truncated_bytes > 0).then(|| self.finish_line())
    }

    fn finish_line(&mut self) -> DecodedOutputLine {
        if self.bytes.last() == Some(&b'\r') {
            self.bytes.pop();
        }
        let mut text = String::from_utf8_lossy(&self.bytes).into_owned();
        let rendered_truncation = truncate_to_byte_limit(&mut text, self.max_bytes);
        text.shrink_to_fit();
        let line = DecodedOutputLine {
            text,
            truncated_bytes: self
                .truncated_bytes
                .saturating_add(rendered_truncation as u64),
        };
        self.bytes.clear();
        self.truncated_bytes = 0;
        line
    }
}

fn truncate_to_byte_limit(value: &mut String, byte_limit: usize) -> usize {
    if value.len() <= byte_limit {
        return 0;
    }
    let original_len = value.len();
    let mut end = byte_limit;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value.truncate(end);
    original_len - end
}

pub(super) struct PlayOutputPump {
    receiver: Receiver<PlayOutputLine>,
    readers: Vec<JoinHandle<()>>,
    deferred: Mutex<Option<PlayOutputLine>>,
    queue_bytes: Arc<OutputByteBudget>,
    counters: Arc<PlayOutputCounters>,
}

pub(super) struct PlayOutputCaptureError {
    message: String,
    readers: Vec<JoinHandle<()>>,
}

impl std::fmt::Debug for PlayOutputCaptureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PlayOutputCaptureError")
            .field("message", &self.message)
            .field("reader_count", &self.readers.len())
            .finish()
    }
}

impl PlayOutputCaptureError {
    pub(super) fn message(&self) -> &str {
        &self.message
    }

    pub(super) fn finish(self) -> String {
        for reader in self.readers {
            let _ = reader.join();
        }
        self.message
    }
}

impl PlayOutputPump {
    pub(super) fn capture(
        stdout: impl Read + Send + 'static,
        stderr: impl Read + Send + 'static,
    ) -> Result<Self, PlayOutputCaptureError> {
        let (sender, receiver) = bounded(PLAY_OUTPUT_QUEUE_CAPACITY);
        let queue_bytes = Arc::new(OutputByteBudget::new(PLAY_OUTPUT_QUEUE_BYTE_CAPACITY));
        let counters = Arc::new(PlayOutputCounters::default());
        let stdout_reader = spawn_reader(
            stdout,
            PlayOutputStream::Stdout,
            sender.clone(),
            Arc::clone(&queue_bytes),
            Arc::clone(&counters),
        )
        .map_err(PlayOutputCaptureError::without_readers)?;
        let stderr_reader = match spawn_reader(
            stderr,
            PlayOutputStream::Stderr,
            sender,
            Arc::clone(&queue_bytes),
            Arc::clone(&counters),
        ) {
            Ok(reader) => reader,
            Err(error) => {
                return Err(PlayOutputCaptureError {
                    message: error,
                    readers: vec![stdout_reader],
                });
            }
        };
        Ok(Self {
            receiver,
            readers: vec![stdout_reader, stderr_reader],
            deferred: Mutex::new(None),
            queue_bytes,
            counters,
        })
    }

    pub(super) fn drain(&self) -> Vec<String> {
        self.drain_limited()
    }

    pub(super) fn finish(mut self) -> Vec<String> {
        let readers = mem::take(&mut self.readers);
        for reader in readers {
            let _ = reader.join();
        }
        self.drain_all()
    }

    fn drain_limited(&self) -> Vec<String> {
        let deadline = Instant::now() + PLAY_OUTPUT_DRAIN_TIME_BUDGET;
        let mut diagnostics = Vec::with_capacity(PLAY_OUTPUT_DRAIN_LIMIT + 4);
        let mut drained_bytes = 0usize;
        let mut oldest_age_ms = 0;

        while diagnostics.len() < PLAY_OUTPUT_DRAIN_LIMIT && Instant::now() < deadline {
            let Some(line) = self.next_line() else {
                break;
            };
            let rendered_bytes = rendered_line_bytes(&line);
            if !diagnostics.is_empty()
                && drained_bytes.saturating_add(rendered_bytes) > PLAY_OUTPUT_DRAIN_BYTE_LIMIT
            {
                self.defer(line);
                break;
            }
            drained_bytes = drained_bytes.saturating_add(rendered_bytes);
            oldest_age_ms = oldest_age_ms.max(elapsed_millis(line.captured_at));
            self.queue_bytes.release(line.queued_bytes);
            diagnostics.push(format_line(&line));
        }
        append_output_budget_diagnostics(&mut diagnostics, &self.counters, oldest_age_ms);
        diagnostics
    }

    fn drain_all(self) -> Vec<String> {
        let mut diagnostics = Vec::new();
        let mut oldest_age_ms = 0;
        let deferred = self
            .deferred
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(line) = deferred {
            oldest_age_ms = oldest_age_ms.max(elapsed_millis(line.captured_at));
            self.queue_bytes.release(line.queued_bytes);
            diagnostics.push(format_line(&line));
        }
        loop {
            match self.receiver.try_recv() {
                Ok(line) => {
                    oldest_age_ms = oldest_age_ms.max(elapsed_millis(line.captured_at));
                    self.queue_bytes.release(line.queued_bytes);
                    diagnostics.push(format_line(&line));
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        append_output_budget_diagnostics(&mut diagnostics, &self.counters, oldest_age_ms);
        diagnostics
    }

    fn next_line(&self) -> Option<PlayOutputLine> {
        let mut deferred = self
            .deferred
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if deferred.is_some() {
            return deferred.take();
        }
        drop(deferred);
        self.receiver.try_recv().ok()
    }

    fn defer(&self, line: PlayOutputLine) {
        let mut deferred = self
            .deferred
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        debug_assert!(
            deferred.is_none(),
            "only one line may wait for the next drain"
        );
        *deferred = Some(line);
    }
}

impl PlayOutputCaptureError {
    fn without_readers(message: String) -> Self {
        Self {
            message,
            readers: Vec::new(),
        }
    }
}

fn spawn_reader(
    reader: impl Read + Send + 'static,
    stream: PlayOutputStream,
    sender: Sender<PlayOutputLine>,
    queue_bytes: Arc<OutputByteBudget>,
    counters: Arc<PlayOutputCounters>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name(format!("zircon-play-{}", stream.label()))
        .spawn(move || {
            let mut reader = reader;
            let mut decoder = BoundedLineDecoder::new(PLAY_OUTPUT_MAX_LINE_BYTES);
            let mut buffer = [0_u8; PLAY_OUTPUT_READ_CHUNK_BYTES];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        if let Some(line) = decoder.finish() {
                            let _ = enqueue_line(&sender, stream, line, &queue_bytes, &counters);
                        }
                        break;
                    }
                    Ok(read) => {
                        for line in decoder.push(&buffer[..read]) {
                            if !enqueue_line(&sender, stream, line, &queue_bytes, &counters) {
                                return;
                            }
                        }
                    }
                    Err(error) => {
                        if let Some(line) = decoder.finish() {
                            if !enqueue_line(&sender, stream, line, &queue_bytes, &counters) {
                                return;
                            }
                        }
                        let _ = enqueue_line(
                            &sender,
                            stream,
                            DecodedOutputLine {
                                text: format!("output read failed: {error}"),
                                truncated_bytes: 0,
                            },
                            &queue_bytes,
                            &counters,
                        );
                        break;
                    }
                }
            }
        })
        .map_err(|error| format!("failed to spawn play {} reader: {error}", stream.label()))
}

fn enqueue_line(
    sender: &Sender<PlayOutputLine>,
    stream: PlayOutputStream,
    line: DecodedOutputLine,
    queue_bytes: &OutputByteBudget,
    counters: &PlayOutputCounters,
) -> bool {
    if line.truncated_bytes > 0 {
        counters.truncated_lines.fetch_add(1, Ordering::Relaxed);
        counters
            .truncated_bytes
            .fetch_add(line.truncated_bytes, Ordering::Relaxed);
    }
    let queued_bytes = line
        .text
        .capacity()
        .saturating_add(mem::size_of::<PlayOutputLine>());
    if !queue_bytes.try_reserve(queued_bytes) {
        record_dropped_line(counters, queued_bytes);
        return true;
    }
    let line = PlayOutputLine {
        stream,
        text: line.text,
        truncated_bytes: line.truncated_bytes,
        queued_bytes,
        captured_at: Instant::now(),
    };
    match sender.try_send(line) {
        Ok(()) => true,
        Err(TrySendError::Full(line)) => {
            queue_bytes.release(line.queued_bytes);
            record_dropped_line(counters, line.queued_bytes);
            true
        }
        Err(TrySendError::Disconnected(line)) => {
            queue_bytes.release(line.queued_bytes);
            false
        }
    }
}

fn record_dropped_line(counters: &PlayOutputCounters, bytes: usize) {
    counters.dropped_lines.fetch_add(1, Ordering::Relaxed);
    counters
        .dropped_bytes
        .fetch_add(bytes as u64, Ordering::Relaxed);
}

fn rendered_line_bytes(line: &PlayOutputLine) -> usize {
    let truncation_suffix = usize::from(line.truncated_bytes > 0) * 64;
    "process."
        .len()
        .saturating_add(line.stream.label().len())
        .saturating_add(2)
        .saturating_add(line.text.len())
        .saturating_add(truncation_suffix)
}

fn format_line(line: &PlayOutputLine) -> String {
    if line.truncated_bytes == 0 {
        format!("process.{}: {}", line.stream.label(), line.text)
    } else {
        format!(
            "process.{}: {} [truncated {} bytes]",
            line.stream.label(),
            line.text,
            line.truncated_bytes
        )
    }
}

fn elapsed_millis(captured_at: Instant) -> u64 {
    u64::try_from(captured_at.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn append_output_budget_diagnostics(
    diagnostics: &mut Vec<String>,
    counters: &PlayOutputCounters,
    oldest_age_ms: u64,
) {
    let dropped_lines = counters.dropped_lines.swap(0, Ordering::Relaxed);
    if dropped_lines > 0 {
        diagnostics.push(format!("process.output_dropped_lines={dropped_lines}"));
    }
    let dropped_bytes = counters.dropped_bytes.swap(0, Ordering::Relaxed);
    if dropped_bytes > 0 {
        diagnostics.push(format!("process.output_dropped_bytes={dropped_bytes}"));
    }
    let truncated_lines = counters.truncated_lines.swap(0, Ordering::Relaxed);
    if truncated_lines > 0 {
        diagnostics.push(format!("process.output_truncated_lines={truncated_lines}"));
    }
    let truncated_bytes = counters.truncated_bytes.swap(0, Ordering::Relaxed);
    if truncated_bytes > 0 {
        diagnostics.push(format!("process.output_truncated_bytes={truncated_bytes}"));
    }
    if oldest_age_ms > 0 {
        diagnostics.push(format!("process.output_oldest_age_ms={oldest_age_ms}"));
    }
}

#[cfg(test)]
mod performance_source_guards {
    use std::io::Cursor;
    use std::mem;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    use crossbeam_channel::bounded;

    use super::{
        BoundedLineDecoder, OutputByteBudget, PlayOutputCounters, PlayOutputLine, PlayOutputPump,
        PlayOutputStream, PLAY_OUTPUT_DRAIN_BYTE_LIMIT, PLAY_OUTPUT_MAX_LINE_BYTES,
        PLAY_OUTPUT_QUEUE_BYTE_CAPACITY,
    };

    fn pump_with_queued_lines(lines: &[String]) -> (PlayOutputPump, Arc<OutputByteBudget>) {
        let (sender, receiver) = bounded(lines.len());
        let queue_bytes = Arc::new(OutputByteBudget::new(PLAY_OUTPUT_QUEUE_BYTE_CAPACITY));
        for text in lines {
            let queued_text = text.clone();
            let queued_bytes = queued_text
                .capacity()
                .saturating_add(mem::size_of::<PlayOutputLine>());
            assert!(queue_bytes.try_reserve(queued_bytes));
            sender
                .send(PlayOutputLine {
                    stream: PlayOutputStream::Stdout,
                    text: queued_text,
                    truncated_bytes: 0,
                    queued_bytes,
                    captured_at: Instant::now(),
                })
                .expect("fixture queue should accept each line");
        }
        drop(sender);
        (
            PlayOutputPump {
                receiver,
                readers: Vec::new(),
                deferred: Mutex::new(None),
                queue_bytes: Arc::clone(&queue_bytes),
                counters: Arc::new(PlayOutputCounters::default()),
            },
            queue_bytes,
        )
    }

    #[test]
    fn bounded_decoder_truncates_an_unterminated_line_without_retaining_its_tail() {
        let mut decoder = BoundedLineDecoder::new(8);
        assert!(decoder.push(b"0123456789").is_empty());

        let line = decoder
            .finish()
            .expect("unterminated output must flush once");
        assert_eq!(line.text, "01234567");
        assert_eq!(line.truncated_bytes, 2);
    }

    #[test]
    fn output_byte_budget_rejects_overflow_and_releases_consumed_bytes() {
        let budget = OutputByteBudget::new(8);
        assert!(budget.try_reserve(5));
        assert!(!budget.try_reserve(4));
        assert_eq!(budget.used(), 5);

        budget.release(5);
        assert_eq!(budget.used(), 0);
        assert!(budget.try_reserve(8));
    }

    #[test]
    fn captured_long_line_reports_truncation_with_bounded_rendered_output() {
        let mut stdout = vec![b'x'; PLAY_OUTPUT_MAX_LINE_BYTES + 1];
        stdout.push(b'\n');
        let pump = PlayOutputPump::capture(Cursor::new(stdout), Cursor::new(Vec::<u8>::new()))
            .expect("fixture readers should start");

        let diagnostics = pump.finish();
        let line = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.starts_with("process.stdout: "))
            .expect("stdout line should be preserved");
        assert!(line.len() <= "process.stdout: ".len() + PLAY_OUTPUT_MAX_LINE_BYTES + 96);
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.starts_with("process.output_truncated_lines=1")));
    }

    #[test]
    fn live_drain_defers_a_line_that_exceeds_the_remaining_byte_budget() {
        let line = "x".repeat(PLAY_OUTPUT_MAX_LINE_BYTES);
        let (pump, queue_bytes) =
            pump_with_queued_lines(&[line.clone(), line.clone(), line.clone(), line.clone()]);

        let first_drain = pump.drain();
        let first_output = first_drain
            .iter()
            .filter(|diagnostic| diagnostic.starts_with("process.stdout: "))
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(first_output.len(), 3);
        assert!(first_output
            .iter()
            .all(|diagnostic| diagnostic.ends_with(line.as_str())));
        assert!(
            queue_bytes.used() > 0,
            "deferred line must keep its reservation"
        );

        let second_drain = pump.drain();
        let expected = format!("process.stdout: {line}");
        assert_eq!(
            second_drain
                .iter()
                .filter(|diagnostic| diagnostic.starts_with("process.stdout: "))
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec![expected.as_str()]
        );
        assert_eq!(queue_bytes.used(), 0);
        assert!(PLAY_OUTPUT_DRAIN_BYTE_LIMIT < 4 * ("process.stdout: ".len() + line.len()));
    }

    #[test]
    fn live_drain_enforces_the_line_budget_without_losing_the_next_line() {
        let lines = (0..65)
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>();
        let (pump, queue_bytes) = pump_with_queued_lines(&lines);

        let first_drain = pump.drain();
        let first_output = first_drain
            .iter()
            .filter(|diagnostic| diagnostic.starts_with("process.stdout: "))
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(first_output.len(), 64);
        assert_eq!(
            first_output.first().copied(),
            Some("process.stdout: line-0")
        );
        assert_eq!(
            first_output.last().copied(),
            Some("process.stdout: line-63")
        );
        assert!(queue_bytes.used() > 0);

        assert_eq!(
            pump.drain()
                .iter()
                .filter(|diagnostic| diagnostic.starts_with("process.stdout: "))
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["process.stdout: line-64"]
        );
        assert_eq!(queue_bytes.used(), 0);
    }

    #[test]
    fn live_output_drain_has_a_per_poll_line_budget() {
        let source = include_str!("output.rs");
        let legacy_line_reader = ["read", "_until"].concat();
        assert!(source.contains("PLAY_OUTPUT_DRAIN_LIMIT"));
        assert!(source.contains("PLAY_OUTPUT_DRAIN_BYTE_LIMIT"));
        assert!(source.contains("PLAY_OUTPUT_DRAIN_TIME_BUDGET"));
        assert!(!source.contains(&legacy_line_reader));
    }
}
