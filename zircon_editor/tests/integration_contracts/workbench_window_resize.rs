use std::fs;
use std::path::PathBuf;

fn source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("{relative} should be readable"))
}

#[test]
fn workbench_shell_window_resize_contract_is_owned_by_rust_host_contract() {
    let slint_host_mod = source("src/ui/slint_host/mod.rs");
    let host_window_contract = source("src/ui/slint_host/host_contract/window.rs");
    let shell_window_test = source("src/tests/host/slint_window/shell_window.rs");

    let generated_include = ["slint::", "include_modules!()"].concat();
    assert!(!slint_host_mod.contains(&generated_include));
    assert!(host_window_contract.contains("pub(crate) struct UiHostWindow"));
    assert!(host_window_contract.contains("pub(crate) fn set_size"));
    assert!(host_window_contract.contains("pub(crate) fn set_maximized"));
    assert!(host_window_contract.contains("pub(crate) fn get_host_window_bootstrap"));
    assert!(shell_window_test.contains("UiHostWindow::new()"));
    assert!(shell_window_test.contains("set_size"));
    assert!(shell_window_test.contains("set_maximized(true)"));
}
