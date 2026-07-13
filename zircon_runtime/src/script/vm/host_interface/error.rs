use std::fmt;

use super::super::{PluginSlotId, VmError};

/// Structured failure returned by VM extension registration and dispatch.
#[derive(Debug)]
pub enum VmHostInterfaceError {
    CapabilityDenied {
        channel: &'static str,
        required: &'static str,
    },
    MissingCaller,
    InvalidIdentifier {
        label: &'static str,
        value: String,
    },
    InvalidSystemStage(String),
    InvalidArgumentCount {
        channel: &'static str,
        expected: usize,
        actual: usize,
    },
    DuplicateRegistration {
        channel: &'static str,
        id: String,
        slot: PluginSlotId,
        generation: u32,
    },
    CallbackTableExhausted(&'static str),
    MissingCallbackTarget {
        slot: PluginSlotId,
        module: u32,
        function: u32,
    },
    CallbackFailed(VmError),
}

impl fmt::Display for VmHostInterfaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapabilityDenied { channel, required } => write!(
                formatter,
                "VM extension channel {channel} requires capability {required}"
            ),
            Self::MissingCaller => {
                formatter.write_str("VM extension registration has no owning package slot")
            }
            Self::InvalidIdentifier { label, value } => {
                write!(formatter, "invalid {label}: `{value}`")
            }
            Self::InvalidSystemStage(stage) => {
                write!(formatter, "unsupported VM system stage `{stage}`")
            }
            Self::InvalidArgumentCount {
                channel,
                expected,
                actual,
            } => write!(
                formatter,
                "VM {channel} registration expected {expected} arguments, received {actual}"
            ),
            Self::DuplicateRegistration {
                channel,
                id,
                slot,
                generation,
            } => write!(
                formatter,
                "duplicate VM {channel} registration `{id}` for slot {} generation {generation}",
                slot.get()
            ),
            Self::CallbackTableExhausted(label) => {
                write!(formatter, "VM callback {label} table exceeds u32 slots")
            }
            Self::MissingCallbackTarget {
                slot,
                module,
                function,
            } => write!(
                formatter,
                "VM callback target is missing for slot {} module {module} function {function}",
                slot.get()
            ),
            Self::CallbackFailed(error) => write!(formatter, "VM callback failed: {error}"),
        }
    }
}

impl std::error::Error for VmHostInterfaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CallbackFailed(error) => Some(error),
            _ => None,
        }
    }
}
