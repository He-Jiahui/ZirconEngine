from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
REGISTRATION = ROOT / "zircon_plugins" / "plugin_sdk" / "src" / "registration.rs"


class PluginSdkRuntimeSystemFactoryPerformanceContractTests(unittest.TestCase):
    def test_sdk_builder_preserves_the_concrete_factory_until_runtime_registration(self) -> None:
        source = REGISTRATION.read_text(encoding="utf-8")

        self.assertIn("RuntimePluginRuntimeSceneSystemBuilder<'registry, F>", source)
        self.assertIn("system_factory: F", source)
        self.assertIn(
            "register_runtime_scene_system(self.owner, self.id, self.stage, system_factory)",
            source,
        )
        self.assertNotIn("system_factory: Arc<dyn Fn() -> S + Send + Sync>", source)


if __name__ == "__main__":
    unittest.main()
