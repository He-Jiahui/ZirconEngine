import subprocess
import unittest
from unittest.mock import patch

from tools import zircon_build
from tools.tests.test_zircon_build_shader_prewarm_acceptance_contract import (
    _FakePrewarmConfig,
)


class ZirconBuildShaderPrewarmAcceptanceHandoffTests(unittest.TestCase):
    def test_prewarm_shaders_runs_acceptance_bundle_after_success(self):
        config = _FakePrewarmConfig()
        events: list[str] = []

        def fake_run(command, cwd, check):
            self.assertFalse(check)
            self.assertEqual(config.repo_root, cwd)
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
