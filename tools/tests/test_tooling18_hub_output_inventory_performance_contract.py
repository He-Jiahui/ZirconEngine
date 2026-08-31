from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_build_hub_outputs import _find_artifact


SCRIPT = Path(__file__).resolve().parents[1] / "zircon_build_hub_outputs.py"


class HubOutputInventoryPerformanceContractTests(unittest.TestCase):
    def test_direct_artifact_hit_does_not_start_recursive_search(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            target_dir = Path(temporary_directory)
            artifact = target_dir / "debug" / "zircon_hub.exe"
            artifact.parent.mkdir()
            artifact.write_bytes(b"hub")

            with mock.patch.object(
                Path, "rglob", side_effect=AssertionError("recursive scan")
            ):
                resolved = _find_artifact(target_dir, "debug", artifact.name)

        self.assertEqual(artifact, resolved)

    def test_installer_inventory_uses_one_directory_walk(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[
            source.index("def stage_hub_tauri_installers(") :
            source.index("def _copy_artifact(")
        ]

        self.assertIn("os.walk(bundle_root)", function)
        self.assertNotIn('bundle_root.rglob("*")', function)
        self.assertNotIn("source.is_dir()", function)
        self.assertNotIn("source.is_file()", function)


if __name__ == "__main__":
    unittest.main()
