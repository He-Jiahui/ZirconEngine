use std::fmt;

use zircon_runtime_interface::reflect::ReflectError;

#[derive(Clone, Debug, PartialEq)]
pub enum CallSiteError {
    UnknownMember {
        type_path: String,
        member_name: String,
    },
    InvalidTypeSlot {
        type_slot: u32,
    },
    InvalidMemberSlot {
        type_slot: u32,
        member_slot: u32,
    },
    NoComponentAdapter {
        type_slot: u32,
    },
    SlotCapacityExceeded {
        slot_kind: &'static str,
        count: usize,
    },
    Reflect(ReflectError),
}

impl fmt::Display for CallSiteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMember {
                type_path,
                member_name,
            } => write!(
                formatter,
                "reflected call site `{type_path}.{member_name}` is not registered"
            ),
            Self::InvalidTypeSlot { type_slot } => {
                write!(
                    formatter,
                    "reflected call-site type slot {type_slot} is invalid"
                )
            }
            Self::InvalidMemberSlot {
                type_slot,
                member_slot,
            } => write!(
                formatter,
                "reflected call-site member slot {type_slot}:{member_slot} is invalid"
            ),
            Self::NoComponentAdapter { type_slot } => write!(
                formatter,
                "reflected call-site type slot {type_slot} has no component adapter"
            ),
            Self::SlotCapacityExceeded { slot_kind, count } => write!(
                formatter,
                "reflected call-site {slot_kind} count {count} exceeds u32 slot capacity"
            ),
            Self::Reflect(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for CallSiteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Reflect(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ReflectError> for CallSiteError {
    fn from(error: ReflectError) -> Self {
        Self::Reflect(error)
    }
}
