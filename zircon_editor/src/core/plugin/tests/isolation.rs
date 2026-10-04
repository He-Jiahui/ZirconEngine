use super::run_editor_plugin_boundary;

#[test]
fn callback_error_stays_a_plugin_diagnostic() {
    let failure = run_editor_plugin_boundary("plugin.sample", "register", || {
        Err::<(), _>("invalid contribution".to_string())
    })
    .expect_err("plugin rejection should not escape the host boundary");

    assert_eq!(
        failure.to_string(),
        "editor plugin `plugin.sample` register failed: invalid contribution"
    );
}

#[test]
fn callback_panic_stays_a_plugin_diagnostic() {
    let failure =
        run_editor_plugin_boundary("plugin.sample", "register", || -> Result<(), String> {
            panic!("fixture panic")
        })
        .expect_err("plugin panic should not escape the host boundary");

    assert_eq!(
        failure.to_string(),
        "editor plugin `plugin.sample` register failed: panic: fixture panic"
    );
}
