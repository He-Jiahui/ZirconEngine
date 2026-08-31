import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorEmbeddedPlaySessionContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_editor_contract_exposes_factory_owned_lease_without_abi_authority(self) -> None:
        contract = self.read(
            "zircon_editor/src/core/play/embedded_backend/session_contract.rs"
        )

        self.assertIn("pub trait PlaySessionFactory", contract)
        self.assertIn("pub trait PlaySessionLease", contract)
        self.assertIn("fn gateway(&self) -> SharedEditorRuntimeGateway", contract)
        self.assertIn("fn retire(&mut self) -> Result<PlaySessionRetireReport, String>", contract)
        self.assertNotIn("ZrRuntimeCreateSession", contract)
        self.assertNotIn("ZrRuntimeDestroySession", contract)

    def test_embedded_backend_materializes_scene_and_keeps_retirement_retryable(self) -> None:
        backend = self.read("zircon_editor/src/core/play/embedded_backend/mod.rs")

        self.assertIn("PlaySnapshotStore", backend)
        self.assertIn("ActiveEmbeddedPlaySession::Running", backend)
        self.assertIn("ActiveEmbeddedPlaySession::Stopped", backend)
        self.assertIn("lease.retire()", backend)
        self.assertIn("*active = ActiveEmbeddedPlaySession::Stopped", backend)
        self.assertNotIn("ProcessPlayBackend", backend)

    def test_start_attaches_backend_gateway_and_terminal_path_retires_after_detach(self) -> None:
        controller = self.read("zircon_editor/src/core/play/controller.rs")
        shutdown = self.read(
            "zircon_editor/src/ui/host/editor_host_event_controller/runtime_shutdown.rs"
        )
        menu = self.read(
            "zircon_editor/src/ui/host/editor_event_execution/menu_action.rs"
        )

        self.assertIn("backend_report.take_gateway()", controller)
        self.assertIn("self.play_domain.attach(gateway)", controller)
        self.assertIn("rollback_failed_gateway_attach", controller)
        self.assertIn('event: "request_play_with_attached_gateway"', controller)
        self.assertIn("backend remains live because stop rollback failed", controller)
        self.assertIn("retire_terminal_backend", controller)
        self.assertLess(
            shutdown.find("shutdown_play_gateway"),
            shutdown.find("shutdown_play_backend_retirement"),
        )
        self.assertLess(
            menu.find("detach_terminal_play_gateway"),
            menu.find("retire_terminal_backend"),
        )

    def test_app_factory_reloads_authenticated_build_set_and_owns_session_destroy(self) -> None:
        factory = self.read(
            "zircon_app/src/entry/entry_runner/editor/play_session_factory.rs"
        )
        library = self.read(
            "zircon_app/src/entry/runtime_library/loaded_runtime.rs"
        )
        session = self.read(
            "zircon_app/src/entry/runtime_library/runtime_session.rs"
        )

        self.assertIn("RuntimeLibraryPreflight", factory)
        self.assertIn("load_after_preflight", factory)
        self.assertIn('b"runtime"', factory)
        self.assertIn("request.scene()", factory)
        self.assertIn("Arc::try_unwrap", factory)
        self.assertIn("session.try_destroy()", factory)
        self.assertIn("artifact_manifest.build_set_id != self.build_set_id", library)
        self.assertIn("pub(in crate::entry) fn try_destroy", session)

    def test_project_product_paths_inject_embedded_backend(self) -> None:
        entry = self.read("zircon_app/src/entry/entry_runner/editor.rs")
        composition = self.read(
            "zircon_app/src/entry/entry_runner/editor/composition.rs"
        )
        startup = self.read("zircon_editor/src/ui/host/editor_host_startup.rs")

        self.assertIn("AppPlaySessionFactory", entry)
        self.assertIn("EmbeddedPlayBackend", entry)
        self.assertIn("with_play_backend", entry)
        self.assertIn("starts_with_project.then", entry)
        self.assertNotIn("runtime_preflight.as_ref().map(|preflight|", entry)
        self.assertIn("AppPlaySessionFactory", composition)
        self.assertIn("EmbeddedPlayBackend", composition)
        self.assertNotIn("ProcessPlayBackend::for_current_install", startup)


if __name__ == "__main__":
    unittest.main()
