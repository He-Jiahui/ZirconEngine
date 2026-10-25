use super::source::{StandaloneCurveChannelId, StandaloneCurveKind};
use zircon_runtime::asset::assets::DataAssetFormat;

#[derive(Debug, thiserror::Error)]
pub enum StandaloneCurveError {
    #[error("standalone curve source format {0:?} is unsupported")]
    UnsupportedFormat(DataAssetFormat),
    #[error("standalone curve source encoding is invalid or unsupported: {0}")]
    Encoding(String),
    #[error("standalone curve source version {0} is unsupported")]
    UnsupportedVersion(u32),
    #[error(
        "standalone curve kind {actual:?} does not match the explicitly selected {expected:?}"
    )]
    KindMismatch {
        expected: StandaloneCurveKind,
        actual: StandaloneCurveKind,
    },
    #[error("DataAsset canonical_json does not match its original source text")]
    CanonicalMismatch,
    #[error("invalid standalone curve field {field}: {reason}")]
    InvalidSource { field: String, reason: &'static str },
    #[error("curve surface cannot represent channel {channel:?}: {reason}")]
    UnsupportedSurface {
        channel: StandaloneCurveChannelId,
        reason: &'static str,
    },
    #[error("curve key {channel:?}/{key_id} no longer matches the admitted edit")]
    StaleKey {
        channel: StandaloneCurveChannelId,
        key_id: String,
    },
}

impl StandaloneCurveError {
    pub(super) fn invalid(field: impl Into<String>, reason: &'static str) -> Self {
        Self::InvalidSource {
            field: field.into(),
            reason,
        }
    }
}
