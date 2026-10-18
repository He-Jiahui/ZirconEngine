import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKSPACE_STATE = ROOT / "zircon_editor/src/ui/host/workspace_state.rs"
MANAGER = ROOT / "zircon_editor/src/ui/host/editor_manager_workspace.rs"
RUNTIME_ACCESS = (
    ROOT
    / "zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs"
)
LAYOUT_COMMANDS = ROOT / "zircon_editor/src/ui/host/layout_commands.rs"
SHELL_STATE = ROOT / "zircon_editor/src/ui/workbench/shell_state.rs"
RENDER_SUBMISSION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs"
)
NATIVE_CLOSE = ROOT / "zircon_editor/src/ui/retained_host/app/native_window_close.rs"
EXTENSION_VIEWS = ROOT / "zircon_editor/src/ui/host/editor_extension_views.rs"
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


class Editor867To871WorkspaceIdentityProjectionPerformanceContract(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workspace_state = WORKSPACE_STATE.read_text(encoding="utf-8")
        cls.manager = MANAGER.read_text(encoding="utf-8")
        cls.runtime_access = RUNTIME_ACCESS.read_text(encoding="utf-8")
        cls.layout_commands = LAYOUT_COMMANDS.read_text(encoding="utf-8")
        cls.shell_state = SHELL_STATE.read_text(encoding="utf-8")
        cls.render_submission = RENDER_SUBMISSION.read_text(encoding="utf-8")
        cls.native_close = NATIVE_CLOSE.read_text(encoding="utf-8")
        cls.extension_views = EXTENSION_VIEWS.read_text(encoding="utf-8")
        cls.lower_regression = (
            LOWER_REGRESSION.read_text(encoding="utf-8")
            if LOWER_REGRESSION.exists()
            else ""
        )

    def test_editor867_layout_commands_use_direct_floating_window_existence(self) -> None:
        for signature in (
            "pub(super) fn attach_view_to_target",
            "pub(super) fn attach_instance",
        ):
            body = function_body(self.layout_commands, signature)
            self.assertIn("self.floating_window_exists", "".join(body.split()))
            self.assertNotIn("current_layout", body)

    def test_editor868_play_preview_queries_only_the_first_matching_identity(self) -> None:
        body = function_body(
            self.shell_state,
            "pub(crate) fn focus_play_preview_view",
        )
        self.assertIn(
            ".manager.view_instance_id_for_descriptor(&descriptor)",
            "".join(body.split()),
        )
        self.assertNotIn("current_view_instances", body)

    def test_editor869_render_retirement_projects_only_scene_view_identities(self) -> None:
        body = function_body(
            self.render_submission,
            "pub(super) fn submit_render_frame_if_dirty",
        )
        compact = "".join(body.split())
        self.assertIn(
            '.runtime.view_instance_ids_for_descriptor_key("editor.scene")',
            compact,
        )
        self.assertNotIn("current_view_instances", body)

    def test_editor870_main_close_collects_ids_only_when_prompt_is_required(self) -> None:
        body = function_body(
            self.native_close,
            "pub(super) fn native_main_window_close_requested",
        )
        compact = "".join(body.split())
        self.assertNotIn("current_view_instances", body)
        self.assertIn("self.runtime.current_view_instance_ids()", compact)
        self.assertLess(
            compact.index("if!dirty.is_empty()"),
            compact.index("self.runtime.current_view_instance_ids()"),
        )

    def test_editor871_extension_retirement_clones_only_matching_ids(self) -> None:
        body = function_body(
            self.extension_views,
            "pub(super) fn retire_extension_views",
        )
        compact = "".join(body.split())
        self.assertNotIn("current_view_instances", body)
        self.assertIn(
            "self.view_instance_ids_for_descriptors(&descriptor_ids)",
            compact,
        )

    def test_identity_queries_delegate_to_the_authoritative_workspace_owner(self) -> None:
        manager_first = function_body(
            self.manager,
            "pub fn view_instance_id_for_descriptor",
        )
        manager_all = function_body(
            self.manager,
            "pub fn current_view_instance_ids",
        )
        manager_filtered = function_body(
            self.manager,
            "pub fn view_instance_ids_for_descriptor_key",
        )
        runtime_all = function_body(
            self.runtime_access,
            "pub(crate) fn current_view_instance_ids",
        )
        runtime_filtered = function_body(
            self.runtime_access,
            "pub(crate) fn view_instance_ids_for_descriptor_key",
        )
        for body in (manager_first, manager_all, manager_filtered):
            self.assertIn("self.host.", "".join(body.split()))
        for body in (runtime_all, runtime_filtered):
            self.assertIn(".manager.", "".join(body.split()))

    def test_owner_borrows_one_session_and_lower_markers_are_wired(self) -> None:
        for signature in (
            "pub(super) fn view_instance_id_for_descriptor",
            "pub(super) fn current_view_instance_ids",
            "pub(super) fn view_instance_ids_for_descriptor_key",
            "pub(super) fn view_instance_ids_for_descriptors",
        ):
            body = function_body(self.workspace_state, signature)
            self.assertIn("lock_session", body)
            self.assertNotIn("current_view_instances", body)

        self.assertIn(
            "#[path = \"tests/workspace_state_identity_projection_tests.rs\"]",
            self.workspace_state,
        )
        for marker in (
            "EDITOR868_PLAY_PREVIEW_IDENTITY_QUERY_BENCH_V1",
            "EDITOR869_SCENE_VIEW_IDENTITY_PROJECTION_BENCH_V1",
            "EDITOR870_MAIN_CLOSE_IDENTITY_PROJECTION_BENCH_V1",
            "EDITOR871_EXTENSION_RETIRE_IDENTITY_PROJECTION_BENCH_V1",
        ):
            self.assertIn(marker, self.lower_regression)


if __name__ == "__main__":
    unittest.main()
