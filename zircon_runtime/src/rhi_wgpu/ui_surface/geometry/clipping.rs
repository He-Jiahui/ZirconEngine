use crate::rhi::UiSurfaceRect;

use super::{SolidVertex, quad_positions};

const SOLID_VERTEX_POSITION_EPSILON: f32 = 1.0e-6;
const MIN_SOLID_TRIANGLE_DOUBLE_AREA: f32 = 1.0e-10;

#[derive(Clone, Copy)]
enum SolidClipEdge {
    Left(f32),
    Right(f32),
    Bottom(f32),
    Top(f32),
}

pub(super) fn clip_solid_triangles_to_rect(
    vertices: Vec<SolidVertex>,
    clip_rect: UiSurfaceRect,
    size: (u32, u32),
) -> Vec<SolidVertex> {
    if vertices
        .iter()
        .any(|vertex| !solid_vertex_is_finite(*vertex))
    {
        return Vec::new();
    }
    let positions = quad_positions(clip_rect, size);
    let edges = [
        SolidClipEdge::Left(positions[0][0]),
        SolidClipEdge::Right(positions[1][0]),
        SolidClipEdge::Bottom(positions[2][1]),
        SolidClipEdge::Top(positions[0][1]),
    ];
    let mut clipped = Vec::with_capacity(vertices.len());
    let mut polygon = Vec::with_capacity(8);
    let mut scratch = Vec::with_capacity(8);
    for triangle in vertices.chunks_exact(3) {
        polygon.clear();
        polygon.extend_from_slice(triangle);
        for edge in edges {
            clip_solid_polygon_against_edge(&polygon, edge, &mut scratch);
            std::mem::swap(&mut polygon, &mut scratch);
            if polygon.is_empty() {
                break;
            }
        }
        let Some(&first) = polygon.first() else {
            continue;
        };
        for pair in polygon.get(1..).unwrap_or_default().windows(2) {
            let [second, third] = pair else {
                continue;
            };
            let triangle = [first, *second, *third];
            if solid_triangle_double_area(triangle).abs() > MIN_SOLID_TRIANGLE_DOUBLE_AREA {
                clipped.extend(triangle);
            }
        }
    }
    clipped
}

fn clip_solid_polygon_against_edge(
    vertices: &[SolidVertex],
    edge: SolidClipEdge,
    clipped: &mut Vec<SolidVertex>,
) {
    clipped.clear();
    let Some(mut previous) = vertices.last().copied() else {
        return;
    };
    let mut previous_inside = solid_vertex_inside_edge(previous, edge);
    for &current in vertices {
        let current_inside = solid_vertex_inside_edge(current, edge);
        if current_inside != previous_inside {
            if let Some(intersection) = solid_edge_intersection(previous, current, edge) {
                push_unique_solid_vertex(clipped, intersection);
            }
        }
        if current_inside {
            push_unique_solid_vertex(clipped, current);
        }
        previous = current;
        previous_inside = current_inside;
    }
    let closes_polygon = clipped
        .first()
        .zip(clipped.last())
        .is_some_and(|(first, last)| solid_vertex_positions_nearly_equal(*first, *last));
    if clipped.len() > 1 && closes_polygon {
        clipped.pop();
    }
}

fn solid_vertex_inside_edge(vertex: SolidVertex, edge: SolidClipEdge) -> bool {
    match edge {
        SolidClipEdge::Left(x) => vertex.position[0] >= x,
        SolidClipEdge::Right(x) => vertex.position[0] <= x,
        SolidClipEdge::Bottom(y) => vertex.position[1] >= y,
        SolidClipEdge::Top(y) => vertex.position[1] <= y,
    }
}

fn solid_edge_intersection(
    start: SolidVertex,
    end: SolidVertex,
    edge: SolidClipEdge,
) -> Option<SolidVertex> {
    let (axis, boundary) = match edge {
        SolidClipEdge::Left(x) | SolidClipEdge::Right(x) => (0, x),
        SolidClipEdge::Bottom(y) | SolidClipEdge::Top(y) => (1, y),
    };
    let axis_delta = end.position[axis] - start.position[axis];
    let t = if axis_delta.abs() <= f32::EPSILON {
        0.0
    } else {
        ((boundary - start.position[axis]) / axis_delta).clamp(0.0, 1.0)
    };
    let mut position = [
        start.position[0] + (end.position[0] - start.position[0]) * t,
        start.position[1] + (end.position[1] - start.position[1]) * t,
    ];
    position[axis] = boundary;
    let vertex = SolidVertex {
        position,
        color: std::array::from_fn(|index| {
            start.color[index] + (end.color[index] - start.color[index]) * t
        }),
    };
    solid_vertex_is_finite(vertex).then_some(vertex)
}

fn push_unique_solid_vertex(vertices: &mut Vec<SolidVertex>, vertex: SolidVertex) {
    if vertices
        .last()
        .is_none_or(|previous| !solid_vertex_positions_nearly_equal(*previous, vertex))
    {
        vertices.push(vertex);
    }
}

fn solid_vertex_positions_nearly_equal(left: SolidVertex, right: SolidVertex) -> bool {
    (left.position[0] - right.position[0]).abs() <= SOLID_VERTEX_POSITION_EPSILON
        && (left.position[1] - right.position[1]).abs() <= SOLID_VERTEX_POSITION_EPSILON
}

fn solid_vertex_is_finite(vertex: SolidVertex) -> bool {
    vertex.position.into_iter().all(f32::is_finite) && vertex.color.into_iter().all(f32::is_finite)
}

fn solid_triangle_double_area(triangle: [SolidVertex; 3]) -> f32 {
    let [first, second, third] = triangle.map(|vertex| vertex.position);
    (second[0] - first[0]) * (third[1] - first[1]) - (second[1] - first[1]) * (third[0] - first[0])
}
