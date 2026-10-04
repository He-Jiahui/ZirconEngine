use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportProfile, ExportTargetPlatform, RuntimeProfileId,
};

/// Product target requested from the App host layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductRoleRequest {
    EditorHost,
    DesktopClient,
    Server,
    WebClient,
    AndroidClient,
    EditorPlayChild,
    Commandlet,
    Embedded,
}

impl ProductRoleRequest {
    pub const ALL: [Self; 8] = [
        Self::EditorHost,
        Self::DesktopClient,
        Self::Server,
        Self::WebClient,
        Self::AndroidClient,
        Self::EditorPlayChild,
        Self::Commandlet,
        Self::Embedded,
    ];

    /// 兼容既有 profile 入口；细分子进程和平台宿主时应显式传入产品角色。
    pub(super) const fn from_entry_profile(profile: crate::entry::EntryProfile) -> Self {
        match profile {
            crate::entry::EntryProfile::Editor => Self::EditorHost,
            crate::entry::EntryProfile::Runtime => Self::DesktopClient,
            crate::entry::EntryProfile::Headless => Self::Server,
        }
    }

    pub(super) const fn from_runtime_profile(profile: RuntimeProfileId) -> Self {
        match profile {
            RuntimeProfileId::Editor | RuntimeProfileId::Dev => Self::EditorHost,
            RuntimeProfileId::Server => Self::Server,
            RuntimeProfileId::Minimal | RuntimeProfileId::Client2d | RuntimeProfileId::Client3d => {
                Self::DesktopClient
            }
        }
    }

    /// 导出包装层先归类宿主形态，随后由 resolve 拒绝当前尚无配置所有者的角色。
    pub(in crate::entry) const fn from_export_profile(profile: &ExportProfile) -> Self {
        match profile.target_platform {
            ExportTargetPlatform::Android => Self::AndroidClient,
            ExportTargetPlatform::WebGpu | ExportTargetPlatform::Wasm => Self::WebClient,
            // The generated iOS Rust library is embedded by the Swift host. Embedded remains
            // unsupported until that host has its own capability admission contract.
            ExportTargetPlatform::Ios => Self::Embedded,
            ExportTargetPlatform::Headless => match profile.target_mode {
                RuntimeTargetMode::ServerRuntime => Self::Server,
                RuntimeTargetMode::EditorHost | RuntimeTargetMode::ClientRuntime => Self::Embedded,
            },
            ExportTargetPlatform::Windows
            | ExportTargetPlatform::Linux
            | ExportTargetPlatform::Macos => match profile.target_mode {
                RuntimeTargetMode::EditorHost => Self::EditorHost,
                RuntimeTargetMode::ServerRuntime => Self::Server,
                RuntimeTargetMode::ClientRuntime => Self::DesktopClient,
            },
        }
    }
}
