use super::EditorHostWindowFailure;

#[test]
fn editor_host_window_failure_is_actionable() {
    let failure = EditorHostWindowFailure::new(
        "editor_host_window",
        "presenter_backend=softbuffer size=1280x720",
        "presenter creation failed: device lost",
        "verify the graphics adapter and restart zircon_editor",
    );

    assert_eq!(
        failure.to_string(),
        "editor startup diagnostic: component=editor_host_window requested=presenter_backend=softbuffer size=1280x720 cause=presenter creation failed: device lost recovery=verify the graphics adapter and restart zircon_editor"
    );
}

#[test]
fn editor_composition_collects_host_failure_before_accepting_run_result() {
    let source = include_str!("../../../app.rs");
    let mut offset = 0;
    for needle in [
        "let run_result = ui.run();",
        "if let Some(error) = ui.take_fatal_failure()",
        "return Err(error.into());",
        "run_result?;",
    ] {
        let index = source[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("editor host failure collection is missing `{needle}`"));
        offset += index + needle.len();
    }
}
