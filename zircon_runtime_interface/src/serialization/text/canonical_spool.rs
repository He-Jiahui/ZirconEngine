use std::cell::{Cell, RefCell};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::write_error::CanonicalTextWriteError;
use super::canonical_writer::{CountingWriter, COPY_BUFFER_BYTES};

const TEMP_FILE_ATTEMPTS: usize = 32;
const CANONICAL_SPOOL_ROOT_ENV: &str = "ZIRCON_CANONICAL_SPOOL_ROOT";
const ATTEMPT_JOURNAL_FILE_NAME: &str = "attempt.journal";
const ATTEMPT_JOURNAL_MAGIC: &str = "ZIRCON_CANONICAL_SPOOL_ATTEMPT\n";
const ATTEMPT_JOURNAL_VERSION: u8 = 1;
pub(super) const MEMORY_SPOOL_BYTES: usize = 64 * 1024;
pub(super) const MAX_CANONICAL_SPOOL_WORK_BYTES: usize = 512 * 1024 * 1024;
pub(super) const MAX_CANONICAL_SPOOL_FILES: usize =
    MAX_CANONICAL_SPOOL_WORK_BYTES / (MEMORY_SPOOL_BYTES + 1);
const CANONICAL_SPOOL_FILES_RESOURCE: &str = "canonical spool files";

static NEXT_ATTEMPT_ID: AtomicU64 = AtomicU64::new(1);

pub(super) struct SpoolAttempt {
    base_root: PathBuf,
    directory: RefCell<Option<PathBuf>>,
    next_file_id: Cell<u64>,
    spilled_file_count: Cell<usize>,
    max_spilled_files: usize,
}

impl SpoolAttempt {
    pub(super) fn new() -> Self {
        let base_root = std::env::var_os(CANONICAL_SPOOL_ROOT_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        Self::new_with_root(base_root)
    }

    fn new_with_root(base_root: PathBuf) -> Self {
        Self::new_with_root_and_file_limit(base_root, MAX_CANONICAL_SPOOL_FILES)
    }

    fn new_with_root_and_file_limit(base_root: PathBuf, max_spilled_files: usize) -> Self {
        Self {
            base_root,
            directory: RefCell::new(None),
            next_file_id: Cell::new(1),
            spilled_file_count: Cell::new(0),
            max_spilled_files,
        }
    }

    fn allocate_file(&self) -> io::Result<(PathBuf, File)> {
        let found = self
            .spilled_file_count
            .get()
            .checked_add(1)
            .unwrap_or(usize::MAX);
        if found > self.max_spilled_files {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                CanonicalSpoolFileLimitExceeded {
                    max: self.max_spilled_files,
                    found,
                },
            ));
        }
        let directory = self.ensure_directory()?;
        for _ in 0..TEMP_FILE_ATTEMPTS {
            let id = self.next_file_id.get();
            self.next_file_id.set(id.saturating_add(1));
            let path = directory.join(format!("value-{id}.tmp"));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    self.spilled_file_count.set(found);
                    return Ok((path, file));
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate a unique canonical text spool file",
        ))
    }

    fn ensure_directory(&self) -> io::Result<PathBuf> {
        if let Some(directory) = self.directory.borrow().as_ref() {
            return Ok(directory.clone());
        }
        fs::create_dir_all(&self.base_root)?;
        for _ in 0..TEMP_FILE_ATTEMPTS {
            let id = NEXT_ATTEMPT_ID.fetch_add(1, Ordering::Relaxed);
            let directory = self
                .base_root
                .join(format!("zircon-canonical-{}-{id}", process::id()));
            match fs::create_dir(&directory) {
                Ok(()) => {
                    if let Err(error) = initialize_attempt_directory(&directory, id) {
                        return Err(error);
                    }
                    self.directory.replace(Some(directory.clone()));
                    return Ok(directory);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate a unique canonical text spool attempt directory",
        ))
    }
}

fn initialize_attempt_directory(directory: &std::path::Path, attempt_id: u64) -> io::Result<()> {
    let journal_path = directory.join(ATTEMPT_JOURNAL_FILE_NAME);
    let result = (|| {
        let mut journal = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(journal_path)?;
        write!(
            journal,
            "{ATTEMPT_JOURNAL_MAGIC}version={ATTEMPT_JOURNAL_VERSION}\nowner_pid={}\nattempt_id={attempt_id}\n",
            process::id()
        )?;
        journal.flush()
    })();
    if let Err(source) = result {
        return match fs::remove_dir_all(directory) {
            Ok(()) => Err(source),
            Err(cleanup) => Err(io::Error::new(
                source.kind(),
                format!(
                    "initialize canonical spool attempt failed: {source}; cleanup failed: {cleanup}"
                ),
            )),
        };
    }
    Ok(())
}

