use std::collections::{BTreeMap, BTreeSet};

use zircon_editor::ui::curve::{CurveInterpolation, CurveKey, CurveModel, CurvePoint, CurveView};
use zircon_runtime::core::math::Real;

use super::{ParticleSourceError, ParticleSystemSource};

/// The horizontal coordinate is normalized particle age, never system playback seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParticleCurveChannel {
    SizeMultiplier,
    Red,
    Green,
    Blue,
    Alpha,
}

impl ParticleCurveChannel {
    fn component(self) -> Option<usize> {
        match self {
            Self::SizeMultiplier => None,
            Self::Red => Some(0),
            Self::Green => Some(1),
            Self::Blue => Some(2),
            Self::Alpha => Some(3),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::SizeMultiplier => "Size multiplier",
            Self::Red => "Color R",
            Self::Green => "Color G",
            Self::Blue => "Color B",
            Self::Alpha => "Color A",
        }
    }
}

/// A display label, key time or sorted vector index never participates in this identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParticleSourceCurveId {
    pub emitter_key: String,
    pub channel: ParticleCurveChannel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleSourceCurveEdit {
    pub curve_id: ParticleSourceCurveId,
    pub key_id: String,
    pub expected: CurvePoint,
    pub replacement: CurvePoint,
}

impl ParticleSourceCurveEdit {
    pub fn is_noop(&self) -> bool {
        self.expected == self.replacement
    }

    fn inverse(&self) -> Self {
        Self {
            curve_id: self.curve_id.clone(),
            key_id: self.key_id.clone(),
            expected: self.replacement,
            replacement: self.expected,
        }
    }
}

/// A short-lived adapter over the document owner's source, with no second source store.
/// The host must admit its transaction/revision before applying a delta and retain the inverse
/// in its existing history. This adapter does not mark dirty, save, run a preview or own history.
pub struct ParticleSourceCurveAdapter<'a> {
    source: &'a mut ParticleSystemSource,
    emitter_index: usize,
    emitter_key: String,
}

impl<'a> ParticleSourceCurveAdapter<'a> {
    pub fn new(
        source: &'a mut ParticleSystemSource,
        emitter_key: &str,
    ) -> Result<Self, ParticleSourceError> {
        // Do not turn missing identities or invalid values into apparently valid view points.
        source.validate()?;
        let emitter_index = source
            .emitters
            .iter()
            .position(|emitter| emitter.key.as_deref() == Some(emitter_key))
            .ok_or_else(|| ParticleSourceError::invalid("selection", "emitter no longer exists"))?;
        Ok(Self {
            source,
            emitter_index,
            emitter_key: emitter_key.to_owned(),
        })
    }

    /// Applies the points from one admitted gesture against the same original source values.
    /// RGBA projections of one source key must agree on its replacement time. Each component
    /// occurs at most once; final times, including untouched keys, must remain distinct.
    /// The host still owns the document revision, transaction, history and gesture cancellation.
    pub fn apply_batch(
        &mut self,
        edits: &[ParticleSourceCurveEdit],
    ) -> Result<Vec<ParticleSourceCurveEdit>, ParticleSourceError> {
        let emitter = &self.source.emitters[self.emitter_index];
        let mut seen = BTreeSet::new();
        let mut colors: BTreeMap<usize, ColorKeyEditPlan> = BTreeMap::new();
        let mut sizes = BTreeMap::new();
        let mut inverse = Vec::with_capacity(edits.len());
        for edit in edits {
            if edit.curve_id.emitter_key != self.emitter_key {
                return Err(ParticleSourceError::invalid(
                    "selection",
                    "curve is outside the selected emitter",
                ));
            }
            let field = if edit.curve_id.channel.component().is_some() {
                "color_over_lifetime"
            } else {
                "size_over_lifetime"
            };
            let path = format!("emitters[{}].{field}[{}]", self.emitter_key, edit.key_id);
            if !seen.insert((edit.curve_id.channel, edit.key_id.as_str())) {
                return Err(ParticleSourceError::invalid(
                    &path,
                    "duplicate component edit in one gesture",
                ));
            }
            if let Some(component) = edit.curve_id.channel.component() {
                let index = emitter
                    .color_over_lifetime
                    .iter()
                    .position(|key| key.key.as_deref() == Some(edit.key_id.as_str()))
                    .ok_or_else(|| ParticleSourceError::invalid(&path, "key no longer exists"))?;
                let key = &emitter.color_over_lifetime[index];
                validate_edit(
                    edit,
                    CurvePoint::new(key.t, key.value[component]),
                    false,
                    &path,
                )?;
                let plan = colors.entry(index).or_insert(ColorKeyEditPlan {
                    time: edit.replacement.time,
                    value: key.value,
                });
                if plan.time != edit.replacement.time {
                    return Err(ParticleSourceError::invalid(
                        &path,
                        "RGBA projections must agree on the source key time",
                    ));
                }
                plan.value[component] = edit.replacement.value;
            } else {
                let index = emitter
                    .size_over_lifetime
                    .iter()
                    .position(|key| key.key.as_deref() == Some(edit.key_id.as_str()))
                    .ok_or_else(|| ParticleSourceError::invalid(&path, "key no longer exists"))?;
                let key = &emitter.size_over_lifetime[index];
                validate_edit(edit, CurvePoint::new(key.t, key.value), true, &path)?;
                sizes.insert(index, edit.replacement);
            }
            inverse.push(edit.inverse());
        }
        let path = format!("emitters[{}]", self.emitter_key);
        if !colors.is_empty() {
            validate_final_times(
                emitter
                    .color_over_lifetime
                    .iter()
                    .enumerate()
                    .map(|(index, key)| colors.get(&index).map_or(key.t, |plan| plan.time)),
                &format!("{path}.color_over_lifetime"),
            )?;
        }
        if !sizes.is_empty() {
            validate_final_times(
                emitter
                    .size_over_lifetime
                    .iter()
                    .enumerate()
                    .map(|(index, key)| sizes.get(&index).map_or(key.t, |point| point.time)),
                &format!("{path}.size_over_lifetime"),
            )?;
        }

        // All fallible validation and inverse allocation precede source writes. Indices exist
        // only under this exclusive borrow; they never become persistent authoring identities.
        let emitter = &mut self.source.emitters[self.emitter_index];
        if !colors.is_empty() {
            for (index, plan) in colors {
                emitter.color_over_lifetime[index].t = plan.time;
                emitter.color_over_lifetime[index].value = plan.value;
            }
            emitter
                .color_over_lifetime
                .sort_unstable_by(|a, b| a.t.total_cmp(&b.t));
        }
        if !sizes.is_empty() {
            for (index, point) in sizes {
                emitter.size_over_lifetime[index].t = point.time;
                emitter.size_over_lifetime[index].value = point.value;
            }
            emitter
                .size_over_lifetime
                .sort_unstable_by(|a, b| a.t.total_cmp(&b.t));
        }
        Ok(inverse)
    }
}

