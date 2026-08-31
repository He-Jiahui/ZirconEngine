use std::error::Error;
use std::fmt::{Display, Formatter};

use zircon_runtime::core::framework::animation::compiler::AnimationCompileDiagnostic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationGraphCompileError {
    SourceDiagnostics(Vec<AnimationCompileDiagnostic>),
    NodeCapacityExceeded,
    ParameterCapacityExceeded,
    InvalidMaskTarget { target: String },
    UnresolvedMaskTarget { target: String },
    AmbiguousMaskTarget { target: String },
}

impl Display for AnimationGraphCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceDiagnostics(diagnostics) => match diagnostics.first() {
                Some(diagnostic) => write!(
                    formatter,
                    "animation graph source rejected by {}: {}",
                    diagnostic.code(),
                    diagnostic.message()
                ),
                None => formatter.write_str("animation graph source rejected without diagnostics"),
            },
            Self::NodeCapacityExceeded => {
                formatter.write_str("animation graph node capacity exceeded")
            }
            Self::ParameterCapacityExceeded => {
                formatter.write_str("animation graph parameter capacity exceeded")
            }
            Self::InvalidMaskTarget { target } => {
                write!(formatter, "invalid mask target `{target}`")
            }
            Self::UnresolvedMaskTarget { target } => {
                write!(formatter, "unresolved mask target `{target}`")
            }
            Self::AmbiguousMaskTarget { target } => {
                write!(formatter, "ambiguous mask target `{target}`")
            }
        }
    }
}

impl Error for AnimationGraphCompileError {}
