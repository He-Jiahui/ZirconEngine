from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
CONTRACT_RS = ROOT / (
    "zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/contract.rs"
)


def source() -> str:
    return CONTRACT_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def input_body() -> str:
    text = source()
    return text.split("enum NativePluginDiscoveryRefreshInput", 1)[1].split(
        "/// Non-empty collector-owned identity", 1
    )[0]


class Plugins21SharedDiscoveryInputPathContract(unittest.TestCase):
    def test_load_manifest_input_shares_its_export_root(self) -> None:
        body = compact(input_body())

        self.assertIn("LoadManifest{export_root:Arc<PathBuf>}", body)
        self.assertNotIn("LoadManifest{export_root:PathBuf}", body)

    def test_load_manifest_constructor_allocates_the_shared_owner_once(self) -> None:
        body = compact(input_body())

        self.assertIn("Self::LoadManifest{export_root:Arc::new(export_root),", body)

    def test_root_scan_remains_a_payload_free_input(self) -> None:
        body = compact(input_body())

        self.assertIn("RootScan,", body)
        self.assertNotIn("RootScan{", body)

    def test_shared_clone_behavior_has_a_direct_rust_contract(self) -> None:
        self.assertIn("load_manifest_input_clones_share_export_root", source())
        self.assertIn("Arc::ptr_eq", source())


if __name__ == "__main__":
    unittest.main()
