use crate::ExportPackagingStrategy;

pub(super) fn default_packaging() -> ExportPackagingStrategy {
    ExportPackagingStrategy::LibraryEmbed
}
