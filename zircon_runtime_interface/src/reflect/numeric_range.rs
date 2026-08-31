use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReflectNumericRange {
    min: Option<f32>,
    max: Option<f32>,
    step: Option<f32>,
    precision: Option<u8>,
}

impl ReflectNumericRange {
    pub fn new(
        min: Option<f32>,
        max: Option<f32>,
        step: Option<f32>,
        precision: Option<u8>,
    ) -> Result<Self, ReflectNumericRangeError> {
        if min.is_some_and(|value| !value.is_finite()) {
            return Err(ReflectNumericRangeError::NonFiniteMin);
        }
        if max.is_some_and(|value| !value.is_finite()) {
            return Err(ReflectNumericRangeError::NonFiniteMax);
        }
        if step.is_some_and(|value| !value.is_finite()) {
            return Err(ReflectNumericRangeError::NonFiniteStep);
        }
        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                return Err(ReflectNumericRangeError::Inverted { min, max });
            }
        }
        if let Some(step) = step.filter(|step| *step <= 0.0) {
            return Err(ReflectNumericRangeError::NonPositiveStep { step });
        }
        Ok(Self {
            min,
            max,
            step,
            precision,
        })
    }

    pub const fn min(&self) -> Option<f32> {
        self.min
    }

    pub const fn max(&self) -> Option<f32> {
        self.max
    }

    pub const fn step(&self) -> Option<f32> {
        self.step
    }

    pub const fn precision(&self) -> Option<u8> {
        self.precision
    }
}

impl<'de> Deserialize<'de> for ReflectNumericRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ReflectNumericRangeWire::deserialize(deserializer)?;
        Self::new(wire.min, wire.max, wire.step, wire.precision).map_err(D::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReflectNumericRangeWire {
    min: Option<f32>,
    max: Option<f32>,
    step: Option<f32>,
    precision: Option<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReflectNumericRangeError {
    NonFiniteMin,
    NonFiniteMax,
    NonFiniteStep,
    Inverted { min: f32, max: f32 },
    NonPositiveStep { step: f32 },
}

impl fmt::Display for ReflectNumericRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteMin => formatter.write_str("reflection numeric range min is not finite"),
            Self::NonFiniteMax => formatter.write_str("reflection numeric range max is not finite"),
            Self::NonFiniteStep => {
                formatter.write_str("reflection numeric range step is not finite")
            }
            Self::Inverted { min, max } => write!(
                formatter,
                "reflection numeric range min {min} exceeds max {max}"
            ),
            Self::NonPositiveStep { step } => write!(
                formatter,
                "reflection numeric range step must be positive, found {step}"
            ),
        }
    }
}

impl std::error::Error for ReflectNumericRangeError {}
