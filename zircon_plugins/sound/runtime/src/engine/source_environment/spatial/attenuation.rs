use zircon_runtime::core::framework::sound::SoundAttenuationMode;

pub(super) fn attenuation_gain(
    distance: f32,
    min_distance: f32,
    max_distance: f32,
    mode: SoundAttenuationMode,
) -> f32 {
    if matches!(mode, SoundAttenuationMode::None) {
        return 1.0;
    }

    let min_distance = min_distance.max(0.0001);
    let max_distance = max_distance.max(min_distance);
    if distance <= min_distance {
        return 1.0;
    }
    if distance >= max_distance {
        return 0.0;
    }

    match mode {
        SoundAttenuationMode::None => 1.0,
        SoundAttenuationMode::Linear => {
            1.0 - ((distance - min_distance) / (max_distance - min_distance).max(0.0001))
        }
        SoundAttenuationMode::InverseDistance => min_distance / distance.max(min_distance),
        SoundAttenuationMode::InverseDistanceSquared => {
            (min_distance / distance.max(min_distance)).powi(2)
        }
    }
    .clamp(0.0, 1.0)
}
