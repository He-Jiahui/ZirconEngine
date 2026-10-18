"""Source contract for the WOC bot's local deterministic product host.

The bot must no longer report identity and exit. It may run only a local,
deterministic policy until transport and persistence owners exist, but that
boundary has to be explicit in its report.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BOT = ROOT / "examples" / "woc" / "native" / "apps" / "woc_bot"


class WocBotProductHostContractTests(unittest.TestCase):
    def test_binary_enters_the_engine_owned_bot_host(self):
        main = (BOT / "src" / "main.rs").read_text(encoding="utf-8")
        self.assertIn("BotProductHost", main)
        self.assertNotIn("identity_report_json", main)

    def test_manifest_enables_the_real_engine_host_binary(self):
        manifest = (BOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('default = ["engine-host"]', manifest)
        self.assertIn('engine-host = ["woc_runtime/engine-host"]', manifest)
        self.assertIn('required-features = ["engine-host"]', manifest)

    def test_bot_host_is_a_real_vm_lifecycle_but_not_a_network_claim(self):
        host = (BOT / "src" / "main.rs").read_text(encoding="utf-8")
        self.assertIn("load_engine_project_vm", host)
        self.assertIn("WocHostRole::Bot", host)
        self.assertIn("vm.activate()", host)
        self.assertIn("vm.deactivate()", host)
        self.assertIn("install_offline_bootstrap", host)
        self.assertIn("checkpoint", host)
        self.assertIn("rollback_checkpoint", host)
        self.assertIn('"networkReady": false', host)
        self.assertIn('"persistenceReady": false', host)
        self.assertIn('"mode": "local_deterministic"', host)

    def test_default_path_is_the_project_root_not_the_script_package(self):
        options = (BOT / "src" / "main.rs").read_text(encoding="utf-8")
        self.assertIn('join("../../..")', options)
        self.assertNotIn('join("../../../scripts/woc_game")', options)


if __name__ == "__main__":
    unittest.main()
