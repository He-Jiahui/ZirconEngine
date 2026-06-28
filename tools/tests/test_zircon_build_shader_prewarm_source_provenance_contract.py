import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_build_shader_prewarm_report_contract import (
    validate_shader_prewarm_report_contract,
)


class ZirconBuildShaderPrewarmSourceProvenanceContractTests(unittest.TestCase):
    def test_validate_report_contract_requires_source_provenance_when_requested(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            report_path = Path(temp_dir) / "shader_variants_report.json"
            report_path.write_text(
                json.dumps({"requested_count": 6, "written_count": 6}),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "did not confirm shader source provenance",
            ):
                validate_shader_prewarm_report_contract(
                    report_path,
                    require_source_provenance=True,
                )

    def test_validate_report_contract_accepts_source_provenance_counts(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            report_path = Path(temp_dir) / "shader_variants_report.json"
            report_path.write_text(
                json.dumps(
                    {
                        "requested_count": 6,
                        "written_count": 6,
                        "failed_count": 0,
                        "source_provenance": {
                            "source_count": 1,
                            "variant_count": 6,
                            "sources": {
                                "res://shader#a#template": {
                                    "source_label": "res://shader",
                                    "source_hash": "a",
                                    "template_revision": "template",
                                    "requested_count": 6,
                                    "written_count": 6,
                                    "failed_count": 0,
                                }
                            },
                        },
                    }
                ),
                encoding="utf-8",
            )

            validate_shader_prewarm_report_contract(
                report_path,
                require_source_provenance=True,
            )

    def test_validate_report_contract_rejects_source_provenance_count_mismatch(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            report_path = Path(temp_dir) / "shader_variants_report.json"
            report_path.write_text(
                json.dumps(
                    {
                        "requested_count": 6,
                        "written_count": 6,
                        "failed_count": 0,
                        "source_provenance": {
                            "source_count": 1,
                            "variant_count": 6,
                            "sources": {
                                "res://shader#a#template": {
                                    "source_label": "res://shader",
                                    "source_hash": "a",
                                    "template_revision": "template",
                                    "requested_count": 6,
                                    "written_count": 5,
                                    "failed_count": 1,
                                }
                            },
                        },
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "shader prewarm source provenance counts did not match report totals",
            ):
                validate_shader_prewarm_report_contract(
                    report_path,
                    require_source_provenance=True,
                )


if __name__ == "__main__":
    unittest.main()