#[derive(Debug)]
struct CanonicalSpoolFileLimitExceeded {
    max: usize,
    found: usize,
}

impl std::fmt::Display for CanonicalSpoolFileLimitExceeded {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "canonical spool file limit {} exceeded (found {})",
            self.max, self.found
        )
    }
}

impl std::error::Error for CanonicalSpoolFileLimitExceeded {}

pub(super) fn canonical_spool_resource_limit(
    error: &io::Error,
) -> Option<(&'static str, usize, usize)> {
    let limit = error
        .get_ref()?
        .downcast_ref::<CanonicalSpoolFileLimitExceeded>()?;
    Some((CANONICAL_SPOOL_FILES_RESOURCE, limit.max, limit.found))
}

impl Drop for SpoolAttempt {
    fn drop(&mut self) {
        if let Some(directory) = self.directory.get_mut().take() {
            let _ = fs::remove_dir_all(directory);
        }
    }
}

enum TempSpoolStorage {
    Memory(Vec<u8>),
    File { path: PathBuf, file: Option<File> },
}

pub(super) struct TempSpool {
    attempt: Rc<SpoolAttempt>,
    storage: TempSpoolStorage,
    pub(super) accounted_bytes: usize,
}

impl TempSpool {
    pub(super) fn new(attempt: Rc<SpoolAttempt>) -> Self {
        Self {
            attempt,
            storage: TempSpoolStorage::Memory(Vec::new()),
            accounted_bytes: 0,
        }
    }

    pub(super) fn finish_write(&mut self) -> Result<(), CanonicalTextWriteError> {
        if let TempSpoolStorage::File { file, .. } = &mut self.storage {
            if let Some(mut writer) = file.take() {
                writer
                    .flush()
                    .map_err(|source| CanonicalTextWriteError::Io {
                        operation: "flush canonical text spool",
                        source,
                    })?;
            }
        }
        Ok(())
    }

