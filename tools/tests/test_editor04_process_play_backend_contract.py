from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PLAY = ROOT / "zircon_editor" / "src" / "core" / "play"


class ProcessPlayBackendContractTests(unittest.TestCase):
    def source(self, relative: str) -> str:
        return (PLAY / relative).read_text(encoding="utf-8")

    def test_play_backend_has_typed_start_stop_and_poll_contract(self) -> None:
        contract = self.source("backend/contract.rs")
        report = self.source("backend/report.rs")

        self.assertIn("pub trait PlayBackend: Send + Sync", contract)
        self.assertIn("fn start(", contract)
        self.assertIn("fn stop(", contract)
        self.assertIn("fn poll(", contract)
        self.assertIn("pub enum PlayBackendPoll", report)
        self.assertIn("Exited", report)

    def test_process_command_uses_the_editor04_editor16_joint_flags(self) -> None:
        command = self.source("process_backend/command.rs")

        for token in [
            '"--project"',
            '"--runtime-session-profile"',
            '"runtime"',
            '"--play-scene"',
            '"--play-report-pipe"',
        ]:
            self.assertIn(token, command)
        self.assertIn("Stdio::piped", self.source("process_backend/child.rs"))

    def test_snapshot_store_owns_atomic_materialization_and_cleanup(self) -> None:
        store = self.source("snapshot/store.rs")
        source = self.source("snapshot/source.rs")

        self.assertIn('join(".zircon").join("play")', store)
        self.assertIn("rename", store)
        self.assertIn("cleanup", store)
        self.assertIn("DynamicScene::from_world", source)
        self.assertIn("to_versioned_json_pretty", source)

    def test_controller_orders_activation_backend_and_inverse_cleanup(self) -> None:
        controller = self.source("controller.rs")

        activate = controller.index(".activate(")
        backend_start = controller.index("backend.start(")
        backend_stop = controller.index("backend.stop(")
        deactivate = controller.index(".deactivate(", backend_stop)
        self.assertLess(activate, backend_start)
        self.assertLess(backend_stop, deactivate)
        self.assertIn("pub fn poll_backend", controller)
        self.assertIn("PlayTransitionCause::Crashed", controller)


if __name__ == "__main__":
    unittest.main()
