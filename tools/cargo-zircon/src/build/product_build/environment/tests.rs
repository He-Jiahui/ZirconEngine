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
