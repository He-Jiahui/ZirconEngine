use thiserror::Error;

/// Invalid data in the versioned shared recent-project registry.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum HubRecentProjectsError {
    #[error("a shared recent-project entry has an empty path")]
    EmptyProjectPath,
    #[error("the shared recent-project registry has more than {limit} entries")]
    TooManyEntries { limit: usize },
    #[error("the shared recent-project registry has more than {limit} deletion tombstones")]
    TooManyTombstones { limit: usize },
    #[error("the shared recent-project registry has duplicate project path `{path_key}`")]
    DuplicateProjectPath { path_key: String },
    #[error("the shared recent-project registry is not in canonical merge order")]
    NonCanonicalOrder,
    #[error("a shared recent-project deletion tombstone has an empty path key")]
    EmptyTombstonePathKey,
    #[error("the shared recent-project registry has duplicate deletion tombstone `{path_key}`")]
    DuplicateTombstonePathKey { path_key: String },
    #[error(
        "the shared recent-project registry retains both a project and deletion tombstone for `{path_key}`"
    )]
    TombstoneOverlapsProject { path_key: String },
    #[error("the shared recent-project deletion tombstones are not in canonical order")]
    NonCanonicalTombstoneOrder,
    #[error("the shared recent-project registry revision cannot advance past u64::MAX")]
    RevisionExhausted,
    #[error("the shared recent-project logical open clock cannot advance past u64::MAX")]
    LogicalClockExhausted,
}
