use crate::ui::host::export_process_support::CapturedOutputChunk;

use super::super::super::output_tail::MAX_OUTPUT_TAIL_LINES;
use super::{
    ArtifactDestination, ExportWizardOutputCapture, IncrementalLineBuffer, OutputCapturePaths,
    MAX_CAPTURED_LINE_BYTES,
};

#[test]
fn optimization_batch_20260830er_incremental_lines_stream_without_a_temporary_vec() {
    let source = include_str!("../output_capture.rs");

    assert!(source.contains(concat!("for_each_line", "(bytes, finalize")));
    assert!(!source.contains(concat!(
        "for line in self.line_buffer",
        ".push(bytes, finalize)"
    )));
}

#[test]
fn optimization_batch_20260830er_incremental_lines_preserve_chunk_order() {
    let mut buffer = IncrementalLineBuffer::default();
    let mut lines = Vec::new();

    buffer.for_each_line(b"alpha\nbeta".to_vec(), false, &mut |line| lines.push(line));
    buffer.for_each_line(b"-tail\ngamma".to_vec(), true, &mut |line| lines.push(line));

    assert_eq!(lines, ["alpha", "beta-tail", "gamma"]);
}

#[test]
#[ignore = "deterministic performance evidence"]
fn optimization_batch_20260830er_incremental_line_callback_benchmark_evidence() {
    const CHUNK_COUNT: usize = 1_000_000;
    const SAMPLE_COUNT: usize = 11;
    const MARKER: &str = "EDITOR551_CALLBACK_LINE_STREAM_BENCH_V1";
    const CHUNK: &[u8] = b"alpha\nbeta\ngamma\n";

    fn median(mut samples: Vec<std::time::Duration>) -> std::time::Duration {
        samples.sort_unstable();
        samples[samples.len() / 2]
    }

    let mut collected_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut callback_samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let mut buffer = IncrementalLineBuffer::default();
        let started = std::time::Instant::now();
        let mut line_bytes = 0_usize;
        for _ in 0..CHUNK_COUNT {
            for line in buffer.push(CHUNK.to_vec(), false) {
                line_bytes = std::hint::black_box(line_bytes.wrapping_add(line.len()));
            }
        }
        collected_samples.push(started.elapsed());
        std::hint::black_box(line_bytes);

        let mut buffer = IncrementalLineBuffer::default();
        let started = std::time::Instant::now();
        let mut line_bytes = 0_usize;
        for _ in 0..CHUNK_COUNT {
            buffer.for_each_line(CHUNK.to_vec(), false, &mut |line| {
                line_bytes = std::hint::black_box(line_bytes.wrapping_add(line.len()));
            });
        }
        callback_samples.push(started.elapsed());
        std::hint::black_box(line_bytes);
    }

    let collected = median(collected_samples);
    let callback = median(callback_samples);
    eprintln!("{MARKER} collected={collected:?} callback={callback:?}");
    assert!(
        callback < collected,
        "callback={callback:?}, collected={collected:?}"
    );
}

#[test]
fn full_output_is_written_while_only_tail_is_retained() {
    let fixture = OutputCaptureFixture::new();
    let stdout = fixture.root.join("stdout.log");
    let stderr = fixture.root.join("stderr.log");
    let manifest = fixture.root.join("output-log.json");
    let mut capture = ExportWizardOutputCapture::open_paths(OutputCapturePaths {
        stdout: destination(&stdout),
        stderr: destination(&stderr),
        manifest: destination(&manifest),
    })
    .unwrap();
    let source = (0..(MAX_OUTPUT_TAIL_LINES + 20))
        .map(|index| format!("line-{index}\n"))
        .collect::<String>()
        .into_bytes();
    let mut emitted = Vec::new();

    capture
        .record(
            CapturedOutputChunk {
                stdout: source.clone(),
                stderr: b"warning\n".to_vec(),
            },
            true,
            &mut |line| emitted.push(line),
        )
        .unwrap();
    let result = capture.finish().unwrap();

    assert_eq!(std::fs::read(&stdout).unwrap(), source);
    assert_eq!(result.stdout_lines.len(), MAX_OUTPUT_TAIL_LINES);
    assert_eq!(
        result.stdout_lines.last().map(String::as_str),
        Some("line-531")
    );
    let manifest = std::fs::read_to_string(manifest).unwrap();
    assert!(manifest.contains("\"byte_count\""));
    assert!(manifest.contains("\"digest\""));
    assert_eq!(result.artifact_lines.len(), 3);
    assert_eq!(emitted.len(), MAX_OUTPUT_TAIL_LINES + 21);
}

#[test]
fn maximum_length_line_split_across_chunks_does_not_emit_an_empty_line() {
    let mut buffer = IncrementalLineBuffer::default();

    assert!(buffer
        .push(vec![b'a'; MAX_CAPTURED_LINE_BYTES], false)
        .is_empty());
    let lines = buffer.push(b"\nnext\n".to_vec(), true);

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].len(), MAX_CAPTURED_LINE_BYTES);
    assert_eq!(lines[1], "next");
}

#[test]
fn single_byte_chunks_preserve_line_boundaries_with_resumable_scanning() {
    let mut buffer = IncrementalLineBuffer::default();
    for _ in 0..MAX_CAPTURED_LINE_BYTES {
        assert!(buffer.push(vec![b'a'], false).is_empty());
    }

    let lines = buffer.push(b"\nnext\n".to_vec(), true);

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].len(), MAX_CAPTURED_LINE_BYTES);
    assert_eq!(lines[1], "next");
}

fn destination(path: &Path) -> ArtifactDestination {
    ArtifactDestination {
        locator: path.display().to_string(),
        io_path: path.to_path_buf(),
    }
}

use std::path::Path;

struct OutputCaptureFixture {
    root: std::path::PathBuf,
}

impl OutputCaptureFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-export-output-{}-{:x}",
            std::process::id(),
            fixture_nonce()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }
}

impl Drop for OutputCaptureFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fixture_nonce() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}
