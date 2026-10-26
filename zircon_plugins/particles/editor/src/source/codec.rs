use std::collections::BTreeSet;
use std::fmt;

use super::ParticleSystemSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParticleSourceError {
    Decode(String),
    Invalid { path: String, message: String },
    Encode(String),
}

impl ParticleSourceError {
    pub(super) fn invalid(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Invalid {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ParticleSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(message) => write!(f, "particle source decode: {message}"),
            Self::Invalid { path, message } => write!(f, "{path}: {message}"),
            Self::Encode(message) => write!(f, "particle source encode: {message}"),
        }
    }
}

impl std::error::Error for ParticleSourceError {}

/// Decode metadata for the document owner. Adopted identities need a real source write.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleSourceRead {
    pub source: ParticleSystemSource,
    pub assigned_keys: usize,
}

impl ParticleSourceRead {
    pub fn requires_identity_persistence(&self) -> bool {
        self.assigned_keys != 0
    }
}

impl ParticleSystemSource {
    /// Value-only convenience. A document host should use `decode_toml` to track key adoption.
    pub fn from_toml(text: &str) -> Result<Self, ParticleSourceError> {
        Ok(Self::decode_toml(text)?.source)
    }

    pub fn decode_toml(text: &str) -> Result<ParticleSourceRead, ParticleSourceError> {
        let mut source: Self =
            toml::from_str(text).map_err(|error| ParticleSourceError::Decode(error.to_string()))?;
        let assigned_keys = source.assign_missing_keys()?;
        source.validate()?;
        Ok(ParticleSourceRead {
            source,
            assigned_keys,
        })
    }

    /// Encodes source only. Successful encoding is not a durable-save acknowledgement.
    pub fn to_toml(&self) -> Result<String, ParticleSourceError> {
        self.validate()?;
        toml::to_string_pretty(self).map_err(|error| ParticleSourceError::Encode(error.to_string()))
    }

    /// Adopts legacy items without deriving identity from a mutable label, time or sorted index.
    /// Existing explicit keys are checked before any allocation and never reassigned.
    pub fn assign_missing_keys(&mut self) -> Result<usize, ParticleSourceError> {
        let mut used = BTreeSet::new();
        for key in self.emitters.iter().flat_map(|e| {
            e.key
                .iter()
                .chain(e.bursts.iter().filter_map(|k| k.key.as_ref()))
                .chain(e.color_over_lifetime.iter().filter_map(|k| k.key.as_ref()))
                .chain(e.size_over_lifetime.iter().filter_map(|k| k.key.as_ref()))
        }) {
            if key.trim().is_empty() || key.trim() != key || !used.insert(key.clone()) {
                return Err(ParticleSourceError::invalid(
                    "key",
                    "keys must be nonempty and unique",
                ));
            }
        }
        let mut assigned_keys = 0;
        for emitter in &mut self.emitters {
            assigned_keys += usize::from(assign(&mut emitter.key, "emitter", &mut used));
            for burst in &mut emitter.bursts {
                assigned_keys += usize::from(assign(&mut burst.key, "burst", &mut used));
            }
            for key in &mut emitter.color_over_lifetime {
                assigned_keys += usize::from(assign(&mut key.key, "color", &mut used));
            }
            for key in &mut emitter.size_over_lifetime {
                assigned_keys += usize::from(assign(&mut key.key, "size", &mut used));
            }
        }
        Ok(assigned_keys)
    }
}

fn assign(key: &mut Option<String>, prefix: &str, used: &mut BTreeSet<String>) -> bool {
    if key.is_some() {
        return false;
    }
    loop {
        // A new item must not reuse a deleted item's key still referenced by document history.
        let candidate = format!("{prefix}:{}", uuid::Uuid::new_v4());
        if used.insert(candidate.clone()) {
            *key = Some(candidate);
            return true;
        }
    }
}
