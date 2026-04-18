use zircon_math::Vec3;

use super::gpu_mesh_resource::GpuMeshResource;
use super::gpu_mesh_vertex::GpuMeshVertex;
use super::mesh_bounds::mesh_bounds;
use super::wire_segments::build_wire_segments;

impl GpuMeshResource {
    pub(crate) fn from_asset(
        device: &wgpu::Device,
        payload: zircon_asset::ModelPrimitiveAsset,
    ) -> Self {
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
            wire_segments: build_wire_segments(&positions, &payload.indices),
            bounds_min,
            bounds_max,
        }
    }
}
