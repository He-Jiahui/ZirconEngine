use std::error::Error;
use std::fmt;

use super::{RagdollProfile, RagdollProfileError};

/// Keep authoring admission errors distinct from an inability to encode the existing schema.
#[derive(Debug)]
pub enum RagdollProfileEncodeError {
    Profile(RagdollProfileError),
    Toml(toml::ser::Error),
}

impl fmt::Display for RagdollProfileEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => fmt::Display::fmt(error, formatter),
            Self::Toml(error) => write!(
                formatter,
                "ragdoll profile cannot be encoded as TOML: {error}"
            ),
        }
    }
}

impl Error for RagdollProfileEncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            Self::Toml(error) => Some(error),
        }
    }
}

impl RagdollProfile {
    /// Encode the same typed schema consumed by `from_toml`, after full profile admission.
    ///
    /// This returns text only; the original document authority still owns CAS, history and the
    /// revision-bound save token. Source comments and formatting are not represented by this DTO.
    /// Referenced collision resources are preserved, without claiming that they are loaded/ready.
    /// A raw-document editor must reject or retain unmodeled nested fields before typed rewriting;
    /// this writer cannot recover fields already discarded by a permissive nested decoder.
    pub fn to_toml(&self) -> Result<String, RagdollProfileEncodeError> {
        self.validate()
            .map_err(RagdollProfileEncodeError::Profile)?;
        toml::to_string_pretty(self).map_err(RagdollProfileEncodeError::Toml)
    }
}
