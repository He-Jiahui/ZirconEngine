import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE_STATE = ROOT / "zircon_editor/src/ui/host/workspace_state.rs"
MANAGER = ROOT / "zircon_editor/src/ui/host/editor_manager_workspace.rs"
RUNTIME_ACCESS = ROOT / "zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs"
FOCUS = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/helpers/callback_surface/"
    "source_window/focus.rs"
)
LOWER_REGRESSION = (
    ROOT / "zircon_editor/src/ui/host/tests/workspace_state_surface_window_query_tests.rs"
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


class Editor861SurfaceWindowDirectQueryPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workspace_state = WORKSPACE_STATE.read_text(encoding="utf-8")
        cls.manager = MANAGER.read_text(encoding="utf-8")
        cls.runtime_access = RUNTIME_ACCESS.read_text(encoding="utf-8")
        cls.focus = FOCUS.read_text(encoding="utf-8")
        cls.lower_regression = (
            LOWER_REGRESSION.read_text(encoding="utf-8")
            if LOWER_REGRESSION.exists()
            else ""
        )

    def test_surface_focus_bypasses_the_full_chrome_snapshot(self) -> None:
        body = function_body(
            self.focus,
            "fn note_focused_floating_window_surface",
        )
        compact = "".join(body.split())

        self.assertIn(
            "self.runtime.floating_window_id_for_surface_key(surface_key)",
            compact,
        )
        self.assertNotIn("chrome_snapshot", body)
        self.assertNotIn("workbench_snapshot_access", body)

    def test_query_delegates_to_the_authoritative_workspace_owner(self) -> None:
        runtime_body = function_body(
            self.runtime_access,
            "pub(crate) fn floating_window_id_for_surface_key",
        )
        manager_body = function_body(
            self.manager,
            "pub fn floating_window_id_for_surface_key",
        )

        self.assertIn(
            ".manager.floating_window_id_for_surface_key",
            "".join(runtime_body.split()),
        )
        self.assertIn(
            "self.host.floating_window_id_for_surface_key",
            "".join(manager_body.split()),
        )

    def test_workspace_query_borrows_layout_and_clones_only_the_match(self) -> None:
        owner_body = function_body(
            self.workspace_state,
            "pub(super) fn floating_window_id_for_surface_key",
        )
        helper_body = function_body(
            self.workspace_state,
            "fn floating_window_id_for_surface_key_in_layout",
        )
        compact_owner = "".join(owner_body.split())
        compact_helper = "".join(helper_body.split())

        self.assertIn("letsession=self.lock_session();", compact_owner)
        self.assertIn(
            "floating_window_id_for_surface_key_in_layout(&session.layout,surface_key)",
            compact_owner,
        )
        self.assertNotIn("current_layout", owner_body)
        self.assertNotIn("chrome_snapshot", owner_body)
        self.assertIn("layout.floating_windows.iter().find", compact_helper)
        self.assertIn("window.window_id.0==surface_key", compact_helper)
        self.assertIn(".map(|window|window.window_id.clone())", compact_helper)

    def test_lower_semantics_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"tests/workspace_state_surface_window_query_tests.rs\"]",
            self.workspace_state,
        )
        self.assertIn("mod workspace_state_surface_window_query_tests;", self.workspace_state)
        self.assertTrue(self.lower_regression)
        self.assertIn(
            "editor861_surface_window_query_preserves_exact_match_semantics",
            self.lower_regression,
        )
        self.assertIn(
            "EDITOR861_SURFACE_WINDOW_DIRECT_QUERY_BENCH_V1",
            self.lower_regression,
        )


if __name__ == "__main__":
    unittest.main()
