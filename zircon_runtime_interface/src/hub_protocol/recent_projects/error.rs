use thiserror::Error;

/// Invalid data in the versioned shared recent-project registry.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum HubRecentProjectsError {
    #[error("a shared recent-project entry has an empty path")]
    EmptyProjectPath,
    #[error("the shared recent-project registry has more than {limit} entries")]
    TooManyEntries { limit: usize },
    #[error("the shared recent-project registry has duplicate project path `{path_key}`")]
    DuplicateProjectPath { path_key: String },
    #[error("the shared recent-project registry is not in canonical merge order")]
    NonCanonicalOrder,
}
