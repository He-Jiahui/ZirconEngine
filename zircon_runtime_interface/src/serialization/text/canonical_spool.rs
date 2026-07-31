use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::write_error::CanonicalTextWriteError;
use super::canonical_writer::{CountingWriter, COPY_BUFFER_BYTES};

const TEMP_FILE_ATTEMPTS: usize = 32;

static NEXT_SPOOL_ID: AtomicU64 = AtomicU64::new(1);

pub(super) struct TempSpool {
    path: PathBuf,
    file: Option<File>,
    pub(super) accounted_bytes: usize,
}

impl TempSpool {
    pub(super) fn new() -> Result<Self, CanonicalTextWriteError> {
        let directory = std::env::temp_dir();
        for _ in 0..TEMP_FILE_ATTEMPTS {
            let id = NEXT_SPOOL_ID.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!("zircon-canonical-{}-{id}.tmp", process::id()));
            match OpenOptions::new()
                .write(true)
                .read(true)
                .create_new(true)
                .open(&path)
            {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        file: Some(file),
                        accounted_bytes: 0,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(source) => {
                    return Err(CanonicalTextWriteError::Io {
                        operation: "create canonical text spool",
                        source,
                    });
                }
            }
        }
        Err(CanonicalTextWriteError::Io {
            operation: "create canonical text spool",
            source: io::Error::new(
                io::ErrorKind::AlreadyExists,
                "could not allocate a unique canonical text spool path",
            ),
        })
    }

    pub(super) fn copy_to<W>(
        &mut self,
        output: &mut CountingWriter<'_, '_, W>,
    ) -> Result<(), CanonicalTextWriteError>
    where
        W: Write + ?Sized,
    {
        let file = self.file_mut()?;
        file.flush().map_err(|source| CanonicalTextWriteError::Io {
            operation: "flush canonical text spool",
            source,
        })?;
        file.seek(SeekFrom::Start(0))
            .map_err(|source| CanonicalTextWriteError::Io {
                operation: "rewind canonical text spool",
                source,
            })?;
        let mut buffer = [0_u8; COPY_BUFFER_BYTES];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|source| CanonicalTextWriteError::Io {
                    operation: "read canonical text spool",
                    source,
                })?;
            if read == 0 {
                return Ok(());
            }
            output.write_preaccounted(&buffer[..read])?;
        }
    }

    pub(super) fn file_mut(&mut self) -> Result<&mut File, CanonicalTextWriteError> {
        self.file
            .as_mut()
            .ok_or_else(|| CanonicalTextWriteError::Io {
                operation: "access canonical text spool",
                source: io::Error::new(
                    io::ErrorKind::NotFound,
                    "canonical text spool file was already closed",
                ),
            })
    }
}

impl Drop for TempSpool {
    fn drop(&mut self) {
        let _ = self.file.take();
        let _ = fs::remove_file(&self.path);
    }
}
