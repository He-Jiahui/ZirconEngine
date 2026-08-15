use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::asset::{AssetUri, AssetUuid};
use crate::core::resource::io::atomic_write;

use super::{AssetMetaDocument, PreviewState, ProjectPaths};

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

pub(crate) struct AssetMetaWriteGuards {
    _guards: Vec<MutexGuard<'static, ()>>,
}

pub(crate) fn lock_meta_document_path(path: &Path) -> AssetMetaWriteGuard {
    AssetMetaWriteGuard {
        _guard: lock_meta_path(path),
    }
}

pub(crate) fn lock_meta_document_paths(paths: &[PathBuf]) -> AssetMetaWriteGuards {
    let mut stripe_indices = paths
        .iter()
        .map(|path| path_stripe(path))
        .collect::<Vec<_>>();
    stripe_indices.sort_unstable();
    stripe_indices.dedup();
    let stripes = META_WRITE_STRIPES.get_or_init(|| std::array::from_fn(|_| Mutex::new(())));
    let guards = stripe_indices
        .into_iter()
        .map(|stripe| {
            stripes[stripe]
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
        })
        .collect();
    AssetMetaWriteGuards { _guards: guards }
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
    let mut hasher = DefaultHasher::new();
    ProjectPaths::filesystem_identity_key(path).hash(&mut hasher);
    hasher.finish() as usize % META_WRITE_STRIPE_COUNT
}
