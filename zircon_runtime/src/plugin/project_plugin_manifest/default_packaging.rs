use crate::plugin::ExportPackagingStrategy;

pub(super) fn default_packaging() -> ExportPackagingStrategy {
    ExportPackagingStrategy::LibraryEmbed
}
