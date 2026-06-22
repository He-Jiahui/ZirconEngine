mod manifest;
mod rows;

use super::BuiltinCatalogDescriptorBuilder;
use manifest::rendering_feature;
use rows::RENDERING_FEATURE_ROWS;

pub(super) fn attach_rendering_features(
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    RENDERING_FEATURE_ROWS
        .iter()
        .fold(descriptor, |descriptor, row| {
            descriptor.with_optional_feature(rendering_feature(row))
        })
}
