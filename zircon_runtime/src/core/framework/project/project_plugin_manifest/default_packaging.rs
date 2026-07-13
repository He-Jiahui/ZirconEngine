use super::super::ExportPackagingStrategy;

pub(super) fn default_packaging() -> ExportPackagingStrategy {
    ExportPackagingStrategy::LibraryEmbed
}
