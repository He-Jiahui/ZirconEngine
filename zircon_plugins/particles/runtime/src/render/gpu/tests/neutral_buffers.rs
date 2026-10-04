use super::*;

#[test]
fn neutral_frame_presence_does_not_depend_on_payload_capacity() {
    let frame = RenderParticleGpuFrameExtract {
        alive_count: 3,
        spawned_total: 5,
        per_emitter_spawned: vec![2, 3, 0],
        indirect_draw_args: [6, 3, 0, 0],
    };

    assert!(!neutral_frame_is_empty(&frame));
    assert!(neutral_frame_is_empty(
        &RenderParticleGpuFrameExtract::default()
    ));
}

#[test]
fn neutral_identity_bundle_has_constant_logical_size() {
    assert_eq!(NEUTRAL_IDENTITY_BYTES, 4);
    assert_eq!(NEUTRAL_INDIRECT_BYTES, 16);
    assert_eq!(6 * NEUTRAL_IDENTITY_BYTES + NEUTRAL_INDIRECT_BYTES, 40);
}

#[test]
fn neutral_source_relies_on_the_runtime_owner_device_epoch() {
    let source = include_str!("../neutral_buffers.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert_eq!(source.matches("device.create_buffer(").count(), 1);
    assert_eq!(source.matches(": create_buffer(").count(), 7);
    assert!(source.contains("if neutral_frame_is_empty(frame)"));
    assert!(!source.contains("device: Option<wgpu::Device>"));
    assert!(!source.contains("device.clone()"));
    assert!(!source.contains("next_power_of_two"));
    assert!(!source.contains("queue.write_buffer"));
    assert!(!source.contains("encoder.copy_buffer_to_buffer"));
    assert!(!source.contains("particle_capacity"));
    assert!(!source.contains("emitter_capacity"));
    assert!(!source.contains("NeutralFrameShadow"));
}

#[test]
fn neutral_buffers_use_wgpu_lazy_zero_initialization_without_host_staging() {
    let source = include_str!("../neutral_buffers.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let create_buffer = &source[source
        .find("fn create_buffer")
        .expect("neutral backing must have one creation helper")..];

    assert!(create_buffer.contains("mapped_at_creation: false"));
    assert!(!create_buffer.contains("mapped_at_creation: true"));
    assert!(!create_buffer.contains("get_mapped_range_mut"));
    assert!(!create_buffer.contains(".fill(0)"));
    assert!(!create_buffer.contains("copy_from_slice"));
    assert!(!create_buffer.contains("buffer.unmap()"));
}
