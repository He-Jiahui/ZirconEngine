use super::{NativeSystemAccessDomain, NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessContractError {
    InvalidDeclaration {
        declaration: String,
    },
    InvalidStableId {
        stable_id: String,
    },
    DuplicateAccess {
        declaration: String,
    },
    ConflictingAccess {
        domain: NativeSystemAccessDomain,
        stable_id: String,
    },
    WorldAccessMustBeExclusive,
    WorkerSafeRequiresExplicitAccess,
    MissingWorkerSafeCapability,
}

impl std::fmt::Display for NativeSystemAccessContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDeclaration { declaration } => write!(
                formatter,
                "access `{declaration}` must use read|write:component|resource:<stable-id>"
            ),
            Self::InvalidStableId { stable_id } => {
                write!(formatter, "access stable id `{stable_id}` is invalid")
            }
            Self::DuplicateAccess { declaration } => {
                write!(
                    formatter,
                    "access `{declaration}` is declared more than once"
                )
            }
            Self::ConflictingAccess { domain, stable_id } => write!(
                formatter,
                "access declares both read and write for {} `{stable_id}`",
                domain.label()
            ),
            Self::WorldAccessMustBeExclusive => {
                formatter.write_str("wildcard `write:world` access must be the only declaration")
            }
            Self::WorkerSafeRequiresExplicitAccess => formatter.write_str(
                "worker-safe systems require explicit component/resource access declarations",
            ),
            Self::MissingWorkerSafeCapability => write!(
                formatter,
                "worker-safe systems require capability `{NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY}`"
            ),
        }
    }
}

impl std::error::Error for NativeSystemAccessContractError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessAuthorityError {
    WorkerSafeCapabilityNotGranted,
    UnknownStableId {
        domain: NativeSystemAccessDomain,
        stable_id: String,
    },
    CapabilityNotGranted {
        stable_id: String,
        required_capability: String,
    },
}

impl std::fmt::Display for NativeSystemAccessAuthorityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkerSafeCapabilityNotGranted => write!(
                formatter,
                "host did not grant `{NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY}`"
            ),
            Self::UnknownStableId { domain, stable_id } => write!(
                formatter,
                "unknown {} access id `{stable_id}`",
                domain.label()
            ),
            Self::CapabilityNotGranted {
                stable_id,
                required_capability,
            } => write!(
                formatter,
                "access to `{stable_id}` requires granted capability `{required_capability}`"
            ),
        }
    }
}

impl std::error::Error for NativeSystemAccessAuthorityError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessResolveError {
    UnknownComponent { stable_id: String },
    ConflictingAccess { stable_id: String, message: String },
}

impl std::fmt::Display for NativeSystemAccessResolveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownComponent { stable_id } => {
                write!(
                    formatter,
                    "component access id `{stable_id}` is not installed"
                )
            }
            Self::ConflictingAccess { stable_id, message } => {
                write!(
                    formatter,
                    "access `{stable_id}` conflicts while resolving: {message}"
                )
            }
        }
    }
}

impl std::error::Error for NativeSystemAccessResolveError {}
