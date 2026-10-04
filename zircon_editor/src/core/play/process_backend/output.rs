use std::io::Read;
use std::mem;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError, TrySendError};
use zircon_runtime::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

const PLAY_OUTPUT_QUEUE_CAPACITY: usize = 1_024;
const PLAY_OUTPUT_QUEUE_BYTE_CAPACITY: usize = 4 * 1024 * 1024;
const PLAY_OUTPUT_MAX_LINE_BYTES: usize = 64 * 1024;
const PLAY_OUTPUT_DRAIN_LIMIT: usize = 64;
const PLAY_OUTPUT_DRAIN_BYTE_LIMIT: usize = 256 * 1024;
const PLAY_OUTPUT_DRAIN_TIME_BUDGET: Duration = Duration::from_millis(2);
const PLAY_OUTPUT_READ_CHUNK_BYTES: usize = 8 * 1024;
const PLAY_OUTPUT_BUDGET_DIAGNOSTIC_COUNT: usize = 5;
const PLAY_OUTPUT_READER_COUNT: usize = 2;

static PLAY_OUTPUT_TASK_POOL: OnceLock<Result<TaskPool, String>> = OnceLock::new();

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

    fn push(&mut self, input: &[u8], mut emit: impl FnMut(DecodedOutputLine) -> bool) -> bool {
        for byte in input {
            if *byte == b'\n' {
                if !emit(self.finish_line()) {
                    return false;
                }
            } else if self.bytes.len() < self.max_bytes {
                self.bytes.push(*byte);
            } else {
                self.truncated_bytes = self.truncated_bytes.saturating_add(1);
            }
        }
        true
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
    reader_completion: Arc<ReaderCompletion>,
    deferred: Mutex<Option<PlayOutputLine>>,
    queue_bytes: Arc<OutputByteBudget>,
    counters: Arc<PlayOutputCounters>,
}

pub(super) struct PlayOutputCaptureError {
    message: String,
}

impl std::fmt::Debug for PlayOutputCaptureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PlayOutputCaptureError")
            .field("message", &self.message)
            .field("reader_count", &0)
            .finish()
    }
}

impl PlayOutputCaptureError {
    pub(super) fn message(&self) -> &str {
        &self.message
    }

    pub(super) fn finish(self) -> String {
        self.message
    }
}

impl PlayOutputPump {
    pub(super) fn capture(
        stdout: impl Read + Send + 'static,
        stderr: impl Read + Send + 'static,
    ) -> Result<Self, PlayOutputCaptureError> {
        let task_pool = play_output_task_pool()
            .map_err(|error| PlayOutputCaptureError::without_readers(error.to_string()))?;
        let (sender, receiver) = bounded(PLAY_OUTPUT_QUEUE_CAPACITY);
        let queue_bytes = Arc::new(OutputByteBudget::new(PLAY_OUTPUT_QUEUE_BYTE_CAPACITY));
        let counters = Arc::new(PlayOutputCounters::default());
        let reader_completion = ReaderCompletion::new(PLAY_OUTPUT_READER_COUNT);
        spawn_reader(
            &task_pool,
            stdout,
            PlayOutputStream::Stdout,
            sender.clone(),
            Arc::clone(&queue_bytes),
            Arc::clone(&counters),
            Arc::clone(&reader_completion),
        );
        spawn_reader(
            &task_pool,
            stderr,
            PlayOutputStream::Stderr,
            sender,
            Arc::clone(&queue_bytes),
            Arc::clone(&counters),
            Arc::clone(&reader_completion),
        );
        Ok(Self {
            receiver,
            reader_completion,
            deferred: Mutex::new(None),
            queue_bytes,
            counters,
        })
    }

    pub(super) fn drain(&self) -> Vec<String> {
        self.drain_limited()
    }

    pub(super) fn finish(self) -> Vec<String> {
        self.reader_completion.wait();
        self.drain_all()
    }

    fn drain_limited(&self) -> Vec<String> {
        let deadline = Instant::now() + PLAY_OUTPUT_DRAIN_TIME_BUDGET;
        let mut diagnostics =
            Vec::with_capacity(PLAY_OUTPUT_DRAIN_LIMIT + PLAY_OUTPUT_BUDGET_DIAGNOSTIC_COUNT);
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
        let deferred = self
            .deferred
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut diagnostics =
            Vec::with_capacity(drain_all_capacity(self.receiver.len(), deferred.is_some()));
        let mut oldest_age_ms = 0;
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

fn drain_all_capacity(queued_lines: usize, has_deferred_line: bool) -> usize {
    let line_count = queued_lines.saturating_add(usize::from(has_deferred_line));
    if line_count == 0 {
        return 0;
    }
    line_count.saturating_add(PLAY_OUTPUT_BUDGET_DIAGNOSTIC_COUNT)
}

impl PlayOutputCaptureError {
    fn without_readers(message: String) -> Self {
        Self { message }
    }
}

fn spawn_reader(
    task_pool: &TaskPool,
    reader: impl Read + Send + 'static,
    stream: PlayOutputStream,
    sender: Sender<PlayOutputLine>,
    queue_bytes: Arc<OutputByteBudget>,
    counters: Arc<PlayOutputCounters>,
    completion: Arc<ReaderCompletion>,
) {
    task_pool.spawn(move || {
        let _completion = ReaderCompletionGuard::new(completion);
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
                    if !decoder.push(&buffer[..read], |line| {
                        enqueue_line(&sender, stream, line, &queue_bytes, &counters)
                    }) {
                        return;
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
    });
}

fn play_output_task_pool() -> Result<TaskPool, String> {
    PLAY_OUTPUT_TASK_POOL
        .get_or_init(|| {
            TaskPool::try_new(
                TaskPoolDescriptor::io()
                    .with_worker_threads(PLAY_OUTPUT_READER_COUNT)
                    .with_thread_name("zircon-play-output-task"),
            )
            .map_err(|error| error.to_string())
        })
        .clone()
}

struct ReaderCompletion {
    remaining: Mutex<usize>,
    ready: Condvar,
}

impl ReaderCompletion {
    fn new(reader_count: usize) -> Arc<Self> {
        Arc::new(Self {
            remaining: Mutex::new(reader_count),
            ready: Condvar::new(),
        })
    }

    fn complete(&self) {
        let mut remaining = self
            .remaining
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *remaining = remaining.saturating_sub(1);
        if *remaining == 0 {
            self.ready.notify_all();
        }
    }

    fn wait(&self) {
        let remaining = self
            .remaining
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _remaining = self
            .ready
            .wait_while(remaining, |remaining| *remaining != 0)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
}

struct ReaderCompletionGuard {
    completion: Arc<ReaderCompletion>,
}

impl ReaderCompletionGuard {
    fn new(completion: Arc<ReaderCompletion>) -> Self {
        Self { completion }
    }
}

impl Drop for ReaderCompletionGuard {
    fn drop(&mut self) {
        self.completion.complete();
    }
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
#[path = "tests/output_performance_source_guards.rs"]
mod performance_source_guards;

#[cfg(test)]
#[path = "output/tests/drain_all_capacity_tests.rs"]
mod drain_all_capacity_tests;
