use super::{
    NativePluginLiveHostLoadingError, NativePluginRegistrationManifestError,
    NativeSystemAccessAuthorityError, RuntimeExtensionRegistryError,
};

#[derive(Debug)]
pub(in super::super) enum NativePluginRegistrationReplayError {
    LiveHostLock(NativePluginLiveHostLoadingError),
    RuntimePluginNotLoaded {
        plugin_id: String,
    },
    UnsupportedManifestSchema {
        plugin_id: String,
        actual: String,
        expected: &'static str,
    },
    MissingRegistrationManifest {
        plugin_id: String,
    },
    InvalidRegistrationManifest {
        plugin_id: String,
        source: NativePluginRegistrationManifestError,
    },
    InvalidRegistrationSystem {
        plugin_id: String,
        system_id: String,
        source: NativePluginRegistrationManifestError,
    },
    BridgeMethodSlot {
        plugin_id: String,
        system_id: String,
        bridge_interface: String,
        bridge_method: String,
        source: String,
    },
    UnknownBridgeInterface {
        plugin_id: String,
        system_id: String,
        bridge_interface: String,
    },
    BridgeCallScope {
        plugin_id: String,
        source: String,
    },
    RegistryInternPluginModule {
        plugin_id: String,
        system_id: String,
        module: String,
        source: RuntimeExtensionRegistryError,
    },
    RegistryInternSystemSet {
        plugin_id: String,
        system_id: String,
        set_name: String,
        source: RuntimeExtensionRegistryError,
    },
    RegisterNativeSystem {
        plugin_id: String,
        system_id: String,
        source: RuntimeExtensionRegistryError,
    },
    InvalidSystemAccessAuthority {
        plugin_id: String,
        system_id: String,
        source: NativeSystemAccessAuthorityError,
    },
}

impl std::fmt::Display for NativePluginRegistrationReplayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LiveHostLock(source) => write!(formatter, "{source}"),
            Self::RuntimePluginNotLoaded { plugin_id } => write!(
                formatter,
                "plugin {plugin_id} is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
            ),
            Self::UnsupportedManifestSchema {
                plugin_id,
                actual,
                expected,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration manifest schema `{actual}` is unsupported; expected {expected}"
            ),
            Self::MissingRegistrationManifest { plugin_id } => write!(
                formatter,
                "runtime plugin {plugin_id} has no registration manifest to replay"
            ),
            Self::InvalidRegistrationManifest { plugin_id, source } => {
                write!(formatter, "runtime plugin {plugin_id} {source}")
            }
            Self::InvalidRegistrationSystem {
                plugin_id,
                system_id,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration system `{system_id}` {source}"
            ),
            Self::BridgeMethodSlot {
                plugin_id,
                system_id,
                bridge_interface,
                bridge_method,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration system `{system_id}` failed to resolve bridge method `{bridge_interface}.{bridge_method}`: {source}"
            ),
            Self::UnknownBridgeInterface {
                plugin_id,
                system_id,
                bridge_interface,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration system `{system_id}` references unknown bridge interface `{bridge_interface}`"
            ),
            Self::BridgeCallScope { plugin_id, source } => write!(
                formatter,
                "runtime plugin {plugin_id} failed to build native registration replay bridge call scope: {source}"
            ),
            Self::RegistryInternPluginModule {
                plugin_id,
                system_id,
                module,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} failed to intern native registration manifest system `{system_id}` module `{module}`: {source}"
            ),
            Self::RegistryInternSystemSet {
                plugin_id,
                system_id,
                set_name,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} failed to intern native registration manifest system `{system_id}` set `{set_name}`: {source}"
            ),
            Self::RegisterNativeSystem {
                plugin_id,
                system_id,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} failed to register native registration manifest system `{system_id}`: {source}"
            ),
            Self::InvalidSystemAccessAuthority {
                plugin_id,
                system_id,
                source,
            } => write!(
                formatter,
                "runtime plugin {plugin_id} registration system `{system_id}` access was denied: {source}"
            ),
        }
    }
}

impl std::error::Error for NativePluginRegistrationReplayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LiveHostLock(source) => Some(source),
            Self::InvalidRegistrationManifest { source, .. }
            | Self::InvalidRegistrationSystem { source, .. } => Some(source),
            Self::RegistryInternPluginModule { source, .. }
            | Self::RegistryInternSystemSet { source, .. }
            | Self::RegisterNativeSystem { source, .. } => Some(source),
            Self::InvalidSystemAccessAuthority { source, .. } => Some(source),
            Self::RuntimePluginNotLoaded { .. }
            | Self::UnsupportedManifestSchema { .. }
            | Self::MissingRegistrationManifest { .. }
            | Self::BridgeMethodSlot { .. }
            | Self::UnknownBridgeInterface { .. }
            | Self::BridgeCallScope { .. } => None,
        }
    }
}
