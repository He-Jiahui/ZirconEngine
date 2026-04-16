use zircon_math::Transform;

pub(in crate::editing::viewport::pointer) fn renderable_pick_radius(transform: Transform) -> f32 {
    let extent = transform
        .scale
        .x
        .abs()
        .max(transform.scale.y.abs())
        .max(transform.scale.z.abs());
    (extent * 0.75).clamp(0.45, 2.5)
}
