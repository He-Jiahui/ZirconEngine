from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_SYSTEMS_PATH = (
    REPO_ROOT / "zircon_runtime/src/dynamic_api/session/script_systems.rs"
)
SCRIPT_SCENE_SYSTEM_PATH = REPO_ROOT / "zircon_runtime/src/script/vm/scene_system.rs"


class Plugins01ScriptSystemRegistryIdentityTests(unittest.TestCase):
    def test_script_domain_does_not_own_plugin_registry_registration(self) -> None:
        source = SCRIPT_SCENE_SYSTEM_PATH.read_text(encoding="utf-8")

        self.assertNotIn("crate::plugin", source)
        self.assertNotIn("RuntimeExtensionRegistry", source)
        self.assertNotIn("PluginModuleId", source)

    def test_builtin_phases_reuse_the_linked_registry_interner(self) -> None:
        source = SCRIPT_SYSTEMS_PATH.read_text(encoding="utf-8")

        self.assertIn("let mut merged = linked_registry.clone();", source)
        self.assertRegex(source, r"merged\s*\.\s*intern_plugin_module")
        self.assertIn("ScriptSceneRuntimeSystem::fixed_update()", source)
        self.assertRegex(source, r"merged\s*\.\s*register_runtime_scene_system")
        self.assertRegex(source, r"merged\s*\.\s*world_runtime_extension_plan\(\)")
        self.assertNotIn("let mut builtins = RuntimeExtensionRegistry::default();", source)
        self.assertNotIn("linked_plan.try_merge", source)
        self.assertNotIn("register_script_scene_runtime_system(&mut merged", source)


if __name__ == "__main__":
    unittest.main()
