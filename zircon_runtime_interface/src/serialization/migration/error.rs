use thiserror::Error;

/// Typed failure produced while validating or executing a migration chain.
#[derive(Debug, Error)]
pub enum MigrateError {
    #[error("invalid migration payload: {0}")]
    InvalidPayload(String),
    #[error(
        "schema {schema_id} has no migration step from version {from_version} toward {target_version}"
    )]
    MissingStep {
        schema_id: String,
        from_version: u32,
        target_version: u32,
    },
    #[error("schema {schema_id} has multiple migration steps from version {from_version}")]
    DuplicateStep {
        schema_id: String,
        from_version: u32,
    },
    #[error(
        "schema {schema_id} migration table expected version {expected_from_version}, found {found_from_version}"
    )]
    OutOfOrderStep {
        schema_id: String,
        expected_from_version: u32,
        found_from_version: u32,
    },
    #[error(
        "schema {schema_id} migration table has unexpected step {from_version} at target version {target_version}"
    )]
    UnexpectedStep {
        schema_id: String,
        from_version: u32,
        target_version: u32,
    },
    #[error("schema {schema_id} migration step from version {from_version} failed: {source}")]
    StepFailed {
        schema_id: String,
        from_version: u32,
        #[source]
        source: Box<MigrateError>,
    },
}

impl MigrateError {
    pub fn invalid_payload(message: impl Into<String>) -> Self {
        Self::InvalidPayload(message.into())
    }
}
