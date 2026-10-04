const SOURCE: &str = include_str!("../resources.rs");
const BLIT_PIPELINE_SOURCE: &str = include_str!("../blit_pipeline.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("light cookie resources should retain a test-module boundary")
}

#[test]
fn light_cookie_graph_pass_owns_initialization_and_measurement() {
    let source = production_source();
    let plan_scope = source
        .find("\"light_cookie\", \"frame_plan\"")
        .expect("light cookie frame-plan scope");
    let plan = source
        .find("build_cookie_frame_plan(cookies)")
        .expect("light cookie frame plan");
    let encode_scope = source
        .find("\"light_cookie\", \"atlas_encode\"")
        .expect("light cookie encode scope");
    let encode = source
        .find("self.blit_pipeline.encode(")
        .expect("light cookie atlas encode");
    let record = source
        .find("self.profile.record_rebuild(")
        .expect("light cookie work record");
    let construct_scope = source
        .find("\"light_cookie\", \"atlas_construct\"")
        .expect("light cookie atlas construction scope");

    assert!(construct_scope < plan_scope);
    assert!(plan_scope < plan);
    assert!(plan < encode_scope);
    assert!(encode_scope < encode);
    assert!(encode < record);
    assert!(
        source.contains("u64::from(LIGHT_COOKIE_ATLAS_SIZE) * u64::from(LIGHT_COOKIE_ATLAS_SIZE)")
    );
    assert!(!source.contains("wgpu::Queue"));
    assert!(!source.contains("queue.write_texture("));
    assert!(!source.contains("TextureUsages::COPY_DST"));
    assert!(!source.contains("initial_white_upload"));
    assert!(BLIT_PIPELINE_SOURCE.contains("load: wgpu::LoadOp::Clear(wgpu::Color::WHITE)"));
}
