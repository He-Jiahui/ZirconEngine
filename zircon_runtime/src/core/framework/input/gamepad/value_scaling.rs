pub(super) fn linear_remapping(
    value: f32,
    old_start: f32,
    old_end: f32,
    new_start: f32,
    new_end: f32,
) -> f32 {
    ((value - old_start) / (old_end - old_start)) * (new_end - new_start) + new_start
}
