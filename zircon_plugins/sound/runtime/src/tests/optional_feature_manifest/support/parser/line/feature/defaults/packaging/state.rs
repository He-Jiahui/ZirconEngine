use super::super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn set_default_packaging(
    feature: &mut PendingOptionalFeatureManifest,
    values: Vec<zircon_runtime::core::framework::project::ExportPackagingStrategy>,
) {
    feature.default_packaging = values;
}
