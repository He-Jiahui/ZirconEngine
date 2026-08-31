use thiserror::Error;

use super::ZrRuntimeTargetModelV1;

/// Classifies a manifest rejection before a dynamic library is allowed to execute code.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeArtifactManifestValidationError {
    #[error("runtime artifact manifest schema {actual} is unsupported; expected {expected}")]
    SchemaVersionMismatch { expected: u32, actual: u32 },
    #[error(
        "runtime artifact manifest BuildSet `{actual}` does not match its artifact identity; expected `{expected}`"
    )]
    BuildSetDigestInvalid { expected: String, actual: String },
    #[error("runtime artifact manifest BuildSet identity cannot be encoded: {message}")]
    BuildSetIdentityEncoding { message: String },
    #[error(
        "runtime artifact manifest BuildSet `{actual}` does not match host BuildSet `{expected}`"
    )]
    BuildSetMismatch { expected: String, actual: String },
    #[error(
        "runtime artifact manifest InterfaceSpec digest `{actual}` does not describe its InterfaceSpec; expected `{expected}`"
    )]
    InterfaceSpecDigestInvalid { expected: String, actual: String },
    #[error("runtime artifact manifest InterfaceSpec cannot be encoded: {message}")]
    InterfaceSpecEncoding { message: String },
    #[error(
        "runtime artifact manifest InterfaceSpec differs from the host's frozen internal InterfaceSpec"
    )]
    InterfaceSpecMismatch,
    #[error(
        "runtime artifact manifest payload schema digest `{actual}` does not match host payload schema digest `{expected}`"
    )]
    PayloadSchemaDigestMismatch { expected: String, actual: String },
    #[error(
        "runtime artifact target model {actual:?} does not match host target model {expected:?}"
    )]
    TargetModelMismatch {
        expected: ZrRuntimeTargetModelV1,
        actual: ZrRuntimeTargetModelV1,
    },
    #[error("runtime artifact manifest does not list the current host artifact `{file_name}`")]
    HostArtifactNotInBuildSet { file_name: String },
    #[error("runtime artifact manifest lists host artifact `{file_name}` with a different digest")]
    HostArtifactDigestMismatch { file_name: String },
    #[error("runtime artifact manifest omits required capabilities: {capabilities:?}")]
    MissingRequiredCapabilities { capabilities: Vec<String> },
    #[error("runtime artifact manifest has an invalid artifact identity: {message}")]
    ArtifactIdentityInvalid { message: String },
    #[error("runtime artifact manifest has an invalid runtime feature `{feature}")]
    RuntimeFeatureInvalid { feature: String },
    #[error("runtime artifact manifest must list at least one host artifact")]
    HostArtifactsMissing,
    #[error("runtime artifact manifest has an invalid host artifact identity: {message}")]
    HostArtifactIdentityInvalid { message: String },
    #[error("runtime artifact manifest lists host artifact `{file_name}` more than once")]
    DuplicateHostArtifact { file_name: String },
}
