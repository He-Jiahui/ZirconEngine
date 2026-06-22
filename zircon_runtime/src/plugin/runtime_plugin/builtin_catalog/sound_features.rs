mod manifest;
mod rows;

use super::BuiltinCatalogDescriptorBuilder;
use manifest::sound_feature;
use rows::SOUND_FEATURE_ROWS;

pub(super) fn attach_sound_features(
    descriptor: BuiltinCatalogDescriptorBuilder,
) -> BuiltinCatalogDescriptorBuilder {
    SOUND_FEATURE_ROWS
        .iter()
        .fold(descriptor, |descriptor, row| {
            descriptor.with_optional_feature(sound_feature(row))
        })
}
