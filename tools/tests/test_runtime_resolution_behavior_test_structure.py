import unittest
from pathlib import Path


class RuntimeResolutionBehaviorTestStructureTests(unittest.TestCase):
    def test_exact_dependency_resolution_tests_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        behavior_path = (
            repo_root / "zircon_runtime/src/core/runtime/tests/resolution/behavior.rs"
        )
        exact_path = (
            repo_root
            / "zircon_runtime/src/core/runtime/tests/resolution/behavior/exact_dependency_resolution.rs"
        )

        behavior = behavior_path.read_text(encoding="utf-8")
        exact = exact_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(behavior.splitlines()), 800)
        self.assertLessEqual(len(exact.splitlines()), 800)
        self.assertIn("mod dependency_cycles;", behavior)
        self.assertIn("mod exact_dependency_resolution;", behavior)
        self.assertIn("mod factory_panics;", behavior)
        self.assertEqual(behavior.count("#[test]"), 10)
        self.assertEqual(exact.count("#[test]"), 2)
        for test_name in (
            "resolve_exact_four_dependencies_initializes_cached_keys_directly",
            "resolve_exact_five_dependencies_initializes_cached_keys_directly",
        ):
            self.assertNotIn(f"fn {test_name}", behavior)
            self.assertIn(f"fn {test_name}", exact)


if __name__ == "__main__":
    unittest.main()
