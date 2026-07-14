use std::fmt;

use zircon_runtime_interface::reflect::ReflectError;

/// Typed failures returned by package-local dense reflection call sites.
#[derive(Clone, Debug, PartialEq)]
pub enum CallSiteError {
    /// A public type/member name pair was absent while resolving the package.
    UnknownMember {
        /// Fully-qualified reflected type path.
        type_path: String,
        /// Reflected field name.
        member_name: String,
    },
    /// A VM supplied a type slot outside the compiled table.
    InvalidTypeSlot {
        /// Invalid dense type slot.
        type_slot: u32,
    },
    /// A VM supplied a field slot outside the compiled type layout.
    InvalidMemberSlot {
        /// Dense type slot containing the invalid member slot.
        type_slot: u32,
        /// Invalid dense member slot.
        member_slot: u32,
    },
    /// A numeric token was not allocated by this immutable call table.
    InvalidToken {
        /// Unknown or stale opaque token.
        token: u64,
    },
    /// A table was compiled for a catalog revision that is no longer current.
    StaleCatalogRevision {
        /// Revision captured while compiling the table.
        compiled_revision: u64,
        /// Latest committed process-wide revision.
        current_revision: u64,
    },
    /// The process-wide opaque token allocator exhausted `u64`.
    TokenCapacityExceeded,
    /// The reflected type has metadata but no component adapter.
    NoComponentAdapter {
        /// Dense type slot without a component adapter.
        type_slot: u32,
    },
    /// A registry exceeded the fixed `u32` ABI slot space.
    SlotCapacityExceeded {
        /// Whether the overflow occurred in the type or member table.
        slot_kind: &'static str,
        /// Number of entries that could not fit.
        count: usize,
    },
    /// The shared reflection adapter rejected the operation.
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
            Self::InvalidToken { token } => {
                write!(formatter, "reflected call-site token {token} is invalid or stale")
            }
            Self::StaleCatalogRevision {
                compiled_revision,
                current_revision,
            } => write!(
                formatter,
                "reflected call table revision {compiled_revision} is stale; current catalog revision is {current_revision}"
            ),
            Self::TokenCapacityExceeded => {
                write!(formatter, "reflected call-site token capacity is exhausted")
            }
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
