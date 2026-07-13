use std::error::Error;
use std::fmt::{Display, Formatter};

use zircon_runtime::core::math::Real;

#[derive(Clone, Debug, PartialEq)]
pub enum StateMachineLayerCompileError {
    InvalidWeight { layer: String, weight: Real },
    InvalidMask { layer: String },
}

impl Display for StateMachineLayerCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWeight { layer, weight } => {
                write!(formatter, "layer `{layer}` has invalid weight {weight}")
            }
            Self::InvalidMask { layer } => {
                write!(formatter, "layer `{layer}` has invalid dense mask weights")
            }
        }
    }
}

impl Error for StateMachineLayerCompileError {}
