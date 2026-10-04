#[test]
fn project_composition_transfers_the_gateway_to_the_retained_host_runner() {
    let source = include_str!("../composition.rs");

    assert!(
        source.contains(
            "let runtime_gateway = runtime_session.editor_gateway(runtime_capabilities)?;"
        ),
        "composition must create the runtime gateway before entering the retained host"
    );
    assert!(
        source.contains(
            "run_retained_host_automation(core.clone(), runtime_gateway, config, bindings)"
        ),
        "composition must transfer automation to zircon_editor's retained host"
    );
}

#[test]
fn project_composition_preflights_the_dynamic_runtime_before_project_materialization() {
    let source = include_str!("../composition.rs");
    let product_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("production composition source must precede its tests");
    let runtime_preflight = product_source
        .find("LoadedRuntime::preflight_default()")
        .expect("composition must preflight the staged runtime BuildSet");
    let project_prepare = product_source
        .find("prepare_editor_gui_startup(Some(startup_request))?")
        .expect("composition must prepare its project after runtime preflight");

    assert!(runtime_preflight < project_prepare);
    assert!(product_source.contains("match runtime_preflight.load_after_preflight()"));
    assert!(!product_source.contains("LoadedRuntime::linked()?"));
    assert!(!product_source.contains("create_linked_with_profile_and_project("));
    assert!(product_source.contains("RuntimeSession::create_with_profile("));
    assert!(product_source.contains("product_composition.fail_with_runtime_until(failure"));
    assert!(!product_source.contains("error.retry_cleanup()"));
}

#[test]
fn project_composition_close_releases_gateway_owners_before_checking_teardown() {
    let source = include_str!("../composition.rs");
    let close = source
        .split("pub fn close(self)")
        .nth(1)
        .expect("project composition should expose explicit close");
    let mut offset = 0;
    for needle in [
        "let runtime_teardown_failure = runtime_session.teardown_failure_state();",
        "let product_failure_ledger = runtime_teardown_failure.failure_ledger();",
        "finish_owned_editor_host(",
        "product_composition",
        "runtime_session",
        "play_backend",
        "&product_failure_ledger",
    ] {
        let index = close[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("composition close path is missing `{needle}`"));
        offset += index + needle.len();
    }
}

#[test]
fn default_drop_retains_the_whole_original_editor_packet_without_cleanup() {
    let source = include_str!("../composition.rs");
    let ownership = include_str!("../ownership.rs");
    assert!(source.contains("ownership: Option<EditorApplicationOwnership>"));
    assert!(source.contains("ownership.retain_unclosed()"));
    assert!(ownership.contains(".with_runtime(self.runtime_session)"));
    assert!(ownership.contains(".with_host_pin(self.play_backend)"));
    assert!(source.contains("#[must_use = \"call close or run_retained_host_automation"));
}
