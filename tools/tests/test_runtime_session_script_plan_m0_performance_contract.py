import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
CONSTRUCTION = ROOT / "zircon_runtime/src/dynamic_api/session/construction.rs"
SCRIPT_SYSTEMS = ROOT / "zircon_runtime/src/dynamic_api/session/script_systems.rs"


class RuntimeSessionScriptPlanM0PerformanceContract(unittest.TestCase):
    def test_construction_does_not_materialize_a_plan_before_script_merge(self):
        source = CONSTRUCTION.read_text(encoding="utf-8")
        start = source.index("let linked_extension_world_plan =")
        end = source.index("    let runtime =", start)
        setup = source[start:end]

        self.assertEqual(setup.count("merge_builtin_script_scene_systems("), 1)
        self.assertNotIn(".world_runtime_extension_plan()", setup)

    def test_script_merge_scans_runtime_systems_once_and_reuses_full_override(self):
        source = SCRIPT_SYSTEMS.read_text(encoding="utf-8")
        start = source.index("pub(super) fn merge_builtin_script_scene_systems(")
        end = source.index("\nfn register_builtin_error", start)
        merge = source[start:end]

        self.assertEqual(merge.count("plugin_runtime_systems()"), 1)
        self.assertIn("return linked_registry", merge)
        self.assertEqual(merge.count("world_runtime_extension_plan()"), 2)

    def test_script_merge_interns_the_shared_set_before_registering_missing_phases(self):
        source = SCRIPT_SYSTEMS.read_text(encoding="utf-8")
        start = source.index("pub(super) fn merge_builtin_script_scene_systems(")
        end = source.index("\nfn register_builtin_error", start)
        merge = source[start:end]

        self.assertEqual(merge.count("intern_plugin_module("), 1)
        self.assertEqual(merge.count("intern_system_set("), 1)
        self.assertLess(
            merge.index("intern_system_set("),
            merge.index("for (system, linked_owns_system) in"),
        )


if __name__ == "__main__":
    unittest.main()
