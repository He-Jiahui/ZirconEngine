pub(in crate::engine::render) fn add_scaled(destination: &mut [f32], source: &[f32], gain: f32) {
    for (destination, source) in destination.iter_mut().zip(source.iter().copied()) {
        *destination += source * gain;
    }
}
