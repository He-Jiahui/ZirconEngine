#[test]
fn shared_resize_surface_replaces_legacy_direct_resize_callback_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_context.slint"
    ));
    let host_resize_layer = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_resize_layer.slint"
    ));
    let host_components = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_components.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let docking = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/workspace_docking.rs"
    ));

    for needle in [
        "callback begin_drawer_resize(x: float, y: float);",
        "callback update_drawer_resize(x: float, y: float);",
        "callback finish_drawer_resize(x: float, y: float);",
        "root.begin_drawer_resize(",
        "root.update_drawer_resize(",
        "root.finish_drawer_resize(",
        "ui.on_begin_drawer_resize(",
        "ui.on_update_drawer_resize(",
        "ui.on_finish_drawer_resize(",
        "fn begin_drawer_resize(",
        "fn update_drawer_resize(",
        "fn finish_drawer_resize(",
    ] {
        let found =
            workbench.contains(needle) || wiring.contains(needle) || docking.contains(needle);
        assert!(
            !found,
            "drawer resize path still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "callback workbench_resize_pointer_event(kind: int, x: float, y: float);",
        "WorkbenchHostContext.workbench_resize_pointer_event(",
    ] {
        assert!(
            workbench.contains(needle)
                || host_context.contains(needle)
                || host_resize_layer.contains(needle)
                || host_components.contains(needle),
            "workbench shell is missing shared resize pointer hook `{needle}`"
        );
    }

    assert!(
        wiring.contains("host_shell.on_workbench_resize_pointer_event("),
        "slint host callback wiring must register shared resize pointer callback"
    );
    assert!(
        docking.contains("fn workbench_resize_pointer_event("),
        "workspace docking host must handle shared resize pointer events"
    );
}
