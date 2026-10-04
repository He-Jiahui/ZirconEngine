#[test]
fn unsupported_mesh_pipeline_invalidates_replay_state_before_skipping_draw() {
    // TODO: [CR-W13-GE-0003] 待确认此守卫的有效覆盖：整文件查找可命中测试自身的
    // 旧赋值、失效和跳过字面量，缺少直接 replay 状态行为验证；下一步截取生产段
    // 并更新真实锚点，或用状态行为夹具核验生产闭包的失效与跳过顺序。
    let source = include_str!("../oit.rs");
    let unsupported = source
        .find("unsupported_shader = Some(command.pipeline_key().shader_id.clone());")
        .expect("OIT replay must record the unsupported shader");
    let invalidate = source[unsupported..]
        .find("replayer.invalidate_state_after_external_pipeline();")
        .map(|offset| unsupported + offset)
        .expect("OIT replay must invalidate a pipeline selection that failed to materialize");
    let skip = source[invalidate..]
        .find("return false;")
        .map(|offset| invalidate + offset)
        .expect("unsupported OIT commands must remain fail-closed");

    assert!(unsupported < invalidate);
    assert!(invalidate < skip);
}
