use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};

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
