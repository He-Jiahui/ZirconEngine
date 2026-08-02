use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::asset::{AssetUri, AssetUuid};
use crate::core::resource::io::atomic_file::atomic_write;

use super::{AssetMetaDocument, PreviewState};

const META_WRITE_STRIPE_COUNT: usize = 64;
static META_WRITE_STRIPES: OnceLock<[Mutex<()>; META_WRITE_STRIPE_COUNT]> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMetaPreviewStateExpectation {
    pub uuid: AssetUuid,
    pub url: AssetUri,
    pub source_digest: String,
}

impl AssetMetaPreviewStateExpectation {
    pub fn from_document(document: &AssetMetaDocument) -> Self {
        Self {
            uuid: document.uuid,
            url: document.url.clone(),
            source_digest: document.source_digest.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetMetaPreviewStateStale {
    Uuid {
        expected: AssetUuid,
        current: AssetUuid,
    },
    Url {
        expected: AssetUri,
        current: AssetUri,
    },
    SourceDigest {
        expected: String,
        current: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetMetaPreviewStateCasResult {
    Updated {
        previous: PreviewState,
        current: PreviewState,
    },
    Stale(AssetMetaPreviewStateStale),
}

impl AssetMetaDocument {
    pub fn compare_and_set_preview_state(
        path: impl AsRef<Path>,
        expected: &AssetMetaPreviewStateExpectation,
        next: PreviewState,
    ) -> io::Result<AssetMetaPreviewStateCasResult> {
        let path = path.as_ref();
        let _write_guard = lock_meta_document_path(path);
        let mut current = Self::load(path)?;
        if current.uuid != expected.uuid {
            return Ok(AssetMetaPreviewStateCasResult::Stale(
                AssetMetaPreviewStateStale::Uuid {
                    expected: expected.uuid,
                    current: current.uuid,
                },
            ));
        }
        if current.url != expected.url {
            return Ok(AssetMetaPreviewStateCasResult::Stale(
                AssetMetaPreviewStateStale::Url {
                    expected: expected.url.clone(),
                    current: current.url,
                },
            ));
        }
        if current.source_digest != expected.source_digest {
            return Ok(AssetMetaPreviewStateCasResult::Stale(
                AssetMetaPreviewStateStale::SourceDigest {
                    expected: expected.source_digest.clone(),
                    current: current.source_digest,
                },
            ));
        }

        let previous = current.preview_state;
        if previous != next {
            current.preview_state = next;
            write_document_locked(&current, path)?;
        }
        Ok(AssetMetaPreviewStateCasResult::Updated {
            previous,
            current: next,
        })
    }
}

pub(super) fn save_document(document: &AssetMetaDocument, path: &Path) -> io::Result<()> {
    let _write_guard = lock_meta_document_path(path);
    write_document_locked(document, path)
}

pub(crate) struct AssetMetaWriteGuard {
    _guard: MutexGuard<'static, ()>,
}

pub(crate) fn lock_meta_document_path(path: &Path) -> AssetMetaWriteGuard {
    AssetMetaWriteGuard {
        _guard: lock_meta_path(path),
    }
}

fn write_document_locked(document: &AssetMetaDocument, path: &Path) -> io::Result<()> {
    let bytes = document.to_pretty_bytes()?;
    atomic_write(path, &bytes)
}

fn lock_meta_path(path: &Path) -> MutexGuard<'static, ()> {
    let stripes = META_WRITE_STRIPES.get_or_init(|| std::array::from_fn(|_| Mutex::new(())));
    let stripe = path_stripe(path);
    stripes[stripe]
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn path_stripe(path: &Path) -> usize {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };
    let mut hasher = DefaultHasher::new();
    #[cfg(windows)]
    absolute
        .to_string_lossy()
        .replace('/', "\\")
        .to_lowercase()
        .hash(&mut hasher);
    #[cfg(not(windows))]
    absolute.hash(&mut hasher);
    hasher.finish() as usize % META_WRITE_STRIPE_COUNT
}
