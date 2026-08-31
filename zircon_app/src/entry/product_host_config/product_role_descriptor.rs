use zircon_runtime::core::framework::platform::RuntimeTargetMode;

use crate::entry::EntryProfile;

use super::{ProductArtifactManifest, ProductHostCapabilityPolicy, ProductRoleRequest};

/// Process or embedding boundary that owns a product entry point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductEntryKind {
    NativeProcess,
    ChildProcess,
    MobileActivity,
    BrowserModule,
    EmbeddedLibrary,
}

impl ProductEntryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NativeProcess => "native_process",
            Self::ChildProcess => "child_process",
            Self::MobileActivity => "mobile_activity",
            Self::BrowserModule => "browser_module",
            Self::EmbeddedLibrary => "embedded_library",
        }
    }
}

/// Host loop family selected before Runtime composition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductRunnerKind {
    RetainedEditorHost,
    DesktopEventLoop,
    HeadlessSchedule,
    BrowserEventLoop,
    MobileLifecycle,
    ChildProcess,
    Commandlet,
    ExternalHost,
}

impl ProductRunnerKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RetainedEditorHost => "retained_editor_host",
            Self::DesktopEventLoop => "desktop_event_loop",
            Self::HeadlessSchedule => "headless_schedule",
            Self::BrowserEventLoop => "browser_event_loop",
            Self::MobileLifecycle => "mobile_lifecycle",
            Self::ChildProcess => "child_process",
            Self::Commandlet => "commandlet",
            Self::ExternalHost => "external_host",
        }
    }
}

/// Link relationship between the product host and Runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductRuntimeLinkage {
    NativeDynamic,
    Static,
    HostProvided,
}

impl ProductRuntimeLinkage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NativeDynamic => "native_dynamic",
            Self::Static => "static",
            Self::HostProvided => "host_provided",
        }
    }
}

/// Authority responsible for driving an orderly product shutdown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductShutdownPolicy {
    ProcessCoordinated,
    PlatformLifecycle,
    ParentCoordinated,
    ExternalHost,
}

impl ProductShutdownPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessCoordinated => "process_coordinated",
            Self::PlatformLifecycle => "platform_lifecycle",
            Self::ParentCoordinated => "parent_coordinated",
            Self::ExternalHost => "external_host",
        }
    }
}

/// Immutable product target policy selected before runtime composition begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProductRoleDescriptor {
    pub(super) role: ProductRoleRequest,
    pub(super) entry_profile: EntryProfile,
    pub(super) target_mode: RuntimeTargetMode,
    pub(super) entry_kind: ProductEntryKind,
    pub(super) runner_kind: ProductRunnerKind,
    pub(super) runtime_linkage: ProductRuntimeLinkage,
    pub(super) capabilities: ProductHostCapabilityPolicy,
    pub(super) shutdown_policy: ProductShutdownPolicy,
    pub(super) artifact_manifest: ProductArtifactManifest,
}

impl ProductRoleDescriptor {
    pub const fn role(self) -> ProductRoleRequest {
        self.role
    }

    pub const fn entry_profile(self) -> EntryProfile {
        self.entry_profile
    }

    pub const fn target_mode(self) -> RuntimeTargetMode {
        self.target_mode
    }

    pub const fn entry_kind(self) -> ProductEntryKind {
        self.entry_kind
    }

    pub const fn runner_kind(self) -> ProductRunnerKind {
        self.runner_kind
    }

    pub const fn runtime_linkage(self) -> ProductRuntimeLinkage {
        self.runtime_linkage
    }

    pub const fn capabilities(self) -> ProductHostCapabilityPolicy {
        self.capabilities
    }

    pub const fn shutdown_policy(self) -> ProductShutdownPolicy {
        self.shutdown_policy
    }

    pub const fn artifact_manifest(self) -> ProductArtifactManifest {
        self.artifact_manifest
    }

    pub fn diagnostic_lines(self) -> Vec<String> {
        let artifact = self.artifact_manifest;
        let capabilities = self.capabilities;
        let mut lines = Vec::with_capacity(12);
        lines.push(format!(
            "entry.product_entry_kind={}",
            self.entry_kind.as_str()
        ));
        lines.push(format!(
            "entry.product_runner={}",
            self.runner_kind.as_str()
        ));
        lines.push(format!(
            "entry.product_runtime_linkage={}",
            self.runtime_linkage.as_str()
        ));
        lines.push(format!(
            "entry.product_artifact.target={}",
            artifact.target_name()
        ));
        lines.push(format!(
            "entry.product_artifact.kind={}",
            artifact.kind().as_str()
        ));
        lines.push(format!(
            "entry.product_artifact.build_feature={}",
            artifact.required_build_feature().unwrap_or("none")
        ));
        lines.push(format!(
            "entry.product_artifact.delivery={}",
            artifact.delivery_status().as_str()
        ));
        lines.push(format!(
            "entry.product_capability.platform={}",
            capabilities.platform().as_str()
        ));
        lines.push(format!(
            "entry.product_capability.window={}",
            capabilities.window().as_str()
        ));
        lines.push(format!(
            "entry.product_capability.input={}",
            capabilities.input().as_str()
        ));
        lines.push(format!(
            "entry.product_capability.render={}",
            capabilities.render().as_str()
        ));
        lines.push(format!(
            "entry.product_shutdown={}",
            self.shutdown_policy.as_str()
        ));
        lines
    }
}
