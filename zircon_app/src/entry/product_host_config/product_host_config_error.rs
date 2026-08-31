use std::fmt;

use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::RuntimeProfileId;
use zircon_runtime::platform::PlatformTarget;

use super::{ProductConfigSource, ProductPlatformClass, ProductRoleRequest};

/// Typed rejection produced before runtime module composition or native host loading begins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductHostConfigError {
    UnsupportedProductRole(ProductRoleRequest),
    ExportRuntimeProfileMissing,
    RuntimeProfileConflict {
        requested: RuntimeProfileId,
        export: RuntimeProfileId,
    },
    RuntimeProfileRoleConflict {
        role: ProductRoleRequest,
        runtime_profile: RuntimeProfileId,
    },
    RuntimePluginRequirementConflict(RuntimePluginId),
    TargetModeConflict {
        source: ProductConfigSource,
        role: ProductRoleRequest,
        expected: RuntimeTargetMode,
        actual: RuntimeTargetMode,
    },
    PlatformTargetConflict {
        role: ProductRoleRequest,
        expected: ProductPlatformClass,
        actual: PlatformTarget,
    },
    EditorSettingsRequireEditorHost,
    RenderCapabilityRequired(ProductRoleRequest),
    RenderCapabilityForbidden(ProductRoleRequest),
    WindowCapabilityRequired(ProductRoleRequest),
    WindowCapabilityForbidden(ProductRoleRequest),
}

impl fmt::Display for ProductHostConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProductRole(role) => {
                write!(
                    formatter,
                    "product role {role:?} has no runtime host owner yet"
                )
            }
            Self::ExportRuntimeProfileMissing => {
                formatter.write_str("export profile must name a runtime profile")
            }
            Self::RuntimeProfileConflict { requested, export } => write!(
                formatter,
                "entry runtime profile {requested:?} conflicts with export runtime profile {export:?}"
            ),
            Self::RuntimeProfileRoleConflict {
                role,
                runtime_profile,
            } => write!(
                formatter,
                "runtime profile {runtime_profile:?} is incompatible with product role {role:?}"
            ),
            Self::RuntimePluginRequirementConflict(plugin_id) => write!(
                formatter,
                "runtime plugin {plugin_id} cannot be both required and optional"
            ),
            Self::TargetModeConflict {
                source,
                role,
                expected,
                actual,
            } => write!(
                formatter,
                "{source:?} target mode {actual:?} conflicts with product role {role:?} target {expected:?}"
            ),
            Self::PlatformTargetConflict {
                role,
                expected,
                actual,
            } => write!(
                formatter,
                "platform target {actual:?} conflicts with product role {role:?} platform class {expected:?}"
            ),
            Self::EditorSettingsRequireEditorHost => formatter.write_str(
                "editor subsystem and sandbox settings require the EditorHost product role",
            ),
            Self::RenderCapabilityRequired(role) => {
                write!(formatter, "product role {role:?} requires rendering")
            }
            Self::RenderCapabilityForbidden(role) => {
                write!(formatter, "product role {role:?} forbids rendering")
            }
            Self::WindowCapabilityRequired(role) => {
                write!(formatter, "product role {role:?} requires a primary window")
            }
            Self::WindowCapabilityForbidden(role) => {
                write!(formatter, "product role {role:?} forbids a primary window")
            }
        }
    }
}

impl std::error::Error for ProductHostConfigError {}
