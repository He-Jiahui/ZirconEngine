use zircon_runtime::core::framework::sound::SoundVolumeDescriptor;

use super::weight::volume_weight;

#[derive(Clone, Copy, Debug)]
pub(in crate::engine::source_environment) struct VolumeInfluence<'a> {
    pub(in crate::engine::source_environment) descriptor: &'a SoundVolumeDescriptor,
    pub(in crate::engine::source_environment) weight: f32,
}

impl VolumeInfluence<'_> {
    pub(in crate::engine::source_environment) fn gain(self) -> f32 {
        self.descriptor.exterior_gain
            + (self.descriptor.interior_gain - self.descriptor.exterior_gain) * self.weight
    }
}

pub(in crate::engine::source_environment) fn strongest_volume_influence(
    source_position: [f32; 3],
    volumes: &[SoundVolumeDescriptor],
) -> Option<VolumeInfluence<'_>> {
    volumes
        .iter()
        .filter_map(|volume| {
            let weight = volume_weight(source_position, volume);
            (weight > 0.0).then_some(VolumeInfluence {
                descriptor: volume,
                weight,
            })
        })
        .max_by(|a, b| {
            a.descriptor
                .priority
                .cmp(&b.descriptor.priority)
                .then_with(|| b.descriptor.id.raw().cmp(&a.descriptor.id.raw()))
        })
}
