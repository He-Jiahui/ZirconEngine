use super::gpu_mesh_resource::GpuMeshResource;
use super::gpu_mesh_vertex::GpuMeshVertex;
use super::mesh_bounds::MeshBoundsAccumulator;
use super::wire_segments::build_wire_segments;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

impl GpuMeshResource {
    /// 从同一批顶点生成缓冲、边界和排序补充签名；签名选取部分通道及索引顺序，资源复用仍由资产版本决定。
    pub(crate) fn from_asset(
        device: &wgpu::Device,
        payload: crate::asset::assets::ModelPrimitiveAsset,
    ) -> Self {
        let mut indirect_order_signature = FNV_OFFSET_BASIS;
        let mut vertices: Vec<GpuMeshVertex> = Vec::with_capacity(payload.vertices.len());
        let mut bounds = MeshBoundsAccumulator::default();
        for vertex in payload.vertices {
            indirect_order_signature = fnv1a_mesh_vertex(indirect_order_signature, &vertex);
            let vertex: GpuMeshVertex = vertex.into();
            bounds.include_position(vertex.position);
            vertices.push(vertex);
        }
        for index in &payload.indices {
            indirect_order_signature = fnv1a_u32(indirect_order_signature, *index);
        }
        let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("zircon-mesh-vertex-buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        let index_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("zircon-mesh-index-buffer"),
                contents: bytemuck::cast_slice(&payload.indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        let (bounds_min, bounds_max) = bounds.finish();

        Self {
            vertex_buffer,
            index_buffer,
            index_count: payload.indices.len() as u32,
            indirect_order_signature,
            wire_segments: build_wire_segments(&vertices, &payload.indices),
            bounds_min,
            bounds_max,
        }
    }
}

fn fnv1a_mesh_vertex(mut hash: u64, vertex: &crate::asset::MeshVertex) -> u64 {
    hash = fnv1a_f32_slice(hash, &vertex.position);
    hash = fnv1a_f32_slice(hash, &vertex.normal);
    hash = fnv1a_f32_slice(hash, &vertex.uv);
    hash = fnv1a_u16_slice(hash, &vertex.joint_indices);
    hash = fnv1a_f32_slice(hash, &vertex.joint_weights);
    hash = fnv1a_f32_slice(hash, &vertex.tangent);
    fnv1a_f32_slice(hash, &vertex.color)
}

fn fnv1a_u16_slice<const N: usize>(mut hash: u64, values: &[u16; N]) -> u64 {
    for value in values {
        hash = fnv1a_u16(hash, *value);
    }
    hash
}

fn fnv1a_u16(mut hash: u64, value: u16) -> u64 {
    for byte in value.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn fnv1a_f32_slice<const N: usize>(mut hash: u64, values: &[f32; N]) -> u64 {
    for value in values {
        hash = fnv1a_u32(hash, value.to_bits());
    }
    hash
}

fn fnv1a_u32(mut hash: u64, value: u32) -> u64 {
    for byte in value.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
#[path = "tests/gpu_mesh_resource_from_asset.rs"]
mod tests;
