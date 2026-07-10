use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityPath;

const ANIMATION_TARGET_NAMESPACE: &[u8] = b"zircon.animation.target.v1";

/// Stable identity for an animation target derived from its import path.
///
/// The identity contains no scene entity handle. Importers and runtime target
/// tables may therefore independently derive the same value from the same
/// ordered path segments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AnimationTargetId([u8; 16]);

impl AnimationTargetId {
    pub fn from_path(path: &EntityPath) -> Self {
        Self::from_segments(path.segments())
    }

    pub fn from_segments<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut hasher = blake3::Hasher::new();
        hasher.update(ANIMATION_TARGET_NAMESPACE);
        for segment in segments {
            let segment = segment.as_ref();
            hasher.update(&(segment.len() as u64).to_le_bytes());
            hasher.update(segment.as_bytes());
        }

        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
        Self(bytes)
    }

    pub fn as_bytes(self) -> [u8; 16] {
        self.0
    }
}

impl From<&EntityPath> for AnimationTargetId {
    fn from(path: &EntityPath) -> Self {
        Self::from_path(path)
    }
}

impl fmt::Display for AnimationTargetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
