//! Observed source bytes and live file-object identity, not an external-writer CAS.
//!
//! Retaining the object prevents a deleted file's identity from being reused while a document
//! still uses its revision. It does not prevent another process from modifying that object.

use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;

#[path = "source_revision/platform.rs"]
mod platform;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceRevision {
    Missing,
    Present(FileRevision),
}

#[derive(Clone, Debug)]
pub struct FileRevision {
    identity: platform::FileIdentity,
    digest: blake3::Hash,
    length: u64,
    // Windows uses a metadata-only handle: a read handle would block ReplaceFileW opening its
    // replacement with no sharing. This object is never used as a content immutability lock.
    _object: Arc<File>,
}

impl PartialEq for FileRevision {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.digest == other.digest
            && self.length == other.length
    }
}

impl Eq for FileRevision {}

#[derive(Debug)]
pub struct SourceSnapshot {
    bytes: Vec<u8>,
    revision: SourceRevision,
}

impl SourceSnapshot {
    /// The returned revision describes these exact bytes, even if the path changes afterwards.
    pub fn read(path: &Path) -> io::Result<Self> {
        Self::read_bounded(path, usize::MAX)
    }

    /// Pins the revision of the same bounded bytes returned to the parser.
    pub fn read_bounded(path: &Path, limit: usize) -> io::Result<Self> {
        ensure_regular_path(path)?;
        let mut file = File::open(path)?;
        ensure_regular_metadata(&file.metadata()?)?;
        let identity = platform::identity(&file)?;
        let mut bytes = Vec::new();
        Read::take(&mut file, (limit as u64).saturating_add(1)).read_to_end(&mut bytes)?;
        if bytes.len() > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "source exceeds bounded read limit",
            ));
        }
        let object = platform::pin(path, &file)?;
        if platform::identity(&object)? != identity {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "source object changed during read",
            ));
        }
        ensure_regular_path(path)?;
        let revision = SourceRevision::Present(FileRevision {
            identity,
            digest: blake3::hash(&bytes),
            length: bytes.len() as u64,
            _object: Arc::new(object),
        });
        Ok(Self { bytes, revision })
    }

    /// Reads and pins an already-open regular file without resolving its path again.
    ///
    /// The caller owns path validation, including any no-follow traversal. Reading starts at the
    /// beginning, and the revision retains this same object even if its pathname is replaced.
    #[cfg(unix)]
    pub fn from_file(mut file: File) -> io::Result<Self> {
        use std::io::Seek;

        ensure_regular_metadata(&file.metadata()?)?;
        let identity = platform::identity(&file)?;
        file.rewind()?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        if platform::identity(&file)? != identity {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "source object changed during read",
            ));
        }
        let revision = SourceRevision::Present(FileRevision {
            identity,
            digest: blake3::hash(&bytes),
            length: bytes.len() as u64,
            _object: Arc::new(file),
        });
        Ok(Self { bytes, revision })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn revision(&self) -> &SourceRevision {
        &self.revision
    }

    pub fn into_parts(self) -> (Vec<u8>, SourceRevision) {
        (self.bytes, self.revision)
    }
}

impl SourceRevision {
    /// Explicit observation for opening a document or preparing an overwrite decision.
    /// Saving must use the revision already owned by that document/decision.
    pub fn observe(path: &Path) -> io::Result<Self> {
        match SourceSnapshot::read(path) {
            Ok(snapshot) => Ok(snapshot.revision),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::Missing),
            Err(error) => Err(error),
        }
    }

    pub fn matches_path(&self, path: &Path) -> io::Result<bool> {
        Self::observe(path).map(|actual| actual == *self)
    }

    pub(crate) fn staged(path: &Path, file: &File, bytes: &[u8]) -> io::Result<Self> {
        ensure_regular_metadata(&file.metadata()?)?;
        let identity = platform::identity(file)?;
        let object = platform::pin(path, file)?;
        if platform::identity(&object)? != identity {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "staged source object changed",
            ));
        }
        Ok(Self::Present(FileRevision {
            identity,
            digest: blake3::hash(bytes),
            length: bytes.len() as u64,
            _object: Arc::new(object),
        }))
    }
}

fn ensure_regular_path(path: &Path) -> io::Result<()> {
    // Validate every existing ancestor before classifying an absent leaf as ExpectedMissing.
    // An absent child below a reparse directory must not authorize publication through it.
    for parent in path.ancestors().skip(1) {
        if parent.as_os_str().is_empty() {
            continue;
        }
        let metadata = match fs::symlink_metadata(parent) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if platform::is_reparse(&metadata) || !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "source parent is not a real directory",
            ));
        }
    }
    ensure_regular_metadata(&fs::symlink_metadata(path)?)
}

fn ensure_regular_metadata(metadata: &Metadata) -> io::Result<()> {
    if platform::is_reparse(metadata) || !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "source is not a regular file",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "source_revision/tests.rs"]
pub(crate) mod tests;
