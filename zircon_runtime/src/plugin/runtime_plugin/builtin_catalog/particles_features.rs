mod manifest;
mod rows;

use super::super::RuntimePluginDescriptor;
use manifest::particles_feature;
use rows::PARTICLES_FEATURE_ROWS;

pub(super) fn attach_particles_features(
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    PARTICLES_FEATURE_ROWS
        .iter()
        .fold(descriptor, |descriptor, row| {
            descriptor.with_optional_feature(particles_feature(row))
        })
}
