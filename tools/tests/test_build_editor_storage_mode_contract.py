# 核对编辑器构建存储模式先校验再传至两个受管构建入口。
from pathlib import Path
import re
import unittest
import json
import hashlib
import io
import os
import struct
import tempfile
import zipfile
from types import SimpleNamespace
from unittest.mock import patch
from tools.build import build_editor
from tools.build import zircon_build_runtime_manifest as runtime_manifest


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_EDITOR = REPO_ROOT / "tools" / "build" / "build-editor.ps1"
SYNTHETIC_SHIPPING_PROFILE = '''schema_version = 1
default_target_triple = "x86_64-pc-windows-msvc"
[profiles.editor-windows-shipping]
product_kind = "editor"
target_triple = "x86_64-pc-windows-msvc"
cargo_profile = "shipping"
app_package = "zircon_app"
app_binary = "zircon_editor"
runtime_package = "zircon_runtime"
runtime_library = "zircon_runtime.dll"
runtime_features = ["shipping-editor"]
app_features = ["shipping-editor"]
plugin_carrier = "none"
asset_scope = "editor"
include_project_pack = false
symbols = "external"
warning_bytes = 500000000
hard_limit_bytes = 400000000
hard_gate = true
archive = true
forbidden_path_tokens = ["source", "cache", "pdb", "zircon_hub", "editor-assets"]
'''


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
                if profile == 'shipping':
                    self.assertEqual('x86_64-pc-windows-msvc', command[command.index('--target') + 1])
                else:
                    self.assertNotIn('--target', command)

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


