pub(super) fn waveshape(buffer: &mut [f32], drive: f32) {
    let drive = drive.max(0.0) + 1.0;
    let normalizer = drive.tanh().max(0.0001);
    for sample in buffer {
        *sample = (*sample * drive).tanh() / normalizer;
    }
}
