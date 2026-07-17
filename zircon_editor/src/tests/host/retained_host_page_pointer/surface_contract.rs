fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

fn method_source<'a>(source: &'a str, method_name: &str, next_method_name: &str) -> &'a str {
    let method_marker = format!("fn {method_name}");
    let next_method_marker = format!("fn {next_method_name}");
    source
        .split(&method_marker)
        .nth(1)
        .and_then(|source| source.split(&next_method_marker).next())
        .unwrap_or_else(|| panic!("method `{method_name}` should exist"))
}

#[test]
fn shared_host_page_surface_uses_rust_owned_pointer_callbacks() {
    let globals = source("src/ui/retained_host/host_contract/globals/ui_context.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/host_shell/chrome.rs");
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout/shell_chrome.rs");
    let host_page_sync = method_source(
        &pointer_layout,
        "sync_host_page_pointer_layout",
        "sync_document_tab_pointer_layout",
    );

    assert!(globals.contains("on_host_page_pointer_clicked"));
    assert!(wiring.contains("host_shell.on_host_page_pointer_clicked("));
    assert!(host_page_sync.contains("self.template_bridge.outer_shell_frames()"));
    assert!(!host_page_sync.contains("self.template_bridge.root_shell_frames()"));
    assert!(host_page_sync.contains("build_host_page_pointer_layout("));
    assert!(!wiring.contains("on_activate_host_page"));
}

#[test]
fn shared_host_page_pointer_layout_keeps_outer_shell_owner_contract() {
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout/shell_chrome.rs");
    let pointer_builder =
        source("src/ui/retained_host/host_page_pointer/build_host_page_pointer_layout.rs");
    let workbench_bridge =
        source("src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs");
    let workbench_layout_frames =
        source("src/ui/retained_host/callback_dispatch/template_bridge/workbench/layout_frames.rs");
    let host_page_sync = method_source(
        &pointer_layout,
        "sync_host_page_pointer_layout",
        "sync_document_tab_pointer_layout",
    );

    assert!(workbench_bridge.contains("pub(crate) fn outer_shell_frames(&self)"));
    assert!(host_page_sync
        .contains("let outer_shell_frames = self.template_bridge.outer_shell_frames();"));
    assert!(pointer_builder.contains("BuiltinHostOuterShellFrames"));
    assert!(pointer_builder.contains("host_page_strip_frame"));
    assert!(
        !workbench_layout_frames.contains("host_page_strip_frame"),
        "componentized Workbench layout frames must not grow an outer host-page strip owner"
    );
}

#[test]
fn host_page_pointer_bridge_uses_route_intent_only() {
    let bridge = source("src/ui/retained_host/host_page_pointer/host_page_pointer_bridge.rs");
    let rebuild = source("src/ui/retained_host/host_page_pointer/rebuild_surface.rs");
    let dispatch = source("src/ui/retained_host/host_page_pointer/dispatch_event.rs");

    assert!(bridge.contains("route_intents: EditorRouteIntentMap"));
    assert!(rebuild.contains("EditorRouteIntent::HostPage"));
    assert!(rebuild.contains("route_intents.bind_node"));
    assert!(dispatch.contains("host_page_route_for_pointer_dispatch"));
    for forbidden in [
        "targets:",
        "HostPagePointerTarget",
        "handled_by",
        "route.target",
    ] {
        assert!(
            !bridge.contains(forbidden)
                && !rebuild.contains(forbidden)
                && !dispatch.contains(forbidden),
            "host page pointer bridge should not keep old hit target marker `{forbidden}`"
        );
    }
}

#[test]
fn host_page_pointer_does_not_maintain_unused_measured_frames() {
    let bridge = source("src/ui/retained_host/host_page_pointer/host_page_pointer_bridge.rs");
    let sync = source("src/ui/retained_host/host_page_pointer/sync.rs");
    let click = source("src/ui/retained_host/host_page_pointer/handle_click.rs");

    for candidate in [&bridge, &sync, &click] {
        assert!(!candidate.contains("measured_frames"));
    }
    assert!(click.contains("let Some(callback_frame)"));
    assert!(!click.contains("self.rebuild_surface()"));
}
