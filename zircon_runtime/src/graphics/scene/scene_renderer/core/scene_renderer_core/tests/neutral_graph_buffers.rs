#[test]
fn neutral_graph_buffers_are_lazy_device_lifetime_owners() {
    let source = include_str!("../neutral_graph_buffers.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(source.contains("light_grid: Option<LightGridNeutralBuffers>"));
    assert!(source.contains("hzb: Option<HzbNeutralBuffers>"));
    assert!(source.contains("plugin: FirstPartyPluginNeutralBuffers"));
    assert_eq!(
        source
            .matches("get_or_insert_with(|| LightGridNeutralBuffers::new(device))")
            .count(),
        1
    );
    assert!(source.contains("mapped_at_creation: true"));
    assert_eq!(
        source
            .matches("get_or_insert_with(|| HzbNeutralBuffers::new(device))")
            .count(),
        1
    );
}

#[test]
fn mapped_neutral_buffers_copy_zeroes_into_the_wgpu_view_before_unmapping() {
    let source = include_str!("../neutral_graph_buffers.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let zeroed_buffer_start = source
        .find("fn zeroed_buffer")
        .expect("neutral backing must have one mapped initialization helper");
    let zeroed_buffer = &source[zeroed_buffer_start..];

    assert!(
        zeroed_buffer.contains("let mut mapped_bytes = buffer.slice(..).get_mapped_range_mut();")
    );
    assert!(zeroed_buffer.contains("let zeroes = vec![0; mapped_bytes.len()];"));
    assert!(zeroed_buffer.contains("mapped_bytes.copy_from_slice(&zeroes);"));
    assert!(zeroed_buffer.contains("drop(mapped_bytes);"));
    assert!(!zeroed_buffer.contains("get_mapped_range_mut().as_mut()"));
    assert!(!zeroed_buffer.contains("mapped_bytes.fill(0);"));

    let zeroes = zeroed_buffer
        .find("let zeroes = vec![0; mapped_bytes.len()];")
        .expect("the mapped range must be initialized");
    let copy = zeroed_buffer
        .find("mapped_bytes.copy_from_slice(&zeroes);")
        .expect("the mapped range must receive the zero bytes");
    let release = zeroed_buffer
        .find("drop(mapped_bytes);")
        .expect("the mapped range must release before unmapping");
    let unmap = zeroed_buffer
        .find("buffer.unmap();")
        .expect("the backing buffer must be unmapped");
    assert!(zeroes < copy && copy < release && release < unmap);
}
