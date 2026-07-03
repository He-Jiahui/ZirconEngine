from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pack_stage import run_pack
from tools.zircon_export.tests.export_test_support import _pack_args


class PackPathResolveErrorTests(unittest.TestCase):
    def test_pack_rejects_repo_root_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            args = _pack_args(out=root / "out")
            args.repo_root = str(repo_root)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(repo_root):
                    raise OSError("simulated pack repo_root resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_pack(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=Pack repo_root", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated pack repo_root resolve failure", output)
            self.assertIn("command=<skipped>", output)

    def test_pack_rejects_asset_manifest_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            asset_manifest = root / "source" / "assets.json"
            args = _pack_args(out=root / "out")
            args.asset_manifest = str(asset_manifest)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(asset_manifest):
                    raise OSError("simulated asset manifest resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_pack(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=asset_manifest", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated asset manifest resolve failure", output)
            self.assertIn("asset_manifest=<invalid>", output)
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--manifest", output)

    def test_pack_rejects_pack_file_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack_file = root / "out" / "custom" / "assets.zrpack"
            args = _pack_args(out=root / "out")
            args.pack_file = str(pack_file)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(pack_file):
                    raise OSError("simulated pack file resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_pack(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=pack_file", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated pack file resolve failure", output)
            self.assertIn("pack=<invalid>", output)
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--output", output)

    def test_pack_rejects_packer_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            packer = root / "tools" / "zircon_export_pack.exe"
            args = _pack_args(out=root / "out")
            args.packer = str(packer)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(packer):
                    raise OSError("simulated packer resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_pack(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=packer", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated packer resolve failure", output)
            self.assertIn("command=<skipped>", output)

    def test_pack_rejects_target_dir_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target_dir = root / "target" / "pack"
            args = _pack_args(out=root / "out")
            args.target_dir = str(target_dir)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(target_dir):
                    raise OSError("simulated target dir resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_pack(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=target_dir", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated target dir resolve failure", output)
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--target-dir", output)


if __name__ == "__main__":
    unittest.main()
