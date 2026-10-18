
from tools.tests.rust_test_files import read_rust_test_file
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DOCUMENT_NODE = ROOT / "zircon_editor/src/ui/workbench/layout/document_node.rs"
WORKSPACE_STATE = ROOT / "zircon_editor/src/ui/host/workspace_state.rs"
MANAGER = ROOT / "zircon_editor/src/ui/host/editor_manager_workspace.rs"
RUNTIME_ACCESS = (
    ROOT
    / "zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs"
)
FLOATING_CLOSE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs"
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


class Editor872FloatingWindowInstanceIdsDirectQueryPerformanceContract(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.document_node = DOCUMENT_NODE.read_text(encoding="utf-8")
        cls.workspace_state = WORKSPACE_STATE.read_text(encoding="utf-8")
        cls.manager = MANAGER.read_text(encoding="utf-8")
        cls.runtime_access = RUNTIME_ACCESS.read_text(encoding="utf-8")
        cls.floating_close = FLOATING_CLOSE.read_text(encoding="utf-8")
        cls.lower_regression = LOWER_REGRESSION.read_text(encoding="utf-8")

    def test_close_projects_target_window_ids_without_a_layout_snapshot(self) -> None:
        body = function_body(
            self.floating_close,
            "pub(super) fn floating_window_close_instance_ids",
        )
        compact = "".join(body.split())
        self.assertIn("self.runtime.floating_window_instance_ids(window_id)", compact)
        self.assertNotIn("current_layout", body)

    def test_query_delegates_to_the_authoritative_workspace_owner(self) -> None:
        manager = function_body(self.manager, "pub fn floating_window_instance_ids")
        runtime = function_body(
            self.runtime_access,
            "pub(crate) fn floating_window_instance_ids",
        )
        self.assertIn("self.host.floating_window_instance_ids", "".join(manager.split()))
        self.assertIn(".manager.floating_window_instance_ids", "".join(runtime.split()))

    def test_owner_borrows_layout_and_clones_only_target_identities(self) -> None:
        body = function_body(
            self.workspace_state,
            "pub(super) fn floating_window_instance_ids",
        )
        helper = function_body(
            self.workspace_state,
            "fn floating_window_instance_ids_in_layout",
        )
        self.assertIn("lock_session", body)
        self.assertNotIn("current_layout", body)
        self.assertIn("window.workspace.instance_count()", helper)
        self.assertIn("window.workspace.append_instance_ids", helper)

    def test_document_node_owns_depth_first_identity_projection(self) -> None:
        count = function_body(self.document_node, "pub(crate) fn instance_count")
        append = function_body(self.document_node, "pub(crate) fn append_instance_ids")
        self.assertIn("saturating_add", count)
        self.assertIn("out.extend", append)
        self.assertIn("first.append_instance_ids", append)
        self.assertIn("second.append_instance_ids", append)

    def test_lower_semantics_and_release_markers_are_wired(self) -> None:
        self.assertIn(
            "EDITOR872_FLOATING_WINDOW_INSTANCE_IDS_DIRECT_QUERY_BENCH_V1",
            self.lower_regression,
        )
        self.assertIn(
            "EDITOR314_FLOATING_WINDOW_INSTANCE_CAPACITY_BENCH_V1",
            read_rust_test_file("zircon_editor/src/ui/retained_host/app/native_window_close/tests/floating_window_optimization_batch_20260830bp_editor_tests.rs"),
        )


if __name__ == "__main__":
    unittest.main()
