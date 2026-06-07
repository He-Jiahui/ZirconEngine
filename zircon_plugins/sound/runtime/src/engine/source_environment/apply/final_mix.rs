use super::super::spatial;

pub(super) fn apply_final_gain_and_pan(buffer: &mut [f32], channels: usize, gain: f32, pan: f32) {
    if gain != 1.0 {
        for sample in buffer.iter_mut() {
            *sample *= gain;
        }
    }
    spatial::apply_source_pan(buffer, channels, pan);
}
