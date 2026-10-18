"""Lock the isolated coordinator launch contract used by the M0.1 smoke test."""

from pathlib import Path
import re
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
SMOKE_SCRIPT = REPO_ROOT / "tools" / "tests" / "session-coordinator-smoke.Tests.ps1"


class Mvp00SessionCoordinatorSmokeContractTests(unittest.TestCase):
    def test_kernel_uses_ephemeral_listener_port(self) -> None:
        source = SMOKE_SCRIPT.read_text(encoding="utf-8")
        kernel = source.split("function Test-Kernel", 1)[1].split(
            "function Test-JsonClientOutput", 1
        )[0]

        launch = re.search(
            r"-ArgumentList\s+@\(\s*\"-m\",\s*\"tools\.session_coordinator\",\s*"
            r"\"--repo-root\",\s*\$repo,\s*\"--port\",\s*\"(?P<port>\d+)\",\s*"
            r"\"serve\"\s*\)",
            kernel,
            re.MULTILINE,
        )
        self.assertIsNotNone(
            launch,
            "Test-Kernel must launch an isolated coordinator with an explicit port.",
        )
        self.assertEqual("0", launch.group("port"))

    def test_kernel_uses_bounded_readiness_with_startup_diagnostics(self) -> None:
        source = SMOKE_SCRIPT.read_text(encoding="utf-8")
        kernel = source.split("function Test-Kernel", 1)[1].split(
            "function Test-JsonClientOutput", 1
        )[0]

        self.assertRegex(
            source,
            r"\$CoordinatorReadinessTimeoutSeconds\s*=\s*30",
            "Coordinator readiness must have a bounded timeout of at least 30 seconds.",
        )
        self.assertRegex(
            source,
            r"function\s+Wait-CoordinatorHealthy",
            "Test-Kernel must use a shared adaptive readiness helper.",
        )
        self.assertRegex(
            kernel,
            r"Wait-CoordinatorHealthy[\s`\r\n]+-Process[\s`\r\n]+\$process",
            "Test-Kernel must call the adaptive readiness helper.",
        )
        self.assertRegex(
            kernel,
            r"-RedirectStandardOutput[\s`\r\n]+\$stdoutPath",
            "Test-Kernel must capture coordinator stdout for readiness diagnostics.",
        )
        self.assertRegex(
            kernel,
            r"-RedirectStandardError[\s`\r\n]+\$stderrPath",
            "Test-Kernel must capture coordinator stderr for readiness diagnostics.",
        )
        self.assertRegex(
            kernel,
            r"lifecycle_global_shutdown_disabled",
            "Test-Kernel must preserve the production global-stop admission guard.",
        )
        self.assertRegex(
            kernel,
            r"\$process\.Kill\(\$true\)",
            "Test-Kernel must terminate only its disposable process tree after the guard assertion.",
        )
        self.assertRegex(
            kernel,
            r"\$global:LASTEXITCODE\s*=\s*0",
            "Test-Kernel must clear the expected rejection exit code before reporting success.",
        )

    def test_lease_patch_uses_same_isolated_readiness_and_cleanup_contract(self) -> None:
        source = SMOKE_SCRIPT.read_text(encoding="utf-8")
        lease_patch = source.split("function Test-LeaseAndPatch", 1)[1].split(
            "function Test-CargoAndCleanup", 1
        )[0]

        launch = re.search(
            r"-ArgumentList\s+@\(\s*\"-m\",\s*\"tools\.session_coordinator\",\s*"
            r"\"--repo-root\",\s*\$repo,\s*\"--port\",\s*\"(?P<port>\d+)\",\s*"
            r"\"serve\"\s*\)",
            lease_patch,
            re.MULTILINE,
        )
        self.assertIsNotNone(
            launch,
            "Test-LeaseAndPatch must launch an isolated coordinator with an explicit port.",
        )
        self.assertEqual("0", launch.group("port"))
        self.assertRegex(
            lease_patch,
            r"Wait-CoordinatorHealthy[\s`\r\n]+-Process[\s`\r\n]+\$process",
            "Test-LeaseAndPatch must use the bounded readiness helper.",
        )
        self.assertRegex(
            lease_patch,
            r"-RedirectStandardOutput[\s`\r\n]+\$stdoutPath",
            "Test-LeaseAndPatch must capture stdout for readiness diagnostics.",
        )
        self.assertRegex(
            lease_patch,
            r"-RedirectStandardError[\s`\r\n]+\$stderrPath",
            "Test-LeaseAndPatch must capture stderr for readiness diagnostics.",
        )
        self.assertRegex(
            lease_patch,
            r"lifecycle_global_shutdown_disabled",
            "Test-LeaseAndPatch must preserve the production global-stop admission guard.",
        )
        self.assertRegex(
            lease_patch,
            r"\$process\.Kill\(\$true\)",
            "Test-LeaseAndPatch must terminate only its disposable process tree after the guard assertion.",
        )
        self.assertRegex(
            lease_patch,
            r"\$global:LASTEXITCODE\s*=\s*0",
            "Test-LeaseAndPatch must clear the expected rejection exit code before reporting success.",
        )

    def test_finalize_and_legacy_use_same_isolated_readiness_and_cleanup_contract(
        self,
    ) -> None:
        source = SMOKE_SCRIPT.read_text(encoding="utf-8")
        lanes = {
            "Test-FinalizeInTempRepo": source.split(
                "function Test-FinalizeInTempRepo", 1
            )[1].split("function Test-LegacyRollout", 1)[0],
            "Test-LegacyRollout": source.split("function Test-LegacyRollout", 1)[
                1
            ].split("$runAll", 1)[0],
        }

        for lane_name, lane in lanes.items():
            with self.subTest(lane=lane_name):
                launch = re.search(
                    r"-ArgumentList\s+@\(\s*\"-m\",\s*\"tools\.session_coordinator\",\s*"
                    r"\"--repo-root\",\s*\$repo,\s*\"--port\",\s*\"(?P<port>\d+)\",\s*"
                    r"\"serve\"\s*\)",
                    lane,
                    re.MULTILINE,
                )
                self.assertIsNotNone(
                    launch,
                    f"{lane_name} must launch an isolated coordinator with an explicit port.",
                )
                self.assertEqual("0", launch.group("port"))
                self.assertRegex(
                    lane,
                    r"Wait-CoordinatorHealthy[\s`\r\n]+-Process[\s`\r\n]+\$process",
                    f"{lane_name} must use the bounded readiness helper.",
                )
                self.assertRegex(
                    lane,
                    r"-RedirectStandardOutput[\s`\r\n]+\$stdoutPath",
                    f"{lane_name} must capture stdout for readiness diagnostics.",
                )
                self.assertRegex(
                    lane,
                    r"-RedirectStandardError[\s`\r\n]+\$stderrPath",
                    f"{lane_name} must capture stderr for readiness diagnostics.",
                )
                self.assertRegex(
                    lane,
                    r"lifecycle_global_shutdown_disabled",
                    f"{lane_name} must preserve the production global-stop admission guard.",
                )
                self.assertRegex(
                    lane,
                    r"\$process\.Kill\(\$true\)",
                    f"{lane_name} must terminate only its disposable process tree after the guard assertion.",
                )
                self.assertRegex(
                    lane,
                    r"\$global:LASTEXITCODE\s*=\s*0",
                    f"{lane_name} must clear the expected rejection exit code before reporting success.",
                )

        legacy = lanes["Test-LegacyRollout"]
        self.assertRegex(
            legacy,
            r"ConvertFrom-Json[\s\S]+\.migration",
            "Legacy repeatability must compare the semantic migration payload.",
        )
        self.assertNotRegex(
            legacy,
            r"Get-FileHash\s+\$reportOne",
            "Legacy repeatability must ignore the unique request envelope id.",
        )


if __name__ == "__main__":
    unittest.main()
