use crate::graphics::resource_limits::POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE;

use super::*;

#[test]
fn post_process_layout_sampled_texture_count_matches_device_request_limit() {
    let entries = post_process_entries(PostProcessDepthSamplingMode::RawDepthTexture);

    assert_eq!(entries.len(), 29);
    assert_eq!(
        sampled_texture_binding_count(&entries),
        POST_PROCESS_REQUIRED_SAMPLED_TEXTURES_PER_SHADER_STAGE
    );
}

#[test]
fn post_process_layout_binds_resolved_exposure_storage_buffer() {
    let entries = post_process_entries(PostProcessDepthSamplingMode::RawDepthTexture);
    let exposure_entry = entries
        .iter()
        .find(|entry| entry.binding == 28)
        .expect("post-process layout should bind resolved exposure");

    assert!(exposure_entry
        .visibility
        .contains(wgpu::ShaderStages::FRAGMENT));
    assert!(matches!(
        exposure_entry.ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            ..
        }
    ));
}

fn sampled_texture_binding_count(entries: &[wgpu::BindGroupLayoutEntry]) -> u32 {
    entries
        .iter()
        .filter(|entry| {
            entry.visibility.contains(wgpu::ShaderStages::FRAGMENT)
                && matches!(entry.ty, wgpu::BindingType::Texture { .. })
        })
        .count() as u32
}
