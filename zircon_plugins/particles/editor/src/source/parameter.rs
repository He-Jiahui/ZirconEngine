use zircon_runtime::core::math::Real;

use super::validation::{nonnegative, range};
use super::{
    parse_particle_seed, ParticleSourceError, ParticleSourceScalarRange, ParticleSystemSource,
};

/// Domain deltas only. The existing document host admits revisions/transactions and owns history.
/// Seed UI payloads must use `parse_particle_seed` before constructing the typed Seed variant.
#[derive(Clone, Debug, PartialEq)]
pub enum ParticleSourceParameterEdit {
    Seed {
        expected: u64,
        replacement: u64,
    },
    SpawnRate {
        emitter_key: String,
        expected: Real,
        replacement: Real,
    },
    Capacity {
        emitter_key: String,
        expected: u32,
        replacement: u32,
    },
    Lifetime {
        emitter_key: String,
        expected: ParticleSourceScalarRange,
        replacement: ParticleSourceScalarRange,
    },
}

impl ParticleSourceParameterEdit {
    /// Both bridge values remain decimal text until the existing strict u64 parser runs.
    pub fn seed_from_decimal(
        expected: &str,
        replacement: &str,
    ) -> Result<Self, ParticleSourceError> {
        Ok(Self::Seed {
            expected: parse_particle_seed(expected)?,
            replacement: parse_particle_seed(replacement)?,
        })
    }

    pub fn is_noop(&self) -> bool {
        match self {
            Self::Seed {
                expected,
                replacement,
            } => expected == replacement,
            Self::SpawnRate {
                expected,
                replacement,
                ..
            } => expected == replacement,
            Self::Capacity {
                expected,
                replacement,
                ..
            } => expected == replacement,
            Self::Lifetime {
                expected,
                replacement,
                ..
            } => expected == replacement,
        }
    }

    fn inverse(&self) -> Self {
        match self {
            Self::Seed {
                expected,
                replacement,
            } => Self::Seed {
                expected: *replacement,
                replacement: *expected,
            },
            Self::SpawnRate {
                emitter_key,
                expected,
                replacement,
            } => Self::SpawnRate {
                emitter_key: emitter_key.clone(),
                expected: *replacement,
                replacement: *expected,
            },
            Self::Capacity {
                emitter_key,
                expected,
                replacement,
            } => Self::Capacity {
                emitter_key: emitter_key.clone(),
                expected: *replacement,
                replacement: *expected,
            },
            Self::Lifetime {
                emitter_key,
                expected,
                replacement,
            } => Self::Lifetime {
                emitter_key: emitter_key.clone(),
                expected: *replacement,
                replacement: *expected,
            },
        }
    }
}

impl ParticleSystemSource {
    /// Read-only domain eligibility. The host must also admit the real document/session/revision,
    /// input owner and history retention; a successful preflight is not a mutation permit.
    pub fn validate_parameter_edit(
        &self,
        edit: &ParticleSourceParameterEdit,
    ) -> Result<(), ParticleSourceError> {
        self.validate()?;
        if let ParticleSourceParameterEdit::Seed { expected, .. } = edit {
            return check_expected(self.system.seed == *expected, "system.seed");
        }
        let emitter_key = match edit {
            ParticleSourceParameterEdit::SpawnRate { emitter_key, .. }
            | ParticleSourceParameterEdit::Capacity { emitter_key, .. }
            | ParticleSourceParameterEdit::Lifetime { emitter_key, .. } => emitter_key,
            ParticleSourceParameterEdit::Seed { .. } => unreachable!("seed handled above"),
        };
        let emitter = self
            .emitters
            .iter()
            .find(|emitter| emitter.key.as_deref() == Some(emitter_key.as_str()))
            .ok_or_else(|| ParticleSourceError::invalid("selection", "emitter no longer exists"))?;
        let path = format!("emitters[{emitter_key}]");
        match edit {
            ParticleSourceParameterEdit::SpawnRate {
                expected,
                replacement,
                ..
            } => {
                let path = format!("{path}.spawn_rate_per_second");
                check_expected(emitter.spawn_rate_per_second == *expected, &path)?;
                nonnegative(&path, *replacement)
            }
            ParticleSourceParameterEdit::Capacity { expected, .. } => check_expected(
                emitter.max_particles == *expected,
                &format!("{path}.max_particles"),
            ),
            ParticleSourceParameterEdit::Lifetime {
                expected,
                replacement,
                ..
            } => {
                let path = format!("{path}.lifetime");
                check_expected(emitter.lifetime == *expected, &path)?;
                range(&path, *replacement, Some(Real::MIN_POSITIVE))
            }
            ParticleSourceParameterEdit::Seed { .. } => unreachable!("seed handled above"),
        }
    }

    /// Rechecks live values immediately before writing and returns the field inverse.
    /// It does not install preview data, mark dirty or acknowledge a durable write.
    pub fn apply_parameter_edit(
        &mut self,
        edit: ParticleSourceParameterEdit,
    ) -> Result<ParticleSourceParameterEdit, ParticleSourceError> {
        self.validate_parameter_edit(&edit)?;
        // Clone inverse identities before any source write; no partially applied range is exposed.
        let inverse = edit.inverse();
        match &edit {
            ParticleSourceParameterEdit::Seed { replacement, .. } => {
                self.system.seed = *replacement;
            }
            ParticleSourceParameterEdit::SpawnRate { emitter_key, .. }
            | ParticleSourceParameterEdit::Capacity { emitter_key, .. }
            | ParticleSourceParameterEdit::Lifetime { emitter_key, .. } => {
                let emitter = self
                    .emitters
                    .iter_mut()
                    .find(|emitter| emitter.key.as_deref() == Some(emitter_key.as_str()))
                    .expect("same source was just validated under the exclusive borrow");
                match &edit {
                    ParticleSourceParameterEdit::SpawnRate { replacement, .. } => {
                        emitter.spawn_rate_per_second = *replacement;
                    }
                    ParticleSourceParameterEdit::Capacity { replacement, .. } => {
                        emitter.max_particles = *replacement;
                    }
                    ParticleSourceParameterEdit::Lifetime { replacement, .. } => {
                        emitter.lifetime = *replacement;
                    }
                    ParticleSourceParameterEdit::Seed { .. } => unreachable!("seed handled above"),
                }
            }
        }
        Ok(inverse)
    }
}

fn check_expected(matches: bool, path: &str) -> Result<(), ParticleSourceError> {
    if matches {
        Ok(())
    } else {
        Err(ParticleSourceError::invalid(path, "stale parameter value"))
    }
}
