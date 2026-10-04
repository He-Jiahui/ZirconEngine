use zircon_plugin_physics_runtime::PhysicsOverlayFrame;
use zircon_runtime::core::framework::physics::{PhysicsColliderShape, PhysicsWorldSyncState};
use zircon_runtime::core::framework::render::{
    OverlayLineSegment, OverlayPickShape, SceneGizmoKind, SceneGizmoOverlayExtract,
};
use zircon_runtime::core::math::{Mat4, Real, Transform, Vec3, Vec4};
use zircon_runtime::scene::EntityId;

pub const PHYSICS_OVERLAY_PROVIDER_ID: &str = "physics.viewport.overlay.provider";

const CURVE_SEGMENTS: usize = 16;
const OVERLAY_PICK_THICKNESS: Real = 0.08;
const BOX_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsOverlayColor {
    Collider,
    Trigger,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsOverlayPrimitive {
    pub entity: EntityId,
    pub shape: PhysicsColliderShape,
    pub transform: Transform,
    pub color: PhysicsOverlayColor,
}

pub fn build_physics_overlay(sync: &PhysicsWorldSyncState) -> Vec<PhysicsOverlayPrimitive> {
    sync.colliders
        .iter()
        .map(|collider| PhysicsOverlayPrimitive {
            entity: collider.entity,
            shape: collider.shape.clone(),
            transform: collider.transform,
            color: if collider.sensor {
                PhysicsOverlayColor::Trigger
            } else {
                PhysicsOverlayColor::Collider
            },
        })
        .collect()
}

pub fn build_physics_overlay_extracts(
    frame: &PhysicsOverlayFrame,
    selected: Option<EntityId>,
) -> Vec<SceneGizmoOverlayExtract> {
    frame
        .colliders
        .iter()
        .filter_map(|collider| {
            let color = overlay_color(if collider.sensor {
                PhysicsOverlayColor::Trigger
            } else {
                PhysicsOverlayColor::Collider
            });
            let mut lines = Vec::new();
            append_shape_lines(&mut lines, &collider.shape, collider.transform, color);
            (!lines.is_empty()).then(|| {
                let pick_shapes = lines
                    .iter()
                    .map(|line| OverlayPickShape::Segment {
                        start: line.start,
                        end: line.end,
                        thickness: OVERLAY_PICK_THICKNESS,
                    })
                    .collect();
                SceneGizmoOverlayExtract::new(
                    collider.entity,
                    SceneGizmoKind::Physics,
                    selected == Some(collider.entity),
                    lines,
                    Vec::new(),
                    Vec::new(),
                    pick_shapes,
                )
            })
        })
        .collect()
}

fn overlay_color(color: PhysicsOverlayColor) -> Vec4 {
    match color {
        PhysicsOverlayColor::Collider => Vec4::new(0.15, 0.72, 0.96, 0.9),
        PhysicsOverlayColor::Trigger => Vec4::new(0.96, 0.56, 0.16, 0.9),
    }
}

fn append_shape_lines(
    lines: &mut Vec<OverlayLineSegment>,
    shape: &PhysicsColliderShape,
    transform: Transform,
    color: Vec4,
) {
    let world_from_local = transform.matrix();
    match shape {
        PhysicsColliderShape::Box { half_extents } => {
            append_box(
                lines,
                world_from_local,
                Vec3::from_array(*half_extents).abs(),
                color,
            );
        }
        PhysicsColliderShape::Sphere { radius } => {
            append_sphere(lines, world_from_local, radius.abs(), color);
        }
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => {
            append_capsule(
                lines,
                world_from_local,
                radius.abs(),
                half_height.abs(),
                color,
            );
        }
        PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        } => {
            append_cylinder(
                lines,
                world_from_local,
                radius.abs(),
                half_height.abs(),
                color,
            );
        }
        PhysicsColliderShape::Compound { children } => {
            for (child_transform, child_shape) in children {
                append_shape_lines(
                    lines,
                    child_shape,
                    compose_transform(transform, *child_transform),
                    color,
                );
            }
        }
        // These variants retain asset handles or points but no resolved edge topology. Emitting
        // inferred bounds or edges here would fabricate debug geometry rather than mirror physics.
        PhysicsColliderShape::ConvexHull { .. }
        | PhysicsColliderShape::TriangleMesh { .. }
        | PhysicsColliderShape::HeightField { .. } => {}
    }
}

