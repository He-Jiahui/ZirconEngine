use std::fs::{self, OpenOptions};
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
        fs::create_dir_all(&self.root)?;
        let line = record.format_line();
        let line_bytes = u64::try_from(line.len()).unwrap_or(u64::MAX);
        let mut state = self.lock_state();
        if state.day != Some(day) {
            state.day = Some(day);
            state.segment = 0;
        }
        let mut path = file_path(&self.root, day, state.segment);
        loop {
            let current_size = fs::metadata(&path)
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            if current_size == 0 || current_size.saturating_add(line_bytes) <= self.max_file_bytes {
                break;
            }
            state.segment = state
                .segment
                .checked_add(1)
                .ok_or(EditorLogError::RollingSegmentExhausted)?;
            path = file_path(&self.root, day, state.segment);
        }
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        file.write_all(line.as_bytes())?;
        file.flush()?;
        Ok(path)
    }

    fn lock_state(&self) -> MutexGuard<'_, RollingFileState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn file_path(root: &Path, day: u64, segment: u64) -> PathBuf {
    root.join(format!("editor-{day}-{segment}.log"))
}
