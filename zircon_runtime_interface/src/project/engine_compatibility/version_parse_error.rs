use thiserror::Error;

/// Reports a running-engine value that is not canonical semantic-version input.
#[derive(Debug, Error)]
pub enum ProjectEngineVersionParseError {
    #[error("project engine version {value:?} is invalid: {source}")]
    Invalid {
        value: String,
        #[source]
        source: semver::Error,
    },
}
