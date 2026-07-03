import os
import tempfile
import types
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_HUB = REPO_ROOT / "tools/zircon_build_hub.py"


class ZirconBuildHubOwnerBoundaryTests(unittest.TestCase):
    def test_hub_tauri_build_lives_in_hub_owner(self):
        self.assertTrue(
            ZIRCON_BUILD_HUB.exists(),
            "Hub/Tauri build and installer staging belong in zircon_build_hub.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        hub_text = ZIRCON_BUILD_HUB.read_text(encoding="utf-8")

        self.assertIn("from .zircon_build_hub import build_hub", build_text)
        self.assertIn("from zircon_build_hub import build_hub", build_text)

        for function_name in (
            "build_hub",
            "tauri_cli_path",
            "run_tauri_build",
            "stage_hub_tauri_outputs",
            "stage_hub_tauri_installers",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in zircon_build_hub.py",
            )
            self.assertIn(f"def {function_name}(", hub_text)

        for constant_name in ("HUB_TAURI_BUNDLE_TARGET", "HUB_INSTALLERS_DIR_NAME"):
            self.assertNotIn(
                f"{constant_name} =",
                build_text,
                f"{constant_name} belongs in zircon_build_hub.py",
            )
            self.assertIn(f"{constant_name} =", hub_text)

        self.assertLessEqual(
            len(hub_text.splitlines()),
            180,
            "zircon_build_hub.py should stay focused on Hub/Tauri staging",
        )

    def test_hub_owner_preserves_staging_semantics(self):
        from tools.zircon_build_hub import (
            build_hub,
            stage_hub_tauri_outputs,
        )

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo_root = root / "repo"
            target_dir = root / "targets" / "hub"
            engine_root = root / "out" / "ZirconEngine"
            profile_root = target_dir / "debug"
            bundle_root = profile_root / "bundle" / "nsis"
            bundle_root.mkdir(parents=True)
            hub_exe = "zircon_hub.exe" if os.name == "nt" else "zircon_hub"
            (profile_root / hub_exe).write_text("hub", encoding="utf-8")
            (bundle_root / "installer.txt").write_text("installer", encoding="utf-8")

            config = types.SimpleNamespace(
                repo_root=repo_root,
                targets_root=root / "targets",
                engine_root=engine_root,
                profile_dir="debug",
                dry_run=False,
                mode="debug",
            )

            stage_hub_tauri_outputs(config, target_dir)

            self.assertEqual(
                "hub",
                (engine_root / hub_exe).read_text(encoding="utf-8"),
            )
            self.assertEqual(
                "installer",
                (engine_root / "installers" / "installer.txt").read_text(
                    encoding="utf-8"
                ),
            )

            config.mode = "profiling"
            with self.assertRaisesRegex(
                SystemExit,
                "--mode profiling is not supported for the hub/Tauri target",
            ):
                build_hub(config)


if __name__ == "__main__":
    unittest.main()
