import unittest
from pathlib import Path


class RuntimeDynamicProjectTestStructureTests(unittest.TestCase):
    def test_dynamic_project_tests_are_folder_backed(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        project_path = repo_root / "zircon_runtime/src/dynamic_api/session/project.rs"
        tests_path = repo_root / "zircon_runtime/src/dynamic_api/session/project/tests.rs"

        project = project_path.read_text(encoding="utf-8")
        tests = tests_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(project.splitlines()), 800)
        self.assertIn('#[path = "project/tests.rs"]', project)
        self.assertIn("mod tests;", project)
        self.assertNotIn("mod tests {", project)
        self.assertIn('#[path = "project/runtime61_characterization.rs"]', project)
        self.assertIn(
            "fn project_startup_snapshot_survives_disk_manifest_rewrite_before_activation()",
            tests,
        )
        self.assertIn("fn project_manifest_filters_startup_script_packages()", tests)
        self.assertEqual(tests.count("#[test]"), 14)


if __name__ == "__main__":
    unittest.main()
