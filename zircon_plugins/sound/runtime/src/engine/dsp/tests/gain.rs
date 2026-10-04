pub(super) fn multiply(buffer: &mut [f32], gain: f32) {
    for sample in buffer {
        *sample *= gain;
    }
}

pub(super) fn wet_mix(buffer: &mut [f32], dry: &[f32], wet: f32) {
    if wet >= 1.0 {
        return;
    }
    let dry_amount = 1.0 - wet;
    for (sample, dry_sample) in buffer.iter_mut().zip(dry.iter().copied()) {
        *sample = *sample * wet + dry_sample * dry_amount;
    }
}
