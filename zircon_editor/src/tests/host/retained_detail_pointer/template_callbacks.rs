fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn inspector_surface_controls_use_pane_surface_host_callbacks() {
    let globals = source("src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/pane_surface/inspector.rs");
    let inspector = [
        "src/ui/retained_host/app/inspector/surface_controls/value_change.rs",
        "src/ui/retained_host/app/inspector/surface_controls/click.rs",
    ]
    .into_iter()
    .map(source)
    .collect::<Vec<_>>()
    .join("\n");

    for required in [
        "on_inspector_control_changed",
        "on_inspector_control_clicked",
        "invoke_inspector_control_changed",
        "invoke_inspector_control_clicked",
    ] {
        assert!(
            globals.contains(required),
            "host globals missing `{required}`"
        );
    }
    assert!(wiring.contains("pane_surface_host.on_inspector_control_changed("));
    assert!(wiring.contains("pane_surface_host.on_inspector_control_clicked("));
    assert!(inspector.contains("dispatch_inspector_control_changed"));
    assert!(inspector.contains("dispatch_inspector_control_clicked"));
}

#[test]
fn inspector_surface_controls_use_the_shared_compact_control_radius() {
    let controls = source("assets/ui/editor/host/inspector_surface_controls.zui");

    assert!(
        controls.contains("corner_radius = \"$editor.control.radius.control\""),
        "inspector fields and actions must consume the shared control radius"
    );
    assert!(
        !controls.contains("corner_radius = 10.0") && !controls.contains("corner_radius = 999.0"),
        "inspector controls must not override the compact control shape"
    );
    assert!(
        controls.contains("gap = \"$editor.density.gap.small\""),
        "inspector controls must use the shared dense spacing rhythm"
    );
}
