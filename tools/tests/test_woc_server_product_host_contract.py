"""Source contract for the WOC server product composition root.

The server binary must exercise the real engine-host adapter and fixed-tick
driver.  This intentionally stays parse-only until the managed Cargo lane can
admit the external ZrVM dependency.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SERVER_MAIN = ROOT / "examples" / "woc" / "native" / "apps" / "woc_server" / "src" / "main.rs"
SERVER_MANIFEST = SERVER_MAIN.parents[1] / "Cargo.toml"
SERVER_DRIVER = SERVER_MAIN.parent / "fixed_tick_driver.rs"
SERVER_TESTS = SERVER_MAIN.parents[1] / "tests" / "fixed_tick_driver.rs"


class WocServerProductHostContractTests(unittest.TestCase):
    def test_server_main_composes_real_engine_host_and_fixed_driver(self):
        source = SERVER_MAIN.read_text(encoding="utf-8")
        for anchor in (
            "load_engine_project_vm",
            "WocHostRole::Server",
            "use woc_server::{FixedServerTickDriver",
            "FixedServerTickDriver",
            "SERVER_TICK_NS",
            ".activate()",
            ".deactivate()",
            "verify_replay",
        ):
            self.assertIn(anchor, source, anchor)

    def test_server_manifest_enables_engine_host_binary(self):
        source = SERVER_MANIFEST.read_text(encoding="utf-8")
        self.assertIn('default = ["engine-host"]', source)
        self.assertIn('engine-host = ["woc_runtime/engine-host"]', source)
        self.assertIn('required-features = ["engine-host"]', source)

    def test_driver_exposes_owned_vm_for_orderly_shutdown(self):
        source = SERVER_DRIVER.read_text(encoding="utf-8")
        self.assertIn("pub fn into_vm(self)", source)
        self.assertIn("new_with_vm_recovery", source)

    def test_faulted_batch_remains_retryable_after_runtime_recovery(self):
        source = SERVER_DRIVER.read_text(encoding="utf-8")
        self.assertIn("restore_pending_batch", source)
        self.assertIn("SessionNotRunning", source)
        tests = SERVER_TESTS.read_text(encoding="utf-8")
        self.assertIn("recovered_server_retries_the_same_failed_batch", tests)

    def test_server_report_does_not_claim_visual_output(self):
        source = SERVER_MAIN.read_text(encoding="utf-8")
        self.assertIn('"presentationReady": false', source)
        self.assertIn('"renderedFrames": 0', source)


if __name__ == "__main__":
    unittest.main()
