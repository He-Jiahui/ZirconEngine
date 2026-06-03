mod manifest;
mod rows;

use super::super::RuntimePluginDescriptor;
use manifest::rendering_feature;
use rows::RENDERING_FEATURE_ROWS;

pub(super) fn attach_rendering_features(
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    RENDERING_FEATURE_ROWS
        .iter()
        .fold(descriptor, |descriptor, row| {
            descriptor.with_optional_feature(rendering_feature(row))
        })
}
