use std::error::Error;
use std::fmt;

use crate::core::framework::window::{
    WindowEffectiveGeneration, WindowId, WindowRequestedGeneration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowStateRegistryError {
    CapacityExhausted,
    DuplicateWindowState {
        window: WindowId,
    },
    UnknownWindowState {
        window: WindowId,
    },
    RequestedGenerationMismatch {
        window: WindowId,
        expected: WindowRequestedGeneration,
        actual: WindowRequestedGeneration,
    },
    RequestedGenerationExhausted {
        window: WindowId,
    },
    ObservedGenerationExhausted {
        window: WindowId,
    },
    EffectiveGenerationExhausted {
        window: WindowId,
    },
    EffectiveRequestGenerationAhead {
        window: WindowId,
        source_requested: WindowRequestedGeneration,
        current_requested: WindowRequestedGeneration,
    },
    EffectiveRequestGenerationRegressed {
        window: WindowId,
        source_requested: WindowRequestedGeneration,
        current_effective: WindowRequestedGeneration,
    },
}

impl fmt::Display for WindowStateRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExhausted => {
                formatter.write_str("window state registry capacity exhausted")
            }
            Self::DuplicateWindowState { window } => write!(
                formatter,
                "window state is already registered for {}:{}:{}",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::UnknownWindowState { window } => write!(
                formatter,
                "window state is not registered for {}:{}:{}",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::RequestedGenerationMismatch {
                window,
                expected,
                actual,
            } => write!(
                formatter,
                "window {}:{}:{} requested state generation {} does not match expected {}",
                window.registry().raw(),
                window.slot(),
                window.generation(),
                actual.get(),
                expected.get()
            ),
            Self::RequestedGenerationExhausted { window } => write!(
                formatter,
                "window {}:{}:{} requested state generation is exhausted",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::ObservedGenerationExhausted { window } => write!(
                formatter,
                "window {}:{}:{} observed state generation is exhausted",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::EffectiveGenerationExhausted { window } => write!(
                formatter,
                "window {}:{}:{} effective state generation is exhausted",
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::EffectiveRequestGenerationAhead {
                window,
                source_requested,
                current_requested,
            } => write!(
                formatter,
                "window {}:{}:{} effective state source request generation {} is ahead of current {}",
                window.registry().raw(),
                window.slot(),
                window.generation(),
                source_requested.get(),
                current_requested.get()
            ),
            Self::EffectiveRequestGenerationRegressed {
                window,
                source_requested,
                current_effective,
            } => write!(
                formatter,
                "window {}:{}:{} effective state source request generation {} regresses current effective source {}",
                window.registry().raw(),
                window.slot(),
                window.generation(),
                source_requested.get(),
                current_effective.get()
            ),
        }
    }
}

impl Error for WindowStateRegistryError {}
