"""Source contract for the WOC headless simulation product."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HEADLESS = ROOT / "examples" / "woc" / "native" / "apps" / "woc_headless"


class WocHeadlessProductHostContractTests(unittest.TestCase):
    def test_headless_binary_runs_a_real_transactional_vm(self):
        main = (HEADLESS / "src" / "main.rs").read_text(encoding="utf-8")
        runner = main
        self.assertIn("run_headless", main)
        self.assertNotIn("identity_report_json", main)
        for anchor in (
            "load_engine_project_vm",
            "WocHostRole::Headless",
            "install_offline_bootstrap",
            "tick(Vec::new())",
            "rollback_checkpoint",
            ".deactivate()",
        ):
            self.assertIn(anchor, runner, anchor)

    def test_headless_manifest_has_an_engine_host_binary_gate(self):
        manifest = (HEADLESS / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('default = ["engine-host"]', manifest)
        self.assertIn('engine-host = ["woc_runtime/engine-host"]', manifest)
        self.assertIn('required-features = ["engine-host"]', manifest)

    def test_headless_report_is_explicitly_non_visual(self):
        runner = (HEADLESS / "src" / "main.rs").read_text(encoding="utf-8")
        self.assertIn('"presentationReady": false', runner)
        self.assertIn('"renderedFrames": 0', runner)


if __name__ == "__main__":
    unittest.main()
