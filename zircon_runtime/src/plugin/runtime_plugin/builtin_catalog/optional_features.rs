use super::super::RuntimePluginDescriptor;
use super::net_features::attach_net_features;
use super::particles_features::attach_particles_features;
use super::rendering_features::attach_rendering_features;
use super::sound_features::attach_sound_features;

pub(super) fn attach_optional_features(
    descriptor: RuntimePluginDescriptor,
) -> RuntimePluginDescriptor {
    match descriptor.package_id.as_str() {
        "sound" => attach_sound_features(descriptor),
        "net" => attach_net_features(descriptor),
        "particles" => attach_particles_features(descriptor),
        "rendering" => attach_rendering_features(descriptor),
        _ => descriptor,
    }
}
