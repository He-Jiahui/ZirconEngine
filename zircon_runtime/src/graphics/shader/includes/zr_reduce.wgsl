// Scalar reduction baseline shared by compute kernels. A future subgroup specialization must
// preserve these functions' workgroup-level semantics rather than changing call-site topology.
fn zr_reduce_min_f32(a: f32, b: f32) -> f32 {
    return min(a, b);
}

fn zr_reduce_max_f32(a: f32, b: f32) -> f32 {
    return max(a, b);
}

fn zr_reduce_add_f32(a: f32, b: f32) -> f32 {
    return a + b;
}
