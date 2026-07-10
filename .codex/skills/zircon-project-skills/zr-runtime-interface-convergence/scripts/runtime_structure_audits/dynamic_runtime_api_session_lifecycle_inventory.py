from __future__ import annotations


HEADLESS_LIFECYCLE_ANCHORS = (
    (
        "zircon_runtime/src/dynamic_api/session.rs",
        "render_bridge: Option<RuntimeRenderBridge>",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/profile.rs",
        "RUNTIME_SESSION_PROFILE_MINIMAL",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/profile.rs",
        "RUNTIME_SESSION_PROFILE_HEADLESS",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/profile.rs",
        "fn uses_render_bridge(self) -> bool",
    ),
    (
        "zircon_runtime/src/dynamic_api/session.rs",
        "runtime_dynamic_session_render_bridge_skipped",
    ),
    ("zircon_runtime/src/dynamic_api/session.rs", "empty_captured_frame(requested)"),
    (
        "zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs",
        "session_destroy_reports_explicit_not_found_after_headless_destroy",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
        "destroyed_headless_session_entry_points_reject_old_handle",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_profiles.rs",
        "create_session_accepts_named_headless_profile_without_render_bridge",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_profiles.rs",
        "minimal_and_headless_profiles_skip_render_bridge_bootstrap",
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs",
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
    ),
    (
        "docs/zircon_runtime/dynamic_api/session.md",
        "frame capture returns an empty encoded frame",
    ),
)
