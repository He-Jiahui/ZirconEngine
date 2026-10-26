use zircon_runtime::core::math::Real;

use super::validation::nonnegative;
use super::{ParticleSourceBurst, ParticleSourceError, ParticleSystemSource};

/// Keep field text exact through this boundary, including counts beyond f32 integer precision.
pub fn parse_particle_burst_count(value: &str) -> Result<u32, ParticleSourceError> {
    if value.is_empty() || !value.bytes().all(|c| c.is_ascii_digit()) {
        return Err(ParticleSourceError::invalid(
            "burst.count",
            "count must contain only unsigned decimal digits",
        ));
    }
    value.parse().map_err(|error: std::num::ParseIntError| {
        ParticleSourceError::invalid("burst.count", error.to_string())
    })
}

/// A burst uses system simulation seconds, independently of normalized lifetime curves.
/// Count remains an integer through the domain boundary; it is never rounded through Real.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleSourceBurstValue {
    pub time_seconds: Real,
    pub count: u32,
}

/// The document owner supplies its admitted revision and retains the returned inverse.
/// Vector order, displayed time and caption are not the identity of a burst.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleSourceBurstEdit {
    pub emitter_key: String,
    pub burst_key: String,
    pub expected: ParticleSourceBurstValue,
    pub replacement: ParticleSourceBurstValue,
}

impl ParticleSourceBurstEdit {
    /// Parses the complete pair before replacing this gesture's proposed value. The captured
    /// keys/expected pair never refresh from the current source; an invalid draft leaves this
    /// delta unchanged. The input owner retains its raw text and handles focus/IME separately.
    pub fn set_replacement_from_text(
        &mut self,
        time_seconds: &str,
        count: &str,
    ) -> Result<(), ParticleSourceError> {
        let parsed_time: Real =
            time_seconds
                .parse()
                .map_err(|error: std::num::ParseFloatError| {
                    ParticleSourceError::invalid("burst.time_seconds", error.to_string())
                })?;
        nonnegative("burst.time_seconds", parsed_time)?;
        // Do not silently turn a nonzero requested occurrence into a burst at simulation zero.
        // This checks representability, not a playback-step epsilon or an invented time limit.
        if parsed_time == 0.0
            && time_seconds
                .split(['e', 'E'])
                .next()
                .unwrap_or("")
                .bytes()
                .any(|c| c.is_ascii_digit() && c != b'0')
        {
            return Err(ParticleSourceError::invalid(
                "burst.time_seconds",
                "nonzero simulation seconds underflow the source numeric type",
            ));
        }
        let count = parse_particle_burst_count(count)?;
        self.replacement = ParticleSourceBurstValue {
            time_seconds: parsed_time,
            count,
        };
        Ok(())
    }

    pub fn is_noop(&self) -> bool {
        self.expected == self.replacement
    }

    fn inverse(&self) -> Self {
        Self {
            emitter_key: self.emitter_key.clone(),
            burst_key: self.burst_key.clone(),
            expected: self.replacement,
            replacement: self.expected,
        }
    }
}

impl ParticleSystemSource {
    /// Captures both expected fields from the actual stable-key target without changing source
    /// or assigning identities. The host must capture its document/view/source revision in the
    /// same admitted read, budget this payload, and keep it through the gesture rather than
    /// recapturing on commit. The initial replacement is a no-op, not a history authorization.
    pub fn capture_burst_edit(
        &self,
        emitter_key: &str,
        burst_key: &str,
    ) -> Result<ParticleSourceBurstEdit, ParticleSourceError> {
        self.validate()?;
        let burst = find_burst(self, emitter_key, burst_key)?;
        let expected = ParticleSourceBurstValue {
            time_seconds: burst.time_seconds,
            count: burst.count,
        };
        Ok(ParticleSourceBurstEdit {
            emitter_key: emitter_key.to_owned(),
            burst_key: burst_key.to_owned(),
            expected,
            replacement: expected,
        })
    }

    /// Eligibility is read-only and rechecked on application. Host input ownership, revision,
    /// transaction, history, preview installation and durable save remain separate contracts.
    pub fn validate_burst_edit(
        &self,
        edit: &ParticleSourceBurstEdit,
    ) -> Result<(), ParticleSourceError> {
        self.validate()?;
        let burst = find_burst(self, &edit.emitter_key, &edit.burst_key)?;
        let path = format!("emitters[{}].bursts[{}]", edit.emitter_key, edit.burst_key);
        if burst.time_seconds != edit.expected.time_seconds || burst.count != edit.expected.count {
            return Err(ParticleSourceError::invalid(&path, "stale burst value"));
        }
        nonnegative(
            &format!("{path}.time_seconds"),
            edit.replacement.time_seconds,
        )
    }

    /// Writes both values after preflight and inverse allocation, preserving source order/keys.
    /// Equal times and zero counts are legal source values, not reasons to merge/delete bursts.
    pub fn apply_burst_edit(
        &mut self,
        edit: ParticleSourceBurstEdit,
    ) -> Result<ParticleSourceBurstEdit, ParticleSourceError> {
        self.validate_burst_edit(&edit)?;
        let inverse = edit.inverse();
        let emitter = self
            .emitters
            .iter_mut()
            .find(|emitter| emitter.key.as_deref() == Some(edit.emitter_key.as_str()))
            .expect("emitter was validated under the same exclusive source borrow");
        let burst = emitter
            .bursts
            .iter_mut()
            .find(|burst| burst.key.as_deref() == Some(edit.burst_key.as_str()))
            .expect("burst was validated under the same exclusive source borrow");
        burst.time_seconds = edit.replacement.time_seconds;
        burst.count = edit.replacement.count;
        Ok(inverse)
    }
}

fn find_burst<'a>(
    source: &'a ParticleSystemSource,
    emitter_key: &str,
    burst_key: &str,
) -> Result<&'a ParticleSourceBurst, ParticleSourceError> {
    let emitter = source
        .emitters
        .iter()
        .find(|emitter| emitter.key.as_deref() == Some(emitter_key))
        .ok_or_else(|| ParticleSourceError::invalid("selection", "emitter no longer exists"))?;
    emitter
        .bursts
        .iter()
        .find(|burst| burst.key.as_deref() == Some(burst_key))
        .ok_or_else(|| {
            ParticleSourceError::invalid(
                format!("emitters[{emitter_key}].bursts[{burst_key}]"),
                "burst no longer exists",
            )
        })
}
