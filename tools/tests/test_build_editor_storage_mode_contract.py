# 核对编辑器构建存储模式先校验再传至两个受管构建入口。
from pathlib import Path
import re
import unittest
import json
import tempfile
from unittest.mock import patch
from tools.build import build_editor


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_EDITOR = REPO_ROOT / "tools" / "build-editor.ps1"


class BuildEditorStorageModeContractTests(unittest.TestCase):
    def test_storage_mode_is_validated_at_independent_entry(self) -> None:
        source = BUILD_EDITOR.read_text(encoding="utf-8")

        self.assertRegex(
            source,
            re.compile(
                r"\[ValidateSet\(\"reuse\", \"compact\", \"diagnostic\"\)\]\s*"
                r"\[string\]\$StorageMode\s*=\s*\"reuse\"",
                re.MULTILINE,
            ),
        )
        self.assertIn("'--storage-mode', $StorageMode", source)
        self.assertNotIn('validate-matrix', source)
        self.assertNotIn('zircon-session', source)
        self.assertIn('$PythonInterpreter', source)

    def test_development_and_shipping_build_actual_dll_with_matching_features(self) -> None:
        for profile, feature in [('development', 'target-editor-host'), ('shipping', 'shipping-editor')]:
            host, runtime = build_editor.build_commands(profile, 2)
            self.assertEqual(['build', '-p', 'zircon_app', '--bin', 'zircon_editor'], host[:5])
            self.assertEqual(['rustc', '-p', 'zircon_runtime', '--lib', '--crate-type', 'cdylib'], runtime[:6])
            for command in (host, runtime):
                self.assertEqual(feature, command[command.index('--features') + 1])
                self.assertIn('--locked', command)
                self.assertIn('--message-format=json-render-diagnostics', command)
                self.assertEqual(profile == 'shipping', '--profile' in command)

    def test_artifact_requires_successful_cargo_message_and_selected_target(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            log = root / 'cargo.log'
            artifact = root / 'target' / 'debug' / 'zircon_runtime.dll'
            report = {'reason': 'compiler-artifact', 'filenames': [str(artifact)]}
            log.write_text(json.dumps(report) + '\n', encoding='utf-8')
            with patch.object(build_editor, 'output_path', side_effect=lambda p: p):
                with self.assertRaises(ValueError):
                    build_editor.reported_artifact(log, artifact.name, root / 'target')
                log.write_text(json.dumps(report) + '\n' + json.dumps({'reason': 'build-finished', 'success': True}), encoding='utf-8')
                self.assertEqual(artifact, build_editor.reported_artifact(log, artifact.name, root / 'target'))
                with self.assertRaises(ValueError):
                    build_editor.reported_artifact(log, artifact.name, root / 'other')

    def test_inventory_detects_dirty_new_and_external_source(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / 'source'
            source.mkdir()
            (source / 'Cargo.toml').write_text('before', encoding='utf-8')
            external = root / 'external'
            external.mkdir()
            (external / 'dependency.rs').write_text('external', encoding='utf-8')
            first = build_editor.source_inventory(source)
            self.assertIn('external/dependency.rs', first)
            (source / 'Cargo.toml').write_text('after', encoding='utf-8')
            (source / 'new.rs').write_text('new', encoding='utf-8')
            second = build_editor.source_inventory(source)
            self.assertNotEqual(build_editor.inventory_digest(first), build_editor.inventory_digest(second))
            self.assertIn('source/new.rs', second)

    def test_shipping_excludes_source_fonts_but_copies_runtime_and_editor_assets(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for owner in ('zircon_runtime', 'zircon_editor'):
                assets = root / 'source' / owner / 'assets'
                assets.mkdir(parents=True)
                (assets / (owner + '.asset')).write_text(owner, encoding='utf-8')
            fonts = root / 'source' / 'zircon_runtime' / 'assets' / 'fonts' / 'editor-ui-sources'
            fonts.mkdir(parents=True)
            (fonts / 'source-font.ttf').write_bytes(b'font')
            destination = root / 'output'
            with patch.object(build_editor, 'output_path', side_effect=lambda p: p):
                build_editor.copy_assets(root / 'source', destination, True)
            self.assertTrue((destination / 'zircon_runtime.asset').is_file())
            self.assertTrue((destination / 'zircon_editor.asset').is_file())
            self.assertFalse((destination / 'fonts' / 'editor-ui-sources').exists())


if __name__ == "__main__":
    unittest.main()
