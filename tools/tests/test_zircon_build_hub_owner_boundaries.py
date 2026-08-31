import os
import tempfile
import types
import unittest
from unittest import mock
from uuid import uuid4
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_HUB = REPO_ROOT / "tools/zircon_build_hub.py"
ZIRCON_BUILD_HUB_OUTPUTS = REPO_ROOT / "tools/zircon_build_hub_outputs.py"


class ZirconBuildHubOwnerBoundaryTests(unittest.TestCase):
    def test_hub_tauri_build_lives_in_hub_owner(self):
        self.assertTrue(
            ZIRCON_BUILD_HUB.exists(),
            "Hub/Tauri execution belongs in zircon_build_hub.py and output staging "
            "belongs in zircon_build_hub_outputs.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        hub_text = ZIRCON_BUILD_HUB.read_text(encoding="utf-8")
        outputs_text = ZIRCON_BUILD_HUB_OUTPUTS.read_text(encoding="utf-8")

        self.assertIn("from .zircon_build_hub import build_hub", build_text)
        self.assertIn("from zircon_build_hub import build_hub", build_text)

        for function_name in (
            "build_hub",
            "tauri_cli_path",
            "run_tauri_build",
            "hub_cargo_environment",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in zircon_build_hub.py",
            )
            self.assertIn(f"def {function_name}(", hub_text)

        for function_name in (
            "stage_hub_tauri_outputs",
            "stage_hub_tauri_installers",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                hub_text,
                f"{function_name} belongs in zircon_build_hub_outputs.py",
            )
            self.assertIn(f"def {function_name}(", outputs_text)

        self.assertIn("from .zircon_build_hub_outputs import", hub_text)
        self.assertIn("from zircon_build_hub_outputs import", hub_text)

        for constant_name in ("HUB_TAURI_BUNDLE_TARGET", "HUB_INSTALLERS_DIR_NAME"):
            self.assertNotIn(f"{constant_name} =", hub_text)
            self.assertIn(f"{constant_name} =", outputs_text)

        self.assertIn("def hub_cargo_environment(", hub_text)
        self.assertIn("zircon_build_cargo_environment", hub_text)
        self.assertIn("managed_cargo_environment", hub_text)

        self.assertLessEqual(
            len(hub_text.splitlines()),
            130,
            "zircon_build_hub.py should stay focused on Hub/Tauri execution",
        )
        self.assertLessEqual(
            len(outputs_text.splitlines()),
            150,
            "zircon_build_hub_outputs.py should stay focused on output staging",
        )

    def test_hub_owner_preserves_staging_semantics(self):
        from tools.zircon_build_hub import build_hub, hub_cargo_environment
        from tools.zircon_build_hub_outputs import stage_hub_tauri_outputs

        with (
            tempfile.TemporaryDirectory() as tmp,
            mock.patch(
                "tools.zircon_build_cargo_environment."
                "assert_managed_windows_build_root"
            ),
            mock.patch(
                "tools.zircon_build_hub_outputs.assert_managed_windows_build_root"
            ),
        ):
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

            cargo_environment = hub_cargo_environment(target_dir)
            self.assertEqual(str(target_dir.resolve()), cargo_environment["CARGO_TARGET_DIR"])
            self.assertEqual(
                str(target_dir.resolve() / "cargo-home"),
                cargo_environment["CARGO_HOME"],
            )
            self.assertEqual(
                str(target_dir.resolve() / "sccache"),
                cargo_environment["SCCACHE_DIR"],
            )
            for name in ("TEMP", "TMP", "TMPDIR"):
                self.assertEqual(
                    str(target_dir.resolve() / "temporary"), cargo_environment[name]
                )

            config.mode = "profiling"
            with self.assertRaisesRegex(
                SystemExit,
                "--mode profiling is not supported for the hub/Tauri target",
            ):
                build_hub(config)

    def test_hub_dry_run_does_not_create_managed_cargo_directories(self):
        from tools.zircon_build_hub import run_tauri_build

        root = Path(r"D:\ZirconBuilds") / f"hub-dry-run-{uuid4().hex}"
        target_dir = root / "targets" / "hub"
        config = types.SimpleNamespace(
            repo_root=root,
            cargo="cargo",
            locked=False,
            jobs="",
            mode="debug",
            dry_run=True,
        )
        with (
            mock.patch(
                "tools.zircon_build_hub.tauri_cli_path",
                return_value=root / "tauri.js",
            ),
            mock.patch("tools.zircon_build_hub.hub_cargo_environment") as environment,
        ):
            run_tauri_build(config, target_dir)

        environment.assert_not_called()
        self.assertFalse(target_dir.exists())

    @unittest.skipUnless(os.name == "nt", "Windows staging roots are Windows-only")
    def test_hub_tauri_build_rejects_unmanaged_target_before_process_start(self):
        from tools.zircon_build_hub import run_tauri_build

        root = Path(r"C:\ZirconBuilds") / f"hub-must-not-be-created-{uuid4().hex}"
        target_dir = root / "targets" / "hub"
        config = types.SimpleNamespace(
            repo_root=root,
            cargo="cargo",
            locked=False,
            jobs="",
            mode="debug",
            dry_run=False,
        )

        self.assertFalse(root.exists())
        with (
            mock.patch(
                "tools.zircon_build_hub.tauri_cli_path",
                return_value=root / "tauri.js",
            ),
            mock.patch("tools.zircon_build_hub.subprocess.run") as run,
            self.assertRaisesRegex(ValueError, "approved build root"),
        ):
            run_tauri_build(config, target_dir)

        run.assert_not_called()
        self.assertFalse(root.exists())


if __name__ == "__main__":
    unittest.main()
