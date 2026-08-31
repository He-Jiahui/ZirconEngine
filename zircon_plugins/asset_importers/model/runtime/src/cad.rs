use std::io::Cursor;

use dxf::{
    entities::{EntityType, Face3D, Polyline, Solid, Trace},
    Drawing, Point,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

use crate::{model_outcome, primitive_from_indexed_mesh};

pub(crate) fn import_dxf_model(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let drawing =
        Drawing::load(&mut Cursor::new(context.source_bytes.as_slice())).map_err(|error| {
            AssetImportError::Parse(format!(
                "parse dxf {}: {error}",
                context.source_path.display()
            ))
        })?;
    let mut builder = DxfMeshBuilder::new(context);
    for entity in drawing.entities() {
        match &entity.specific {
            EntityType::Face3D(face) => builder.push_face3d(face)?,
            EntityType::Solid(solid) => builder.push_solid(solid)?,
            EntityType::Trace(trace) => builder.push_trace(trace)?,
            EntityType::Polyline(polyline) => builder.push_polyface(polyline)?,
            _ => {}
        }
    }
    if builder.indices.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "parse dxf {}: file contains no supported mesh faces (3DFACE, SOLID, TRACE, or POLYLINE polyface mesh)",
            context.source_path.display()
        )));
    }

    let source_hint = context.uri.to_string();
    let primitive = primitive_from_indexed_mesh(
        &builder.positions,
        &[],
        &[],
        &builder.indices,
        context
            .source_path
            .file_stem()
            .and_then(|stem| stem.to_str()),
        &source_hint,
        context.mesh_sdf_cook_request()?.settings(),
    )?;

    model_outcome(context, vec![primitive])
}

struct DxfMeshBuilder<'a> {
    context: &'a AssetImportContext,
    positions: Vec<f32>,
    indices: Vec<u32>,
}

