#[test]
fn shared_drag_capture_surface_replaces_legacy_direct_drop_callback_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_context.slint"
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
        "callback drop_tab(tab_id: string, target_group: string, pointer_x: float, pointer_y: float);",
        "callback update_drag_target(x: float, y: float);",
        "root.update_drag_target(root.drag_pointer_x, root.drag_pointer_y);",
        "root.drop_tab(",
        "ui.on_drop_tab(",
        "ui.on_update_drag_target(",
        "fn drop_tab(",
        "fn update_drag_target(",
    ] {
        let found =
            workbench.contains(needle) || wiring.contains(needle) || docking.contains(needle);
        assert!(
            !found,
            "drag capture path still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "callback workbench_drag_pointer_event(kind: int, x: float, y: float);",
        "WorkbenchHostContext.workbench_drag_pointer_event(",
    ] {
        assert!(
            workbench.contains(needle)
                || host_context.contains(needle)
                || host_components.contains(needle),
            "workbench shell is missing shared drag pointer hook `{needle}`"
        );
    }

    assert!(
        wiring.contains("host_shell.on_workbench_drag_pointer_event("),
        "slint host callback wiring must register shared drag pointer callback"
    );
    assert!(
        docking.contains("fn workbench_drag_pointer_event("),
        "workspace docking host must handle shared drag pointer events"
    );
}
