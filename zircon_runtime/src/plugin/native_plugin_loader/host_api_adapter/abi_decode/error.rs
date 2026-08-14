use std::str::Utf8Error;

use zircon_runtime_interface::{ZrNativeSystemAccessV1, ZrSystemRegistrationV2};

use crate::plugin::{PluginModuleId, RuntimeExtensionRegistryError};

use super::super::super::registration_manifest::{
    NativeSystemAccessAuthorityError, NativeSystemAccessContractError,
};

use super::system::MAX_NATIVE_SYSTEM_ACCESS_ENTRIES;

pub(in super::super) type AbiDecodeResult<T> = Result<T, AbiDecodeError>;
pub(in super::super) type NativeHostApiAdapterResult<T> = Result<T, NativeHostApiAdapterError>;

#[derive(Debug)]
pub(in super::super) enum AbiDecodeError {
    InvalidUtf8 { source: Utf8Error },
    UnknownSystemStage { stage: u32 },
    InvalidV4RegistrationAbiVersion { actual: u32 },
    InvalidV4RegistrationSize { actual: usize },
    EmptyV4AccessContract,
    InvalidV4StringListPointer { field: &'static str, count: usize },
    InvalidV4AccessPointer { count: usize },
    TooManyV4Accesses { count: usize },
    InvalidV4AccessAbiVersion { actual: u32 },
    InvalidV4AccessSize { actual: usize },
    InvalidV4AccessMode { mode: u32 },
    InvalidV4AccessDomain { domain: u32 },
    InvalidV4ThreadAffinity { affinity: u32 },
}

impl std::fmt::Display for AbiDecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUtf8 { source } => {
                write!(
                    formatter,
                    "native host API string field is not valid UTF-8: {source}"
                )
            }
            Self::UnknownSystemStage { stage } => {
                write!(formatter, "unknown native system stage {stage}")
            }
            Self::InvalidV4RegistrationAbiVersion { actual } => {
                write!(
                    formatter,
                    "native host API V4 registration ABI must be 4, got {actual}"
                )
            }
            Self::InvalidV4RegistrationSize { actual } => write!(
                formatter,
                "native host API V4 registration size must be {}, got {actual}",
                std::mem::size_of::<ZrSystemRegistrationV2>()
            ),
            Self::EmptyV4AccessContract => formatter.write_str(
                "native host API V4 systems must declare at least one component or resource access",
            ),
            Self::InvalidV4StringListPointer { field, count } => write!(
                formatter,
                "native host API V4 {field} pointer is null for {count} entries"
            ),
            Self::InvalidV4AccessPointer { count } => write!(
                formatter,
                "native host API V4 access pointer is null for {count} entries"
            ),
            Self::TooManyV4Accesses { count } => write!(
                formatter,
                "native host API V4 access list has {count} entries, maximum is {MAX_NATIVE_SYSTEM_ACCESS_ENTRIES}"
            ),
            Self::InvalidV4AccessAbiVersion { actual } => {
                write!(
                    formatter,
                    "native host API V4 access ABI must be 1, got {actual}"
                )
            }
            Self::InvalidV4AccessSize { actual } => write!(
                formatter,
                "native host API V4 access size must be {}, got {actual}",
                std::mem::size_of::<ZrNativeSystemAccessV1>()
            ),
            Self::InvalidV4AccessMode { mode } => {
                write!(
                    formatter,
                    "native host API V4 access mode {mode} is unsupported"
                )
            }
            Self::InvalidV4AccessDomain { domain } => {
                write!(
                    formatter,
                    "native host API V4 access domain {domain} is unsupported"
                )
            }
            Self::InvalidV4ThreadAffinity { affinity } => write!(
                formatter,
                "native host API V4 thread affinity {affinity} is unsupported"
            ),
        }
    }
}

impl std::error::Error for AbiDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUtf8 { source } => Some(source),
            Self::UnknownSystemStage { .. }
            | Self::InvalidV4RegistrationAbiVersion { .. }
            | Self::InvalidV4RegistrationSize { .. }
            | Self::EmptyV4AccessContract
            | Self::InvalidV4StringListPointer { .. }
            | Self::InvalidV4AccessPointer { .. }
            | Self::TooManyV4Accesses { .. }
            | Self::InvalidV4AccessAbiVersion { .. }
            | Self::InvalidV4AccessSize { .. }
            | Self::InvalidV4AccessMode { .. }
            | Self::InvalidV4AccessDomain { .. }
            | Self::InvalidV4ThreadAffinity { .. } => None,
        }
    }
}

#[derive(Debug)]
pub(in super::super) enum NativeHostApiAdapterError {
    InvalidPluginModuleOwner {
        source: RuntimeExtensionRegistryError,
    },
    InvalidSystemSet {
        source: RuntimeExtensionRegistryError,
    },
    RegisterSystem {
        source: RuntimeExtensionRegistryError,
    },
    UnknownPluginModuleOwner {
        owner: PluginModuleId,
    },
    RegisterComponent {
        source: RuntimeExtensionRegistryError,
    },
    InvalidV4RuntimeModuleName {
        module_name: String,
    },
    AbiDecode {
        source: AbiDecodeError,
    },
    InvalidV4SystemAccess {
        source: NativeSystemAccessContractError,
    },
    UnauthorizedV4SystemAccess {
        source: NativeSystemAccessAuthorityError,
    },
}

impl std::fmt::Display for NativeHostApiAdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPluginModuleOwner { source } => {
                write!(
                    formatter,
                    "native host API plugin module owner is invalid: {source}"
                )
            }
            Self::InvalidSystemSet { source } => {
                write!(formatter, "native host API system set is invalid: {source}")
            }
            Self::RegisterSystem { source } => {
                write!(
                    formatter,
                    "native host API system registration failed: {source}"
                )
            }
            Self::UnknownPluginModuleOwner { owner } => {
                write!(formatter, "unknown plugin module owner {}", owner.raw())
            }
            Self::RegisterComponent { source } => {
                write!(
                    formatter,
                    "native host API component registration failed: {source}"
                )
            }
            Self::InvalidV4RuntimeModuleName { module_name } => write!(
                formatter,
                "native host API V4 requires a <plugin>.runtime module owner, got `{module_name}`"
            ),
            Self::AbiDecode { source } => source.fmt(formatter),
            Self::InvalidV4SystemAccess { source } => {
                write!(
                    formatter,
                    "native host API V4 access contract is invalid: {source}"
                )
            }
            Self::UnauthorizedV4SystemAccess { source } => {
                write!(
                    formatter,
                    "native host API V4 access is not authorized: {source}"
                )
            }
        }
    }
}

impl std::error::Error for NativeHostApiAdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPluginModuleOwner { source }
            | Self::InvalidSystemSet { source }
            | Self::RegisterSystem { source }
            | Self::RegisterComponent { source } => Some(source),
            Self::AbiDecode { source } => std::error::Error::source(source),
            Self::InvalidV4SystemAccess { source } => Some(source),
            Self::UnauthorizedV4SystemAccess { source } => Some(source),
            Self::UnknownPluginModuleOwner { .. } | Self::InvalidV4RuntimeModuleName { .. } => None,
        }
    }
}

impl From<AbiDecodeError> for NativeHostApiAdapterError {
    fn from(source: AbiDecodeError) -> Self {
        Self::AbiDecode { source }
    }
}
