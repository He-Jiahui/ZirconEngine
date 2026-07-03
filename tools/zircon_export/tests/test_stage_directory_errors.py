from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.cli import (
    run_compile_host,
    run_cook_assets,
    run_platform_bundle,
    run_source_template,
    run_validate,
)
from tools.zircon_export.native_dynamic import run_native_dynamic
from tools.zircon_export.pack_stage import run_pack
from tools.zircon_export.tests.native_dynamic_test_support import (
    _export_args as _native_dynamic_args,
    _write_validate_report_with_native_dynamic_exports,
)
from tools.zircon_export.tests.export_test_support import (
    VALID_TEMPLATE,
    _compile_host_args,
    _cook_assets_args,
    _export_args,
    _pack_args,
    _platform_bundle_args,
    _source_template_args,
    _source_template_validate_report,
    json_dumps,
    json_loads,
)


def _printed_json_object(output: str) -> dict[str, object]:
    json_start = output.find("{\n")
    if json_start == -1:
        raise AssertionError(f"no JSON object found in output:\n{output}")
    parsed = json_loads(output[json_start:])
    if not isinstance(parsed, dict):
        raise AssertionError(f"expected JSON object, got {type(parsed).__name__}")
    return parsed


class StageDirectoryErrorTests(unittest.TestCase):
    def test_pack_preflight_reports_stage_directory_create_error_to_stdout(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            stage_dir = (out / "stages" / "pack").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == stage_dir:
                    raise OSError("simulated pack stage dir create failure")
                original_mkdir(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_pack(_pack_args(out=out, dry_run=False))

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse((stage_dir / "report.json").exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "Pack stage directory" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated pack stage dir create failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )

    def test_compile_host_reports_stage_directory_create_error_to_stdout(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            stage_dir = (out / "stages" / "compile_host").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == stage_dir:
                    raise OSError("simulated compile_host stage dir create failure")
                original_mkdir(path, *args, **kwargs)

            stdout = io.StringIO()
            args = _compile_host_args(out=out)
            args.dry_run = False
            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_compile_host(args)

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse((stage_dir / "report.json").exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "CompileHost stage directory" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated compile_host stage dir create failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )

    def test_cook_assets_reports_stage_directory_create_error_to_stdout(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_dir = root / "source"
            source_dir.mkdir()
            (source_dir / "main.scene").write_text("scene", encoding="utf-8")
            source_manifest = source_dir / "assets.json"
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": "main.scene",
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            out = root / "out"
            stage_dir = (out / "stages" / "cook_assets").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == stage_dir:
                    raise OSError("simulated cook_assets stage dir create failure")
                original_mkdir(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_cook_assets(
                        _cook_assets_args(out=out, asset_manifest=source_manifest)
                    )

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse((stage_dir / "report.json").exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "CookAssets stage directory" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated cook_assets stage dir create failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )

    def test_source_template_reports_stage_directory_create_error_to_stdout(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            out = root / "out"
            stage_dir = (out / "stages" / "source_template").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == stage_dir:
                    raise OSError("simulated source_template stage dir create failure")
                original_mkdir(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_source_template(
                        _source_template_args(
                            out=out,
                            validate_report=validate_report,
                            dry_run=False,
                        )
                    )

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse((stage_dir / "report.json").exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "SourceTemplate stage directory" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated source_template stage dir create failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )

    def test_native_dynamic_reports_stage_directory_create_error_to_stdout(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            stage_dir = (out / "stages" / "native_dynamic").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == stage_dir:
                    raise OSError("simulated native_dynamic stage dir create failure")
                original_mkdir(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_native_dynamic(
                        _native_dynamic_args(
                            out=out,
                            stage="native_dynamic",
                            dry_run=False,
                        )
                    )

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse((stage_dir / "report.json").exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic stage directory" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated native_dynamic stage dir create failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )

    def test_platform_bundle_reports_stage_directory_create_error_to_stdout(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack_file = root / "assets.zrpack"
            pack_file.write_text("pack placeholder", encoding="utf-8")
            stage_dir = (out / "stages" / "platform_bundle").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == stage_dir:
                    raise OSError("simulated platform_bundle stage dir create failure")
                original_mkdir(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_platform_bundle(
                        _platform_bundle_args(
                            out=out,
                            template_dir=VALID_TEMPLATE,
                            pack_file=pack_file,
                            target_platform="windows-x86_64",
                            profile="windows-release",
                        )
                    )

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse((stage_dir / "report.json").exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "PlatformBundle stage directory" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated platform_bundle stage dir create failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )

    def test_validate_reports_stage_directory_create_error_to_stdout(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            stage_dir = (out / "stages" / "validate").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == stage_dir:
                    raise OSError("simulated validate stage dir create failure")
                original_mkdir(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_validate(
                        _export_args(
                            out=out,
                            stage="validate",
                            dry_run=False,
                        )
                    )

            printed_report = _printed_json_object(stdout.getvalue())
            self.assertEqual(exit_code, 2)
            self.assertFalse((stage_dir / "report.json").exists())
            self.assertTrue(printed_report["fatal"], printed_report["diagnostics"])
            self.assertTrue(
                any(
                    "Validate stage directory" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated validate stage dir create failure" in diagnostic
                    for diagnostic in printed_report["diagnostics"]
                ),
                printed_report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
