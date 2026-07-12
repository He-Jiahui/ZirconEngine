use std::ffi::CStr;
use std::ptr;

use joltc_sys::{
    JPC_Shape, JPC_Shape_Release, JPC_StaticCompoundShapeSettings,
    JPC_StaticCompoundShapeSettings_Create, JPC_String, JPC_String_c_str, JPC_String_delete,
    JPC_SubShapeSettings, JPC_TriangleShapeSettings, JPC_TriangleShapeSettings_Create, JPC_Vec3,
};
use zircon_runtime::core::framework::physics::PhysicsMeshAsset;
use zircon_runtime::core::math::Real;

use crate::backend::{PhysicsBackendError, PhysicsBackendObjectKind};

pub(super) fn validate_mesh_asset(asset: &PhysicsMeshAsset) -> Result<(), String> {
    match asset {
        PhysicsMeshAsset::TriangleMesh { vertices, indices } => {
            if vertices.len() < 3 || indices.is_empty() {
                return Err(
                    "triangle mesh assets require at least three vertices and one triangle"
                        .to_string(),
                );
            }
            if !vertices
                .iter()
                .flatten()
                .all(|coordinate| coordinate.is_finite())
            {
                return Err("triangle mesh vertices must be finite".to_string());
            }
            for triangle in indices {
                if triangle[0] == triangle[1]
                    || triangle[1] == triangle[2]
                    || triangle[0] == triangle[2]
                    || triangle
                        .iter()
                        .any(|index| *index as usize >= vertices.len())
                {
                    return Err(
                        "triangle mesh indices must be distinct and reference existing vertices"
                            .to_string(),
                    );
                }
            }
        }
        PhysicsMeshAsset::HeightField {
            resolution,
            heights,
        } => {
            let sample_count = (resolution[0] as usize)
                .checked_mul(resolution[1] as usize)
                .ok_or_else(|| "height-field resolution exceeds addressable memory".to_string())?;
            if resolution[0] < 2 || resolution[1] < 2 || heights.len() != sample_count {
                return Err(
                    "height-field assets require a resolution of at least 2x2 and one height per sample"
                        .to_string(),
                );
            }
            if !heights.iter().all(|height| height.is_finite()) {
                return Err("height-field samples must be finite".to_string());
            }
        }
    }
    Ok(())
}

pub(super) unsafe fn create_mesh_shape(
    asset: &PhysicsMeshAsset,
) -> Result<*mut JPC_Shape, PhysicsBackendError> {
    let triangles = match asset {
        PhysicsMeshAsset::TriangleMesh { vertices, indices } => indices
            .iter()
            .map(|triangle| {
                [
                    vertices[triangle[0] as usize],
                    vertices[triangle[1] as usize],
                    vertices[triangle[2] as usize],
                ]
            })
            .collect(),
        PhysicsMeshAsset::HeightField {
            resolution,
            heights,
        } => height_field_triangles(*resolution, heights),
    };
    create_triangle_set(&triangles)
}

fn height_field_triangles(resolution: [u32; 2], heights: &[Real]) -> Vec<[[Real; 3]; 3]> {
    let width = resolution[0] as usize;
    let depth = resolution[1] as usize;
    let mut triangles = Vec::with_capacity((width - 1) * (depth - 1) * 2);
    for z in 0..depth - 1 {
        for x in 0..width - 1 {
            let vertex = |sample_x: usize, sample_z: usize| {
                [
                    sample_x as Real,
                    heights[sample_z * width + sample_x],
                    sample_z as Real,
                ]
            };
            let v00 = vertex(x, z);
            let v10 = vertex(x + 1, z);
            let v01 = vertex(x, z + 1);
            let v11 = vertex(x + 1, z + 1);
            triangles.push([v00, v01, v10]);
            triangles.push([v10, v01, v11]);
        }
    }
    triangles
}

unsafe fn create_triangle_set(
    triangles: &[[[Real; 3]; 3]],
) -> Result<*mut JPC_Shape, PhysicsBackendError> {
    let mut owned_shapes = Vec::with_capacity(triangles.len());
    for triangle in triangles {
        match create_triangle(*triangle) {
            Ok(shape) => owned_shapes.push(shape),
            Err(error) => {
                release_shapes(owned_shapes);
                return Err(error);
            }
        }
    }
    if owned_shapes.len() == 1 {
        return Ok(owned_shapes[0]);
    }

    let sub_shapes = owned_shapes
        .iter()
        .map(|shape| JPC_SubShapeSettings {
            Shape: *shape,
            ..JPC_SubShapeSettings::default()
        })
        .collect::<Vec<_>>();
    let mut native_shape = ptr::null_mut();
    let mut native_error = ptr::null_mut();
    let created = JPC_StaticCompoundShapeSettings_Create(
        &JPC_StaticCompoundShapeSettings {
            SubShapes: sub_shapes.as_ptr(),
            SubShapesLen: sub_shapes.len(),
            ..JPC_StaticCompoundShapeSettings::default()
        },
        &mut native_shape,
        &mut native_error,
    );
    release_shapes(owned_shapes);
    finish_shape_creation(created, native_shape, native_error)
}

unsafe fn create_triangle(vertices: [[Real; 3]; 3]) -> Result<*mut JPC_Shape, PhysicsBackendError> {
    let mut native_shape = ptr::null_mut();
    let mut native_error = ptr::null_mut();
    let created = JPC_TriangleShapeSettings_Create(
        &JPC_TriangleShapeSettings {
            V1: vec3(vertices[0]),
            V2: vec3(vertices[1]),
            V3: vec3(vertices[2]),
            ..JPC_TriangleShapeSettings::default()
        },
        &mut native_shape,
        &mut native_error,
    );
    finish_shape_creation(created, native_shape, native_error)
}

fn vec3(values: [Real; 3]) -> JPC_Vec3 {
    JPC_Vec3 {
        x: values[0],
        y: values[1],
        z: values[2],
        _w: values[2],
    }
}

unsafe fn finish_shape_creation(
    created: bool,
    native_shape: *mut JPC_Shape,
    native_error: *mut JPC_String,
) -> Result<*mut JPC_Shape, PhysicsBackendError> {
    if created && !native_shape.is_null() {
        return Ok(native_shape);
    }
    Err(PhysicsBackendError::InvalidDescriptor {
        kind: PhysicsBackendObjectKind::Shape,
        detail: take_native_error(native_error),
    })
}

unsafe fn release_shapes(shapes: Vec<*mut JPC_Shape>) {
    for shape in shapes {
        JPC_Shape_Release(shape);
    }
}

unsafe fn take_native_error(error: *mut JPC_String) -> String {
    if error.is_null() {
        return "JoltC rejected mesh shape creation without an error message".to_string();
    }
    let message = JPC_String_c_str(error);
    let detail = if message.is_null() {
        "JoltC rejected mesh shape creation with an empty error".to_string()
    } else {
        CStr::from_ptr(message).to_string_lossy().into_owned()
    };
    JPC_String_delete(error);
    detail
}
