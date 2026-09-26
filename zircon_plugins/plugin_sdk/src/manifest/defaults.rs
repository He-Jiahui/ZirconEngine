use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};

/// SDK 写入包清单的版本，原生包加载策略据此核对宿主兼容性。
pub const SDK_API_VERSION: &str = "0.2.0";

pub fn default_supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

pub fn default_export_packaging() -> [ExportPackagingStrategy; 2] {
    [
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
    ]
}
