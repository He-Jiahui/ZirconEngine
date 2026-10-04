//! 目标三元组到 Cargo 链接器变量的合法映射和非法字符拒绝。
//! 测试 harness 和所属生产模块调用本文件；受控性能或夹具数据只验证对应局部契约，不代表真实 Cargo 产品编译。

#[test]
fn cargo_linker_environment_key_maps_and_rejects_target_triples() {
    assert_eq!(
        super::cargo_linker_environment_key("x86_64-pc-windows-msvc").unwrap(),
        "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER"
    );
    assert_eq!(
        super::cargo_linker_environment_key("AARCH64_PC_WINDOWS_MSVC").unwrap(),
        "CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER"
    );

    let error = super::cargo_linker_environment_key("x86_64 pc-windows-msvc")
        .err()
        .unwrap();
    assert!(error
        .to_string()
        .contains("cannot form a Cargo linker environment key"));
}
