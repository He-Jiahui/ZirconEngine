import unittest
from pathlib import Path


class RuntimePluginDescriptorProvidedInterfaceProjectionTests(unittest.TestCase):
    def test_descriptor_projects_provided_interfaces_to_package_manifest(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        sources = {
            "descriptor": repo_root
            / "zircon_runtime/src/plugin/runtime_plugin/descriptor.rs",
            "builder": repo_root
            / "zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs",
            "access": repo_root
            / "zircon_runtime/src/plugin/runtime_plugin/descriptor/access.rs",
            "rows": repo_root
            / "zircon_runtime/src/plugin/runtime_plugin/descriptor/package_manifest/rows.rs",
            "physics plugin": repo_root / "zircon_plugins/physics/runtime/src/plugin.rs",
            "descriptor tests": repo_root
            / "zircon_runtime/src/tests/plugin_extensions/runtime_plugin_descriptor.rs",
        }
        text = {
            name: path.read_text(encoding="utf-8")
            for name, path in sources.items()
        }

        expected = {
            "descriptor": [
                "provided_interfaces: Vec<PluginInterfaceManifest>",
                "self.provided_interfaces == other.provided_interfaces",
            ],
            "builder": [
                "pub fn with_provided_interface(",
                "pub fn with_provided_interface_id(",
                ".provided_interfaces",
                "PluginInterfaceManifest::new(interface_id)",
            ],
            "access": [
                "pub fn provided_interfaces(&self) -> &[PluginInterfaceManifest]",
                "&self.provided_interfaces",
            ],
            "rows": [
                "for interface in &descriptor.provided_interfaces",
                "manifest.with_provided_interface(interface.clone())",
            ],
            "physics plugin": [
                ".with_provided_interface_id(PHYSICS_QUERY_INTERFACE_ID)",
                ".export_interface::<dyn PhysicsQueryInterface>(",
            ],
            "descriptor tests": [
                '.with_provided_interface_id("weather.query.v1")',
                "manifest.provides_interfaces[0].id",
            ],
        }

        failures: list[str] = []
        for source_name, phrases in expected.items():
            for phrase in phrases:
                if phrase not in text[source_name]:
                    failures.append(f"{source_name}: missing {phrase}")

        if failures:
            self.fail(
                "Runtime plugin descriptor provided-interface projection drifted:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
