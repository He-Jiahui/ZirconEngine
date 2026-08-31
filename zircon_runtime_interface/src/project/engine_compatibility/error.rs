use thiserror::Error;

/// Reports an engine requirement that cannot be safely evaluated during preflight.
#[derive(Debug, Error)]
pub enum ProjectEngineCompatibilityError {
    #[error("project engine-version requirement {requirement:?} is invalid: {source}")]
    InvalidRequirement {
        requirement: String,
        #[source]
        source: semver::Error,
    },
}
