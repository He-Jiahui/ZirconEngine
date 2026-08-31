use thiserror::Error;

/// Reports malformed textual identity values before they reach a loader boundary.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeIdentityFormatError {
    #[error("{kind} must be exactly 64 lowercase hexadecimal characters, received `{value}`")]
    Digest { kind: &'static str, value: String },
    #[error("runtime artifact file name must be a single non-empty file name, received `{value}`")]
    ArtifactFileName { value: String },
    #[error("runtime target architecture and operating system must be non-empty")]
    TargetNameMissing,
    #[error("runtime pointer width must be 32 or 64 bits, received {pointer_width}")]
    PointerWidth { pointer_width: u8 },
}
