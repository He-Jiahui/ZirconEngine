use std::error::Error;
use std::fmt::{Display, Formatter};

use zircon_runtime::core::framework::animation::compiler::AnimationCompileDiagnostic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateMachineLayerCompileError {
    SourceDiagnostics(Vec<AnimationCompileDiagnostic>),
}

impl Display for StateMachineLayerCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceDiagnostics(diagnostics) => match diagnostics.first() {
                Some(diagnostic) => write!(
                    formatter,
                    "animation state-machine source rejected by {}: {}",
                    diagnostic.code(),
                    diagnostic.message()
                ),
                None => formatter
                    .write_str("animation state-machine source rejected without diagnostics"),
            },
        }
    }
}

impl Error for StateMachineLayerCompileError {}
