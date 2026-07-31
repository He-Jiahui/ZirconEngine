use crate::core::{
    framework::{
        render::{
            OverlayLineSegment, RenderOverlayExtract, SceneGizmoKind, SceneGizmoOverlayExtract,
        },
        scene::EntityId,
    },
    math::{Mat4, Real, Transform, Vec3, Vec4},
};

use super::{GizmoAxis, GizmoBuffer, GizmoColorPolicy, GizmoCommand, RetainedGizmo};

const DEFAULT_CIRCLE_SEGMENTS: usize = 32;
const MIN_CIRCLE_SEGMENTS: usize = 3;
const CIRCLE_BASIS_AXIS_DOT_LIMIT: Real = 0.9;

pub struct GizmoOverlayExtractRequest<'a> {
    pub owner: EntityId,
    pub kind: SceneGizmoKind,
    pub selected: bool,
    pub buffers: Vec<&'a GizmoBuffer>,
    pub retained: Vec<&'a RetainedGizmo>,
}

impl<'a> GizmoOverlayExtractRequest<'a> {
    pub fn new(owner: EntityId, kind: SceneGizmoKind) -> Self {
        Self {
            owner,
            kind,
            selected: false,
            buffers: Vec::new(),
            retained: Vec::new(),
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn with_buffer(mut self, buffer: &'a GizmoBuffer) -> Self {
        self.buffers.push(buffer);
        self
    }

    pub fn with_retained(mut self, retained: &'a RetainedGizmo) -> Self {
        self.retained.push(retained);
        self
    }
}

pub fn extract_gizmo_overlay(
    request: GizmoOverlayExtractRequest<'_>,
) -> Option<SceneGizmoOverlayExtract> {
    let mut lines = Vec::new();
    for buffer in request.buffers {
        if buffer.config().enabled {
            push_commands(
                &mut lines,
                buffer.commands(),
                Transform::identity(),
                buffer.config().color_policy,
            );
        }
    }

    for retained in request.retained {
        if retained.config.enabled {
            push_commands(
                &mut lines,
                retained.asset.commands(),
                retained.transform,
                retained.config.color_policy,
            );
        }
    }

    (!lines.is_empty()).then(|| {
        SceneGizmoOverlayExtract::new(
            request.owner,
            request.kind,
            request.selected,
            lines,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    })
}

pub fn append_gizmo_overlay(
    packet: &mut RenderOverlayExtract,
    request: GizmoOverlayExtractRequest<'_>,
) {
    if let Some(overlay) = extract_gizmo_overlay(request) {
        packet.scene_gizmos.push(overlay);
    }
}

fn push_commands(
    lines: &mut Vec<OverlayLineSegment>,
    commands: &[GizmoCommand],
    transform: Transform,
    color_policy: GizmoColorPolicy,
) {
    lines.reserve(commands.iter().map(estimated_line_count).sum());
    let transform = transform.matrix();
    for command in commands {
        push_command(lines, command, transform, color_policy);
    }
}

fn estimated_line_count(command: &GizmoCommand) -> usize {
    match command {
        GizmoCommand::Line { .. } | GizmoCommand::Ray { .. } | GizmoCommand::Axis { .. } => 1,
        GizmoCommand::LineStrip { points, .. } => points.len().saturating_sub(1),
        GizmoCommand::Rect { .. } => 4,
        GizmoCommand::Circle { radius, .. } => {
            if *radius <= 0.0 {
                0
            } else {
                DEFAULT_CIRCLE_SEGMENTS
            }
        }
        GizmoCommand::Sphere { radius, .. } => {
            if *radius <= 0.0 {
                0
            } else {
                DEFAULT_CIRCLE_SEGMENTS * 3
            }
        }
        GizmoCommand::Cube { .. } | GizmoCommand::Aabb { .. } => 12,
    }
}

fn push_command(
    lines: &mut Vec<OverlayLineSegment>,
    command: &GizmoCommand,
    transform: Mat4,
    color_policy: GizmoColorPolicy,
) {
    match command {
        GizmoCommand::Line { start, end, color } => push_line(
            lines,
            transform_point(transform, *start),
            transform_point(transform, *end),
            color_policy.apply(*color),
        ),
        GizmoCommand::Ray {
            start,
            vector,
            color,
        } => push_line(
            lines,
            transform_point(transform, *start),
            transform_point(transform, *start + *vector),
            color_policy.apply(*color),
        ),
        GizmoCommand::LineStrip { points, color } => push_linestrip(
            lines,
            points
                .iter()
                .map(|point| transform_point(transform, *point)),
            color_policy.apply(*color),
        ),
        GizmoCommand::Rect {
            transform: rect_transform,
            size,
            color,
        } => push_rect(
            lines,
            transform * rect_transform.matrix(),
            *size,
            color_policy.apply(*color),
        ),
        GizmoCommand::Circle {
            center,
            normal,
            radius,
            color,
        } => push_circle(
            lines,
            transform_point(transform, *center),
            transform_vector(transform, *normal),
            *radius,
            color_policy.apply(*color),
        ),
        GizmoCommand::Sphere {
            center,
            radius,
            color,
        } => push_sphere(
            lines,
            transform_point(transform, *center),
            *radius,
            color_policy.apply(*color),
        ),
        GizmoCommand::Cube {
            transform: cube_transform,
            size,
            color,
        } => push_cube(
            lines,
            transform * cube_transform.matrix(),
            *size,
            color_policy.apply(*color),
        ),
        GizmoCommand::Aabb { min, max, color } => push_aabb(
            lines,
            transform_point(transform, *min),
            transform_point(transform, *max),
            color_policy.apply(*color),
        ),
        GizmoCommand::Axis {
            origin,
            axis,
            length,
            color,
        } => push_axis(
            lines,
            transform_point(transform, *origin),
            *axis,
            *length,
            color_policy.apply(*color),
        ),
    }
}

fn push_line(lines: &mut Vec<OverlayLineSegment>, start: Vec3, end: Vec3, color: Vec4) {
    lines.push(OverlayLineSegment { start, end, color });
}

fn push_linestrip(
    lines: &mut Vec<OverlayLineSegment>,
    points: impl IntoIterator<Item = Vec3>,
    color: Vec4,
) {
    let mut previous = None;
    for point in points {
        if let Some(start) = previous {
            push_line(lines, start, point, color);
        }
        previous = Some(point);
    }
}

fn push_rect(
    lines: &mut Vec<OverlayLineSegment>,
    transform: Mat4,
    size: crate::core::math::Vec2,
    color: Vec4,
) {
    let half = size * 0.5;
    let corners = [
        Vec3::new(-half.x, -half.y, 0.0),
        Vec3::new(half.x, -half.y, 0.0),
        Vec3::new(half.x, half.y, 0.0),
        Vec3::new(-half.x, half.y, 0.0),
    ]
    .map(|point| transform_point(transform, point));
    push_loop(lines, &corners, color);
}

fn push_circle(
    lines: &mut Vec<OverlayLineSegment>,
    center: Vec3,
    normal: Vec3,
    radius: Real,
    color: Vec4,
) {
    push_circle_segments(
        lines,
        center,
        normal,
        radius,
        color,
        DEFAULT_CIRCLE_SEGMENTS,
    );
}

fn push_sphere(lines: &mut Vec<OverlayLineSegment>, center: Vec3, radius: Real, color: Vec4) {
    push_circle_segments(
        lines,
        center,
        Vec3::X,
        radius,
        color,
        DEFAULT_CIRCLE_SEGMENTS,
    );
    push_circle_segments(
        lines,
        center,
        Vec3::Y,
        radius,
        color,
        DEFAULT_CIRCLE_SEGMENTS,
    );
    push_circle_segments(
        lines,
        center,
        Vec3::Z,
        radius,
        color,
        DEFAULT_CIRCLE_SEGMENTS,
    );
}

fn push_circle_segments(
    lines: &mut Vec<OverlayLineSegment>,
    center: Vec3,
    normal: Vec3,
    radius: Real,
    color: Vec4,
    segments: usize,
) {
    if radius <= 0.0 {
        return;
    }
    let segments = segments.max(MIN_CIRCLE_SEGMENTS);
    let (tangent, bitangent) = circle_basis(normal);
    let mut first = None;
    let mut previous = None;
    for index in 0..segments {
        let angle = std::f32::consts::TAU * (index as Real / segments as Real);
        let point = center + radius * (angle.cos() * tangent + angle.sin() * bitangent);
        if let Some(previous) = previous {
            push_line(lines, previous, point, color);
        } else {
            first = Some(point);
        }
        previous = Some(point);
    }
    if let (Some(previous), Some(first)) = (previous, first) {
        push_line(lines, previous, first, color);
    }
}

fn circle_basis(normal: Vec3) -> (Vec3, Vec3) {
    let normal = normal.normalize_or_zero();
    if normal == Vec3::ZERO {
        return (Vec3::X, Vec3::Y);
    }
    let seed = if normal.x.abs() < CIRCLE_BASIS_AXIS_DOT_LIMIT {
        Vec3::X
    } else {
        Vec3::Y
    };
    let tangent = normal.cross(seed).normalize_or_zero();
    let bitangent = normal.cross(tangent).normalize_or_zero();
    (tangent, bitangent)
}

fn push_cube(lines: &mut Vec<OverlayLineSegment>, transform: Mat4, size: Vec3, color: Vec4) {
    let half = size * 0.5;
    let corners = [
        Vec3::new(-half.x, -half.y, -half.z),
        Vec3::new(half.x, -half.y, -half.z),
        Vec3::new(half.x, half.y, -half.z),
        Vec3::new(-half.x, half.y, -half.z),
        Vec3::new(-half.x, -half.y, half.z),
        Vec3::new(half.x, -half.y, half.z),
        Vec3::new(half.x, half.y, half.z),
        Vec3::new(-half.x, half.y, half.z),
    ]
    .map(|point| transform_point(transform, point));
    push_box_edges(lines, &corners, color);
}

fn push_aabb(lines: &mut Vec<OverlayLineSegment>, min: Vec3, max: Vec3, color: Vec4) {
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(min.x, max.y, max.z),
    ];
    push_box_edges(lines, &corners, color);
}

fn push_box_edges(lines: &mut Vec<OverlayLineSegment>, corners: &[Vec3; 8], color: Vec4) {
    push_loop(lines, &corners[0..4], color);
    push_loop(lines, &corners[4..8], color);
    push_line(lines, corners[0], corners[4], color);
    push_line(lines, corners[1], corners[5], color);
    push_line(lines, corners[2], corners[6], color);
    push_line(lines, corners[3], corners[7], color);
}

fn push_axis(
    lines: &mut Vec<OverlayLineSegment>,
    origin: Vec3,
    axis: GizmoAxis,
    length: Real,
    color: Vec4,
) {
    push_line(lines, origin, origin + axis.direction() * length, color);
}

fn push_loop(lines: &mut Vec<OverlayLineSegment>, points: &[Vec3], color: Vec4) {
    if points.len() < 2 {
        return;
    }
    push_linestrip(
        lines,
        points.iter().copied().chain(points.first().copied()),
        color,
    );
}

fn transform_point(transform: Mat4, point: Vec3) -> Vec3 {
    transform.transform_point3(point)
}

fn transform_vector(transform: Mat4, vector: Vec3) -> Vec3 {
    transform.transform_vector3(vector)
}
