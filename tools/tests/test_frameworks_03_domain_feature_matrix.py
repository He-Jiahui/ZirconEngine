from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MATRIX_SCRIPT = REPO_ROOT / "tools" / "check-runtime-domain-features.ps1"
EXPECTED_DOMAIN_FEATURES = (
    "ai-contracts",
    "animation",
    "diagnostic-log",
    "dynamic-api",
    "graphics",
    "navigation",
    "net-contracts",
    "physics-contracts",
    "script",
    "sound-contracts",
    "text",
    "ui",
)


class Frameworks03DomainFeatureMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.script = MATRIX_SCRIPT.read_text(encoding="utf-8")

    def test_matrix_lists_every_runtime_domain_feature(self) -> None:
        feature_block = self.script.split("$domainFeatures = @(", 1)[1].split(")", 1)[0]
        actual_features = tuple(re.findall(r'"([a-z0-9-]+)"', feature_block))
        self.assertEqual(actual_features, EXPECTED_DOMAIN_FEATURES)

    def test_matrix_checks_one_domain_over_the_minimal_baseline(self) -> None:
        for argument in (
            '"--lib"',
            '"--no-default-features"',
            '"--features"',
            '"core-min,$domainFeature"',
            '"--locked"',
        ):
            self.assertIn(argument, self.script)

    def test_matrix_cannot_silently_skip_a_failing_domain(self) -> None:
        self.assertIn("$failedFeatures.Add($domainFeature)", self.script)
        self.assertIn("exit 1", self.script)
        self.assertNotIn("SkipMissing", self.script)

    def test_matrix_isolates_all_cargo_derived_data_on_managed_storage(self) -> None:
        self.assertIn("WindowsPathResolver.psm1", self.script)
        self.assertIn("Resolve-ZirconWindowsPath", self.script)
        self.assertIn("cargo-targets\\zircon-runtime-domain-matrix", self.script)
        self.assertIn("CARGO_HOME", self.script)
        self.assertIn("SCCACHE_DIR", self.script)
        for name in ("TEMP", "TMP", "TMPDIR"):
            self.assertIn(name, self.script)
        self.assertIn("D:\\cargo-targets, E:\\cargo-targets, or F:\\cargo-targets", self.script)


if __name__ == "__main__":
    unittest.main()
