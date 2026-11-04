"""Exercise first-frame validation and failure receipts without launching a GUI."""
import json
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import zlib

from tools.build.editor_startup_smoke import inspect_first_frame, run_startup_smoke
from tools.build.zircon_build_support import platform_executable_name, platform_runtime_library_name


def png_bytes(*, blank=False, transparent=False, width=640, height=360):
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
    row = b''.join(bytes([0 if blank or x < width // 2 else 255] * 3 + [0 if transparent else 255])
                   for x in range(width))
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress((b'\0' + row) * height)) + chunk(b'IEND', b''))


class StartupSmokeTests(unittest.TestCase):
    def test_nonblank_exact_window_size_and_hash(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'frame.png'
            path.write_bytes(png_bytes())
            result = inspect_first_frame(path, (640, 360))
            self.assertGreater(result['luma_stddev'], 8)
            self.assertEqual(len(result['sha256']), 64)
            with self.assertRaisesRegex(ValueError, 'size differs'):
                inspect_first_frame(path, (1280, 720))

    def test_blank_transparent_small_and_corrupt_frames_fail(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'frame.png'
            for payload in [png_bytes(blank=True), png_bytes(transparent=True),
                            png_bytes(width=16, height=16), b'not a PNG']:
                path.write_bytes(payload)
                with self.assertRaises((ValueError, RuntimeError)):
                    inspect_first_frame(path)

    def test_launch_records_only_a_verified_first_frame_as_passed(self):
        self.run_case('success')

    def test_standalone_gpu_is_recorded_without_claiming_shared_presentation(self):
        self.run_case('standalone')

    def test_native_smoke_rejects_relative_and_aliased_paths(self):
        import os
        from tools.build.editor_startup_smoke import smoke_output_path
        if os.name == 'nt':
            self.skipTest('Native POSIX output validation')
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            alias = root / 'alias'
            alias.symlink_to(root, target_is_directory=True)
            for path in (Path('relative'), root / '..' / 'escape', alias / 'evidence'):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    smoke_output_path(path)

    def test_failure_timeout_missing_frame_or_software_never_claim_success(self):
        for mode in ['exit', 'timeout', 'missing', 'blank', 'software', 'no-backend',
                     'forced-software', 'missing-presenter', 'repeated-presenter']:
            with self.subTest(mode=mode):
                self.run_case(mode)

    def run_case(self, mode):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            staging, evidence = root / 'staged', root / 'evidence'
            staging.mkdir()
            evidence.mkdir()
            calls = []

            def launch(command, **kwargs):
                calls.append(command)
                self.assertEqual(kwargs['timeout'], 3)
                self.assertEqual(kwargs['env']['WGPU_BACKEND'], 'dx12')
                self.assertEqual(command[0], str(staging / platform_executable_name('zircon_editor')))
                self.assertEqual(kwargs['env']['ZIRCON_RUNTIME_LIBRARY'], str(staging / platform_runtime_library_name()))
                self.assertEqual(kwargs['env']['XDG_CACHE_HOME'], str(evidence / 'profile/cache'))
                if '--help' in command:
                    kwargs['stdout'].write(b'Usage: zircon_editor')
                else:
                    self.assertIn('--create-project', command)
                    self.assertEqual(kwargs['env']['ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME'], '1')
                    if mode == 'timeout':
                        raise subprocess.TimeoutExpired(command, 3)
                    if mode != 'missing':
                        Path(kwargs['env']['ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG']).write_bytes(png_bytes(blank=mode == 'blank'))
                    if mode != 'no-backend':
                        kwargs['stderr'].write(b'gpu_backend=dx12 adapter="test" device_type=Cpu\n')
                    if mode != 'missing-presenter':
                        backend = 'softbuffer' if mode == 'forced-software' else 'gpu'
                        shared = 'false' if mode in ['standalone', 'forced-software'] else 'true'
                        marker = (f'[zircon_editor] startup_boundary marker=presenter_selected '
                                  f'presenter_backend={backend} runtime_shared={shared}\n').encode()
                        kwargs['stderr'].write(marker)
                        if mode == 'repeated-presenter':
                            kwargs['stderr'].write(marker)
                    if mode == 'software':
                        kwargs['stderr'].write(b'falling back to softbuffer')
                    if mode == 'exit':
                        return subprocess.CompletedProcess(command, 2)
                return subprocess.CompletedProcess(command, 0)

            with patch('tools.build.build_editor.output_path', side_effect=lambda path: path), \
                    patch('tools.build.editor_startup_smoke.subprocess.run', side_effect=launch):
                if mode in ['success', 'standalone']:
                    result = run_startup_smoke(staging, evidence, {'WGPU_BACKEND': 'dx12'}, timeout_seconds=3)
                    self.assertTrue(result['first_frame_verified'])
                    self.assertEqual(result['presenter_backend'], 'gpu')
                    self.assertEqual(result['runtime_shared'], mode == 'success')
                else:
                    with self.assertRaises(ValueError):
                        run_startup_smoke(staging, evidence, {'WGPU_BACKEND': 'dx12'}, timeout_seconds=3)
            receipt = json.loads((evidence / 'startup-smoke-receipt.json').read_text())
            succeeded = mode in ['success', 'standalone']
            self.assertEqual(receipt['status'], 'passed' if succeeded else 'failed')
            self.assertEqual(receipt['first_frame_verified'], succeeded)
            self.assertEqual(len(calls), 2)
            if succeeded:
                self.assertEqual([entry['label'] for entry in receipt['launches']], ['help', 'first-frame'])
                self.assertGreaterEqual(receipt['launches'][1]['wall_seconds'], 0)


if __name__ == '__main__':
    unittest.main()
