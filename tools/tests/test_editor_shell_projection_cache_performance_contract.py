from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SHELL = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "shell_presentation.rs"
)
PROJECTION_CACHE = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/"
    "projection_cache.rs"
)
APPLY = ROOT / "zircon_editor/src/ui/retained_host/ui/apply_presentation.rs"


class EditorShellProjectionCachePerformanceContractTests(unittest.TestCase):
    def test_shell_presentation_has_one_cache_required_constructor(self) -> None:
        shell = SHELL.read_text(encoding="utf-8")

        self.assertNotIn("HostChromeProjectionCache::default()", shell)
        self.assertNotIn("from_state_with_template_v2_data", shell)
        self.assertEqual(
            len(re.findall(r"pub\(crate\) fn from_state\(", shell)),
            1,
        )
        self.assertIn("chrome_projection_cache: &mut HostChromeProjectionCache", shell)

    def test_production_and_identity_test_use_the_authoritative_entry(self) -> None:
        apply = APPLY.read_text(encoding="utf-8")
        cache = PROJECTION_CACHE.read_text(encoding="utf-8")

        self.assertIn("ShellPresentation::from_state(", apply)
        self.assertEqual(cache.count("ShellPresentation::from_state("), 2)
        self.assertNotIn("ShellPresentation::from_state_with_", apply + cache)


if __name__ == "__main__":
    unittest.main()
