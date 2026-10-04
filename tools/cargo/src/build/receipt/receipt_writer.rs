#[cfg(not(windows))]
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{canonical::bytes_to_hex, ProductReceipt, ProductReceiptError};

#[cfg(windows)]
mod windows_publication;

static TEMPORARY_RECEIPT_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const RECEIPT_WRITE_BUFFER_CAPACITY: usize = 64 * 1024;

#[derive(Debug)]
pub(crate) enum ReceiptWriteError {
    Serialize(serde_json::Error),
    Io(io::Error),
}

impl From<io::Error> for ReceiptWriteError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub(crate) fn write_new_after_verification(
    receipt: &ProductReceipt,
    output_path: &Path,
) -> Result<(), ProductReceiptError> {
    // Callers expose this only after integrity provenance and attestation are verified.
    write_new_json(receipt, output_path)
}

pub(crate) fn write_new_json(
    value: &impl Serialize,
    output_path: &Path,
) -> Result<(), ProductReceiptError> {
    write_new_json_with(output_path, |file| {
        write_and_flush(file, value)?;
        Ok(())
    })
}

pub(crate) fn write_new_canonical_json_with_sha256(
    value: &impl Serialize,
    output_path: &Path,
) -> Result<String, ProductReceiptError> {
    write_new_json_with(output_path, |file| {
        let sha256 = write_canonical_json_with_sha256(&mut *file, value)?;
        file.sync_all()?;
        Ok(sha256)
    })
}

fn write_new_json_with<T>(
    output_path: &Path,
    write: impl FnOnce(&mut File) -> Result<T, ReceiptWriteError>,
) -> Result<T, ProductReceiptError> {
    let (temporary_path, file) = create_temporary_receipt(output_path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not create temporary product receipt for `{}`: {error}",
            output_path.display()
        ))
    })?;
    let mut temporary = TemporaryReceipt {
        path: temporary_path,
        file,
        needs_cleanup: true,
    };
    let written = write(&mut temporary.file).map_err(|error| match error {
        ReceiptWriteError::Serialize(error) => {
            ProductReceiptError::new(format!("could not serialize product receipt: {error}"))
        }
        ReceiptWriteError::Io(error) => ProductReceiptError::new(format!(
            "could not write product receipt `{}`: {error}",
            output_path.display()
        )),
    });
    let written = match written {
        Ok(written) => written,
        Err(error) => return Err(temporary.cleanup_after_error(error)),
    };
    let published =
        publish_locked_receipt(&temporary.file, &temporary.path, output_path).map_err(|error| {
            ProductReceiptError::new(format!(
                "could not create product receipt `{}`: {error}",
                output_path.display()
            ))
        });
    if let Err(error) = published {
        return Err(temporary.cleanup_after_error(error));
    }
    #[cfg(windows)]
    {
        // Rename moved the locked identity to the final path; it must now survive Drop.
        temporary.needs_cleanup = false;
    }
    #[cfg(not(windows))]
    temporary.cleanup().map_err(|error| {
        ProductReceiptError::new(format!(
            "product receipt was published, but temporary receipt cleanup failed: {error}"
        ))
    })?;
    Ok(written)
}

struct TemporaryReceipt {
    path: PathBuf,
    file: File,
    needs_cleanup: bool,
}

impl TemporaryReceipt {
    fn cleanup(&mut self) -> io::Result<()> {
        if self.needs_cleanup {
            #[cfg(windows)]
            windows_publication::retire(&self.file)?;
            #[cfg(not(windows))]
            fs::remove_file(&self.path)?;
            self.needs_cleanup = false;
        }
        Ok(())
    }

    fn cleanup_after_error(&mut self, error: ProductReceiptError) -> ProductReceiptError {
        match self.cleanup() {
            Ok(()) => error,
            Err(cleanup) => ProductReceiptError::new(format!(
                "{error}; temporary receipt cleanup failed for `{}`: {cleanup}",
                self.path.display()
            )),
        }
    }
}

