use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::codec::{deserialize_f32, deserialize_optional_f32};
use super::StandaloneCurveError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandaloneCurveKind {
    Float,
    Vector,
    LinearColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandaloneCurvePrecision {
    F32,
}

/// Coordinates declare a source unit; neither choice creates a playback clock or an FPS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandaloneCurveInputUnit {
    Seconds,
    Unitless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandaloneCurveChannelId {
    Value,
    X,
    Y,
    Z,
    R,
    G,
    B,
    A,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandaloneCurveInterpolation {
    Constant,
    Linear,
    Cubic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandaloneCurveTangentMode {
    User,
}

/// Other extrapolation modes need an explicit lossless source/surface contract first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandaloneCurveExtrapolation {
    Constant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandaloneCurveKey {
    pub id: String,
    #[serde(deserialize_with = "deserialize_f32")]
    pub input: f32,
    #[serde(deserialize_with = "deserialize_f32")]
    pub value: f32,
    pub interpolation: StandaloneCurveInterpolation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tangent_mode: Option<StandaloneCurveTangentMode>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    pub in_tangent: Option<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    pub out_tangent: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandaloneCurveChannel {
    pub id: StandaloneCurveChannelId,
    pub display_name: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_value: Option<f32>,
    pub pre_extrapolation: StandaloneCurveExtrapolation,
    pub post_extrapolation: StandaloneCurveExtrapolation,
    pub keys: Vec<StandaloneCurveKey>,
}

/// This is a document-owned value, not a registry entry, asset identity or second store.
/// The host must explicitly choose this codec from its actual registered semantic type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandaloneCurveSource {
    pub version: u32,
    pub kind: StandaloneCurveKind,
    pub precision: StandaloneCurvePrecision,
    pub input_unit: StandaloneCurveInputUnit,
    pub channels: Vec<StandaloneCurveChannel>,
}

impl StandaloneCurveSource {
    pub fn validate(&self) -> Result<(), StandaloneCurveError> {
        if self.version != 1 {
            return Err(StandaloneCurveError::UnsupportedVersion(self.version));
        }
        let expected = match self.kind {
            StandaloneCurveKind::Float => &[StandaloneCurveChannelId::Value][..],
            StandaloneCurveKind::Vector => &[
                StandaloneCurveChannelId::X,
                StandaloneCurveChannelId::Y,
                StandaloneCurveChannelId::Z,
            ][..],
            StandaloneCurveKind::LinearColor => &[
                StandaloneCurveChannelId::R,
                StandaloneCurveChannelId::G,
                StandaloneCurveChannelId::B,
                StandaloneCurveChannelId::A,
            ][..],
        };
        let ids = self
            .channels
            .iter()
            .map(|channel| channel.id)
            .collect::<BTreeSet<_>>();
        if ids.len() != self.channels.len() || ids != expected.iter().copied().collect() {
            return Err(StandaloneCurveError::invalid(
                "channels",
                "missing, duplicate or wrong channel for the explicit source kind",
            ));
        }
        for channel in &self.channels {
            let field = format!("channels[{:?}]", channel.id);
            if channel
                .default_value
                .is_some_and(|value| !value.is_finite())
            {
                return Err(StandaloneCurveError::invalid(
                    &field,
                    "default must be finite or unset",
                ));
            }
            let mut keys = BTreeSet::new();
            let mut previous = None;
            for key in &channel.keys {
                if key.id.trim().is_empty()
                    || key.id.trim() != key.id
                    || !keys.insert(key.id.as_str())
                {
                    return Err(StandaloneCurveError::invalid(
                        &field,
                        "key identities must be nonempty, stable and unique within a channel",
                    ));
                }
                if !key.input.is_finite()
                    || !key.value.is_finite()
                    || previous.is_some_and(|input| key.input <= input)
                    || key.in_tangent.is_some_and(|value| !value.is_finite())
                    || key.out_tangent.is_some_and(|value| !value.is_finite())
                {
                    return Err(StandaloneCurveError::invalid(
                        &field,
                        "keys and tangents must be finite, with strictly increasing inputs",
                    ));
                }
                previous = Some(key.input);
                let cubic = key.interpolation == StandaloneCurveInterpolation::Cubic;
                if cubic
                    && (key.tangent_mode != Some(StandaloneCurveTangentMode::User)
                        || key.in_tangent.is_none()
                        || key.out_tangent.is_none())
                {
                    return Err(StandaloneCurveError::invalid(
                        &field,
                        "cubic keys require explicit manual arrive and leave slopes",
                    ));
                }
                if cubic && key.in_tangent != key.out_tangent {
                    return Err(StandaloneCurveError::invalid(
                        &field,
                        "user tangents must be linked; independent break tangents are unsupported",
                    ));
                }
                if !cubic
                    && (key.tangent_mode.is_some()
                        || key.in_tangent.is_some()
                        || key.out_tangent.is_some())
                {
                    return Err(StandaloneCurveError::invalid(
                        &field,
                        "tangents on a non-cubic key are unsupported",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_surface(&self) -> Result<(), StandaloneCurveError> {
        self.validate()?;
        for channel in &self.channels {
            let unsupported = |reason| StandaloneCurveError::UnsupportedSurface {
                channel: channel.id,
                reason,
            };
            if channel.default_value.is_some() {
                return Err(unsupported(
                    "CurveView has no authored default-value representation",
                ));
            }
            if let Some(first) = channel.keys.first() {
                // CurveView has one mode for the whole channel, including endpoint tangents.
                // Preserve mixed source modes in the codec; never homogenize them for display.
                if channel
                    .keys
                    .iter()
                    .any(|key| key.interpolation != first.interpolation)
                {
                    return Err(unsupported(
                        "per-segment mixed interpolation cannot be represented by CurveView",
                    ));
                }
            }
        }
        Ok(())
    }
}
