"""Exercise independent snapshot, asset merge, CRT, and BuildSet assembly."""
import json
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.build import build_editor as owner
from tools.build import zircon_build_runtime_manifest as runtime_manifest


def synthetic_pe(*, dll):
    data = bytearray(128)
    data[:2] = b'MZ'
    struct.pack_into('<I', data, 0x3c, 64)
    data[64:68] = b'PE\0\0'
    struct.pack_into('<H', data, 68, 0x8664)
    struct.pack_into('<H', data, 86, 0x2002 if dll else 0x0002)
    return bytes(data)


class IndependentEditorTests(unittest.TestCase):
    def test_snapshot_copies_untracked_and_sibling_dependency(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / 'checkout'
            (source / 'app').mkdir(parents=True)
            (source / 'Cargo.toml').write_text('[workspace]\nmembers=["app"]\n')
            (source / 'app/Cargo.toml').write_text('[package]\nname="app"\nversion="0.1.0"\n[dependencies]\nother={path="../../zr_vm"}\n')
            (source / 'app/untracked.rs').write_text('source')
            (root / 'zr_vm').mkdir()
            (root / 'zr_vm/Cargo.toml').write_text('[package]\nname="other"\nversion="0.1.0"\n')
            with patch.object(owner.local_cargo, 'free_bytes', return_value=100 * 1024**3):
                receipt = owner.prepare_snapshot(source, root / 'snapshot')
            frozen = Path(receipt['source'])
            inventory = owner.source_inventory(frozen)
            self.assertIn('source/app/untracked.rs', inventory)
            self.assertEqual(receipt['source_digest'], owner.inventory_digest(inventory))
            self.assertTrue(owner.dependency_closure(frozen, inventory))

    def test_identical_asset_merge_and_conflict_rejection(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ('zircon_runtime', 'zircon_editor'):
                assets = root / 'source' / name / 'assets'
                assets.mkdir(parents=True)
                (assets / 'shared').write_bytes(b'same')
            owner.copy_assets(root / 'source', root / 'ok', False)
            (root / 'source/zircon_editor/assets/shared').write_bytes(b'different')
            with self.assertRaises(ValueError):
                owner.copy_assets(root / 'source', root / 'bad', False)

    def test_crt_requires_x64_pe_and_microsoft_signature(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            redist, stage = root / 'redist', root / 'stage'
            redist.mkdir()
            stage.mkdir()
            for name in ('msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll'):
                (redist / name).write_bytes(synthetic_pe(dll=True))
            with patch.object(owner.subprocess, 'run', return_value=SimpleNamespace(returncode=0)) as verifier:
                self.assertEqual(3, len(owner.copy_crt(redist, stage, Path('powershell.exe'))))
            self.assertEqual(3, verifier.call_count)
            self.assertIn('Get-AuthenticodeSignature', verifier.call_args.args[0][-1])
            with patch.object(owner.subprocess, 'run', return_value=SimpleNamespace(returncode=2)), self.assertRaises(ValueError):
                owner.copy_crt(redist, stage, Path('powershell.exe'))

    def test_mock_build_generates_actual_buildset_and_separates_evidence(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            repo, target, bundle = root / 'repo', root / 'target', root / 'bundle'
            repo.mkdir()
            (repo / 'Cargo.toml').write_text('[workspace]\nmembers=[]\n', encoding='utf-8')
            for name in ('zircon_runtime', 'zircon_editor'):
                assets = repo / name / 'assets'
                assets.mkdir(parents=True)
                (assets / name).write_bytes(b'asset')
            fonts = repo / 'zircon_runtime/assets/fonts'
            fonts.mkdir()
            for name, payload in [('default.font.toml', 'default.ttc'), ('editor-ui.font.toml', 'editor-ui.ttc')]:
                (fonts / name).write_text(f'source = "{payload}"\nfamily = "Synthetic"\n', encoding='utf-8')
                (fonts / payload).write_bytes(b'synthetic packaged font')
            def run(command, **kwargs):
                if command[-1] == '--help':
                    return SimpleNamespace(returncode=0, stdout=b'Usage: zircon_editor', stderr=b'')
                name = 'zircon_runtime.dll' if 'rustc' in command else 'zircon_editor.exe'
                artifact = target / 'debug' / name
                artifact.parent.mkdir(parents=True, exist_ok=True)
                artifact.write_bytes(synthetic_pe(dll=name.endswith('.dll')))
                rows = [{'reason': 'compiler-artifact', 'filenames': [str(artifact)]}, {'reason': 'build-finished', 'success': True}]
                kwargs['stdout'].write(('\n'.join(json.dumps(row) for row in rows) + '\n').encode())
                return SimpleNamespace(returncode=0)
            with patch.object(owner.local_cargo, 'free_bytes', return_value=100 * 1024**3), \
                    patch('tools.jenkins.frontend.sole_entry_enforced', return_value=False), \
                    patch.object(runtime_manifest.platform, 'machine', return_value='AMD64'), \
                    patch.object(runtime_manifest, '_load_interface_spec', return_value={'family': 'synthetic-fixture'}), \
                    patch.object(runtime_manifest, '_load_payload_schema_set_digest', return_value='a' * 64), \
                    patch.object(owner.subprocess, 'run', side_effect=run), patch.object(owner, 'copy_crt', return_value={}):
                self.assertEqual(0, owner.main(['--repo-root', str(repo), '--target-dir', str(target), '--output-directory', str(bundle)]))
            manifest = json.loads((bundle / 'zircon_runtime.dll.manifest.json').read_bytes())
            self.assertEqual(owner.sha(bundle / 'zircon_runtime.dll'), manifest['artifact']['sha256'])
            self.assertEqual(owner.sha(bundle / 'zircon_editor.exe'), manifest['host_artifacts'][0]['sha256'])
            self.assertFalse((bundle / 'build-receipt.json').exists())
            self.assertFalse((bundle / 'temporary').exists())
            self.assertFalse((root / 'bundle.zip').exists())
            receipts = list((root / 'evidence').glob('*/build-receipt.json'))
            self.assertEqual(1, len(receipts))
            self.assertEqual('preview_built', json.loads(receipts[0].read_text(encoding='utf-8'))['status'])


if __name__ == '__main__':
    unittest.main()
