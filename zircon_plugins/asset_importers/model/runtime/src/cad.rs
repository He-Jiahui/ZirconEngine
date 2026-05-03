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
        let mut polygon = vec![points[0], points[1], points[2]];
        if distinct_point(points[3], points[0])
            && distinct_point(points[3], points[1])
            && distinct_point(points[3], points[2])
        {
            polygon.push(points[3]);
        }
        self.push_polygon(&polygon)
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
