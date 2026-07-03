from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _run_stage_quiet,
    _write_validate_report_with_native_dynamic_exports,
    json_loads,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _write_native_dynamic_package_fixture,
)


class NativeDynamicSigningFileReadTests(unittest.TestCase):
    def test_native_dynamic_signing_rejects_artifact_listing_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            signer = write_fake_sign_script(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _signed_args(out, repo_root, signer)
            package_dir = (
                out / "stages" / "native_dynamic" / "plugins" / "animation"
            ).resolve()
            original_rglob = Path.rglob
            rglob_counts: dict[Path, int] = {}

            def rglob_or_fail(path: Path, pattern: str):
                resolved = path.resolve()
                if resolved == package_dir:
                    rglob_counts[resolved] = rglob_counts.get(resolved, 0) + 1
                if resolved == package_dir and rglob_counts[resolved] == 2:
                    raise OSError("simulated signing artifact listing failure")
                return original_rglob(path, pattern)

            with mock.patch.object(Path, "rglob", rglob_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            signing = report["native_signing"]
            package_signing = signing["packages"][0]
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(signing["fatal"], signing["diagnostics"])
            self.assertEqual(package_signing["artifact_count"], 0)
            self.assertEqual(package_signing["artifacts"], [])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertIsNone(report["loader_manifest"])
            self.assertFalse((package_dir / "native_dynamic_package.toml").exists())
            self.assertTrue(
                any(
                    "NativeDynamic signing for package animation package directory"
                    in diagnostic
                    and "could not be listed" in diagnostic
                    and "simulated signing artifact listing failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_signing_rejects_before_hash_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            signer = write_fake_sign_script(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _signed_args(out, repo_root, signer)
            unreadable_file = _stage_artifact(out)

            exit_code = run_stage_with_read_failure(
                args,
                unreadable_file,
                "simulated before hash read failure",
            )

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            signing = report["native_signing"]
            artifact_signing = signing["packages"][0]["artifacts"][0]
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(signing["fatal"], signing["diagnostics"])
            self.assertIsNone(artifact_signing["before_sha256"])
            self.assertIsNone(artifact_signing["after_sha256"])
            self.assertIsNone(artifact_signing["exit_code"])
            self.assertTrue(report["payload_cleaned"])
            self.assertTrue(
                any(
                    "NativeDynamic signing for package animation artifact"
                    in diagnostic
                    and "could not be read before command" in diagnostic
                    and "simulated before hash read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_signing_rejects_after_hash_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            signer = write_fake_sign_script(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _signed_args(out, repo_root, signer)
            unreadable_file = _stage_artifact(out)

            exit_code = run_stage_with_second_read_failure(
                args,
                unreadable_file,
                "simulated after hash read failure",
            )

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            signing = report["native_signing"]
            artifact_signing = signing["packages"][0]["artifacts"][0]
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(signing["fatal"], signing["diagnostics"])
            self.assertIsInstance(artifact_signing["before_sha256"], str)
            self.assertIsNone(artifact_signing["after_sha256"])
            self.assertEqual(artifact_signing["exit_code"], 0)
            self.assertTrue(report["payload_cleaned"])
            self.assertTrue(
                any(
                    "NativeDynamic signing for package animation artifact"
                    in diagnostic
                    and "could not be read after command" in diagnostic
                    and "simulated after hash read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def _signed_args(
    out: Path,
    repo_root: Path,
    signer: Path,
) -> object:
    args = _export_args(out=out, stage="native_dynamic", dry_run=False)
    args.repo_root = str(repo_root)
    args.native_dynamic_sign_command = sys.executable
    args.native_dynamic_sign_arg = [
        str(signer),
        "{artifact}",
        "{package_id}",
        "{target_platform}",
    ]
    return args


def _stage_artifact(out: Path) -> Path:
    return (
        out
        / "stages"
        / "native_dynamic"
        / "plugins"
        / "animation"
        / "native"
        / "zircon_plugin_animation.dll"
    ).resolve()


def write_fake_sign_script(repo_root: Path) -> Path:
    script = repo_root / "fake_sign.py"
    script.parent.mkdir(parents=True, exist_ok=True)
    script.write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import sys",
                "artifact = Path(sys.argv[1])",
                "package_id = sys.argv[2]",
                "target_platform = sys.argv[3]",
                "with artifact.open('a', encoding='utf-8') as handle:",
                "    handle.write(f'\\nsigned:{package_id}:{target_platform}')",
            ]
        ),
        encoding="utf-8",
    )
    return script


def run_stage_with_read_failure(
    args: object,
    unreadable_file: Path,
    message: str,
) -> int:
    original_read_bytes = Path.read_bytes

    def read_bytes_or_fail(path: Path) -> bytes:
        if path.resolve() == unreadable_file:
            raise OSError(message)
        return original_read_bytes(path)

    with mock.patch.object(Path, "read_bytes", read_bytes_or_fail):
        return _run_stage_quiet(args)


def run_stage_with_second_read_failure(
    args: object,
    unreadable_file: Path,
    message: str,
) -> int:
    original_read_bytes = Path.read_bytes
    read_counts: dict[Path, int] = {}

    def read_bytes_or_fail(path: Path) -> bytes:
        resolved = path.resolve()
        if resolved == unreadable_file:
            read_counts[resolved] = read_counts.get(resolved, 0) + 1
            if read_counts[resolved] == 2:
                raise OSError(message)
        return original_read_bytes(path)

    with mock.patch.object(Path, "read_bytes", read_bytes_or_fail):
        return _run_stage_quiet(args)


if __name__ == "__main__":
    unittest.main()
