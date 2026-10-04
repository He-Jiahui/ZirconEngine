import unittest
from pathlib import Path

# 约束插件系统注册的测试组织：插件系统注册测试应目录支撑。
class RuntimePluginSystemRegistrationTestStructureTests(unittest.TestCase):
    def test_plugin_system_registration_tests_are_folder_backed(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/plugin/extension_registry/register/system_registration.rs"
        )
        tests_path = (
            repo_root / "zircon_runtime/src/plugin/extension_registry/register/system_registration/tests/cases.rs"
        )

        owner = owner_path.read_text(encoding="utf-8")
        tests = tests_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 800)
        self.assertIn("#[path = \"system_registration/tests/cases.rs\"]", owner)
        self.assertIn("mod tests;", owner)
        self.assertNotIn("mod tests {", owner)
        self.assertIn("fn retire(&mut self, world: &mut World)", owner)
        self.assertIn("self.state.retire(world);", owner)
        self.assertEqual(tests.count("#[test]"), 3)
        self.assertIn("fn typed_scene_system_callback_state_is_private_per_world()", tests)
        self.assertIn("fn external_scene_system_callback_state_is_private_per_world()", tests)
        self.assertIn("fn external_scene_system_callbacks_overlap_across_worlds()", tests)


if __name__ == "__main__":
    unittest.main()