@unittest.skipUnless(os.name == 'nt', 'Windows editor packaging')
class BuildEditorPackageContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        self.root = Path(self.folder.name)
        self.source = self.root / 'source'
        self.profile_path = self.source / 'tools/export/product_build_profiles.toml'
        self.profile_path.parent.mkdir(parents=True)
        self.profile_path.write_text(SYNTHETIC_SHIPPING_PROFILE, encoding='utf-8')
        (self.source / 'Cargo.toml').write_text('[workspace]\nmembers=[]\n', encoding='utf-8')
        for owner in ('zircon_runtime', 'zircon_editor'):
            assets = self.source / owner / 'assets'
            assets.mkdir(parents=True)
            (assets / (owner + '.asset')).write_bytes(owner.encode())
        fonts = self.source / 'zircon_runtime' / 'assets' / 'fonts' / 'editor-ui-sources'
        fonts.mkdir(parents=True)
        (fonts / 'source-font.ttf').write_bytes(b'synthetic font')
        for name, payload in [('default.font.toml', 'default.ttc'), ('editor-ui.font.toml', 'editor-ui.ttc')]:
            (fonts.parent / name).write_text(f'source = "{payload}"\nfamily = "Synthetic"\n', encoding='utf-8')
            (fonts.parent / payload).write_bytes(b'synthetic packaged font')
        self.source_files = build_editor.source_inventory(self.source)
        self.target = self.root / 'target'
        self.target.mkdir()
        self.artifacts = {}
        for name in ('zircon_editor.exe', 'zircon_runtime.dll'):
            origin = self.target / name
            origin.write_bytes(self.pe_bytes(dll=name.endswith('.dll')))
            self.artifacts[name] = (origin, build_editor.sha(origin))
        self.crt = self.root / 'crt'
        self.crt.mkdir()
        for name in ('msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll'):
            (self.crt / name).write_bytes(self.pe_bytes(dll=True))

    @staticmethod
    def pe_bytes(*, dll: bool) -> bytes:
        data = bytearray(152)
        data[:2] = b'MZ'
        struct.pack_into('<I', data, 0x3c, 128)
        data[128:132] = b'PE\0\0'
        struct.pack_into('<H', data, 132, 0x8664)
        struct.pack_into('<H', data, 150, 0x2002 if dll else 0x0002)
        return bytes(data)

    def stage(self, name: str, profile: str = 'development') -> tuple[Path, dict]:
        staging = self.root / name
        staging.mkdir()
        with patch.object(build_editor.subprocess, 'run', return_value=SimpleNamespace(returncode=0)) as signatures, \
                patch.object(runtime_manifest.platform, 'machine', return_value='AMD64'), \
                patch.object(runtime_manifest, '_load_interface_spec', return_value={}), \
                patch.object(runtime_manifest, '_load_payload_schema_set_digest', return_value='a' * 64):
            result = build_editor.stage_editor_bundle(self.source, staging, self.artifacts,
                cargo_profile=profile, source_files=self.source_files, crt_directory=self.crt,
                shell=Path('synthetic-powershell.exe'))
        self.assertEqual(3, signatures.call_count)
        return staging, result

    def test_stage_and_publish_keep_buildset_and_complete_content_closure(self) -> None:
        build_sets = []
        for profile in ('development', 'shipping'):
            staging, package = self.stage(profile + '.staging', profile)
            self.assertEqual(profile != 'shipping', (staging / 'assets/fonts/editor-ui-sources/source-font.ttf').exists())
            for owner in ('zircon_runtime', 'zircon_editor'):
                self.assertEqual(owner.encode(), (staging / 'assets' / (owner + '.asset')).read_bytes())
            identity = package['build_set']
            self.assertEqual({'file_name': 'zircon_runtime.dll', 'sha256': self.artifacts['zircon_runtime.dll'][1]}, identity['artifact'])
            self.assertEqual([{'file_name': 'zircon_editor.exe', 'sha256': self.artifacts['zircon_editor.exe'][1]}], identity['host_artifacts'])
            build_sets.append(identity['build_set_id'])
            final = self.root / profile
            published = build_editor.publish_editor_bundle(staging, final, package['output_closure'])
            self.assertFalse(staging.exists())
            self.assertEqual(str(final), published['root'])
            self.assertEqual(package['output_closure']['files'], published['files'])
            self.assertEqual(package['output_closure']['digest'], published['digest'])
            paths = {entry['path'] for entry in published['files']}
            self.assertTrue({'zircon_editor.exe', 'zircon_runtime.dll', 'msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll'}.issubset(paths))
            self.assertTrue(any(path.endswith('.manifest.json') for path in paths))
        self.assertNotEqual(*build_sets)

    def test_publication_rejects_changed_extra_and_missing_content(self) -> None:
        for mutation in ('changed', 'extra', 'missing'):
            with self.subTest(mutation=mutation):
                staging, package = self.stage(mutation + '.staging')
                asset = staging / 'assets/zircon_editor.asset'
                if mutation == 'changed':
                    asset.write_bytes(b'changed')
                elif mutation == 'extra':
                    (staging / 'extra.file').write_bytes(b'extra')
                else:
                    asset.unlink()
                final = self.root / mutation
                with self.assertRaisesRegex(ValueError, 'staged closure'):
                    build_editor.publish_editor_bundle(staging, final, package['output_closure'])
                self.assertTrue(staging.is_dir())
                self.assertFalse(final.exists())

    def test_staging_rejects_changed_source_and_artifact_identity(self) -> None:
        self.artifacts['zircon_runtime.dll'] = (self.target / 'zircon_runtime.dll', '0' * 64)
        with self.assertRaisesRegex(ValueError, 'reported identity'):
            self.stage('artifact.staging')
        origin = self.target / 'zircon_runtime.dll'
        self.artifacts['zircon_runtime.dll'] = (origin, build_editor.sha(origin))
        (self.source / 'zircon_editor/assets/zircon_editor.asset').write_bytes(b'new source')
        with self.assertRaisesRegex(ValueError, 'Source changed'):
            self.stage('source.staging')

    def test_closure_capture_rejects_changes_after_buildset_was_written(self) -> None:
        capture = build_editor.editor_bundle_closure
        def changed(directory: Path) -> dict:
            (directory / 'assets/zircon_editor.asset').write_bytes(b'changed after copy')
            return capture(directory)
        with patch.object(build_editor, 'editor_bundle_closure', side_effect=changed):
            with self.assertRaisesRegex(ValueError, 'copied asset and artifact identities'):
                self.stage('capture.staging')

    def test_staging_rejects_missing_and_escaping_mandatory_font_sources(self) -> None:
        font_root = self.source / 'zircon_runtime/assets/fonts'
        (font_root / 'default.ttc').unlink()
        self.source_files = build_editor.source_inventory(self.source)
        with self.assertRaisesRegex(ValueError, 'Mandatory font payload'):
            self.stage('missing-font.staging')
        (font_root / 'default.font.toml').write_text('source = "../../../outside.ttc"\n', encoding='utf-8')
        self.source_files = build_editor.source_inventory(self.source)
        with self.assertRaisesRegex(ValueError, 'escapes the staged asset directory'):
            self.stage('escaping-font.staging')

    def test_artifact_format_rejects_wrong_pe_role_and_truncated_binary(self) -> None:
        executable = self.target / 'zircon_editor.exe'
        with self.assertRaises(ValueError):
            build_editor.verify_x64_pe(executable, dll=True)
        executable.write_bytes(b'truncated')
        with self.assertRaises(ValueError):
            build_editor.verify_x64_pe(executable, dll=False)

    def test_shipping_publishes_repeatable_zip_and_report_for_the_frozen_buildset(self) -> None:
        archives = []
        for name in ('shipping-first', 'shipping-second'):
            staging, package = self.stage(name + '.staging', 'shipping')
            manifest = (staging / 'zircon_runtime.dll.manifest.json').read_bytes()
            final = self.root / name
            published = build_editor.publish_editor_shipping_bundle(self.source, staging, final, package,
                source_files=self.source_files)
            archive = self.root / (name + '.zip')
            archives.append(archive.read_bytes())
            self.assertEqual(package['output_closure']['files'], published['output_closure']['files'])
            self.assertEqual(manifest, (final / 'zircon_runtime.dll.manifest.json').read_bytes())
            self.assertEqual(build_editor.sha(archive), published['archive']['sha256'])
            report = published['size_report']
            self.assertEqual(package['build_set']['build_set_id'], report['build_set_id'])
            self.assertEqual('editor-windows-shipping', report['product_profile'])
            self.assertEqual(['shipping-editor'], report['features']['runtime'])
            self.assertEqual(['shipping-editor'], report['features']['app'])
            self.assertEqual(400000000, report['budget']['hard_limit_bytes'])
            self.assertEqual(500000000, report['budget']['warning_bytes'])
            self.assertEqual(archive.name, report['archive']['path'])
            self.assertEqual(build_editor.sha(archive), report['archive']['sha256'])
            with zipfile.ZipFile(archive) as zipped:
                self.assertEqual({entry['path'] for entry in package['output_closure']['files']}, set(zipped.namelist()))
        self.assertEqual(*archives)

    def test_shipping_budget_and_forbidden_payload_fail_before_compression_or_publication(self) -> None:
        for violation in ('budget', 'forbidden'):
            with self.subTest(violation=violation):
                profile = SYNTHETIC_SHIPPING_PROFILE.replace('hard_limit_bytes = 400000000', 'hard_limit_bytes = 1') if violation == 'budget' else SYNTHETIC_SHIPPING_PROFILE
                self.profile_path.write_text(profile, encoding='utf-8')
                self.source_files = build_editor.source_inventory(self.source)
                staging, package = self.stage(violation + '.staging', 'shipping')
                if violation == 'forbidden':
                    (staging / 'leaked.pdb').write_bytes(b'synthetic symbol')
                    package['output_closure'] = build_editor.editor_bundle_closure(staging)
                final = self.root / violation
                with patch.object(build_editor, 'write_deterministic_archive', side_effect=AssertionError('no compression')) as writer:
                    with self.assertRaisesRegex(ValueError, 'product size gate failed'):
                        build_editor.publish_editor_shipping_bundle(self.source, staging, final, package, source_files=self.source_files)
                writer.assert_not_called()
                self.assertTrue(staging.is_dir())
                self.assertFalse(final.exists())
                self.assertFalse((self.root / (violation + '.zip')).exists())

    def test_shipping_rejects_existing_zip_and_private_helper_temporary(self) -> None:
        for collision in ('public', 'helper'):
            with self.subTest(collision=collision):
                staging, package = self.stage(collision + '.staging', 'shipping')
                final = self.root / collision
                blocked = self.root / (collision + '.zip' if collision == 'public' else collision + '.zip.pending-fixture.tmp')
                blocked.write_bytes(b'preserved foreign data')
                with patch.object(build_editor.uuid, 'uuid4', return_value=SimpleNamespace(hex='fixture')), \
                        patch.object(build_editor, 'write_deterministic_archive', side_effect=AssertionError('no compression')) as writer:
                    with self.assertRaisesRegex(ValueError, 'fresh'):
                        build_editor.publish_editor_shipping_bundle(self.source, staging, final, package, source_files=self.source_files)
                writer.assert_not_called()
                self.assertEqual(b'preserved foreign data', blocked.read_bytes())
                self.assertTrue(staging.is_dir())
                self.assertFalse(final.exists())

    def test_shipping_rejects_policy_and_frozen_target_mismatches_before_publication(self) -> None:
        for mismatch in ('policy', 'target'):
            with self.subTest(mismatch=mismatch):
                profile = SYNTHETIC_SHIPPING_PROFILE.replace('app_features = ["shipping-editor"]', 'app_features = ["shipping-runtime"]') if mismatch == 'policy' else SYNTHETIC_SHIPPING_PROFILE
                self.profile_path.write_text(profile, encoding='utf-8')
                self.source_files = build_editor.source_inventory(self.source)
                staging, package = self.stage(mismatch + '.staging', 'shipping')
                if mismatch == 'target':
                    package['build_set']['target']['target_triple'] = 'x86_64-pc-windows-gnu'
                final = self.root / ('mismatch-' + mismatch)
                with patch.object(build_editor, 'write_deterministic_archive', side_effect=AssertionError('no compression')) as writer:
                    with self.assertRaisesRegex(ValueError, 'built composition or frozen BuildSet target'):
                        build_editor.publish_editor_shipping_bundle(self.source, staging, final, package, source_files=self.source_files)
                writer.assert_not_called()
                self.assertTrue(staging.is_dir())
                self.assertFalse(final.exists())

    def test_archive_verifier_rejects_unsafe_duplicate_extra_missing_and_changed_members(self) -> None:
        expected = {'files': [{'path': 'asset.bin', 'size': 4, 'sha256': hashlib.sha256(b'data').hexdigest()}]}
        cases = {'unsafe': [('../asset.bin', b'data')], 'duplicate': [('asset.bin', b'data'), ('asset.bin', b'data')],
            'extra': [('asset.bin', b'data'), ('extra.bin', b'other')], 'missing': [], 'changed': [('asset.bin', b'else')],
            'size': [('asset.bin', b'larger')]}
        for name, members in cases.items():
            with self.subTest(mutation=name):
                archive = self.root / (name + '.zip')
                with zipfile.ZipFile(archive, 'w') as zipped:
                    for path, content in members:
                        zipped.writestr(path, content)
                with self.assertRaisesRegex(ValueError, 'Editor ZIP'):
                    build_editor.verify_editor_archive(archive, expected)

    def test_shipping_recaptures_staging_after_archive_creation(self) -> None:
        staging, package = self.stage('mutated.staging', 'shipping')
        writer = build_editor.write_deterministic_archive
        def mutate_after_zip(directory, archive, **kwargs):
            result = writer(directory, archive, **kwargs)
            (directory / 'assets/zircon_editor.asset').write_bytes(b'changed after ZIP')
            return result
        final = self.root / 'mutated'
        with patch.object(build_editor, 'write_deterministic_archive', side_effect=mutate_after_zip):
            with self.assertRaisesRegex(ValueError, 'staged closure'):
                build_editor.publish_editor_shipping_bundle(self.source, staging, final, package, source_files=self.source_files)
        self.assertTrue(staging.is_dir())
        self.assertFalse(final.exists())
        self.assertFalse((self.root / 'mutated.zip').exists())

    def test_shipping_entry_writes_completion_receipt_last_and_preserves_folder_on_zip_rename_failure(self) -> None:
        for failed_zip_rename in (False, True):
            with self.subTest(failed_zip_rename=failed_zip_rename):
                final = self.root / ('entry-failed' if failed_zip_rename else 'entry-complete')
                evidence_root = self.root / 'evidence'
                evidence_before = set(evidence_root.glob('*'))
                receipts_before = {path: build_editor.sha(path) for path in evidence_root.glob('*/build-receipt.json')}
                staged_closures = []
                def run(command, **kwargs):
                    if command[-1] == '--help':
                        return SimpleNamespace(returncode=0, stdout=b'Usage: zircon_editor', stderr=b'')
                    if '-File' not in command:
                        return SimpleNamespace(returncode=0)
                    name = 'zircon_runtime.dll' if 'rustc' in command else 'zircon_editor.exe'
                    artifact = self.target / 'shipping' / name
                    artifact.parent.mkdir(parents=True, exist_ok=True)
                    artifact.write_bytes(self.pe_bytes(dll=name.endswith('.dll')))
                    rows = [{'reason': 'compiler-artifact', 'filenames': [str(artifact)]}, {'reason': 'build-finished', 'success': True}]
                    kwargs['stdout'].write(('\n'.join(json.dumps(row) for row in rows) + '\n').encode())
                    return SimpleNamespace(returncode=0)
                rename = Path.rename
                def rename_output(path, destination):
                    if failed_zip_rename and '.zip.pending-' in path.name:
                        raise OSError('synthetic ZIP rename failure')
                    if destination == final:
                        staged_closures.append(build_editor.editor_bundle_closure(path))
                    return rename(path, destination)
                with patch.object(build_editor.local_cargo, 'free_bytes', return_value=100 * 1024**3), \
                        patch('tools.jenkins.frontend.sole_entry_enforced', return_value=False), \
                        patch.object(runtime_manifest.platform, 'machine', return_value='AMD64'), \
                        patch.object(runtime_manifest, '_load_interface_spec', return_value={'family': 'synthetic-fixture'}), \
                        patch.object(runtime_manifest, '_load_payload_schema_set_digest', return_value='a' * 64), \
                        patch.object(build_editor.subprocess, 'run', side_effect=run), \
                        patch.object(Path, 'rename', new=rename_output), \
                        patch.object(build_editor.sys, 'stderr', new=io.StringIO()) as errors:
                    result = build_editor.main(['--repo-root', str(self.source), '--target-dir', str(self.target),
                        '--output-directory', str(final), '--cargo-profile', 'shipping', '--crt-directory', str(self.crt)])
                self.assertEqual(2 if failed_zip_rename else 0, result)
                self.assertTrue(final.is_dir())
                self.assertEqual(not failed_zip_rename, (self.root / (final.name + '.zip')).exists())
                evidence_after = set(evidence_root.glob('*')) - evidence_before
                self.assertEqual(1, len(evidence_after))
                evidence = evidence_after.pop()
                receipt_path = evidence / 'build-receipt.json'
                report_path = evidence / 'product-size-report.json'
                receipts_after = {path: build_editor.sha(path) for path in evidence_root.glob('*/build-receipt.json')}
                for path, expected_hash in receipts_before.items():
                    self.assertEqual(expected_hash, receipts_after.get(path))
                self.assertEqual(1, len(staged_closures))
                published = build_editor.verify_editor_bundle(final, staged_closures[0])
                if failed_zip_rename:
                    self.assertIn('synthetic ZIP rename failure', errors.getvalue())
                    self.assertFalse(receipt_path.exists())
                    self.assertFalse(report_path.exists())
                    self.assertEqual(receipts_before, receipts_after)
                    continue
                self.assertEqual(set(receipts_before) | {receipt_path}, set(receipts_after))
                receipt = json.loads(receipt_path.read_text(encoding='utf-8'))
                self.assertEqual('shipping_packaged', receipt['status'])
                self.assertTrue(receipt['completed_publication'])
                self.assertFalse(receipt['full_build_acceptance'])
                self.assertFalse(receipt['native_ui_acceptance'])
                self.assertFalse(receipt['shipping_acceptance'])
                self.assertEqual(report_path, Path(receipt['product_size_report']))
                self.assertEqual(build_editor.sha(report_path), receipt['product_size_report_sha256'])
                self.assertEqual(published, receipt['output_closure'])

    def test_shipping_rejects_zip_mutation_during_final_report_before_completion_receipt(self) -> None:
        for mutate_before_report in (True, False):
            with self.subTest(mutate_before_report=mutate_before_report):
                final = self.root / ('report-mutated-before' if mutate_before_report else 'report-mutated-after')
                archive = self.root / (final.name + '.zip')
                evidence_before = set((self.root / 'evidence').glob('*'))
                captured = []
                report_builder = build_editor.build_product_size_report
                def mutate_zip():
                    with archive.open('ab') as stream:
                        stream.write(b'synthetic trailing ZIP bytes')
                def report_with_zip_mutation(directory, *, archive_path=None, **kwargs):
                    if archive_path is not None:
                        self.assertEqual(final, directory)
                        self.assertEqual(archive, archive_path)
                        captured.append(build_editor.verify_editor_archive(archive, build_editor.editor_bundle_closure(final)))
                        if mutate_before_report:
                            mutate_zip()
                    report = report_builder(directory, archive_path=archive_path, **kwargs)
                    if archive_path is not None and not mutate_before_report:
                        mutate_zip()
                    return report
                def run(command, **kwargs):
                    if command[-1] == '--help':
                        return SimpleNamespace(returncode=0, stdout=b'Usage: zircon_editor', stderr=b'')
                    if '-File' not in command:
                        return SimpleNamespace(returncode=0)
                    name = 'zircon_runtime.dll' if 'rustc' in command else 'zircon_editor.exe'
                    artifact = self.target / 'shipping' / name
                    artifact.parent.mkdir(parents=True, exist_ok=True)
                    artifact.write_bytes(self.pe_bytes(dll=name.endswith('.dll')))
                    rows = [{'reason': 'compiler-artifact', 'filenames': [str(artifact)]}, {'reason': 'build-finished', 'success': True}]
                    kwargs['stdout'].write(('\n'.join(json.dumps(row) for row in rows) + '\n').encode())
                    return SimpleNamespace(returncode=0)
                with patch.object(build_editor.local_cargo, 'free_bytes', return_value=100 * 1024**3), \
                        patch('tools.jenkins.frontend.sole_entry_enforced', return_value=False), \
                        patch.object(runtime_manifest.platform, 'machine', return_value='AMD64'), \
                        patch.object(runtime_manifest, '_load_interface_spec', return_value={'family': 'synthetic-fixture'}), \
                        patch.object(runtime_manifest, '_load_payload_schema_set_digest', return_value='a' * 64), \
                        patch.object(build_editor.subprocess, 'run', side_effect=run), \
                        patch.object(build_editor, 'build_product_size_report', side_effect=report_with_zip_mutation), \
                        patch.object(build_editor.sys, 'stderr', new=io.StringIO()) as errors:
                    result = build_editor.main(['--repo-root', str(self.source), '--target-dir', str(self.target),
                        '--output-directory', str(final), '--cargo-profile', 'shipping', '--crt-directory', str(self.crt)])
                self.assertEqual(2, result)
                self.assertIn('Editor ZIP identity differs', errors.getvalue())
                self.assertTrue(final.is_dir())
                self.assertTrue(archive.is_file())
                self.assertEqual(1, len(captured))
                observed = build_editor.verify_editor_archive(archive, build_editor.editor_bundle_closure(final))
                self.assertNotEqual(captured[0]['sha256'], observed['sha256'])
                evidence_after = set((self.root / 'evidence').glob('*')) - evidence_before
                self.assertEqual(1, len(evidence_after))
                for evidence in evidence_after:
                    self.assertFalse((evidence / 'build-receipt.json').exists())
                    self.assertFalse((evidence / 'product-size-report.json').exists())

    def test_unsupported_storage_modes_fail_before_inventory_or_execution(self) -> None:
        for mode in ('compact', 'diagnostic'):
            with self.subTest(mode=mode), \
                    patch.object(build_editor, 'source_inventory', side_effect=AssertionError('no inventory')), \
                    patch.object(build_editor, 'prepare_snapshot', side_effect=AssertionError('no source preparation')), \
                    patch.object(build_editor.subprocess, 'run', side_effect=AssertionError('no execution')), \
                    patch.object(build_editor.sys, 'stderr', new=io.StringIO()) as errors:
                self.assertEqual(2, build_editor.main(['--repo-root', str(REPO_ROOT), '--storage-mode', mode,
                    '--prepare-source-snapshot', str(self.root / 'unused-preparation')]))
                self.assertIn('no supported executor', errors.getvalue())


if __name__ == "__main__":
    unittest.main()
