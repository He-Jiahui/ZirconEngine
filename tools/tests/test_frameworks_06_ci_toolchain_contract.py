from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_TOOLCHAIN = "1.94.1"


class Frameworks06CiToolchainContractTests(unittest.TestCase):
    def test_convention_job_installs_the_runner_named_toolchain(self) -> None:
        workflow = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )
        runner = (REPO_ROOT / "tools" / "check_conventions.py").read_text(
            encoding="utf-8"
        )
        rust_job_match = re.search(
            r"(?ms)^  rust:\s*$\n(?P<body>.*?)(?=^  [A-Za-z0-9_-]+:\s*$|\Z)",
            workflow,
        )

        self.assertIsNotNone(rust_job_match)
        rust_job = rust_job_match.group("body")
        self.assertRegex(
            runner,
            rf'"cargo",\s*"\+{re.escape(RUST_TOOLCHAIN)}",\s*"fmt"',
        )
        self.assertRegex(
            runner,
            rf'"cargo",\s*"\+{re.escape(RUST_TOOLCHAIN)}",\s*"clippy"',
        )
        self.assertRegex(
            rust_job,
            rf"(?m)^\s*uses:\s*dtolnay/rust-toolchain@{re.escape(RUST_TOOLCHAIN)}\s*$",
        )
        self.assertRegex(
            rust_job,
            r"(?m)^\s*components:\s*rustfmt,\s*clippy\s*$",
        )
        self.assertEqual(
            rust_job.count(
                "python -m unittest "
                "tools.tests.test_check_conventions "
                "tools.tests.test_frameworks_06_ci_toolchain_contract -v"
            ),
            1,
        )


if __name__ == "__main__":
    unittest.main()
