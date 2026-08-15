from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorPluginRegistrationAtomicityContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_consumer_registry_validates_a_candidate_before_replacement(self) -> None:
        registry = self.read(
            "zircon_editor/src/core/runtime_event_consumer/registration.rs"
        )

        self.assertIn("let mut candidate = self.clone();", registry)
        self.assertIn("candidate.register(registration)?;", registry)
        self.assertIn("*self = candidate;", registry)
        self.assertIn(
            "duplicate_late_in_batch_leaves_active_registry_unchanged", registry
        )

    def test_plugin_registration_commits_consumers_after_extensions(self) -> None:
        host = self.read(
            "zircon_editor/src/ui/host/editor_extension_registration.rs"
        )
        controller = self.read(
            "zircon_editor/src/ui/host/editor_host_event_controller.rs"
        )
        start = host.index("    pub fn register_editor_plugin_registration(")
        end = host.index("\n    ///", start)
        registration = host[start:end]
        standalone_start = controller.index("    pub fn register_runtime_event_consumers(")
        standalone_end = controller.index("\n    ///", standalone_start)
        standalone_registration = controller[standalone_start:standalone_end]

        self.assertIn("plugin_registration_gate", registration)
        self.assertIn("plugin_registration_gate", standalone_registration)
        self.assertLess(
            registration.index(".prepare_registration("),
            registration.index(".register_editor_extension_owned("),
        )
        self.assertLess(
            registration.index(".register_editor_extension_owned("),
            registration.index(".install_prepared_registration("),
        )
        self.assertLess(
            standalone_registration.index("plugin_registration_gate"),
            standalone_registration.index(".register(registry)"),
        )

    def test_registration_regressions_are_split_by_semantic_owner(self) -> None:
        root = self.read(
            "zircon_editor/src/tests/editor_event/runtime/extensions_registration.rs"
        )
        module_paths = (
            "zircon_editor/src/tests/editor_event/runtime/extensions_registration/"
            "overlay_lifecycle.rs",
            "zircon_editor/src/tests/editor_event/runtime/extensions_registration/"
            "operation_and_view_registration.rs",
            "zircon_editor/src/tests/editor_event/runtime/extensions_registration/"
            "plugin_contributions.rs",
        )
        sources = [self.read(path) for path in module_paths]
        tests = re.findall(
            r"(?m)^#\[test\]\nfn ([a-zA-Z0-9_]+)\(", "\n".join(sources)
        )

        self.assertEqual(
            "mod operation_and_view_registration;\n"
            "mod overlay_lifecycle;\n"
            "mod plugin_contributions;\n",
            root,
        )
        self.assertEqual(20, len(tests))
        self.assertEqual(len(tests), len(set(tests)))
        self.assertTrue(all(len(source.splitlines()) < 900 for source in sources))
        for test_name in (
            "rejected_plugin_extension_does_not_publish_runtime_event_consumers",
            "editor_runtime_folds_menu_capabilities_into_the_shared_command_descriptor",
            "editor_runtime_consumes_plugin_registration_reports_with_capability_gate",
        ):
            self.assertIn(test_name, tests)


if __name__ == "__main__":
    unittest.main()
