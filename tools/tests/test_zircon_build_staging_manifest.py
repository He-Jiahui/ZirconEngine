import hashlib
import json
import tempfile
import types
import unittest
from pathlib import Path


class ZirconBuildStagingManifestTests(unittest.TestCase):
    def test_manifest_binds_staged_files_to_sources_and_hashes(self):
        from tools.zircon_build_staging_manifest import write_staging_manifest

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo_root = root / "repo"
            out_root = root / "out"
            engine_root = out_root / "ZirconEngine"
            targets_root = out_root / "targets"
            self._write(
                targets_root / "editor" / "debug" / "zircon_editor.exe",
                b"editor-binary",
            )
            self._write(
                targets_root / "runtime" / "bin" / "debug" / "zircon_runtime.exe",
                b"runtime-binary",
            )
            self._write(
                targets_root / "runtime" / "lib" / "debug" / "zircon_runtime.dll",
                b"runtime-library",
            )
            self._write(
                repo_root / "zircon_editor" / "assets" / "ui" / "editor.zui",
                b"editor-ui",
            )
            self._write(
                repo_root / "zircon_runtime" / "assets" / "fonts" / "default.font.toml",
                b"runtime-font",
            )
            self._write(engine_root / "zircon_editor.exe", b"editor-binary")
            self._write(engine_root / "zircon_runtime.exe", b"runtime-binary")
            self._write(engine_root / "zircon_runtime.dll", b"runtime-library")
            self._write(engine_root / "assets" / "ui" / "editor.zui", b"editor-ui")
            self._write(
                engine_root / "assets" / "fonts" / "default.font.toml",
                b"runtime-font",
            )
            config = types.SimpleNamespace(
                repo_root=repo_root,
                out_root=out_root,
                engine_root=engine_root,
                targets_root=targets_root,
                mode="debug",
                targets=("editor", "runtime"),
                runtime_features=("target-client",),
                plugins=(),
                dry_run=False,
            )

            manifest_path = write_staging_manifest(config)
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

            self.assertEqual(1, manifest["schema_version"])
            self.assertEqual("debug", manifest["build"]["mode"])
            self.assertEqual(["editor", "runtime"], manifest["build"]["targets"])
            entries = manifest["artifacts"]
            self.assertEqual(
                sorted(entry["target_path"] for entry in entries),
                [entry["target_path"] for entry in entries],
            )
            by_path = {entry["target_path"]: entry for entry in entries}
            self.assertEqual(
                {
                    "kind": "build_artifact",
                    "path": "targets/editor/debug/zircon_editor.exe",
                },
                by_path["zircon_editor.exe"]["source"],
            )
            self.assertEqual(
                {
                    "kind": "source_asset",
                    "path": "zircon_editor/assets/ui/editor.zui",
                },
                by_path["assets/ui/editor.zui"]["source"],
            )
            self.assertEqual(
                hashlib.sha256(b"runtime-library").hexdigest(),
                by_path["zircon_runtime.dll"]["sha256"],
            )

    def test_manifest_rejects_staged_files_without_source_provenance(self):
        from tools.zircon_build_staging_manifest import write_staging_manifest

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            engine_root = root / "out" / "ZirconEngine"
            self._write(engine_root / "untracked.bin", b"untracked")
            config = types.SimpleNamespace(
                repo_root=root / "repo",
                out_root=root / "out",
                engine_root=engine_root,
                targets_root=root / "out" / "targets",
                mode="debug",
                targets=("editor",),
                runtime_features=("target-editor-host",),
                plugins=(),
                dry_run=False,
            )

            with self.assertRaisesRegex(SystemExit, "no source provenance"):
                write_staging_manifest(config)

    def test_manifest_records_generated_plugin_load_manifest(self):
        from tools.zircon_build_staging_manifest import write_staging_manifest

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            engine_root = root / "out" / "ZirconEngine"
            self._write(engine_root / "plugins.toml", b"[plugins]\n")
            config = types.SimpleNamespace(
                repo_root=root / "repo",
                out_root=root / "out",
                engine_root=engine_root,
                targets_root=root / "out" / "targets",
                mode="debug",
                targets=("editor",),
                runtime_features=("target-editor-host",),
                plugins=(),
                dry_run=False,
            )
            self._write(config.repo_root / "tools" / "zircon_build.py", b"builder")

            manifest_path = write_staging_manifest(config)
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

            self.assertEqual(
                {
                    "kind": "generated",
                    "path": "tools/zircon_build.py",
                },
                manifest["artifacts"][0]["source"],
            )

    def test_manifest_binds_native_plugin_payload_to_build_and_source_inputs(self):
        from tools.zircon_build_staging_manifest import write_staging_manifest

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo_root = root / "repo"
            out_root = root / "out"
            engine_root = out_root / "ZirconEngine"
            targets_root = out_root / "targets"
            package_root = repo_root / "zircon_plugins" / "fixture"
            package = types.SimpleNamespace(
                plugin_id="fixture",
                package_root=package_root,
            )
            self._write(package_root / "plugin.toml", b"id = 'fixture'\n")
            self._write(
                targets_root / "plugins" / "fixture" / "debug" / "fixture.dll",
                b"fixture-library",
            )
            self._write(
                engine_root / "plugins" / "fixture" / "plugin.toml",
                b"id = 'fixture'\n",
            )
            self._write(
                engine_root / "plugins" / "fixture" / "native" / "fixture.dll",
                b"fixture-library",
            )
            config = types.SimpleNamespace(
                repo_root=repo_root,
                out_root=out_root,
                engine_root=engine_root,
                targets_root=targets_root,
                mode="debug",
                targets=("editor", "plugins"),
                runtime_features=("target-editor-host",),
                plugins=(package,),
                dry_run=False,
            )

            manifest_path = write_staging_manifest(config)
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            by_path = {entry["target_path"]: entry for entry in manifest["artifacts"]}

            self.assertEqual(
                {
                    "kind": "plugin_source",
                    "path": "zircon_plugins/fixture/plugin.toml",
                },
                by_path["plugins/fixture/plugin.toml"]["source"],
            )
            self.assertEqual(
                {
                    "kind": "build_artifact",
                    "path": "targets/plugins/fixture/debug/fixture.dll",
                },
                by_path["plugins/fixture/native/fixture.dll"]["source"],
            )

    @staticmethod
    def _write(path: Path, contents: bytes) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(contents)


if __name__ == "__main__":
    unittest.main()