impl Drop for TemporaryReceipt {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            eprintln!(
                "temporary receipt cleanup failed for `{}`: {error}",
                self.path.display()
            );
        }
    }
}

#[cfg(windows)]
fn publish_locked_receipt(
    file: &File,
    _temporary_path: &Path,
    output_path: &Path,
) -> io::Result<()> {
    windows_publication::publish(file, output_path)
}

#[cfg(not(windows))]
fn publish_locked_receipt(
    _file: &File,
    temporary_path: &Path,
    output_path: &Path,
) -> io::Result<()> {
    fs::hard_link(temporary_path, output_path)
}

fn create_temporary_receipt(output_path: &Path) -> io::Result<(PathBuf, File)> {
    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = output_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "receipt path has no file name")
        })?;
    #[cfg(windows)]
    windows_publication::recover_temporary_receipts(parent, file_name)?;
    for _ in 0..32 {
        let sequence = TEMPORARY_RECEIPT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary_path = parent.join(format!(
            ".{file_name}.receipt-{}-{sequence}.tmp",
            std::process::id()
        ));
        match open_locked_temporary_receipt(&temporary_path) {
            Ok(file) => return Ok((temporary_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists =>
            {
                #[cfg(windows)]
                if windows_publication::recover_temporary_receipt(&temporary_path)? {
                    match open_locked_temporary_receipt(&temporary_path) {
                        Ok(file) => return Ok((temporary_path, file)),
                        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                        Err(error) => return Err(error),
                    }
                }
            }
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a unique temporary receipt path",
    ))
}

#[cfg(windows)]
fn open_locked_temporary_receipt(path: &Path) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Foundation::GENERIC_WRITE;
    use windows_sys::Win32::Storage::FileSystem::{DELETE, FILE_SHARE_READ};

    OpenOptions::new()
        .write(true)
        .access_mode(GENERIC_WRITE | DELETE)
        .create_new(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
}

#[cfg(not(windows))]
fn open_locked_temporary_receipt(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

fn write_and_flush(file: &mut File, value: &impl Serialize) -> Result<(), ReceiptWriteError> {
    write_pretty_json(&mut *file, value)?;
    file.sync_all()?;
    Ok(())
}

fn write_pretty_json(
    destination: impl Write,
    value: &impl Serialize,
) -> Result<(), ReceiptWriteError> {
    let mut writer = BufWriter::with_capacity(RECEIPT_WRITE_BUFFER_CAPACITY, destination);
    serde_json::to_writer_pretty(&mut writer, value).map_err(ReceiptWriteError::Serialize)?;
    writer.flush()?;
    Ok(())
}

pub(crate) fn write_canonical_json_with_sha256(
    destination: impl Write,
    value: &impl Serialize,
) -> Result<String, ReceiptWriteError> {
    let mut destination = Sha256Writer::new(destination);
    {
        let mut writer = BufWriter::with_capacity(RECEIPT_WRITE_BUFFER_CAPACITY, &mut destination);
        serde_json::to_writer(&mut writer, value).map_err(ReceiptWriteError::Serialize)?;
        writer.flush()?;
    }
    Ok(destination.finish())
}

struct Sha256Writer<W> {
    destination: W,
    hasher: Sha256,
}

impl<W> Sha256Writer<W> {
    fn new(destination: W) -> Self {
        Self {
            destination,
            hasher: Sha256::new(),
        }
    }

    fn finish(self) -> String {
        bytes_to_hex(&self.hasher.finalize())
    }
}

impl<W: Write> Write for Sha256Writer<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.destination.write(bytes)?;
        self.hasher.update(&bytes[..written]);
        Ok(written)
    }

    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.destination.write_all(bytes)?;
        self.hasher.update(bytes);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.destination.flush()
    }
}

#[cfg(test)]
#[path = "receipt_writer/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "receipt_writer/tests/cases.rs"]
mod tests;

#[cfg(all(test, windows))]
#[path = "tests/receipt_writer_windows_tests.rs"]
mod windows_tests;
