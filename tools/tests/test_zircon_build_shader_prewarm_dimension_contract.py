import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_build_shader_prewarm import validate_shader_prewarm_report_contract


class ZirconBuildShaderPrewarmDimensionContractTests(unittest.TestCase):
    def test_validate_report_contract_requires_requested_pass_types(self):
        report_path = _write_report(
            {
                "dimension_summary": {
                    "pass_types": {
                        "shadow": {
                            "requested_count": 1,
                            "written_count": 1,
                            "failed_count": 0,
                        },
                    },
                },
            }
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "missing requested pass types: forward",
        ):
            validate_shader_prewarm_report_contract(
                report_path,
                expected_pass_types=("forward",),
            )

    def test_validate_report_contract_requires_requested_quality_tiers(self):
        report_path = _write_report(
            {
                "dimension_summary": {
                    "quality_tiers": {
                        "medium": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                    },
                },
            }
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "missing requested quality tiers",
        ):
            validate_shader_prewarm_report_contract(
                report_path,
                expected_quality_tiers=("medium", "high"),
            )

    def test_validate_report_contract_requires_requested_geometry_sources(self):
        report_path = _write_report(
            {
                "dimension_summary": {
                    "geometry_source_ids": {
                        "0": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                    },
                },
            }
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "missing requested geometry sources",
        ):
            validate_shader_prewarm_report_contract(
                report_path,
                expected_geometry_sources=("static", "skinned"),
            )

    def test_validate_report_contract_rejects_incomplete_requested_dimension_counts(
        self,
    ):
        report_path = _write_report(
            {
                "dimension_summary": {
                    "pass_types": {
                        "forward": {
                            "requested_count": 6,
                            "written_count": 5,
                            "failed_count": 1,
                        },
                    },
                },
            }
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "did not fully write requested pass types: "
            "forward requested=6 written=5 failed=1",
        ):
            validate_shader_prewarm_report_contract(
                report_path,
                expected_pass_types=("forward",),
            )

    def test_validate_report_contract_rejects_dimension_count_total_mismatch(self):
        report_path = _write_report(
            {
                "requested_count": 6,
                "written_count": 6,
                "failed_count": 0,
                "dimension_summary": {
                    "pass_types": {
                        "forward": {
                            "requested_count": 7,
                            "written_count": 7,
                            "failed_count": 0,
                        },
                    },
                },
            }
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "shader prewarm report pass type counts did not match report totals: "
            "requested=6/7 written=6/7 failed=0/0",
        ):
            validate_shader_prewarm_report_contract(
                report_path,
                expected_pass_types=("forward",),
            )

    def test_validate_report_contract_requires_requested_geometry_source_ids(self):
        report_path = _write_report(
            {
                "dimension_summary": {
                    "geometry_source_ids": {
                        "0": {"requested_count": 6},
                    },
                },
            }
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "missing requested shader geometry source ids: custom:virtual_geometry=4",
        ):
            validate_shader_prewarm_report_contract(
                report_path,
                expected_geometry_source_ids=("custom:virtual_geometry=4",),
            )

    def test_validate_report_contract_requires_requested_shading_model_ids(self):
        report_path = _write_report(
            {
                "dimension_summary": {
                    "shading_model_ids": {
                        "2": {"requested_count": 6},
                    },
                },
            }
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "missing requested shader shading model ids: custom:toon=16",
        ):
            validate_shader_prewarm_report_contract(
                report_path,
                expected_shading_model_ids=("toon=16",),
            )

    def test_validate_report_contract_accepts_requested_dimensions(self):
        report_path = _write_report(
            {
                "dimension_summary": {
                    "pass_types": {
                        "forward": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                    },
                    "quality_tiers": {
                        "medium": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                        "high": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                    },
                    "geometry_source_ids": {
                        "0": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                        "1": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                        "4": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                    },
                    "shading_model_ids": {
                        "16": {
                            "requested_count": 6,
                            "written_count": 6,
                            "failed_count": 0,
                        },
                    },
                },
            }
        )

        validate_shader_prewarm_report_contract(
            report_path,
            expected_pass_types=("forward",),
            expected_quality_tiers=("medium", "high"),
            expected_geometry_sources=("static", "skinned"),
            expected_geometry_source_ids=("custom:virtual_geometry=4",),
            expected_shading_model_ids=("toon=16",),
        )


def _write_report(report: dict) -> Path:
    temp_dir = tempfile.TemporaryDirectory()
    path = Path(temp_dir.name) / "shader_variants_report.json"
    path.write_text(json.dumps(report), encoding="utf-8")
    _TEMP_DIRS.append(temp_dir)
    return path


_TEMP_DIRS: list[tempfile.TemporaryDirectory] = []


if __name__ == "__main__":
    unittest.main()
