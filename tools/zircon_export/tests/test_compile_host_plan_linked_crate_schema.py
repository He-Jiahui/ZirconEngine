from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _compile_host_args,
    _compile_host_plan,
    _run_compile_host_quiet,
    json_dumps,
    json_loads,
)


class CompileHostPlanLinkedCrateSchemaTests(unittest.TestCase):
    def test_compile_host_rejects_plan_with_padded_linked_crate_field(
        self,
    ) -> None:
        cases = (
            (
                "path",
                " zircon_plugins/rendering/runtime ",
                "CompileHost plan linked_runtime_crates[0].path must be a "
                "non-empty trimmed string",
            ),
            (
                "registration_kind",
                " runtime_plugin ",
                "CompileHost plan linked_runtime_crates[0].registration_kind "
                "must be a non-empty trimmed string",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    linked_crate = {
                        "crate_name": "zircon_plugin_rendering_runtime",
                        "path": "zircon_plugins/rendering/runtime",
                        "provider_package_id": "rendering",
                        "registration_kind": "runtime_plugin",
                    }
                    linked_crate[field] = value
                    compile_plan["linked_runtime_crates"] = [linked_crate]
                    validate_report = root / "validate.json"
                    validate_report.write_text(
                        json_dumps(
                            {
                                "stage": "Validate",
                                "profile": "windows-release",
                                "fatal": False,
                                "diagnostics": [],
                                "plan_summary": {
                                    "library_embed_compile_host": compile_plan,
                                },
                            }
                        ),
                        encoding="utf-8",
                    )
                    args = _compile_host_args(
                        out=root / "out",
                        validate_report=validate_report,
                    )
                    args.dry_run = False

                    with mock.patch(
                        "tools.zircon_export.compile_host.subprocess.run",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as cargo_call:
                        exit_code = _run_compile_host_quiet(args)

                    report = json_loads(
                        (
                            root
                            / "out"
                            / "stages"
                            / "compile_host"
                            / "report.json"
                        ).read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    cargo_call.assert_not_called()
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertIsNone(report["host_executable"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
