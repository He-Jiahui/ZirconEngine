use super::net_features::attach_net_features;
use super::particles_features::attach_particles_features;
use super::rendering_features::attach_rendering_features;
use super::sound_features::attach_sound_features;
use super::IdentifiedBuiltinCatalogDescriptorBuilder;

// 可选特性只附着到其所属基础包；实际启用仍由项目选择和具体提供者决定。
pub(super) fn attach_optional_features(
    (package_id, descriptor): IdentifiedBuiltinCatalogDescriptorBuilder,
) -> IdentifiedBuiltinCatalogDescriptorBuilder {
    let descriptor = match package_id {
        "sound" => attach_sound_features(descriptor),
        "net" => attach_net_features(descriptor),
        "particles" => attach_particles_features(descriptor),
        "rendering" => attach_rendering_features(descriptor),
        _ => descriptor,
    };
    (package_id, descriptor)
}
