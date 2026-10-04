use std::fs;
use std::path::Path;

use super::super::support::collect_rust_files;
use super::support::source;

#[test]
fn editor_v2_projection_collectors_use_explicit_stacks() {
    let projection = source("src/ui/template_runtime/runtime/projection.rs");

    for required in [
        "let mut stack = vec![V2ProjectionFrame::Enter(root)]",
        "struct HostProjectionFrame",
        "let mut stack = vec![HostProjectionFrame",
        "let mut stack = vec![node_id]",
        "node_bindings_from_ids(",
    ] {
        assert!(
            projection.contains(required),
            "projection.rs missing explicit-stack marker `{required}`"
        );
    }
    for forbidden in [
        "collect_host_nodes(child,",
        "collect_surface_host_nodes(tree, *child_id",
    ] {
        assert!(
            !projection.contains(forbidden),
            "projection.rs should not recurse through host collectors via `{forbidden}`"
        );
    }
}

#[test]
fn pointer_handlers_do_not_force_slow_path_rebuilds() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "src/ui/retained_host/app/asset_content_pointer.rs",
        "src/ui/retained_host/app/asset_reference_pointer.rs",
        "src/ui/retained_host/app/asset_tree_pointer.rs",
        "src/ui/retained_host/app/detail_scroll_pointer.rs",
        "src/ui/retained_host/app/hierarchy_pointer.rs",
        "src/ui/retained_host/app/menu_pointer.rs",
        "src/ui/retained_host/app/viewport.rs",
        "src/ui/retained_host/app/welcome_recent_pointer.rs",
        "src/ui/retained_host/app/workbench_pointer.rs",
        "src/ui/retained_host/app/workspace_docking.rs",
    ] {
        let parent = manifest_root.join(relative);
        let mut owners = vec![parent.clone()];
        owners.extend(collect_rust_files(&parent.with_extension("")));
        let mut uses_committed_layout = false;
        for path in owners {
            let text =
                fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
            uses_committed_layout |= text.split("#[cfg(test)]").next().is_some_and(|production| {
                production.contains("self.use_committed_pointer_layout();")
            });
            for forbidden in ["recompute_if_dirty(", "chrome_snapshot("] {
                assert!(
                    !text.contains(forbidden),
                    "{path:?} should use committed layout and chrome caches inside pointer callbacks, not `{forbidden}`"
                );
            }
        }
        assert!(
            uses_committed_layout,
            "{relative} or its child owners should route pointer events against the last committed layout"
        );
    }

    let lifecycle = source("src/ui/retained_host/app/host_lifecycle/tick.rs");
    let pointer_layout = lifecycle
        .split("fn use_committed_pointer_layout(&self)")
        .nth(1)
        .and_then(|tail| tail.split("\n    }").next())
        .expect("host lifecycle tick should expose the committed pointer-layout entrypoint");
    assert!(
        pointer_layout.contains("self.publish_refresh_invalidation_diagnostics();"),
        "committed pointer-layout entrypoint should publish diagnostics without rebuilding"
    );
    assert!(!pointer_layout.contains("recompute_if_dirty("));
    assert!(!pointer_layout.contains("chrome_snapshot("));
}

#[test]
fn host_dirty_flags_route_through_invalidation_root_outside_lifecycle_owner() {
    let lifecycle =
        source("src/ui/retained_host/app/host_lifecycle/invalidation_bridge/dirty_marking.rs");
    for required in [
        "fn mark_presentation_dirty(&mut self)",
        "HostInvalidationMask::PRESENTATION_DATA",
    ] {
        assert!(
            lifecycle.contains(required),
            "host lifecycle missing invalidation entrypoint marker `{required}`"
        );
    }
    let dirty_flags =
        source("src/ui/retained_host/app/host_lifecycle/invalidation_bridge/dirty_flags.rs");
    assert!(dirty_flags.contains("self.invalidation.invalidate(mask);"));
    assert!(dirty_flags.contains("self.invalidation.invalidate_shell_content(scope, mask);"));

    let app_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/retained_host/app");
    let dirty_flags_path = app_root.join("host_lifecycle/invalidation_bridge/dirty_flags.rs");
    for path in collect_rust_files(&app_root) {
        if path == dirty_flags_path {
            continue;
        }
        let text =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {:?}: {error}", path));
        for forbidden in [
            "self.presentation_dirty = true",
            "self.layout_dirty = true",
            "self.window_metrics_dirty = true",
            "self.render_dirty = true",
        ] {
            assert!(
                !text.contains(forbidden),
                "{:?} should route host dirty state through HostInvalidationRoot, not `{forbidden}`",
                path.file_name().expect("file name")
            );
        }
    }

    let ui_asset_editor = source("src/ui/retained_host/app/ui_asset_editor/actions.rs");
    assert!(
        ui_asset_editor.contains("mark_presentation_dirty()"),
        "UI Asset Editor actions should mark presentation changes through dirty-domain invalidation"
    );
}

#[test]
fn retained_event_effects_route_dirty_domains_through_invalidation_mask() {
    let event_bridge = source("src/ui/retained_host/event_bridge.rs");
    for required in [
        "pub dirty_domains: HostInvalidationMask",
        "fn request_presentation(&mut self)",
        "fn request_layout(&mut self)",
        "fn request_render(&mut self)",
        "fn request_render_and_presentation(&mut self)",
        "fn request_paint_only(&mut self)",
        "fn dirty_domains(&self) -> HostInvalidationMask",
        "fn merge_dirty_domains(&mut self, dirty_domains: HostInvalidationMask)",
    ] {
        assert!(
            event_bridge.contains(required),
            "retained event bridge missing dirty-domain marker `{required}`"
        );
    }

    let lifecycle = source("src/ui/retained_host/app/host_lifecycle/dispatch_effects.rs");
    assert!(
        lifecycle.contains("let dirty_domains = effects.dirty_domains();")
            && lifecycle.contains("self.invalidate_host_for_shell_content(scope, dirty_domains);")
            && lifecycle.contains("self.invalidate_host(dirty_domains);"),
        "host lifecycle should consume event dirty domains for scoped and unscoped invalidation"
    );

    let common_effects = source("src/ui/retained_host/callback_dispatch/common/effects.rs");
    assert!(
        common_effects.contains("target.merge_dirty_domains(source.dirty_domains())"),
        "callback effect merging should preserve dirty-domain provenance"
    );

    let drawer_resize = source("src/ui/retained_host/drawer_resize.rs");
    let resize_movement =
        source("src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs");
    assert!(
        drawer_resize.contains("dispatch_resize_to_group(")
            && drawer_resize.contains("Result<UiHostEventEffects, String>")
            && resize_movement.contains("self.apply_dispatch_effects(effects);"),
        "drawer resize effects should reach the host dirty-domain invalidation path"
    );

    let retained_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/retained_host");
    let dirty_flags_path =
        retained_root.join("app/host_lifecycle/invalidation_bridge/dirty_flags.rs");
    for path in collect_rust_files(&retained_root) {
        if path == dirty_flags_path || path == retained_root.join("event_bridge.rs") {
            continue;
        }
        let text =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {:?}: {error}", path));
        for forbidden in [
            ".presentation_dirty |=",
            ".layout_dirty |=",
            ".render_dirty |=",
            ".presentation_dirty = true",
            ".layout_dirty = true",
            ".render_dirty = true",
        ] {
            assert!(
                !text.contains(forbidden),
                "{:?} should merge event dirty state through UiHostEventEffects dirty domains, not `{forbidden}`",
                path.file_name().expect("file name")
            );
        }
    }
}
