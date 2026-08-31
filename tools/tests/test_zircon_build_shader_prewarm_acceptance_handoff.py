import subprocess
import unittest
from unittest.mock import patch

from tools import zircon_build
from tools.tests.test_zircon_build_shader_prewarm_acceptance_contract import (
    _FakePrewarmConfig,
)


class ZirconBuildShaderPrewarmAcceptanceHandoffTests(unittest.TestCase):
    @patch.object(zircon_build, "managed_cargo_environment")
    def test_prewarm_shaders_runs_acceptance_bundle_after_success(
        self, managed_cargo_environment
    ):
        config = _FakePrewarmConfig()
        events: list[str] = []
        managed_environment = {"ZR_TEST_MANAGED_ENV": "1"}
        managed_cargo_environment.return_value = managed_environment

        def fake_run(command, cwd, check, env):
            self.assertFalse(check)
            self.assertEqual(config.repo_root, cwd)
            self.assertIs(managed_environment, env)
            events.append("run")
            return subprocess.CompletedProcess(command, 0)

        with patch.object(zircon_build.subprocess, "run", side_effect=fake_run):
            with patch.object(
                zircon_build,
                "print_shader_prewarm_report_dimensions",
                side_effect=lambda report_path: events.append(f"summary:{report_path}"),
            ):
                with patch.object(
                    zircon_build,
                    "validate_staged_shader_prewarm_acceptance_contract",
                    side_effect=lambda actual_config: events.append(
                        f"acceptance:{actual_config is config}"
                    ),
                ):
                    zircon_build.prewarm_shaders(config)

        managed_cargo_environment.assert_called_once_with(
            config.targets_root / "shader_prewarm", config.targets_root
        )

        self.assertEqual(
            [
                "run",
                f"summary:{config.shader_prewarm_report_path}",
                "acceptance:True",
            ],
            events,
        )


if __name__ == "__main__":
    unittest.main()
