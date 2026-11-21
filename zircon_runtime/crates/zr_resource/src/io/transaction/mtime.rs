//! Exact physical last-write stamp for an explicitly opted-in existing-target transaction.
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RetainedMtime {
    before_epoch: bool,
    seconds_from_epoch: u64,
    nanoseconds: u32,
}

impl RetainedMtime {
    pub(super) fn from_time(time: SystemTime) -> Self {
        let (before_epoch, delta) = match time.duration_since(UNIX_EPOCH) {
            Ok(delta) => (false, delta),
            Err(error) => (true, error.duration()),
        };
        Self {
            before_epoch,
            seconds_from_epoch: delta.as_secs(),
            nanoseconds: delta.subsec_nanos(),
        }
    }

    pub(super) fn as_time(self) -> io::Result<SystemTime> {
        let invalid = || {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid retained file modification time",
            )
        };
        if self.nanoseconds >= 1_000_000_000 {
            return Err(invalid());
        }
        let delta = Duration::new(self.seconds_from_epoch, self.nanoseconds);
        if self.before_epoch {
            UNIX_EPOCH.checked_sub(delta)
        } else {
            UNIX_EPOCH.checked_add(delta)
        }
        .ok_or_else(invalid)
    }

    pub(super) fn set_on(self, staging: &File) -> io::Result<()> {
        staging.set_modified(self.as_time()?)?;
        staging.sync_all()
    }

    pub(super) fn verify(self, modified: SystemTime) -> io::Result<()> {
        if self.as_time()? != modified {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "retained file modification time changed",
            ));
        }
        Ok(())
    }

    pub(super) fn verify_path(self, path: &Path, allow_missing: bool) -> io::Result<()> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                self.verify(metadata.modified()?)
            }
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "timestamp target is not a regular file",
            )),
            Err(error) if allow_missing && error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}
