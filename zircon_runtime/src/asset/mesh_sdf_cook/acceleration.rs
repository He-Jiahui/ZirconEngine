use crate::asset::{MeshSdfCookError, MeshVertex};
use crate::core::math::Vec3;

use super::distance::{point_triangle_distance_squared, positive_x_ray_intersects_triangle};

const BVH_LEAF_TRIANGLE_COUNT: usize = 8;
const DEGENERATE_TRIANGLE_AREA_SQUARED: f32 = 1.0e-16;

#[derive(Clone, Copy, Debug)]
pub(super) struct Aabb {
    pub(super) min: Vec3,
    pub(super) max: Vec3,
}

impl Aabb {
    fn from_triangle(triangle: Triangle) -> Self {
        Self {
            min: triangle.a.min(triangle.b).min(triangle.c),
            max: triangle.a.max(triangle.b).max(triangle.c),
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    fn distance_squared(self, point: Vec3) -> f32 {
        let below = (self.min - point).max(Vec3::ZERO);
        let above = (point - self.max).max(Vec3::ZERO);
        (below + above).length_squared()
    }

    fn intersects_positive_x_ray(self, origin: Vec3) -> bool {
        self.max.x >= origin.x
            && (self.min.y..=self.max.y).contains(&origin.y)
            && (self.min.z..=self.max.z).contains(&origin.z)
    }
}

#[derive(Clone, Copy, Debug)]
struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3,
    centroid: Vec3,
}

impl Triangle {
    fn new(a: Vec3, b: Vec3, c: Vec3) -> Option<Self> {
        ((b - a).cross(c - a).length_squared() > DEGENERATE_TRIANGLE_AREA_SQUARED).then_some(Self {
            a,
            b,
            c,
            centroid: (a + b + c) / 3.0,
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum BvhNodeContent {
    Leaf { first: usize, count: usize },
    Branch { left: usize, right: usize },
}

#[derive(Clone, Copy, Debug)]
struct BvhNode {
    bounds: Aabb,
    content: BvhNodeContent,
}

pub(super) struct TriangleBvh {
    triangles: Vec<Triangle>,
    triangle_order: Vec<usize>,
    nodes: Vec<BvhNode>,
    root: usize,
    source_bounds: Aabb,
}

impl TriangleBvh {
    pub(super) fn build(
        vertices: &[MeshVertex],
        indices: &[u32],
    ) -> Result<Self, MeshSdfCookError> {
        if vertices.is_empty() || indices.is_empty() {
            return Err(MeshSdfCookError::EmptyGeometry);
        }
        if indices.len() % 3 != 0 {
            return Err(MeshSdfCookError::InvalidTriangleIndexCount);
        }
        if vertices
            .iter()
            .any(|vertex| !Vec3::from_array(vertex.position).is_finite())
        {
            return Err(MeshSdfCookError::NonFinitePosition);
        }
        if indices
            .iter()
            .any(|index| usize::try_from(*index).map_or(true, |index| index >= vertices.len()))
        {
            return Err(MeshSdfCookError::IndexOutOfRange);
        }

        let triangles = indices
            .chunks_exact(3)
            .filter_map(|triangle| {
                Triangle::new(
                    Vec3::from_array(vertices[triangle[0] as usize].position),
                    Vec3::from_array(vertices[triangle[1] as usize].position),
                    Vec3::from_array(vertices[triangle[2] as usize].position),
                )
            })
            .collect::<Vec<_>>();
        if triangles.is_empty() {
            return Err(MeshSdfCookError::DegenerateGeometry);
        }
        let source_bounds = triangles
            .iter()
            .copied()
            .map(Aabb::from_triangle)
            .reduce(Aabb::union)
            .ok_or(MeshSdfCookError::DegenerateGeometry)?;
        let mut triangle_order = (0..triangles.len()).collect::<Vec<_>>();
        let mut nodes = Vec::with_capacity(triangles.len().saturating_mul(2));
        let triangle_order_len = triangle_order.len();
        let root = build_node(
            &triangles,
            &mut triangle_order,
            0,
            triangle_order_len,
            &mut nodes,
        );
        Ok(Self {
            triangles,
            triangle_order,
            nodes,
            root,
            source_bounds,
        })
    }

    pub(super) fn source_bounds(&self) -> Aabb {
        self.source_bounds
    }

    pub(super) fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub(super) fn nearest_distance_squared(&self, point: Vec3) -> f32 {
        self.nearest_distance_squared_in_node(self.root, point, f32::INFINITY)
    }

    pub(super) fn positive_x_intersection_count(&self, origin: Vec3) -> u32 {
        self.positive_x_intersections_in_node(self.root, origin)
    }

    fn nearest_distance_squared_in_node(
        &self,
        node_index: usize,
        point: Vec3,
        mut best: f32,
    ) -> f32 {
        let node = self.nodes[node_index];
        if node.bounds.distance_squared(point) >= best {
            return best;
        }
        match node.content {
            BvhNodeContent::Leaf { first, count } => {
                for triangle_index in &self.triangle_order[first..first + count] {
                    let triangle = self.triangles[*triangle_index];
                    best = best.min(point_triangle_distance_squared(
                        point, triangle.a, triangle.b, triangle.c,
                    ));
                }
                best
            }
            BvhNodeContent::Branch { left, right } => {
                let left_distance = self.nodes[left].bounds.distance_squared(point);
                let right_distance = self.nodes[right].bounds.distance_squared(point);
                let (near, far) = if left_distance <= right_distance {
                    (left, right)
                } else {
                    (right, left)
                };
                best = self.nearest_distance_squared_in_node(near, point, best);
                self.nearest_distance_squared_in_node(far, point, best)
            }
        }
    }

    fn positive_x_intersections_in_node(&self, node_index: usize, origin: Vec3) -> u32 {
        let node = self.nodes[node_index];
        if !node.bounds.intersects_positive_x_ray(origin) {
            return 0;
        }
        match node.content {
            BvhNodeContent::Leaf { first, count } => self.triangle_order[first..first + count]
                .iter()
                .filter(|triangle_index| {
                    let triangle = self.triangles[**triangle_index];
                    positive_x_ray_intersects_triangle(origin, triangle.a, triangle.b, triangle.c)
                })
                .count()
                .try_into()
                .unwrap_or(u32::MAX),
            BvhNodeContent::Branch { left, right } => self
                .positive_x_intersections_in_node(left, origin)
                .saturating_add(self.positive_x_intersections_in_node(right, origin)),
        }
    }
}

fn build_node(
    triangles: &[Triangle],
    triangle_order: &mut [usize],
    first: usize,
    count: usize,
    nodes: &mut Vec<BvhNode>,
) -> usize {
    debug_assert!(count > 0);
    let first_triangle_index = triangle_order[first];
    let bounds = triangle_order[first + 1..first + count].iter().fold(
        Aabb::from_triangle(triangles[first_triangle_index]),
        |bounds, index| bounds.union(Aabb::from_triangle(triangles[*index])),
    );
    let node_index = nodes.len();
    nodes.push(BvhNode {
        bounds,
        content: BvhNodeContent::Leaf { first, count },
    });
    if count <= BVH_LEAF_TRIANGLE_COUNT {
        return node_index;
    }

    let first_centroid = triangles[first_triangle_index].centroid;
    let centroid_bounds = triangle_order[first + 1..first + count].iter().fold(
        Aabb {
            min: first_centroid,
            max: first_centroid,
        },
        |bounds, index| {
            let centroid = triangles[*index].centroid;
            bounds.union(Aabb {
                min: centroid,
                max: centroid,
            })
        },
    );
    let extent = centroid_bounds.max - centroid_bounds.min;
    let axis = if extent.x >= extent.y && extent.x >= extent.z {
        0
    } else if extent.y >= extent.z {
        1
    } else {
        2
    };
    triangle_order[first..first + count].sort_unstable_by(|left, right| {
        triangles[*left].centroid[axis].total_cmp(&triangles[*right].centroid[axis])
    });
    let left_count = count / 2;
    let right_count = count - left_count;
    let left = build_node(triangles, triangle_order, first, left_count, nodes);
    let right = build_node(
        triangles,
        triangle_order,
        first + left_count,
        right_count,
        nodes,
    );
    nodes[node_index].content = BvhNodeContent::Branch { left, right };
    node_index
}
