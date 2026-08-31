use zircon_runtime::core::framework::platform::RuntimeTargetMode;

use crate::entry::EntryProfile;

use super::{
    ProductArtifactDeliveryStatus, ProductArtifactKind, ProductArtifactManifest,
    ProductCapabilityRequirement, ProductEntryKind, ProductHostCapabilityPolicy,
    ProductPlatformClass, ProductRoleDescriptor, ProductRoleRequest, ProductRunnerKind,
    ProductRuntimeLinkage, ProductShutdownPolicy,
};

const REQUIRED_WINDOWED_CAPABILITIES: ProductHostCapabilityPolicy =
    ProductHostCapabilityPolicy::new(
        ProductPlatformClass::Desktop,
        ProductCapabilityRequirement::Required,
        ProductCapabilityRequirement::Required,
        ProductCapabilityRequirement::Required,
    );

const DESKTOP_CLIENT_CAPABILITIES: ProductHostCapabilityPolicy = ProductHostCapabilityPolicy::new(
    ProductPlatformClass::Desktop,
    ProductCapabilityRequirement::Optional,
    ProductCapabilityRequirement::Optional,
    ProductCapabilityRequirement::Optional,
);

const HEADLESS_CAPABILITIES: ProductHostCapabilityPolicy = ProductHostCapabilityPolicy::new(
    ProductPlatformClass::DesktopOrHeadless,
    ProductCapabilityRequirement::Forbidden,
    ProductCapabilityRequirement::Optional,
    ProductCapabilityRequirement::Forbidden,
);

const EDITOR_HOST: ProductRoleDescriptor = ProductRoleDescriptor {
    role: ProductRoleRequest::EditorHost,
    entry_profile: EntryProfile::Editor,
    target_mode: RuntimeTargetMode::EditorHost,
    entry_kind: ProductEntryKind::NativeProcess,
    runner_kind: ProductRunnerKind::RetainedEditorHost,
    runtime_linkage: ProductRuntimeLinkage::NativeDynamic,
    capabilities: REQUIRED_WINDOWED_CAPABILITIES,
    shutdown_policy: ProductShutdownPolicy::ProcessCoordinated,
    artifact_manifest: ProductArtifactManifest::new(
        "zircon_editor",
        ProductArtifactKind::NativeExecutable,
        Some("target-editor-host"),
        ProductArtifactDeliveryStatus::Runnable,
    ),
};

const DESKTOP_CLIENT: ProductRoleDescriptor = ProductRoleDescriptor {
    role: ProductRoleRequest::DesktopClient,
    entry_profile: EntryProfile::Runtime,
    target_mode: RuntimeTargetMode::ClientRuntime,
    entry_kind: ProductEntryKind::NativeProcess,
    runner_kind: ProductRunnerKind::DesktopEventLoop,
    runtime_linkage: ProductRuntimeLinkage::NativeDynamic,
    capabilities: DESKTOP_CLIENT_CAPABILITIES,
    shutdown_policy: ProductShutdownPolicy::ProcessCoordinated,
    artifact_manifest: ProductArtifactManifest::new(
        "zircon_runtime",
        ProductArtifactKind::NativeExecutable,
        Some("target-client"),
        ProductArtifactDeliveryStatus::Preview,
    ),
};

const SERVER: ProductRoleDescriptor = ProductRoleDescriptor {
    role: ProductRoleRequest::Server,
    entry_profile: EntryProfile::Headless,
    target_mode: RuntimeTargetMode::ServerRuntime,
    entry_kind: ProductEntryKind::NativeProcess,
    runner_kind: ProductRunnerKind::HeadlessSchedule,
    runtime_linkage: ProductRuntimeLinkage::Static,
    capabilities: HEADLESS_CAPABILITIES,
    shutdown_policy: ProductShutdownPolicy::ProcessCoordinated,
    artifact_manifest: ProductArtifactManifest::new(
        "zircon_server",
        ProductArtifactKind::NativeExecutable,
        Some("target-server"),
        ProductArtifactDeliveryStatus::ConfigurationOnly,
    ),
};

const WEB_CLIENT: ProductRoleDescriptor = ProductRoleDescriptor {
    role: ProductRoleRequest::WebClient,
    entry_profile: EntryProfile::Runtime,
    target_mode: RuntimeTargetMode::ClientRuntime,
    entry_kind: ProductEntryKind::BrowserModule,
    runner_kind: ProductRunnerKind::BrowserEventLoop,
    runtime_linkage: ProductRuntimeLinkage::Static,
    capabilities: ProductHostCapabilityPolicy::new(
        ProductPlatformClass::Browser,
        ProductCapabilityRequirement::Required,
        ProductCapabilityRequirement::Required,
        ProductCapabilityRequirement::Required,
    ),
    shutdown_policy: ProductShutdownPolicy::PlatformLifecycle,
    artifact_manifest: ProductArtifactManifest::new(
        "zircon_web",
        ProductArtifactKind::BrowserModule,
        None,
        ProductArtifactDeliveryStatus::Unavailable,
    ),
};

