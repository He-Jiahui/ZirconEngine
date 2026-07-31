use std::io::{BufRead, BufReader, Read};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

const PLAY_OUTPUT_QUEUE_CAPACITY: usize = 1_024;
const PLAY_OUTPUT_DRAIN_LIMIT: usize = 64;

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
}

pub(super) struct PlayOutputPump {
    receiver: Receiver<PlayOutputLine>,
    readers: Vec<JoinHandle<()>>,
    dropped_lines: Arc<AtomicU64>,
}

impl PlayOutputPump {
    pub(super) fn capture(
        stdout: impl Read + Send + 'static,
        stderr: impl Read + Send + 'static,
    ) -> Result<Self, String> {
        let (sender, receiver) = bounded(PLAY_OUTPUT_QUEUE_CAPACITY);
        let dropped_lines = Arc::new(AtomicU64::new(0));
        let readers = vec![
            spawn_reader(
                stdout,
                PlayOutputStream::Stdout,
                sender.clone(),
                dropped_lines.clone(),
            )?,
            spawn_reader(
                stderr,
                PlayOutputStream::Stderr,
                sender,
                dropped_lines.clone(),
            )?,
        ];
        Ok(Self {
            receiver,
            readers,
            dropped_lines,
        })
    }

    pub(super) fn drain(&self) -> Vec<String> {
        let mut diagnostics = self
            .receiver
            .try_iter()
            .take(PLAY_OUTPUT_DRAIN_LIMIT)
            .map(format_line)
            .collect::<Vec<_>>();
        append_dropped_line_diagnostic(&mut diagnostics, &self.dropped_lines);
        diagnostics
    }

    pub(super) fn finish(self) -> Vec<String> {
        for reader in self.readers {
            let _ = reader.join();
        }
        let mut diagnostics = self
            .receiver
            .try_iter()
            .map(format_line)
            .collect::<Vec<_>>();
        append_dropped_line_diagnostic(&mut diagnostics, &self.dropped_lines);
        diagnostics
    }
}

fn spawn_reader(
    reader: impl Read + Send + 'static,
    stream: PlayOutputStream,
    sender: Sender<PlayOutputLine>,
    dropped_lines: Arc<AtomicU64>,
) -> Result<JoinHandle<()>, String> {
    thread::Builder::new()
        .name(format!("zircon-play-{}", stream.label()))
        .spawn(move || {
            let mut reader = BufReader::new(reader);
            let mut bytes = Vec::new();
            loop {
                bytes.clear();
                match reader.read_until(b'\n', &mut bytes) {
                    Ok(0) => break,
                    Ok(_) => {
                        while bytes
                            .last()
                            .is_some_and(|byte| matches!(*byte, b'\r' | b'\n'))
                        {
                            bytes.pop();
                        }
                        let text = String::from_utf8_lossy(&bytes).into_owned();
                        match sender.try_send(PlayOutputLine { stream, text }) {
                            Ok(()) => {}
                            Err(TrySendError::Full(_)) => {
                                dropped_lines.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(TrySendError::Disconnected(_)) => break,
                        }
                    }
                    Err(error) => {
                        let _ = sender.try_send(PlayOutputLine {
                            stream,
                            text: format!("output read failed: {error}"),
                        });
                        break;
                    }
                }
            }
        })
        .map_err(|error| format!("failed to spawn play {} reader: {error}", stream.label()))
}

fn format_line(line: PlayOutputLine) -> String {
    format!("process.{}: {}", line.stream.label(), line.text)
}

fn append_dropped_line_diagnostic(diagnostics: &mut Vec<String>, dropped_lines: &AtomicU64) {
    let dropped = dropped_lines.swap(0, Ordering::Relaxed);
    if dropped > 0 {
        diagnostics.push(format!("process.output_dropped_lines={dropped}"));
    }
}

#[cfg(test)]
mod performance_source_guards {
    #[test]
    fn live_output_drain_has_a_per_poll_line_budget() {
        let source = include_str!("output.rs");
        let body = source
            .split("pub(super) fn drain")
            .nth(1)
            .and_then(|body| body.split("pub(super) fn finish").next())
            .expect("drain body should remain available");

        assert!(body.contains("take(PLAY_OUTPUT_DRAIN_LIMIT)"));
    }
}
