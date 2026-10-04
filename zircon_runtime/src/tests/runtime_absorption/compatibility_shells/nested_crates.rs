//! 运行时吸收完成后，嵌套兼容包的移除状态由此处约束。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn runtime_absorption_does_not_keep_nested_compatibility_shells() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    assert!(
        !runtime_root.join("crates").exists(),
        "zircon_runtime should not keep nested compatibility crates after absorption"
    );
}