const ANDROID_CLIENT: ProductRoleDescriptor = ProductRoleDescriptor {
    role: ProductRoleRequest::AndroidClient,
    entry_profile: EntryProfile::Runtime,
    target_mode: RuntimeTargetMode::ClientRuntime,
    entry_kind: ProductEntryKind::MobileActivity,
    runner_kind: ProductRunnerKind::MobileLifecycle,
    runtime_linkage: ProductRuntimeLinkage::Static,
    capabilities: ProductHostCapabilityPolicy::new(
        ProductPlatformClass::Mobile,
        ProductCapabilityRequirement::Required,
        ProductCapabilityRequirement::Required,
        ProductCapabilityRequirement::Required,
    ),
    shutdown_policy: ProductShutdownPolicy::PlatformLifecycle,
    artifact_manifest: ProductArtifactManifest::new(
        "zircon_android",
        ProductArtifactKind::MobileApplication,
        None,
        ProductArtifactDeliveryStatus::Unavailable,
    ),
};

const EDITOR_PLAY_CHILD: ProductRoleDescriptor = ProductRoleDescriptor {
    role: ProductRoleRequest::EditorPlayChild,
    entry_profile: EntryProfile::Runtime,
    target_mode: RuntimeTargetMode::ClientRuntime,
    entry_kind: ProductEntryKind::ChildProcess,
    runner_kind: ProductRunnerKind::ChildProcess,
    runtime_linkage: ProductRuntimeLinkage::NativeDynamic,
    capabilities: REQUIRED_WINDOWED_CAPABILITIES,
    shutdown_policy: ProductShutdownPolicy::ParentCoordinated,
    artifact_manifest: ProductArtifactManifest::new(
        "zircon_play_child",
        ProductArtifactKind::NativeExecutable,
        None,
        ProductArtifactDeliveryStatus::Unavailable,
    ),
};

const COMMANDLET: ProductRoleDescriptor = ProductRoleDescriptor {
    role: ProductRoleRequest::Commandlet,
    entry_profile: EntryProfile::Headless,
    target_mode: RuntimeTargetMode::ServerRuntime,
    entry_kind: ProductEntryKind::NativeProcess,
    runner_kind: ProductRunnerKind::Commandlet,
    runtime_linkage: ProductRuntimeLinkage::Static,
    capabilities: HEADLESS_CAPABILITIES,
    shutdown_policy: ProductShutdownPolicy::ProcessCoordinated,
    artifact_manifest: ProductArtifactManifest::new(
        "zircon_commandlet",
        ProductArtifactKind::NativeExecutable,
        None,
        ProductArtifactDeliveryStatus::Unavailable,
    ),
};

const EMBEDDED: ProductRoleDescriptor = ProductRoleDescriptor {
    role: ProductRoleRequest::Embedded,
    entry_profile: EntryProfile::Runtime,
    target_mode: RuntimeTargetMode::ClientRuntime,
    entry_kind: ProductEntryKind::EmbeddedLibrary,
    runner_kind: ProductRunnerKind::ExternalHost,
    runtime_linkage: ProductRuntimeLinkage::HostProvided,
    capabilities: ProductHostCapabilityPolicy::new(
        ProductPlatformClass::HostProvided,
        ProductCapabilityRequirement::HostProvided,
        ProductCapabilityRequirement::HostProvided,
        ProductCapabilityRequirement::HostProvided,
    ),
    shutdown_policy: ProductShutdownPolicy::ExternalHost,
    artifact_manifest: ProductArtifactManifest::new(
        "zircon_embedded",
        ProductArtifactKind::Library,
        None,
        ProductArtifactDeliveryStatus::Unavailable,
    ),
};

impl ProductRoleRequest {
    pub const fn descriptor(self) -> &'static ProductRoleDescriptor {
        match self {
            Self::EditorHost => &EDITOR_HOST,
            Self::DesktopClient => &DESKTOP_CLIENT,
            Self::Server => &SERVER,
            Self::WebClient => &WEB_CLIENT,
            Self::AndroidClient => &ANDROID_CLIENT,
            Self::EditorPlayChild => &EDITOR_PLAY_CHILD,
            Self::Commandlet => &COMMANDLET,
            Self::Embedded => &EMBEDDED,
        }
    }
}
