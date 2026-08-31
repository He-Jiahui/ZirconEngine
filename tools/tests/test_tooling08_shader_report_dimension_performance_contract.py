from __future__ import annotations

import unittest

from tools import zircon_build_shader_prewarm_report_contract as report_contract


class _CountingGroup(dict[str, object]):
    def __init__(self, values: dict[str, object]) -> None:
        super().__init__(values)
        self.get_count = 0

    def get(self, key, default=None):
        self.get_count += 1
        return super().get(key, default)


def _complete_counts() -> dict[str, int]:
    return {"requested": 1, "written": 1, "failed": 0}


class Tooling08ShaderReportDimensionPerformanceContractTests(unittest.TestCase):
    def test_pass_dimension_reads_each_expected_entry_once(self) -> None:
        expected = tuple(f"pass-{index}" for index in range(512))
        group = _CountingGroup({key: _complete_counts() for key in expected})

        report_contract._validate_expected_pass_types(
            {"pass_types": group},
            expected,
        )

        self.assertEqual(len(expected), group.get_count)

    def test_custom_id_dimension_reads_each_expected_entry_once(self) -> None:
        expected = tuple(f"custom:geometry-{index}={index}" for index in range(256))
        group = _CountingGroup(
            {str(index): _complete_counts() for index in range(256)}
        )

        report_contract._validate_expected_shader_dimension_ids(
            {"geometry_source_ids": group},
            "geometry_source_ids",
            "shader geometry source id",
            expected,
        )

        self.assertEqual(len(expected), group.get_count)

    def test_missing_dimension_error_still_precedes_incomplete_error(self) -> None:
        group = {
            "forward": {"requested": 2, "written": 1, "failed": 1},
        }

        with self.assertRaisesRegex(RuntimeError, "missing requested pass types"):
            report_contract._validate_expected_pass_types(
                {"pass_types": group},
                ("forward", "shadow"),
            )

    def test_structure_guard_helpers_remain_available(self) -> None:
        source = report_contract.Path(report_contract.__file__).read_text(
            encoding="utf-8"
        )
        self.assertIn("def _expected_dimension_count_failures(", source)
        self.assertIn("def _dimension_has_requested_count(", source)
        self.assertIn("def _incomplete_dimension_counts(", source)


if __name__ == "__main__":
    unittest.main()
