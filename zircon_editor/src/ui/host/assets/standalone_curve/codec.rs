use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use zircon_runtime::asset::assets::{DataAsset, DataAssetFormat};

use crate::ui::curve::CurveModel;

use super::{
    StandaloneCurveAdapter, StandaloneCurveDelta, StandaloneCurveError, StandaloneCurveKind,
    StandaloneCurveSource,
};

pub struct StandaloneCurveCodec;

impl StandaloneCurveCodec {
    /// Prepares one complete replacement from an immutable source and an existing-key delta.
    /// Semantic admission must supply the explicit kind from the same captured catalog.
    /// The original document host still owns its revision/activation guards, one permit,
    /// retention budget, exact before/after history, CAS and SaveCtx. This does not install
    /// a document, mark it dirty, perform I/O or call back into the host.
    /// None is a no-op only after all expected keys and the complete candidate pass validation.
    pub fn prepare_edit(
        asset: &DataAsset,
        expected: StandaloneCurveKind,
        delta: StandaloneCurveDelta,
    ) -> Result<Option<DataAsset>, StandaloneCurveError> {
        let mut source = Self::decode(asset, expected)?;
        let inverse = StandaloneCurveAdapter::new(&mut source)?.apply(delta)?;
        if inverse.is_noop() {
            return Ok(None);
        }
        let mut candidate = asset.clone();
        let changed = Self::encode_into(&source, &mut candidate, expected)?;
        Ok(changed.then_some(candidate))
    }

    /// Explicit expected kind comes from the caller's semantic admission, never its URI suffix.
    pub fn decode(
        asset: &DataAsset,
        expected: StandaloneCurveKind,
    ) -> Result<StandaloneCurveSource, StandaloneCurveError> {
        if asset.format != DataAssetFormat::Json {
            return Err(StandaloneCurveError::UnsupportedFormat(asset.format));
        }
        // Decode the actual text first so duplicate/unknown fields are not lost in a Value map.
        let source: StandaloneCurveSource = serde_json::from_str(&asset.text)
            .map_err(|error| StandaloneCurveError::Encoding(error.to_string()))?;
        source.validate()?;
        if source.kind != expected {
            return Err(StandaloneCurveError::KindMismatch {
                expected,
                actual: source.kind,
            });
        }
        let canonical: serde_json::Value = serde_json::from_str(&asset.text)
            .map_err(|error| StandaloneCurveError::Encoding(error.to_string()))?;
        if !asset.canonical_json.is_null() && asset.canonical_json != canonical {
            return Err(StandaloneCurveError::CanonicalMismatch);
        }
        Ok(source)
    }

    /// Updates only the owner's in-memory DataAsset after complete validation. The original
    /// DocumentToolkit/SaveCtx still owns source CAS, dirty state, I/O and durable cancellation.
    pub fn encode_into(
        source: &StandaloneCurveSource,
        asset: &mut DataAsset,
        expected: StandaloneCurveKind,
    ) -> Result<bool, StandaloneCurveError> {
        let before = Self::decode(asset, expected)?;
        source.validate()?;
        if source.kind != expected {
            return Err(StandaloneCurveError::KindMismatch {
                expected,
                actual: source.kind,
            });
        }
        if before == *source {
            return Ok(false);
        }
        let text = serde_json::to_string_pretty(source)
            .map_err(|error| StandaloneCurveError::Encoding(error.to_string()))?
            + "\n";
        let reopened: StandaloneCurveSource = serde_json::from_str(&text)
            .map_err(|error| StandaloneCurveError::Encoding(error.to_string()))?;
        if reopened != *source {
            return Err(StandaloneCurveError::invalid(
                "source",
                "numeric source cannot round trip losslessly",
            ));
        }
        let canonical_json = serde_json::from_str(&text)
            .map_err(|error| StandaloneCurveError::Encoding(error.to_string()))?;
        asset.text = text;
        asset.canonical_json = canonical_json;
        Ok(true)
    }
}

pub(super) fn deserialize_optional_f32<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<f32>, D::Error> {
    #[derive(Deserialize)]
    struct Scalar(#[serde(deserialize_with = "deserialize_f32")] f32);
    Option::<Scalar>::deserialize(deserializer).map(|value| value.map(|value| value.0))
}

pub(super) fn deserialize_f32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    // Inspect the original numeric token: going through f64 first would discard decimal
    // precision before this check (for example, 1.000000000000000000000001 becomes 1).
    let raw = Box::<serde_json::value::RawValue>::deserialize(deserializer)?;
    let text = raw.get();
    let scalar = text.parse::<f32>().map_err(D::Error::custom)?;
    if !scalar.is_finite() {
        return Err(D::Error::custom("numeric source exceeds finite f32 range"));
    }
    let decimal = normalized_decimal(text)
        .ok_or_else(|| D::Error::custom("unsupported f32 numeric spelling"))?;
    // Accept canonical f32 decimals and exact values, including exponent/trailing-zero
    // equivalents. Every finite f32 has an exact decimal expansion within 149 places.
    if Some(&decimal) != normalized_decimal(&scalar.to_string()).as_ref()
        && Some(&decimal) != normalized_decimal(&format!("{scalar:.149}")).as_ref()
    {
        return Err(D::Error::custom("numeric source loses precision in f32"));
    }
    Ok(scalar)
}

fn normalized_decimal(text: &str) -> Option<(bool, String, i32)> {
    let (negative, text) = text
        .strip_prefix('-')
        .map_or((false, text), |text| (true, text));
    let (mantissa, exponent) = match text.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, exponent.parse::<i32>().ok()?),
        None => (text, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let coefficient = format!("{whole}{fraction}");
    if coefficient.is_empty() || !coefficient.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let coefficient = coefficient.trim_start_matches('0');
    if coefficient.is_empty() {
        return Some((false, "0".into(), 0));
    }
    let trimmed = coefficient.trim_end_matches('0');
    let exponent = exponent
        .checked_sub(i32::try_from(fraction.len()).ok()?)?
        .checked_add(i32::try_from(coefficient.len() - trimmed.len()).ok()?)?;
    Some((negative, trimmed.to_string(), exponent))
}