struct ColorKeyEditPlan {
    time: Real,
    value: [Real; 4],
}

fn validate_final_times(
    times: impl IntoIterator<Item = Real>,
    path: &str,
) -> Result<(), ParticleSourceError> {
    let mut times: Vec<_> = times.into_iter().collect();
    times.sort_unstable_by(Real::total_cmp);
    if times.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ParticleSourceError::invalid(path, "duplicate key time"));
    }
    Ok(())
}

impl CurveModel for ParticleSourceCurveAdapter<'_> {
    type CurveId = ParticleSourceCurveId;
    // One shared-curve gesture is one domain mutation and one complete inverse, including
    // all selected RGBA projections. A consumer must not split this array into point calls.
    type Delta = Vec<ParticleSourceCurveEdit>;
    type Error = ParticleSourceError;

    fn curves(&self) -> Vec<CurveView<Self::CurveId>> {
        let emitter = &self.source.emitters[self.emitter_index];
        let name = emitter.display_name.as_deref().unwrap_or(&emitter.id);
        [
            ParticleCurveChannel::SizeMultiplier,
            ParticleCurveChannel::Red,
            ParticleCurveChannel::Green,
            ParticleCurveChannel::Blue,
            ParticleCurveChannel::Alpha,
        ]
        .into_iter()
        .map(|channel| {
            let keys = if let Some(component) = channel.component() {
                emitter
                    .color_over_lifetime
                    .iter()
                    .map(|key| {
                        CurveKey::new(
                            key.key.as_ref().expect("validated source key").clone(),
                            CurvePoint::new(key.t, key.value[component]),
                        )
                    })
                    .collect()
            } else {
                emitter
                    .size_over_lifetime
                    .iter()
                    .map(|key| {
                        CurveKey::new(
                            key.key.as_ref().expect("validated source key").clone(),
                            CurvePoint::new(key.t, key.value),
                        )
                    })
                    .collect()
            };
            CurveView {
                id: ParticleSourceCurveId {
                    emitter_key: self.emitter_key.clone(),
                    channel,
                },
                display_name: format!("{name} / {}", channel.label()),
                interpolation: CurveInterpolation::Linear,
                keys,
            }
        })
        .collect()
    }

    fn apply(&mut self, edits: Self::Delta) -> Result<Self::Delta, Self::Error> {
        self.apply_batch(&edits)
    }
}

fn validate_edit(
    edit: &ParticleSourceCurveEdit,
    current: CurvePoint,
    size_multiplier: bool,
    path: &str,
) -> Result<(), ParticleSourceError> {
    if current != edit.expected {
        return Err(ParticleSourceError::invalid(path, "stale curve key value"));
    }
    if !edit.replacement.time.is_finite() || !(0.0..=1.0).contains(&edit.replacement.time) {
        return Err(ParticleSourceError::invalid(
            path,
            "key time must be finite normalized particle age in 0..=1",
        ));
    }
    if !edit.replacement.value.is_finite() || size_multiplier && edit.replacement.value < 0.0 {
        return Err(ParticleSourceError::invalid(
            path,
            "value must be finite; size multipliers must be nonnegative",
        ));
    }
    Ok(())
}
