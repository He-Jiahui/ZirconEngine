use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;
use std::sync::{Condvar, Mutex, MutexGuard};

use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPathIdentity};
use zircon_runtime::core::resource::io::atomic_write;

/// Serializes canonical document publication by normalized project source identity.
#[derive(Debug, Default)]
pub(crate) struct DocumentSourceWriteAuthority {
    active_sources: Mutex<BTreeSet<ResolvedProjectPathIdentity>>,
    source_released: Condvar,
}

#[derive(Debug)]
pub(crate) enum DocumentSourceWriteOutcome {
    DurableBestEffort,
    PublishedNotDurable(io::Error),
    NotPublished(io::Error),
    SourceChanged,
}

#[derive(Debug)]
pub(crate) struct DocumentSourceWriteReceipt {
    _private: (),
}

#[derive(Debug)]
pub(crate) enum DocumentSourceWritePublication {
    Durable(DocumentSourceWriteReceipt),
    PublishedNotDurable(io::Error),
}

#[derive(Debug)]
enum SourceBeforePublication {
    Missing,
    MatchesReplacement,
    DiffersFromReplacement,
    Unknown,
}

impl SourceBeforePublication {
    fn proves_visible_replacement(&self) -> bool {
        match self {
            Self::Missing | Self::DiffersFromReplacement => true,
            Self::MatchesReplacement | Self::Unknown => false,
        }
    }
}

impl DocumentSourceWriteOutcome {
    pub(crate) const fn source_changed(&self) -> bool {
        matches!(self, Self::SourceChanged)
    }

    pub(crate) fn into_publication(self) -> io::Result<DocumentSourceWritePublication> {
        match self {
            Self::DurableBestEffort => Ok(DocumentSourceWritePublication::Durable(
                DocumentSourceWriteReceipt { _private: () },
            )),
            Self::PublishedNotDurable(error) => {
                Ok(DocumentSourceWritePublication::PublishedNotDurable(error))
            }
            Self::NotPublished(error) => Err(error),
            Self::SourceChanged => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "source changed before document publication",
            )),
        }
    }
}

impl DocumentSourceWriteReceipt {
    #[cfg(test)]
    pub(in crate::core::extension::toolkit) const fn fixture_for_report_contract() -> Self {
        Self { _private: () }
    }
}

impl DocumentSourceWriteAuthority {
    pub(crate) fn acquire<'a>(
        &'a self,
        project_root: &Path,
        source_path: &Path,
    ) -> io::Result<DocumentSourceWriteLease<'a>> {
        self.acquire_with_wait_hook(project_root, source_path, || {})
    }

    fn acquire_with_wait_hook<'a>(
        &'a self,
        project_root: &Path,
        source_path: &Path,
        mut on_wait: impl FnMut(),
    ) -> io::Result<DocumentSourceWriteLease<'a>> {
        let source_path = resolve_project_source_identity(project_root, source_path)?;
        let mut active_sources = self.lock_active_sources();
        let mut wait_reported = false;
        while active_sources.contains(&source_path) {
            if !wait_reported {
                on_wait();
                wait_reported = true;
            }
            active_sources = self
                .source_released
                .wait(active_sources)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        active_sources.insert(source_path.clone());
        drop(active_sources);
        Ok(DocumentSourceWriteLease {
            authority: self,
            source_path,
        })
    }

    fn lock_active_sources(&self) -> MutexGuard<'_, BTreeSet<ResolvedProjectPathIdentity>> {
        self.active_sources
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    fn try_acquire<'a>(
        &'a self,
        project_root: &Path,
        source_path: &Path,
    ) -> io::Result<Option<DocumentSourceWriteLease<'a>>> {
        let source_path = resolve_project_source_identity(project_root, source_path)?;
        let mut active_sources = self.lock_active_sources();
        if !active_sources.insert(source_path.clone()) {
            return Ok(None);
        }
        drop(active_sources);
        Ok(Some(DocumentSourceWriteLease {
            authority: self,
            source_path,
        }))
    }
}

#[derive(Debug)]
pub(crate) struct DocumentSourceWriteLease<'a> {
    authority: &'a DocumentSourceWriteAuthority,
    source_path: ResolvedProjectPathIdentity,
}

