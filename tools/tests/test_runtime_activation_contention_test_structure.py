import unittest
from pathlib import Path


class RuntimeActivationContentionTestStructureTests(unittest.TestCase):
    def test_activation_contention_tests_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        activation_path = (
            repo_root
            / "zircon_runtime/src/core/runtime/tests/activation/behavior/activation.rs"
        )
        contention_path = (
            repo_root
            / "zircon_runtime/src/core/runtime/tests/activation/behavior/activation/contention.rs"
        )

        activation = activation_path.read_text(encoding="utf-8")
        contention = contention_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(activation.splitlines()), 800)
        self.assertLessEqual(len(contention.splitlines()), 800)
        self.assertIn("mod contention;", activation)
        self.assertEqual(activation.count("#[test]"), 11)
        self.assertEqual(contention.count("#[test]"), 2)
        for test_name in (
            "concurrent_activation_joiners_share_one_build_within_contention_budget",
            "concurrent_activation_joiners_release_benchmark_evidence",
        ):
            self.assertNotIn(f"fn {test_name}", activation)
            self.assertIn(f"fn {test_name}", contention)

        for anchor in (
            "Duration::from_millis(750)",
            "let joiner_count = 7;",
            "const SAMPLE_COUNT: usize = 21;",
            "PERF_RESULT runtime01_activation_join",
        ):
            self.assertIn(anchor, contention)


if __name__ == "__main__":
    unittest.main()
