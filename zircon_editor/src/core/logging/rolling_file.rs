use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{EditorLogError, LogRecord};

pub struct RollingFileLogSink {
    root: PathBuf,
    max_file_bytes: u64,
    state: Mutex<RollingFileState>,
}

#[derive(Default)]
struct RollingFileState {
    day: Option<u64>,
    segment: u64,
    current: Option<RollingFileSegment>,
    #[cfg(test)]
    io_counters: RollingFileIoCounters,
}

struct RollingFileSegment {
    path: PathBuf,
    file: File,
    bytes: u64,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RollingFileIoCounters {
    directory_preparations: u64,
    metadata_probes: u64,
    file_opens: u64,
    flushes: u64,
}

impl RollingFileLogSink {
    pub fn new(root: impl Into<PathBuf>, max_file_bytes: u64) -> Result<Self, EditorLogError> {
        if max_file_bytes == 0 {
            return Err(EditorLogError::InvalidRollingFileByteLimit);
        }
        Ok(Self {
            root: root.into(),
            max_file_bytes,
            state: Mutex::new(RollingFileState::default()),
        })
    }

    pub fn append(&self, record: &LogRecord) -> Result<PathBuf, EditorLogError> {
        let day = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| EditorLogError::ClockBeforeUnixEpoch)?
            .as_secs()
            / 86_400;
        self.append_for_day(record, day)
    }

    pub(super) fn append_for_day(
        &self,
        record: &LogRecord,
        day: u64,
    ) -> Result<PathBuf, EditorLogError> {
        let line = record.format_line();
        let line_bytes = u64::try_from(line.len()).unwrap_or(u64::MAX);
        let mut state = self.lock_state();
        if state.day != Some(day) {
            state.day = Some(day);
            state.segment = 0;
            state.current = None;
        }
        self.ensure_current_segment(&mut state, day, line_bytes)?;

        let write_result = state
            .current
            .as_mut()
            .expect("rolling segment exists after preparation")
            .file
            .write_all(line.as_bytes());
        if let Err(error) = write_result {
            state.current = None;
            return Err(error.into());
        }
        #[cfg(test)]
        {
            state.io_counters.flushes = state.io_counters.flushes.saturating_add(1);
        }
        let current = state
            .current
            .as_mut()
            .expect("rolling segment remains open after a successful write");
        current.bytes = current.bytes.saturating_add(line_bytes);
        current.file.flush()?;
        Ok(current.path.clone())
    }

    fn ensure_current_segment(
        &self,
        state: &mut RollingFileState,
        day: u64,
        line_bytes: u64,
    ) -> Result<(), EditorLogError> {
        if state.current.as_ref().is_some_and(|current| {
            current.bytes == 0 || current.bytes.saturating_add(line_bytes) <= self.max_file_bytes
        }) {
            return Ok(());
        }
        if state.current.take().is_some() {
            state.segment = state
                .segment
                .checked_add(1)
                .ok_or(EditorLogError::RollingSegmentExhausted)?;
        }

        #[cfg(test)]
        {
            state.io_counters.directory_preparations =
                state.io_counters.directory_preparations.saturating_add(1);
        }
        fs::create_dir_all(&self.root)?;
        loop {
            let path = file_path(&self.root, day, state.segment);
            #[cfg(test)]
            {
                state.io_counters.metadata_probes =
                    state.io_counters.metadata_probes.saturating_add(1);
            }
            let current_size = fs::metadata(&path)
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            if current_size == 0 || current_size.saturating_add(line_bytes) <= self.max_file_bytes {
                #[cfg(test)]
                {
                    state.io_counters.file_opens = state.io_counters.file_opens.saturating_add(1);
                }
                let file = OpenOptions::new().create(true).append(true).open(&path)?;
                state.current = Some(RollingFileSegment {
                    path,
                    file,
                    bytes: current_size,
                });
                return Ok(());
            }
            state.segment = state
                .segment
                .checked_add(1)
                .ok_or(EditorLogError::RollingSegmentExhausted)?;
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, RollingFileState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    fn io_counters(&self) -> RollingFileIoCounters {
        self.lock_state().io_counters
    }
}

fn file_path(root: &Path, day: u64, segment: u64) -> PathBuf {
    root.join(format!("editor-{day}-{segment}.log"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    use super::RollingFileLogSink;
    use crate::core::logging::{LogEntry, LogRecord, LogSeverity, LogSource};

    static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    fn temp_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "zircon_editor_rolling_{label}_{}_{}",
            std::process::id(),
            NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn record(sequence: u64) -> LogRecord {
        LogRecord::new(
            sequence,
            LogEntry::new(
                LogSource::editor(),
                LogSeverity::Info,
                "cached rolling segment",
                sequence,
                None,
            )
            .unwrap(),
        )
    }

    #[test]
    fn optimization_wave_20260824c_editor11_stable_segment_reuses_file_control_path() {
        const WRITES: u64 = 64;

        let directory = temp_directory("reuse");
        let sink = RollingFileLogSink::new(&directory, 1 << 20).unwrap();
        for sequence in 1..=WRITES {
            sink.append_for_day(&record(sequence), 20_000).unwrap();
        }

        let counters = sink.io_counters();
        assert_eq!(counters.directory_preparations, 1);
        assert_eq!(counters.metadata_probes, 1);
        assert_eq!(counters.file_opens, 1);
        assert_eq!(counters.flushes, WRITES);

        drop(sink);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn optimization_wave_20260824c_editor11_cached_segment_rotates_when_full() {
        let directory = temp_directory("rotation");
        let sink = RollingFileLogSink::new(&directory, 1).unwrap();

        let first = sink.append_for_day(&record(1), 20_000).unwrap();
        let second = sink.append_for_day(&record(2), 20_000).unwrap();

        assert_ne!(first, second);
        assert!(first
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("-0.log"));
        assert!(second
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("-1.log"));
        let counters = sink.io_counters();
        assert_eq!(counters.file_opens, 2);
        assert_eq!(counters.flushes, 2);

        drop(sink);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn optimization_wave_20260824c_editor11_rolling_segment_cache_evidence() {
        const WRITES: u64 = 2_000;
        const MAX_ELAPSED_NS: u128 = 5_000_000_000;

        let directory = temp_directory("evidence");
        let sink = RollingFileLogSink::new(&directory, 1 << 20).unwrap();
        let started = Instant::now();
        for sequence in 1..=WRITES {
            sink.append_for_day(&record(sequence), 20_000).unwrap();
        }
        let elapsed_ns = started.elapsed().as_nanos();
        let counters = sink.io_counters();
        let legacy_control_operations = WRITES.saturating_mul(3);
        let optimized_control_operations = counters
            .directory_preparations
            .saturating_add(counters.metadata_probes)
            .saturating_add(counters.file_opens);
        let control_operation_reduction_bps = legacy_control_operations
            .saturating_sub(optimized_control_operations)
            .saturating_mul(10_000)
            / legacy_control_operations;

        println!(
            "EDITOR_ROLLING_LOG_BENCH_V1 writes={WRITES} legacy_control_operations={legacy_control_operations} optimized_control_operations={optimized_control_operations} control_operation_reduction_bps={control_operation_reduction_bps} flushes={} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}",
            counters.flushes,
        );

        assert_eq!(optimized_control_operations, 3);
        assert!(control_operation_reduction_bps >= 9_990);
        assert_eq!(counters.flushes, WRITES);
        assert!(elapsed_ns <= MAX_ELAPSED_NS);

        drop(sink);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
