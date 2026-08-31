import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins" / "plugin_sdk" / "src" / "native.rs"


class PluginSdkNativeCapabilityNegotiationPerformanceContractTests(unittest.TestCase):
    def test_entry_negotiation_validates_the_host_once_before_capability_scans(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        entry = re.search(
            r"pub fn entry_report\(.*?\n    \}\n\}", source, flags=re.DOTALL
        )
        self.assertIsNotNone(entry)
        entry_source = entry.group(0)

        self.assertEqual(
            entry_source.count("host_functions_v3_are_compatible(host_functions)"), 1
        )
        self.assertNotIn("host_supports_all_capabilities_v3(", entry_source)
        self.assertNotIn("host_supports_any_capability_v3(", entry_source)
        self.assertIn("host_supports_capability_with_compatible_host_v3", source)


if __name__ == "__main__":
    unittest.main()
