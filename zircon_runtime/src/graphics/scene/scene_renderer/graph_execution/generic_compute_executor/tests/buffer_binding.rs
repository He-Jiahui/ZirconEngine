use std::num::NonZeroU64;

use crate::render_graph::{BindingSchemaEntry, ComputeBindingKind};

use super::{resolve_buffer_binding_range, validate_buffer_binding_offset};

#[test]
fn buffer_offsets_follow_the_device_binding_alignment() {
    let limits = wgpu::Limits {
        min_uniform_buffer_offset_alignment: 256,
        min_storage_buffer_offset_alignment: 16,
        ..wgpu::Limits::default()
    };
    let uniform = BindingSchemaEntry::new(0, "params", ComputeBindingKind::UniformBuffer)
        .with_buffer_range(256, Some(512));
    let storage = BindingSchemaEntry::new(1, "weights", ComputeBindingKind::StorageBufferRead)
        .with_buffer_range(16, Some(128));

    assert!(validate_buffer_binding_offset(&uniform, 256, &limits).is_ok());
    assert!(validate_buffer_binding_offset(&storage, 16, &limits).is_ok());
    assert!(validate_buffer_binding_offset(&uniform, 16, &limits)
        .expect_err("uniform offsets must honor the device alignment")
        .contains("align to 256 bytes"));
}

#[test]
fn buffer_ranges_preserve_explicit_nonzero_binding_windows() {
    let limits = wgpu::Limits {
        min_uniform_buffer_offset_alignment: 256,
        ..wgpu::Limits::default()
    };
    let binding = BindingSchemaEntry::new(0, "params", ComputeBindingKind::UniformBuffer)
        .with_buffer_range(256, Some(512));

    let resolved = resolve_buffer_binding_range(&binding, 1_024, &limits)
        .expect("a contained nonzero range is valid");

    assert_eq!(resolved.offset, 256);
    assert_eq!(resolved.size, Some(NonZeroU64::new(512).unwrap()));
}

#[test]
fn buffer_ranges_reject_empty_or_out_of_bounds_windows() {
    let limits = wgpu::Limits {
        min_storage_buffer_offset_alignment: 16,
        ..wgpu::Limits::default()
    };
    let empty = BindingSchemaEntry::new(0, "params", ComputeBindingKind::UniformBuffer)
        .with_buffer_range(0, Some(0));
    let overrun = BindingSchemaEntry::new(1, "params", ComputeBindingKind::StorageBufferRead)
        .with_buffer_range(16, Some(1_009));

    assert!(resolve_buffer_binding_range(&empty, 1_024, &limits)
        .err()
        .expect("empty windows cannot produce a WGPU buffer binding")
        .contains("must not be empty"));
    assert!(resolve_buffer_binding_range(&overrun, 1_024, &limits)
        .err()
        .expect("ranges must fit their resolved buffer")
        .contains("exceeds its 1024 byte buffer"));
}
