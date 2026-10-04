use crate::asset::AssetImportError;
use crate::core::math::Vec3;

// indexed_mesh_projection 在所有法线策略中先用此入口拒绝不完整或越界拓扑，
// 让后续顶点投影和法线回填可以按三角形索引访问。
pub(super) fn validate_triangle_indices(
    indices: &[u32],
    vertex_count: usize,
) -> Result<(), AssetImportError> {
    if indices.len() % 3 != 0 {
        return Err(AssetImportError::Parse(format!(
            "triangle index count {} was not a multiple of 3",
            indices.len()
        )));
    }
    for (element, &index) in indices.iter().enumerate() {
        let index = usize::try_from(index).map_err(|_| {
            AssetImportError::Parse(format!(
                "mesh index {index} at element {element} exceeds platform limits"
            ))
        })?;
        if index >= vertex_count {
            return Err(AssetImportError::Parse(format!(
                "mesh index {index} at element {element} exceeds vertex count {vertex_count}"
            )));
        }
    }
    Ok(())
}

// 仅用于缺失法线且选择 Smooth 的网格；已提供法线或 Flat 策略由调用者走其他路径。
pub(super) fn generate_normals(
    positions: &[f32],
    indices: &[u32],
) -> Result<Vec<f32>, AssetImportError> {
    let vertex_count = positions.len() / 3;
    validate_triangle_indices(indices, vertex_count)?;
    let mut normals = vec![0.0_f32; vertex_count * 3];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        let position = |index: usize| -> Vec3 {
            Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            )
        };
        let position_a = position(a);
        let position_b = position(b);
        let position_c = position(c);
        let face_normal = (position_b - position_a)
            .cross(position_c - position_a)
            .normalize_or_zero();
        for index in [a, b, c] {
            normals[index * 3] += face_normal.x;
            normals[index * 3 + 1] += face_normal.y;
            normals[index * 3 + 2] += face_normal.z;
        }
    }

    Ok(normals)
}

#[cfg(test)]
#[path = "tests/generate_normals.rs"]
mod tests;
