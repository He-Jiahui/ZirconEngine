import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE_STATE = ROOT / "zircon_editor/src/ui/host/workspace_state.rs"
MANAGER = ROOT / "zircon_editor/src/ui/host/editor_manager_workspace.rs"
RUNTIME_ACCESS = ROOT / "zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs"
DISPATCH = (
    ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/layout/floating_window/dispatch.rs"
)
LOWER_REGRESSION = (
    ROOT / "zircon_editor/src/ui/host/tests/workspace_state_floating_focus_query_tests.rs"
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


class Editor860FloatingFocusDirectQueryPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workspace_state = WORKSPACE_STATE.read_text(encoding="utf-8")
        cls.manager = MANAGER.read_text(encoding="utf-8")
        cls.runtime_access = RUNTIME_ACCESS.read_text(encoding="utf-8")
        cls.dispatch = DISPATCH.read_text(encoding="utf-8")
        cls.lower_regression = (
            LOWER_REGRESSION.read_text(encoding="utf-8")
            if LOWER_REGRESSION.exists()
            else ""
        )

    def test_focus_dispatch_bypasses_chrome_and_workbench_model_projection(self) -> None:
        body = function_body(
            self.dispatch,
            "pub(crate) fn dispatch_builtin_floating_window_focus(",
        )
        compact = "".join(body.split())

        self.assertIn(
            "letinstance_id=runtime.floating_window_focus_target(window_id)?;",
            compact,
        )
        self.assertNotIn("chrome_snapshot", body)
        self.assertNotIn("project_command_eval_snapshot", body)
        self.assertNotIn("build_workbench_view_model", body)
        self.assertNotIn("resolve_floating_window_focus_instance", body)

    def test_query_delegates_to_the_authoritative_workspace_owner(self) -> None:
        runtime_body = function_body(
            self.runtime_access,
            "pub(crate) fn floating_window_focus_target",
        )
        manager_body = function_body(
            self.manager,
            "pub fn floating_window_focus_target",
        )

        self.assertIn(
            ".manager.floating_window_focus_target",
            "".join(runtime_body.split()),
        )
        self.assertIn(
            "self.host.floating_window_focus_target",
            "".join(manager_body.split()),
        )

    def test_workspace_query_borrows_layout_and_scans_tabs_once(self) -> None:
        owner_body = function_body(
            self.workspace_state,
            "pub(super) fn floating_window_focus_target",
        )
        layout_body = function_body(
            self.workspace_state,
            "fn floating_window_focus_target_in_layout",
        )
        visitor_body = function_body(
            self.workspace_state,
            "fn visit_document_focus_target",
        )
        compact_owner = "".join(owner_body.split())
        compact_layout = "".join(layout_body.split())
        compact_visitor = "".join(visitor_body.split())

        self.assertIn("letsession=self.lock_session();", compact_owner)
        self.assertIn("floating_window_focus_target_in_layout(&session.layout,window_id)", compact_owner)
        self.assertNotIn("current_layout", owner_body)
        self.assertNotIn("chrome_snapshot", owner_body)

        self.assertIn("layout.floating_windows.iter().find", compact_layout)
        self.assertIn("visit_document_focus_target", layout_body)
        self.assertIn(".or(active).or(first)", compact_layout)
        self.assertIn("DocumentNode::SplitNode", visitor_body)
        self.assertIn("DocumentNode::Tabs", visitor_body)
        self.assertIn("fortabin&stack.tabs", compact_visitor)
        self.assertIn("focused==Some(tab)", compact_visitor)
        self.assertIn("active.is_none()", visitor_body)
        self.assertNotIn("collect::<Vec", visitor_body)

    def test_lower_priority_semantics_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"tests/workspace_state_floating_focus_query_tests.rs\"]",
            self.workspace_state,
        )
        self.assertIn("mod workspace_state_floating_focus_query_tests;", self.workspace_state)
        self.assertTrue(self.lower_regression)
        self.assertIn(
            "editor860_floating_focus_query_preserves_focused_active_first_priority",
            self.lower_regression,
        )
        self.assertIn(
            "editor860_floating_focus_query_rejects_missing_or_empty_windows",
            self.lower_regression,
        )
        self.assertIn(
            "EDITOR860_FLOATING_FOCUS_DIRECT_QUERY_BENCH_V1",
            self.lower_regression,
        )


if __name__ == "__main__":
    unittest.main()
