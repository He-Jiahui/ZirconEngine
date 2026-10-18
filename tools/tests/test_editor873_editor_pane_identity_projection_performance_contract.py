import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE_STATE = ROOT / "zircon_editor/src/ui/host/workspace_state.rs"
MANAGER = ROOT / "zircon_editor/src/ui/host/editor_manager_workspace.rs"
RUNTIME_ACCESS = (
    ROOT
    / "zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs"
)
PANE_PAYLOADS = (
    ROOT / "zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs"
)
EDITOR_PANES = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads/editor_panes.rs"
)
LOWER_REGRESSION = (
    ROOT / "zircon_editor/src/ui/host/tests/workspace_state_identity_projection_tests.rs"
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


class Editor873EditorPaneIdentityProjectionPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workspace_state = WORKSPACE_STATE.read_text(encoding="utf-8")
        cls.manager = MANAGER.read_text(encoding="utf-8")
        cls.runtime_access = RUNTIME_ACCESS.read_text(encoding="utf-8")
        cls.pane_payloads = PANE_PAYLOADS.read_text(encoding="utf-8")
        cls.editor_panes = EDITOR_PANES.read_text(encoding="utf-8")
        cls.lower_regression = LOWER_REGRESSION.read_text(encoding="utf-8")

    def test_host_lifecycle_projects_only_requested_editor_pane_identities(self) -> None:
        body = function_body(
            self.pane_payloads,
            "pub(super) fn collect_host_lifecycle_pane_payloads",
        )
        compact = "".join(body.split())
        self.assertIn("self.runtime.editor_pane_instance_ids(", compact)
        self.assertNotIn("current_view_instances", body)

    def test_workspace_owner_filters_both_pane_families_in_one_session_scan(self) -> None:
        owner = function_body(
            self.workspace_state,
            "fn editor_pane_instance_ids_in_session",
        )
        public_query = function_body(
            self.workspace_state,
            "pub(super) fn editor_pane_instance_ids",
        )
        compact = "".join(owner.split())
        self.assertEqual(compact.count("for(instance_id,instance)in&session.open_view_instances"), 1)
        self.assertIn("UI_ASSET_EDITOR_DESCRIPTOR_ID", owner)
        self.assertIn("editor.animation_sequence", owner)
        self.assertIn("editor.animation_graph", owner)
        self.assertIn("lock_session", public_query)

    def test_query_delegates_through_manager_and_runtime_access(self) -> None:
        manager = function_body(self.manager, "pub fn editor_pane_instance_ids")
        runtime = function_body(
            self.runtime_access,
            "pub(crate) fn editor_pane_instance_ids",
        )
        self.assertIn("self.host.editor_pane_instance_ids", "".join(manager.split()))
        self.assertIn(".manager.editor_pane_instance_ids", "".join(runtime.split()))

    def test_editor_pane_collector_consumes_targeted_identity_vectors(self) -> None:
        body = function_body(self.editor_panes, "pub(super) fn collect_editor_panes")
        self.assertIn("Vec<ViewInstanceId>", self.editor_panes)
        self.assertNotIn("Vec<ViewInstance>", body)
        self.assertNotIn("descriptor_id", body)
        self.assertIn("for instance_id in ui_asset_instance_ids", body)
        self.assertIn("for instance_id in animation_instance_ids", body)

    def test_lower_semantics_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "EDITOR873_EDITOR_PANE_IDENTITY_PROJECTION_BENCH_V1",
            self.lower_regression,
        )
        self.assertIn("editor873_editor_pane_identity_projection", self.lower_regression)


if __name__ == "__main__":
    unittest.main()
