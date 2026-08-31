use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{canonical::bytes_to_hex, ProductReceipt, ProductReceiptError};

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
    let (temporary_path, mut file) = create_temporary_receipt(output_path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not create temporary product receipt for `{}`: {error}",
            output_path.display()
        ))
    })?;
    let written = write(&mut file).map_err(|error| match error {
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
        Err(error) => {
            drop(file);
            let _ = fs::remove_file(&temporary_path);
            return Err(error);
        }
    };
    let published = fs::hard_link(&temporary_path, output_path).map_err(|error| {
        ProductReceiptError::new(format!(
            "could not create product receipt `{}`: {error}",
            output_path.display()
        ))
    });
    if let Err(error) = published {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    // Keep the flushed file identity locked through publication so its path cannot be swapped.
    drop(file);
    let _ = fs::remove_file(&temporary_path);
    Ok(written)
}

fn create_temporary_receipt(output_path: &Path) -> io::Result<(PathBuf, File)> {
    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = output_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "receipt path has no file name")
        })?;
    for _ in 0..32 {
        let sequence = TEMPORARY_RECEIPT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary_path = parent.join(format!(
            ".{file_name}.receipt-{}-{sequence}.tmp",
            std::process::id()
        ));
        match open_locked_temporary_receipt(&temporary_path) {
            Ok(file) => return Ok((temporary_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
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

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .share_mode(0x0000_0001)
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
mod performance_tests;

#[cfg(test)]
mod tests;

#[cfg(all(test, windows))]
mod windows_tests {
    use std::fs::{self, OpenOptions};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::open_locked_temporary_receipt;

    #[test]
    fn publication_file_denies_replacement_until_its_handle_is_released() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "cargo-zircon-receipt-publication-lock-{}-{nonce}.tmp",
            std::process::id()
        ));
        let published = path.with_extension("published.json");
        let file = open_locked_temporary_receipt(&path).unwrap();

        assert!(OpenOptions::new().write(true).open(&path).is_err());
        assert!(fs::remove_file(&path).is_err());
        fs::hard_link(&path, &published).unwrap();
        assert!(fs::remove_file(&published).is_err());

        drop(file);
        fs::remove_file(published).unwrap();
        fs::remove_file(path).unwrap();
    }
}
