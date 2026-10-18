import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE_STATE = ROOT / "zircon_editor/src/ui/host/workspace_state.rs"
MANAGER = ROOT / "zircon_editor/src/ui/host/editor_manager_workspace.rs"
RUNTIME_ACCESS = (
    ROOT
    / "zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs"
)
DRAWER_TOGGLE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/layout/drawer_toggle.rs"
)
TAB_DROP = (
    ROOT / "zircon_editor/src/ui/retained_host/callback_dispatch/layout/tab_drop.rs"
)
FLOATING_CLOSE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs"
)
TOOLBAR_SIZE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs"
)
LOWER_REGRESSION = (
    ROOT / "zircon_editor/src/ui/host/tests/workspace_state_direct_query_batch_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class Editor863To866WorkspaceDirectQueryPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workspace_state = WORKSPACE_STATE.read_text(encoding="utf-8")
        cls.manager = MANAGER.read_text(encoding="utf-8")
        cls.runtime_access = RUNTIME_ACCESS.read_text(encoding="utf-8")
        cls.drawer_toggle = DRAWER_TOGGLE.read_text(encoding="utf-8")
        cls.tab_drop = TAB_DROP.read_text(encoding="utf-8")
        cls.floating_close = FLOATING_CLOSE.read_text(encoding="utf-8")
        cls.toolbar_size = TOOLBAR_SIZE.read_text(encoding="utf-8")
        cls.lower_regression = (
            LOWER_REGRESSION.read_text(encoding="utf-8")
            if LOWER_REGRESSION.exists()
            else ""
        )

    def test_editor863_drawer_toggle_uses_one_borrowed_state_query(self) -> None:
        body = function_body(
            self.drawer_toggle,
            "pub(crate) fn dispatch_builtin_host_drawer_toggle",
        )
        compact = "".join(body.split())

        self.assertIn(
            "runtime.active_drawer_toggle_state(slot,instance_id)?",
            compact,
        )
        self.assertNotIn("current_layout", body)

    def test_editor864_tab_drop_queries_only_the_target_drawer_mode(self) -> None:
        body = function_body(self.tab_drop, "pub(crate) fn dispatch_tab_drop")
        compact = "".join(body.split())

        self.assertIn("runtime.active_drawer_mode(*slot)", compact)
        self.assertNotIn("current_layout", body)

    def test_editor865_close_checks_floating_window_existence_without_layout_clone(self) -> None:
        body = function_body(
            self.floating_close,
            "pub(super) fn close_floating_window_without_prompt",
        )
        compact = "".join(body.split())

        self.assertIn("self.runtime.floating_window_exists(window_id)", compact)
        self.assertNotIn("current_layout", body)

    def test_editor866_toolbar_queries_only_the_matching_view_host(self) -> None:
        body = function_body(
            self.toolbar_size,
            "pub(in crate::ui::retained_host::app) fn viewport_toolbar_surface_size",
        )
        compact = "".join(body.split())

        self.assertIn(
            "self.runtime.view_host_for_instance_key(surface_key)",
            compact,
        )
        self.assertNotIn("current_view_instances", body)

    def test_queries_delegate_to_the_authoritative_workspace_owner(self) -> None:
        for signature, manager_call, owner_call in (
            (
                "pub(crate) fn active_drawer_toggle_state",
                ".manager.active_drawer_toggle_state",
                "self.host.active_drawer_toggle_state",
            ),
            (
                "pub(crate) fn active_drawer_mode",
                ".manager.active_drawer_mode",
                "self.host.active_drawer_mode",
            ),
            (
                "pub(crate) fn floating_window_exists",
                ".manager.floating_window_exists",
                "self.host.floating_window_exists",
            ),
            (
                "pub(crate) fn view_host_for_instance_key",
                ".manager.view_host_for_instance_key",
                "self.host.view_host_for_instance_key",
            ),
        ):
            runtime_body = function_body(self.runtime_access, signature)
            manager_signature = signature.replace("pub(crate)", "pub")
            manager_body = function_body(self.manager, manager_signature)
            self.assertIn(manager_call, "".join(runtime_body.split()))
            self.assertIn(owner_call, "".join(manager_body.split()))

    def test_owner_borrows_session_and_clones_only_the_requested_view_host(self) -> None:
        toggle_body = function_body(
            self.workspace_state,
            "pub(super) fn active_drawer_toggle_state",
        )
        mode_body = function_body(
            self.workspace_state,
            "pub(super) fn active_drawer_mode",
        )
        exists_body = function_body(
            self.workspace_state,
            "pub(super) fn floating_window_exists",
        )
        host_body = function_body(
            self.workspace_state,
            "pub(super) fn view_host_for_instance_key",
        )
        host_helper_body = function_body(
            self.workspace_state,
            "fn view_host_for_instance_key_in_session",
        )

        for body in (toggle_body, mode_body, exists_body, host_body):
            self.assertIn("lock_session", body)
            self.assertNotIn("current_layout", body)
            self.assertNotIn("current_view_instances", body)
        self.assertIn(
            ".map(|instance| instance.host.clone())",
            host_helper_body,
        )

    def test_lower_semantics_and_release_markers_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"tests/workspace_state_direct_query_batch_tests.rs\"]",
            self.workspace_state,
        )
        self.assertIn(
            "mod workspace_state_direct_query_batch_tests;",
            self.workspace_state,
        )
        self.assertTrue(self.lower_regression)
        for marker in (
            "EDITOR863_DRAWER_TOGGLE_DIRECT_QUERY_BENCH_V1",
            "EDITOR864_TAB_DROP_DRAWER_MODE_DIRECT_QUERY_BENCH_V1",
            "EDITOR865_FLOATING_WINDOW_EXISTS_DIRECT_QUERY_BENCH_V1",
            "EDITOR866_VIEW_HOST_DIRECT_QUERY_BENCH_V1",
        ):
            self.assertIn(marker, self.lower_regression)


if __name__ == "__main__":
    unittest.main()
