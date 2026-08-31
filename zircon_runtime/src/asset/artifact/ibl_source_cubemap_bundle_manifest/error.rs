use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum IblSourceCubemapBundleManifestError {
    #[error("IBL source bundle manifest has {actual} bytes, expected {expected}")]
    InvalidSize { actual: usize, expected: usize },
    #[error("IBL source bundle manifest magic is invalid")]
    InvalidMagic,
    #[error("IBL source bundle manifest schema {actual} is not current schema {expected}")]
    StaleSchema { actual: u32, expected: u32 },
    #[error(
        "IBL source bundle manifest staging version {actual} is not current version {expected}"
    )]
    StaleStagingVersion { actual: u64, expected: u64 },
    #[error("IBL source bundle manifest bake version {actual} is not current version {expected}")]
    StaleBakeVersion { actual: u64, expected: u64 },
    #[error(
        "IBL source bundle manifest wire platform {actual} is not supported platform {expected}"
    )]
    UnsupportedWirePlatform { actual: u32, expected: u32 },
    #[error("IBL source bundle manifest checksum is invalid")]
    ChecksumMismatch,
}
