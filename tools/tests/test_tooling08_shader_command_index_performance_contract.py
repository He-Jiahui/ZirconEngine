from __future__ import annotations

import unittest
from unittest import mock

from tools import zircon_build_shader_prewarm as shader_prewarm
from tools.tests.shader_prewarm_test_support import FakePrewarmConfig


class Tooling08ShaderCommandIndexPerformanceContractTests(unittest.TestCase):
    def test_validation_builds_one_shared_command_flag_index(self) -> None:
        command = shader_prewarm.build_shader_prewarm_command(FakePrewarmConfig())
        index_builder = getattr(shader_prewarm, "_command_flag_index", None)

        self.assertIsNotNone(index_builder)
        with mock.patch.object(
            shader_prewarm,
            "_command_flag_index",
            wraps=index_builder,
        ) as build_index:
            shader_prewarm.validate_shader_prewarm_command_contract(
                FakePrewarmConfig(),
                command,
            )

        self.assertEqual(1, build_index.call_count)

    def test_validation_does_not_repeat_per_flag_command_scans(self) -> None:
        config = FakePrewarmConfig()
        command = shader_prewarm.build_shader_prewarm_command(config)

        with mock.patch.object(
            shader_prewarm,
            "_command_flag_values",
            wraps=shader_prewarm._command_flag_values,
        ) as scan_for_flag:
            shader_prewarm.validate_shader_prewarm_command_contract(config, command)

        self.assertEqual(0, scan_for_flag.call_count)

    def test_trailing_value_flag_still_reports_missing_value(self) -> None:
        config = FakePrewarmConfig()
        command = shader_prewarm.build_shader_prewarm_command(config)
        command.append("--quality-tier")

        with self.assertRaisesRegex(RuntimeError, "missing a value"):
            shader_prewarm.validate_shader_prewarm_command_contract(config, command)


if __name__ == "__main__":
    unittest.main()
