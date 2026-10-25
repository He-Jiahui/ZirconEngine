use std::collections::BTreeSet;

use crate::ui::curve::{CurveInterpolation, CurveKey, CurveModel, CurvePoint, CurveView};

use super::{
    StandaloneCurveChannelId, StandaloneCurveError, StandaloneCurveInterpolation,
    StandaloneCurveKey, StandaloneCurveSource,
};

#[derive(Clone, Debug, PartialEq)]
pub struct StandaloneCurveKeyEdit {
    pub channel: StandaloneCurveChannelId,
    pub expected: StandaloneCurveKey,
    pub replacement: StandaloneCurveKey,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StandaloneCurveDelta {
    pub edits: Vec<StandaloneCurveKeyEdit>,
}

impl StandaloneCurveDelta {
    pub fn is_noop(&self) -> bool {
        self.edits
            .iter()
            .all(|edit| edit.expected == edit.replacement)
    }
}

/// Borrows the actual document source. No source copy survives a transaction, and no registry,
/// history, generation, dirty flag, evaluator, playback clock or save backend is introduced.
pub struct StandaloneCurveAdapter<'a> {
    source: &'a mut StandaloneCurveSource,
}

impl<'a> StandaloneCurveAdapter<'a> {
    pub fn new(source: &'a mut StandaloneCurveSource) -> Result<Self, StandaloneCurveError> {
        source.validate_surface()?;
        Ok(Self { source })
    }
}

impl CurveModel for StandaloneCurveAdapter<'_> {
    type CurveId = StandaloneCurveChannelId;
    type Delta = StandaloneCurveDelta;
    type Error = StandaloneCurveError;

    fn curves(&self) -> Vec<CurveView<Self::CurveId>> {
        self.source
            .channels
            .iter()
            .map(|channel| CurveView {
                id: channel.id,
                display_name: channel.display_name.clone(),
                interpolation: match channel.keys.first().map(|key| key.interpolation) {
                    Some(StandaloneCurveInterpolation::Constant) => CurveInterpolation::Step,
                    Some(StandaloneCurveInterpolation::Cubic) => CurveInterpolation::Hermite,
                    _ => CurveInterpolation::Linear,
                },
                keys: channel
                    .keys
                    .iter()
                    .map(|key| CurveKey {
                        id: key.id.clone(),
                        point: CurvePoint::new(key.input, key.value),
                        in_tangent: key.in_tangent,
                        out_tangent: key.out_tangent,
                    })
                    .collect(),
            })
            .collect()
    }

    fn apply(&mut self, delta: Self::Delta) -> Result<Self::Delta, Self::Error> {
        // Preflight every expected key and the final complete model before publishing any edit.
        // This is field-level conflict detection, not the host's document/activation CAS.
        let mut candidate = self.source.clone();
        let mut seen = BTreeSet::new();
        let mut inverse = Vec::with_capacity(delta.edits.len());
        for edit in delta.edits {
            if edit.expected.id != edit.replacement.id
                || !seen.insert((edit.channel, edit.expected.id.clone()))
            {
                return Err(StandaloneCurveError::invalid(
                    "edit",
                    "key identity changes or duplicate edits are unsupported",
                ));
            }
            let key = candidate
                .channels
                .iter_mut()
                .find(|channel| channel.id == edit.channel)
                .and_then(|channel| {
                    channel
                        .keys
                        .iter_mut()
                        .find(|key| key.id == edit.expected.id)
                })
                .ok_or_else(|| StandaloneCurveError::StaleKey {
                    channel: edit.channel,
                    key_id: edit.expected.id.clone(),
                })?;
            if *key != edit.expected {
                return Err(StandaloneCurveError::StaleKey {
                    channel: edit.channel,
                    key_id: edit.expected.id,
                });
            }
            *key = edit.replacement.clone();
            inverse.push(StandaloneCurveKeyEdit {
                channel: edit.channel,
                expected: edit.replacement,
                replacement: edit.expected,
            });
        }
        candidate.validate_surface()?;
        if candidate != *self.source {
            *self.source = candidate;
        }
        Ok(StandaloneCurveDelta { edits: inverse })
    }
}
