use super::*;

#[test]
fn bindable_storage_buffer_size_keeps_zero_capacity_buffers_bindable() {
    assert_eq!(
        bindable_storage_buffer_size(0),
        INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES
    );
    assert_eq!(bindable_storage_buffer_size(20), 20);
}

#[test]
fn bindable_draw_count_buffer_size_keeps_zero_capacity_buffers_bindable() {
    assert_eq!(
        bindable_draw_count_buffer_size(0),
        INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES
    );
    assert_eq!(bindable_draw_count_buffer_size(8), 8);
}

#[test]
fn mesh_indirect_compaction_resources_reserve_expected_wgpu_usages() {
    let source = include_str!("../indirect_compaction_resources.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation source");

    assert!(implementation.contains("ensure_buffer_capacity"));
    assert!(implementation.contains("visible-instance-index"));
    assert!(implementation.contains("compacted-indirect-args"));
    assert!(implementation.contains("wgpu::BufferUsages::STORAGE"));
    assert!(implementation.contains("wgpu::BufferUsages::COPY_DST"));
    assert!(implementation.contains("wgpu::BufferUsages::COPY_SRC"));
    assert!(implementation.contains("wgpu::BufferUsages::INDIRECT"));
}

#[test]
fn indirect_buffer_capacity_is_grow_only_and_power_of_two() {
    assert_eq!(grow_indirect_buffer_capacity(0, 1), 1);
    assert_eq!(grow_indirect_buffer_capacity(1, 20), 32);
    assert_eq!(grow_indirect_buffer_capacity(32, 20), 32);
    assert_eq!(grow_indirect_buffer_capacity(32, 33), 64);
}

#[test]
fn mesh_indirect_compaction_resources_clear_outputs_without_rewriting_metadata() {
    let source = include_str!("../indirect_compaction_resources.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation source");

    assert!(implementation.contains("pub(crate) fn encode_clear_outputs"));
    assert!(implementation.contains("encoder.clear_buffer("));
    assert!(implementation.contains("visible_instance_index_buffer_allocation_byte_size"));
    assert!(implementation.contains("draw_count_buffer"));
    assert!(implementation.contains("compacted_indirect_args_buffer"));
    assert!(!implementation.contains("clear_buffer(\n            &self.metadata_buffer"));
}
