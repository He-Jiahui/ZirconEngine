import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_build_shader_prewarm_report_contract import (
    validate_shader_prewarm_report_contract,
)


class ZirconBuildShaderPrewarmWgpuReportContractTests(unittest.TestCase):
    def test_validate_report_contract_requires_wgpu_validation_when_requested(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            report_path = Path(temp_dir) / "shader_variants_report.json"
            report_path.write_text(
                json.dumps({"requested_count": 6, "written_count": 6}),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "did not confirm WGPU module validation",
            ):
                validate_shader_prewarm_report_contract(
                    report_path,
                    require_wgpu_module_validation=True,
                )

    def test_validate_report_contract_accepts_wgpu_validation_counts(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            report_path = Path(temp_dir) / "shader_variants_report.json"
            report_path.write_text(
                json.dumps(
                    {
                        "requested_count": 6,
                        "written_count": 6,
                        "failed_count": 0,
                        "wgpu_module_validation": {
                            "enabled": True,
                            "requested_count": 6,
                            "validated_count": 6,
                            "failed_count": 0,
                            "skipped_count": 0,
                        },
                    }
                ),
                encoding="utf-8",
            )

            validate_shader_prewarm_report_contract(
                report_path,
                require_wgpu_module_validation=True,
            )

    def test_validate_report_contract_rejects_wgpu_validation_total_mismatch(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            report_path = Path(temp_dir) / "shader_variants_report.json"
            report_path.write_text(
                json.dumps(
                    {
                        "requested_count": 6,
                        "written_count": 6,
                        "failed_count": 0,
                        "wgpu_module_validation": {
                            "enabled": True,
                            "requested_count": 5,
                            "validated_count": 5,
                            "failed_count": 0,
                            "skipped_count": 0,
                        },
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "WGPU module validation counts did not match report totals",
            ):
                validate_shader_prewarm_report_contract(
                    report_path,
                    require_wgpu_module_validation=True,
                )


if __name__ == "__main__":
    unittest.main()
