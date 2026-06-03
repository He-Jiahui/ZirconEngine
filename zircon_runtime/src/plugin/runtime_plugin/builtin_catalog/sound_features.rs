mod manifest;
mod rows;

use super::super::RuntimePluginDescriptor;
use manifest::sound_feature;
use rows::SOUND_FEATURE_ROWS;

pub(super) fn attach_sound_features(
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    SOUND_FEATURE_ROWS
        .iter()
        .fold(descriptor, |descriptor, row| {
            descriptor.with_optional_feature(sound_feature(row))
        })
}
