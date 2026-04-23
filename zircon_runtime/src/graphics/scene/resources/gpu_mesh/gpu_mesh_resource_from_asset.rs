use crate::core::math::Vec3;

use super::gpu_mesh_resource::GpuMeshResource;
use super::gpu_mesh_vertex::GpuMeshVertex;
use super::mesh_bounds::mesh_bounds;
use super::wire_segments::build_wire_segments;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

impl GpuMeshResource {
    pub(crate) fn from_asset(
        device: &wgpu::Device,
        payload: crate::asset::assets::ModelPrimitiveAsset,
    ) -> Self {
        let indirect_order_signature = indirect_order_signature(&payload);
        let positions: Vec<Vec3> = payload
            .vertices
            .iter()
            .map(|vertex| Vec3::from_array(vertex.position))
            .collect();
        let vertices: Vec<GpuMeshVertex> = payload.vertices.into_iter().map(Into::into).collect();
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
        let (bounds_min, bounds_max) = mesh_bounds(&positions);

        Self {
            vertex_buffer,
            index_buffer,
            index_count: payload.indices.len() as u32,
            indirect_order_signature,
            wire_segments: build_wire_segments(&positions, &payload.indices),
            bounds_min,
            bounds_max,
        }
    }
}

fn indirect_order_signature(payload: &crate::asset::assets::ModelPrimitiveAsset) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for vertex in &payload.vertices {
        hash = fnv1a_f32_slice(hash, &vertex.position);
        hash = fnv1a_f32_slice(hash, &vertex.normal);
        hash = fnv1a_f32_slice(hash, &vertex.uv);
        hash = fnv1a_u16_slice(hash, &vertex.joint_indices);
        hash = fnv1a_f32_slice(hash, &vertex.joint_weights);
    }
    for index in &payload.indices {
        hash = fnv1a_u32(hash, *index);
    }
    hash
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