    pub(super) fn copy_to<W>(
        &mut self,
        output: &mut CountingWriter<'_, '_, W>,
    ) -> Result<(), CanonicalTextWriteError>
    where
        W: Write + ?Sized,
    {
        self.finish_write()?;
        if let TempSpoolStorage::Memory(bytes) = &self.storage {
            return output.write_preaccounted(bytes);
        }
        let TempSpoolStorage::File { path, .. } = &self.storage else {
            unreachable!("canonical spool storage variants are exhaustive");
        };
        let mut file = File::open(path).map_err(|source| CanonicalTextWriteError::Io {
            operation: "open canonical text spool for reading",
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

    #[cfg(test)]
    fn is_memory_backed(&self) -> bool {
        matches!(self.storage, TempSpoolStorage::Memory(_))
    }

    #[cfg(test)]
    fn has_open_file(&self) -> bool {
        matches!(self.storage, TempSpoolStorage::File { file: Some(_), .. })
    }
}

impl Write for TempSpool {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let should_spill = match &self.storage {
            TempSpoolStorage::Memory(buffer) => {
                buffer.len().saturating_add(bytes.len()) > MEMORY_SPOOL_BYTES
            }
            TempSpoolStorage::File { .. } => false,
        };
        if should_spill {
            let TempSpoolStorage::Memory(buffer) =
                std::mem::replace(&mut self.storage, TempSpoolStorage::Memory(Vec::new()))
            else {
                unreachable!("only in-memory spools can cross the spill threshold");
            };
            let (path, mut file) = match self.attempt.allocate_file() {
                Ok(file) => file,
                Err(error) => {
                    self.storage = TempSpoolStorage::Memory(buffer);
                    return Err(error);
                }
            };
            if let Err(error) = file.write_all(&buffer) {
                self.storage = TempSpoolStorage::File {
                    path,
                    file: Some(file),
                };
                return Err(error);
            }
            self.storage = TempSpoolStorage::File {
                path,
                file: Some(file),
            };
        }

        match &mut self.storage {
            TempSpoolStorage::Memory(buffer) => {
                buffer.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            TempSpoolStorage::File {
                file: Some(file), ..
            } => file.write(bytes),
            TempSpoolStorage::File { file: None, .. } => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "canonical text spool was already finalized",
            )),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.storage {
            TempSpoolStorage::Memory(_) => Ok(()),
            TempSpoolStorage::File {
                file: Some(file), ..
            } => file.flush(),
            TempSpoolStorage::File { file: None, .. } => Ok(()),
        }
    }
}

impl Drop for TempSpool {
    fn drop(&mut self) {
        if let TempSpoolStorage::File { path, file } = &mut self.storage {
            let _ = file.take();
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::rc::Rc;

    use super::{
        canonical_spool_resource_limit, initialize_attempt_directory, SpoolAttempt, TempSpool,
        ATTEMPT_JOURNAL_FILE_NAME, ATTEMPT_JOURNAL_MAGIC, MEMORY_SPOOL_BYTES,
    };

    #[test]
    fn small_canonical_values_stay_in_memory_without_creating_a_temp_file() {
        let root = test_root("memory");
        let attempt = Rc::new(SpoolAttempt::new_with_root(root.clone()));
        let mut spool = TempSpool::new(Rc::clone(&attempt));

        spool
            .write_all(&vec![b'x'; MEMORY_SPOOL_BYTES])
            .expect("the in-memory threshold should accept an exact-size value");
        spool
            .finish_write()
            .expect("finishing an in-memory spool should be infallible");

        assert!(spool.is_memory_backed());
        assert!(!spool.has_open_file());
        assert!(!root.exists());
    }

    #[test]
    fn large_canonical_values_close_the_spill_file_after_serialization() {
        let root = test_root("spill");
        let attempt = Rc::new(SpoolAttempt::new_with_root(root.clone()));
        let mut spool = TempSpool::new(Rc::clone(&attempt));

        spool
            .write_all(&vec![b'x'; MEMORY_SPOOL_BYTES + 1])
            .expect("a large value should spill into the attempt directory");
        assert!(!spool.is_memory_backed());
        assert!(spool.has_open_file());

        spool
            .finish_write()
            .expect("finishing a spill must close its retained writer");
        assert!(!spool.has_open_file());

        drop(spool);
        drop(attempt);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spilled_attempt_records_versioned_recovery_evidence() {
        let root = test_root("journal");
        let attempt = SpoolAttempt::new_with_root(root.clone());

        let (value_path, value_file) = attempt
            .allocate_file()
            .expect("a spill attempt should create its recovery journal");
        drop(value_file);
        let directory = value_path.parent().expect("attempt directory");
        let journal = std::fs::read_to_string(directory.join(ATTEMPT_JOURNAL_FILE_NAME))
            .expect("attempt journal should be readable before any recovery decision");

        assert!(journal.starts_with(ATTEMPT_JOURNAL_MAGIC));
        assert!(journal.contains("\nversion=1\n"));
        assert!(journal.contains(&format!("owner_pid={}\n", std::process::id())));
        let attempt_id = directory
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.rsplit('-').next())
            .expect("attempt id in directory name");
        assert!(journal.contains(&format!("attempt_id={attempt_id}\n")));

        drop(attempt);
        assert!(!directory.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn failed_attempt_journal_initialization_rolls_back_the_directory() {
        let root = test_root("journal-rollback");
        let directory = root.join("attempt");
        std::fs::create_dir_all(directory.join(ATTEMPT_JOURNAL_FILE_NAME))
            .expect("fixture should block journal file creation");

        initialize_attempt_directory(&directory, 1)
            .expect_err("a directory at the journal path must reject initialization");

        assert!(!directory.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spill_file_count_is_bounded_per_attempt_before_directory_allocation() {
        let root = test_root("file-limit");
        let attempt = SpoolAttempt::new_with_root_and_file_limit(root.clone(), 0);

        let error = attempt
            .allocate_file()
            .expect_err("the attempt must reject a spill above its file budget");

        assert_eq!(
            canonical_spool_resource_limit(&error),
            Some(("canonical spool files", 0, 1))
        );
        assert!(!root.exists());
    }

    fn test_root(label: &str) -> std::path::PathBuf {
        std::env::current_dir()
            .expect("test working directory")
            .join(format!(
                "target/canonical-spool-tests/{}-{label}",
                std::process::id()
            ))
    }
}
