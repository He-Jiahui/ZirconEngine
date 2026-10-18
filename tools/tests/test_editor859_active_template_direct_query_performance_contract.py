import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE_STATE = ROOT / "zircon_editor/src/ui/host/workspace_state.rs"
MANAGER = ROOT / "zircon_editor/src/ui/host/editor_manager_workspace.rs"
RUNTIME_ACCESS = ROOT / "zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs"
RETAINED_ACCESS = ROOT / "zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs"
LOWER_REGRESSION = (
    ROOT / "zircon_editor/src/ui/host/tests/workspace_state_active_template_query_tests.rs"
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


class Editor859ActiveTemplateDirectQueryPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workspace_state = WORKSPACE_STATE.read_text(encoding="utf-8")
        cls.manager = MANAGER.read_text(encoding="utf-8")
        cls.runtime_access = RUNTIME_ACCESS.read_text(encoding="utf-8")
        cls.retained_access = RETAINED_ACCESS.read_text(encoding="utf-8")
        cls.lower_regression = (
            LOWER_REGRESSION.read_text(encoding="utf-8")
            if LOWER_REGRESSION.exists()
            else ""
        )

    def test_hot_predicate_bypasses_the_full_chrome_snapshot(self) -> None:
        body = function_body(
            self.retained_access,
            "pub(super) fn active_activity_window_template_document_is",
        )
        compact = "".join(body.split())

        self.assertIn(
            "self.runtime.active_activity_window_template_document_is(document_id)",
            compact,
        )
        self.assertNotIn("chrome_snapshot()", body)
        self.assertNotIn("active_activity_window_template_document_id(&chrome)", body)

    def test_query_delegates_to_the_authoritative_workspace_owner(self) -> None:
        runtime_body = function_body(
            self.runtime_access,
            "pub(crate) fn active_activity_window_template_document_is",
        )
        manager_body = function_body(
            self.manager,
            "pub fn active_activity_window_template_document_is",
        )
        compact_runtime = "".join(runtime_body.split())
        compact_manager = "".join(manager_body.split())

        self.assertIn(".manager.active_activity_window_template_document_is", compact_runtime)
        self.assertIn(
            "self.host.active_activity_window_template_document_is", compact_manager
        )

    def test_workspace_query_borrows_layout_and_registry_without_projection(self) -> None:
        body = function_body(
            self.workspace_state,
            "pub(super) fn active_activity_window_template_document_is",
        )
        helper = function_body(
            self.workspace_state,
            "fn active_activity_window_template_descriptor_id",
        )
        compact_helper = "".join(helper.split())

        self.assertIn("let session = self.lock_session();", body)
        self.assertIn("active_activity_window_template_descriptor_id(&session)", body)
        self.assertIn("let registry = self.lock_view_registry();", body)
        self.assertIn("registry.descriptor(descriptor_id)", body)
        self.assertIn("descriptor_capability_error", body)
        self.assertNotIn("current_layout", body)
        self.assertNotIn("list_descriptors", body)
        self.assertNotIn("EditorChromeSnapshot", body)

        self.assertIn("layout.main_pages.iter()", compact_helper)
        self.assertIn("MainHostPageLayout::WorkbenchPage", helper)
        self.assertIn("MainHostPageLayout::ExclusiveActivityWindowPage", helper)
        self.assertIn("layout.activity_windows().get(activity_window)", compact_helper)
        self.assertIn("session.open_view_instances.get(window_instance)", compact_helper)

    def test_lower_semantics_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"tests/workspace_state_active_template_query_tests.rs\"]",
            self.workspace_state,
        )
        self.assertIn("mod workspace_state_active_template_query_tests;", self.workspace_state)
        self.assertTrue(self.lower_regression)
        self.assertIn(
            "editor859_active_template_query_preserves_workbench_and_exclusive_semantics",
            self.lower_regression,
        )
        self.assertIn(
            "editor859_active_template_query_rejects_missing_authority",
            self.lower_regression,
        )
        self.assertIn(
            "EDITOR859_ACTIVE_TEMPLATE_DIRECT_QUERY_BENCH_V1",
            self.lower_regression,
        )


if __name__ == "__main__":
    unittest.main()
