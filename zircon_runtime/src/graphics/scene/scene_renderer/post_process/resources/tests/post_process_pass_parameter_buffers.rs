use super::PostProcessPassParameterBuffers;

#[test]
fn one_persistent_slot_exists_per_post_process_parameter_producer() {
    let source = include_str!("../post_process_pass_parameter_buffers.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("persistent post-process parameter slot source");

    assert_eq!(production.matches("create_parameter_buffer(").count(), 10);
    assert_eq!(
        std::mem::size_of::<PostProcessPassParameterBuffers>(),
        9 * std::mem::size_of::<wgpu::Buffer>()
    );
}