fn append_box(
    lines: &mut Vec<OverlayLineSegment>,
    world_from_local: Mat4,
    half_extents: Vec3,
    color: Vec4,
) {
    let vertices = [
        Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new(half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new(half_extents.x, half_extents.y, -half_extents.z),
        Vec3::new(-half_extents.x, half_extents.y, -half_extents.z),
        Vec3::new(-half_extents.x, -half_extents.y, half_extents.z),
        Vec3::new(half_extents.x, -half_extents.y, half_extents.z),
        Vec3::new(half_extents.x, half_extents.y, half_extents.z),
        Vec3::new(-half_extents.x, half_extents.y, half_extents.z),
    ];
    for (start, end) in BOX_EDGES {
        push_line(
            lines,
            world_from_local,
            vertices[start],
            vertices[end],
            color,
        );
    }
}

fn append_sphere(
    lines: &mut Vec<OverlayLineSegment>,
    world_from_local: Mat4,
    radius: Real,
    color: Vec4,
) {
    append_ring(
        lines,
        world_from_local,
        Vec3::ZERO,
        Vec3::X,
        Vec3::Y,
        radius,
        color,
    );
    append_ring(
        lines,
        world_from_local,
        Vec3::ZERO,
        Vec3::X,
        Vec3::Z,
        radius,
        color,
    );
    append_ring(
        lines,
        world_from_local,
        Vec3::ZERO,
        Vec3::Y,
        Vec3::Z,
        radius,
        color,
    );
}

fn append_cylinder(
    lines: &mut Vec<OverlayLineSegment>,
    world_from_local: Mat4,
    radius: Real,
    half_height: Real,
    color: Vec4,
) {
    if radius <= 0.0 {
        return;
    }
    let lower = Vec3::Y * -half_height;
    let upper = Vec3::Y * half_height;
    append_ring(
        lines,
        world_from_local,
        lower,
        Vec3::X,
        Vec3::Z,
        radius,
        color,
    );
    append_ring(
        lines,
        world_from_local,
        upper,
        Vec3::X,
        Vec3::Z,
        radius,
        color,
    );
    for radial in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
        push_line(
            lines,
            world_from_local,
            lower + radial * radius,
            upper + radial * radius,
            color,
        );
    }
}

fn append_capsule(
    lines: &mut Vec<OverlayLineSegment>,
    world_from_local: Mat4,
    radius: Real,
    half_height: Real,
    color: Vec4,
) {
    if radius <= 0.0 {
        return;
    }
    let lower = Vec3::Y * -half_height;
    let upper = Vec3::Y * half_height;
    append_ring(
        lines,
        world_from_local,
        lower,
        Vec3::X,
        Vec3::Z,
        radius,
        color,
    );
    append_ring(
        lines,
        world_from_local,
        upper,
        Vec3::X,
        Vec3::Z,
        radius,
        color,
    );
    for horizontal_axis in [Vec3::X, Vec3::Z] {
        append_arc(
            lines,
            world_from_local,
            upper,
            horizontal_axis,
            radius,
            1.0,
            color,
        );
        append_arc(
            lines,
            world_from_local,
            lower,
            horizontal_axis,
            radius,
            -1.0,
            color,
        );
    }
}

fn append_ring(
    lines: &mut Vec<OverlayLineSegment>,
    world_from_local: Mat4,
    center: Vec3,
    first_axis: Vec3,
    second_axis: Vec3,
    radius: Real,
    color: Vec4,
) {
    if radius <= 0.0 {
        return;
    }
    let mut first = None;
    let mut previous = None;
    for index in 0..CURVE_SEGMENTS {
        let radians = std::f32::consts::TAU * index as Real / CURVE_SEGMENTS as Real;
        let point =
            center + first_axis * (radius * radians.cos()) + second_axis * (radius * radians.sin());
        if let Some(previous) = previous {
            push_line(lines, world_from_local, previous, point, color);
        } else {
            first = Some(point);
        }
        previous = Some(point);
    }
    if let (Some(first), Some(previous)) = (first, previous) {
        push_line(lines, world_from_local, previous, first, color);
    }
}

fn append_arc(
    lines: &mut Vec<OverlayLineSegment>,
    world_from_local: Mat4,
    center: Vec3,
    horizontal_axis: Vec3,
    radius: Real,
    vertical_sign: Real,
    color: Vec4,
) {
    let half_segments = CURVE_SEGMENTS / 2;
    let mut previous = None;
    for index in 0..=half_segments {
        let radians = std::f32::consts::PI * index as Real / half_segments as Real;
        let point = center
            + horizontal_axis * (radius * radians.cos())
            + Vec3::Y * (vertical_sign * radius * radians.sin());
        if let Some(previous) = previous {
            push_line(lines, world_from_local, previous, point, color);
        }
        previous = Some(point);
    }
}

fn push_line(
    lines: &mut Vec<OverlayLineSegment>,
    world_from_local: Mat4,
    start: Vec3,
    end: Vec3,
    color: Vec4,
) {
    lines.push(OverlayLineSegment {
        start: world_from_local.transform_point3(start),
        end: world_from_local.transform_point3(end),
        color,
    });
}

fn compose_transform(parent: Transform, local: Transform) -> Transform {
    Transform {
        translation: parent.translation + parent.rotation * (parent.scale * local.translation),
        rotation: parent.rotation * local.rotation,
        scale: parent.scale * local.scale,
    }
}
