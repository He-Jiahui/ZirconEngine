from __future__ import annotations

import os
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock
from uuid import uuid4


class ZirconBuildHubManagedRootsTests(unittest.TestCase):
    def test_hub_environment_routes_npm_cache_to_the_managed_target(self) -> None:
        from tools.zircon_build_hub import hub_cargo_environment

        managed_root = r"D:\ZirconBuilds" if os.name == "nt" else None
        with tempfile.TemporaryDirectory(dir=managed_root) as temporary_root:
            target_dir = Path(temporary_root) / "targets" / "hub"

            environment = hub_cargo_environment(target_dir)

            npm_cache = target_dir.resolve() / "npm-cache"
            self.assertEqual(str(npm_cache), environment["npm_config_cache"])
            self.assertTrue(npm_cache.is_dir())

    @unittest.skipUnless(os.name == "nt", "Windows staging roots are Windows-only")
    def test_staging_rejects_unmanaged_engine_root_before_copy(self) -> None:
        from tools.zircon_build_hub import stage_hub_tauri_outputs

        root = Path(r"C:\ZirconBuilds") / f"hub-must-not-be-created-{uuid4().hex}"
        config = types.SimpleNamespace(
            engine_root=root / "ZirconEngine",
            profile_dir="debug",
            dry_run=False,
        )
        target_dir = Path(r"D:\ZirconBuilds") / f"hub-target-{uuid4().hex}"

        self.assertFalse(root.exists())
        with (
            mock.patch("tools.zircon_build_hub._copy_artifact") as copy_artifact,
            mock.patch(
                "tools.zircon_build_hub.stage_hub_tauri_installers"
            ) as stage_installers,
            self.assertRaisesRegex(ValueError, "approved build root"),
        ):
            stage_hub_tauri_outputs(config, target_dir)

        copy_artifact.assert_not_called()
        stage_installers.assert_not_called()
        self.assertFalse(root.exists())

    @unittest.skipUnless(os.name == "nt", "Windows staging roots are Windows-only")
    def test_installer_staging_rejects_unmanaged_root_before_deletion(self) -> None:
        from tools.zircon_build_hub import stage_hub_tauri_installers

        root = Path(r"C:\ZirconBuilds") / f"hub-must-not-be-created-{uuid4().hex}"
        config = types.SimpleNamespace(dry_run=False)

        self.assertFalse(root.exists())
        with self.assertRaisesRegex(ValueError, "approved build root"):
            stage_hub_tauri_installers(
                Path(r"D:\ZirconBuilds") / f"bundle-{uuid4().hex}",
                root / "installers",
                config,
            )
        self.assertFalse(root.exists())


if __name__ == "__main__":
    unittest.main()
