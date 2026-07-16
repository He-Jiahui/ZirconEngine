use std::sync::Arc;

use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::GPU_MATERIAL_UNIFORM_MIN_SIZE;
use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;

const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

pub(crate) fn create_standard_mesh_pipeline_layout(
    device: &wgpu::Device,
    label: &'static str,
) -> wgpu::PipelineLayout {
    let scene_layout = create_test_scene_layout(device, label);
    let shadow_receiver_layout = create_empty_shadow_receiver_layout(device, label);
    let material_layout = create_test_material_layout(device, label);
    let gpu_scene = create_test_gpu_scene(device, label);
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("zircon-test-{label}-mesh-layout")),
        bind_group_layouts: &[
            Some(&scene_layout),
            Some(&shadow_receiver_layout),
            Some(&material_layout),
            Some(gpu_scene.scene_bind_group_layout()),
        ],
        immediate_size: 0,
    })
}

fn create_test_scene_layout(device: &wgpu::Device, label: &str) -> wgpu::BindGroupLayout {
    let scene_layout_entries = scene_bind_group_layout_entries();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&format!("zircon-test-{label}-scene-layout")),
        entries: &scene_layout_entries,
    })
}

fn create_empty_shadow_receiver_layout(
    device: &wgpu::Device,
    label: &str,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&format!("zircon-test-{label}-empty-shadow-receiver-layout")),
        entries: &[],
    })
}

fn create_test_material_layout(device: &wgpu::Device, label: &str) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&format!("zircon-test-{label}-material-set-layout")),
        entries: &[
            material_uniform_entry(0),
            material_texture_entry(1),
            material_sampler_entry(2),
            material_texture_entry(3),
            material_sampler_entry(4),
            material_texture_entry(5),
            material_sampler_entry(6),
            material_texture_entry(7),
            material_sampler_entry(8),
            material_texture_entry(9),
            material_sampler_entry(10),
            material_texture_entry(11),
            material_sampler_entry(12),
        ],
    })
}

fn material_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    }
}

fn material_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn material_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(GPU_MATERIAL_UNIFORM_MIN_SIZE as u64),
        },
        count: None,
    }
}

fn create_test_gpu_scene(device: &wgpu::Device, label: &str) -> GpuScene {
    GpuScene::new(
        device,
        test_skinned_joint_palette_buffer(device, label),
        test_skinned_joint_palette_min_binding_size(),
    )
}

fn test_skinned_joint_palette_buffer(device: &wgpu::Device, label: &str) -> Arc<wgpu::Buffer> {
    Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("zircon-test-{label}-skinned-joint-palette-buffer")),
        size: test_skinned_joint_palette_min_binding_size().get(),
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    }))
}

fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
    wgpu::BufferSize::new(
        TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
            + TEST_SKINNED_JOINT_PARAMS_BYTES,
    )
    .expect("test skinned joint palette storage size is non-zero")
}