impl<'a> DxfMeshBuilder<'a> {
    fn new(context: &'a AssetImportContext) -> Self {
        Self {
            context,
            positions: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn push_face3d(&mut self, face: &Face3D) -> Result<(), AssetImportError> {
        self.push_surface([
            &face.first_corner,
            &face.second_corner,
            &face.third_corner,
            &face.fourth_corner,
        ])
    }

    fn push_solid(&mut self, solid: &Solid) -> Result<(), AssetImportError> {
        self.push_surface([
            &solid.first_corner,
            &solid.second_corner,
            &solid.third_corner,
            &solid.fourth_corner,
        ])
    }

    fn push_trace(&mut self, trace: &Trace) -> Result<(), AssetImportError> {
        self.push_surface([
            &trace.first_corner,
            &trace.second_corner,
            &trace.third_corner,
            &trace.fourth_corner,
        ])
    }

    fn push_polyface(&mut self, polyline: &Polyline) -> Result<(), AssetImportError> {
        const POLYFACE_MESH_FLAG: i32 = 64;
        const POLYFACE_FACE_FLAG: i32 = 128;

        if polyline.flags & POLYFACE_MESH_FLAG == 0 {
            return Ok(());
        }

        let vertices = polyline.vertices().collect::<Vec<_>>();
        let control_points = vertices
            .iter()
            .filter(|vertex| vertex.flags & POLYFACE_FACE_FLAG == 0)
            .map(|vertex| &vertex.location)
            .collect::<Vec<_>>();
        for face in vertices
            .iter()
            .filter(|vertex| vertex.flags & POLYFACE_FACE_FLAG != 0)
        {
            let mut points = Vec::new();
            for index in [
                face.polyface_mesh_vertex_index1,
                face.polyface_mesh_vertex_index2,
                face.polyface_mesh_vertex_index3,
                face.polyface_mesh_vertex_index4,
            ] {
                let Some(index) = polyface_index(index) else {
                    continue;
                };
                let point = control_points.get(index).ok_or_else(|| {
                    AssetImportError::Parse(format!(
                        "parse dxf {}: polyface references missing vertex {}",
                        self.context.source_path.display(),
                        index + 1
                    ))
                })?;
                points.push(*point);
            }
            self.push_polygon(&points)?;
        }
        Ok(())
    }

    fn push_surface(&mut self, points: [&Point; 4]) -> Result<(), AssetImportError> {
        let point_count = surface_point_count(points);
        self.push_polygon(&points[..point_count])
    }

    fn push_polygon(&mut self, points: &[&Point]) -> Result<(), AssetImportError> {
        if points.len() < 3 {
            return Ok(());
        }
        let first = points[0];
        for triangle in 1..points.len() - 1 {
            self.push_triangle([first, points[triangle], points[triangle + 1]])?;
        }
        Ok(())
    }

    fn push_triangle(&mut self, points: [&Point; 3]) -> Result<(), AssetImportError> {
        if is_degenerate_triangle(points) {
            return Ok(());
        }
        let base = u32::try_from(self.positions.len() / 3).map_err(|_| {
            AssetImportError::Parse(format!(
                "parse dxf {}: vertex count exceeds u32",
                self.context.source_path.display()
            ))
        })?;
        for point in points {
            self.positions
                .extend_from_slice(&point_to_f32(point, self.context)?);
        }
        self.indices.extend([base, base + 1, base + 2]);
        Ok(())
    }
}

fn surface_point_count(points: [&Point; 4]) -> usize {
    if distinct_point(points[3], points[0])
        && distinct_point(points[3], points[1])
        && distinct_point(points[3], points[2])
    {
        4
    } else {
        3
    }
}

fn polyface_index(index: i32) -> Option<usize> {
    if index == 0 {
        None
    } else {
        Some(index.unsigned_abs() as usize - 1)
    }
}

fn distinct_point(a: &Point, b: &Point) -> bool {
    const EPSILON: f64 = 1.0e-9;
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    dx * dx + dy * dy + dz * dz > EPSILON
}

fn is_degenerate_triangle(points: [&Point; 3]) -> bool {
    let ab = (
        points[1].x - points[0].x,
        points[1].y - points[0].y,
        points[1].z - points[0].z,
    );
    let ac = (
        points[2].x - points[0].x,
        points[2].y - points[0].y,
        points[2].z - points[0].z,
    );
    let cross = (
        ab.1 * ac.2 - ab.2 * ac.1,
        ab.2 * ac.0 - ab.0 * ac.2,
        ab.0 * ac.1 - ab.1 * ac.0,
    );
    cross.0 * cross.0 + cross.1 * cross.1 + cross.2 * cross.2 <= 1.0e-18
}

fn point_to_f32(point: &Point, context: &AssetImportContext) -> Result<[f32; 3], AssetImportError> {
    let mut values = [0.0_f32; 3];
    for (output, value) in values.iter_mut().zip([point.x, point.y, point.z]) {
        if !value.is_finite() || value < f32::MIN as f64 || value > f32::MAX as f64 {
            return Err(AssetImportError::Parse(format!(
                "parse dxf {}: coordinate {value} cannot be represented as f32",
                context.source_path.display()
            )));
        }
        *output = value as f32;
    }
    Ok(values)
}

#[cfg(test)]
mod hotpath_tests {
    use super::*;
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn plugins07_model_hotpath_surface_point_count_preserves_triangles_and_quads() {
        let first = Point::new(0.0, 0.0, 0.0);
        let second = Point::new(1.0, 0.0, 0.0);
        let third = Point::new(0.0, 1.0, 0.0);
        let fourth = Point::new(1.0, 1.0, 0.0);

        assert_eq!(surface_point_count([&first, &second, &third, &first]), 3);
        assert_eq!(surface_point_count([&first, &second, &third, &fourth]), 4);
    }

    #[test]
    #[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
    fn plugins07_model_hotpath_release_stack_surface_points_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const SURFACES: usize = 262_144;
        const THRESHOLD_PERCENT: u128 = 80;
        let first = Point::new(0.0, 0.0, 0.0);
        let second = Point::new(1.0, 0.0, 0.0);
        let third = Point::new(0.0, 1.0, 0.0);
        let fourth = Point::new(1.0, 1.0, 0.0);
        let points = [&first, &second, &third, &fourth];
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let legacy = || measure_heap_surface_points(points, SURFACES);
            let optimized = || measure_stack_surface_points(points, SURFACES);
            if pair % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }

        emit_cad_performance_gate(
            &legacy_samples,
            &optimized_samples,
            THRESHOLD_PERCENT,
            &format!(
                "surfaces_per_sample={SURFACES} points_per_surface=4 legacy_temporary_vec_allocations_per_sample={SURFACES} optimized_temporary_vec_allocations_per_sample=0"
            ),
        );
    }

    fn measure_heap_surface_points(points: [&Point; 4], surfaces: usize) -> u128 {
        let started = Instant::now();
        let mut point_count = 0_usize;
        for _ in 0..surfaces {
            let points = black_box(points);
            let mut polygon = vec![points[0], points[1], points[2]];
            if surface_point_count(points) == 4 {
                polygon.push(points[3]);
            }
            point_count += black_box(polygon.as_slice()).len();
        }
        black_box(point_count);
        started.elapsed().as_nanos()
    }

    fn measure_stack_surface_points(points: [&Point; 4], surfaces: usize) -> u128 {
        let started = Instant::now();
        let mut point_count = 0_usize;
        for _ in 0..surfaces {
            let points = black_box(points);
            let count = surface_point_count(points);
            point_count += black_box(&points[..count]).len();
        }
        black_box(point_count);
        started.elapsed().as_nanos()
    }

    fn emit_cad_performance_gate(
        legacy_samples: &[u128],
        optimized_samples: &[u128],
        threshold_percent: u128,
        workload: &str,
    ) {
        let legacy_p95 = nearest_rank_cad_p95(legacy_samples);
        let optimized_p95 = nearest_rank_cad_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_cad_stack_surface_points sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
            cad_samples_csv(legacy_samples),
            cad_samples_csv(optimized_samples),
        );
        assert!(
            improvement_percent >= threshold_percent,
            "CAD stack surface points must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
        );
    }

    fn nearest_rank_cad_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn cad_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