impl DocumentSourceWriteLease<'_> {
    pub(crate) fn commit_if_matches(
        &self,
        expected_source: &[u8],
        replacement: &[u8],
    ) -> DocumentSourceWriteOutcome {
        self.commit_if_matches_with_publisher(expected_source, replacement, atomic_write)
    }

    fn commit_if_matches_with_publisher(
        &self,
        expected_source: &[u8],
        replacement: &[u8],
        publisher: impl FnOnce(&Path, &[u8]) -> io::Result<()>,
    ) -> DocumentSourceWriteOutcome {
        let source_path = self.source_path.operation_path();
        let current_source = match fs::read(source_path) {
            Ok(source) => source,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return DocumentSourceWriteOutcome::SourceChanged;
            }
            Err(error) => return DocumentSourceWriteOutcome::NotPublished(error),
        };
        if current_source != expected_source {
            return DocumentSourceWriteOutcome::SourceChanged;
        }

        if let Err(error) = ensure_source_is_writable(source_path) {
            return DocumentSourceWriteOutcome::NotPublished(error);
        }
        let source_before_publication = if current_source == replacement {
            SourceBeforePublication::MatchesReplacement
        } else {
            SourceBeforePublication::DiffersFromReplacement
        };
        self.publish_after_admission(replacement, source_before_publication, publisher)
    }

    pub(crate) fn replace(&self, replacement: &[u8]) -> DocumentSourceWriteOutcome {
        self.replace_with_publisher(replacement, atomic_write)
    }

    fn replace_with_publisher(
        &self,
        replacement: &[u8],
        publisher: impl FnOnce(&Path, &[u8]) -> io::Result<()>,
    ) -> DocumentSourceWriteOutcome {
        self.replace_with_publisher_and_observer(replacement, publisher, |path| {
            source_before_publication(path, replacement)
        })
    }

    fn replace_with_publisher_and_observer(
        &self,
        replacement: &[u8],
        publisher: impl FnOnce(&Path, &[u8]) -> io::Result<()>,
        observe_source: impl FnOnce(&Path) -> SourceBeforePublication,
    ) -> DocumentSourceWriteOutcome {
        let source_path = self.source_path.operation_path();
        if let Err(error) = ensure_source_is_writable(source_path) {
            return DocumentSourceWriteOutcome::NotPublished(error);
        }
        let source_before_publication = observe_source(source_path);
        self.publish_after_admission(replacement, source_before_publication, publisher)
    }

    fn publish_after_admission(
        &self,
        replacement: &[u8],
        source_before_publication: SourceBeforePublication,
        publisher: impl FnOnce(&Path, &[u8]) -> io::Result<()>,
    ) -> DocumentSourceWriteOutcome {
        let source_path = self.source_path.operation_path();
        match publisher(source_path, replacement) {
            Ok(()) => DocumentSourceWriteOutcome::DurableBestEffort,
            Err(error) => match fs::read(source_path) {
                Ok(published)
                    if published == replacement
                        && source_before_publication.proves_visible_replacement() =>
                {
                    DocumentSourceWriteOutcome::PublishedNotDurable(error)
                }
                _ => DocumentSourceWriteOutcome::NotPublished(error),
            },
        }
    }

    pub(crate) fn remove_if_exists(&self) -> io::Result<bool> {
        let source_path = self.source_path.operation_path();
        match fs::metadata(source_path) {
            Ok(_) => ensure_source_is_writable(source_path)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error),
        }
        fs::remove_file(source_path)?;
        Ok(true)
    }
}

impl Drop for DocumentSourceWriteLease<'_> {
    fn drop(&mut self) {
        let mut active_sources = self.authority.lock_active_sources();
        let removed = active_sources.remove(&self.source_path);
        debug_assert!(removed, "document source write lease must own its source");
        drop(active_sources);
        self.authority.source_released.notify_all();
    }
}

fn resolve_project_source_identity(
    project_root: &Path,
    source_path: &Path,
) -> io::Result<ResolvedProjectPathIdentity> {
    let project_root = ProjectPaths::resolve_existing(project_root)?;
    if !project_root.operation_path().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "active project root is not a directory: {}",
                project_root.display_path().display()
            ),
        ));
    }
    let source_path = if source_path.is_absolute() {
        ProjectPaths::resolve_path(source_path)
    } else {
        ProjectPaths::resolve_path_from(&project_root, source_path)
    }?;
    let source_identity = ResolvedProjectPathIdentity::from(source_path.clone());
    let project_identity = ResolvedProjectPathIdentity::from(project_root.clone());
    if !source_identity.is_within(&project_identity) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "document source {} is outside active project root {}",
                source_path.display_path().display(),
                project_root.display_path().display()
            ),
        ));
    }
    Ok(source_identity)
}

fn ensure_source_is_writable(path: &Path) -> io::Result<()> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.permissions().readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("document source is read-only: {}", path.display()),
        ));
    }
    OpenOptions::new().write(true).open(path).map(drop)
}

fn source_before_publication(path: &Path, replacement: &[u8]) -> SourceBeforePublication {
    match fs::read(path) {
        Ok(source) if source == replacement => SourceBeforePublication::MatchesReplacement,
        Ok(_) => SourceBeforePublication::DiffersFromReplacement,
        Err(error) if error.kind() == io::ErrorKind::NotFound => SourceBeforePublication::Missing,
        Err(_) => SourceBeforePublication::Unknown,
    }
}

#[cfg(test)]
mod tests;
