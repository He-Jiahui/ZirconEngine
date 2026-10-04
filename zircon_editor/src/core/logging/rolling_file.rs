use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{EditorLogError, LogRecord};

/// 可选的按日/大小滚动磁盘副本；权威日志仍在内存存储。
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

    /// 服务已分配序号后调用；失败仅影响磁盘副本，并由 LogWriteReport 保留诊断。
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
#[path = "tests/rolling_file.rs"]
mod tests;
