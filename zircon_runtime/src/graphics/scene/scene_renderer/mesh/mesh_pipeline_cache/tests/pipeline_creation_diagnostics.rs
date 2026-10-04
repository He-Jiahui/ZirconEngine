fn production_source() -> &'static str {
    include_str!("../pipeline_creation_diagnostics.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("pipeline diagnostic owner test boundary")
}

#[test]
fn resolved_pipeline_diagnostics_do_not_own_the_device_timeline() {
    let source = production_source();
    let track = source
        .split("fn track_pipeline_creation_error_scope(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn consume_resolved_pipeline_creation_diagnostics(")
                .next()
        })
        .expect("pipeline diagnostic resolution owner");
    let resolved = track
        .find("pollster::block_on(error_scope.pop())")
        .expect("error scope must resolve before diagnostic retention");
    let retained = track
        .find(".push(PendingPipelineCreationDiagnostic")
        .expect("resolved diagnostic retention");

    assert!(resolved < retained);
    assert!(!source.contains("wgpu::Device"));
    assert!(!source.contains("device.poll("));
    assert!(source.contains("pub(crate) fn drain_pipeline_creation_diagnostics(&mut self)"));
    assert!(source.contains(
        "pub(crate) fn finish_pipeline_creation_diagnostics_for_variant(\n        &mut self,\n        key: &ShaderVariantKey,"
    ));
}
