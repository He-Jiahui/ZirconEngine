"""Static hard-cut contract for Editor06's inspector customization migration."""

from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def read(relative_path: str) -> str:
    return (ROOT / relative_path).read_text(encoding="utf-8")


class Editor06InspectorCustomizationContractTests(unittest.TestCase):
    def test_old_component_drawer_registry_family_is_absent(self) -> None:
        registry = read("zircon_editor/src/core/editor_extension.rs")
        descriptors = read(
            "zircon_editor/src/core/editor_extension/contribution_descriptors.rs"
        )
        store = "\n".join(
            (
                read("zircon_editor/src/core/extension/store/model.rs"),
                read("zircon_editor/src/core/extension/store/model/snapshot.rs"),
            )
        )

        for source in (registry, descriptors, store):
            self.assertNotIn("ComponentDrawerDescriptor", source)
            self.assertNotIn("component_drawers", source)
            self.assertNotIn("register_component_drawer", source)

    def test_new_customization_store_and_snapshot_path_are_canonical(self) -> None:
        registry = read("zircon_editor/src/core/editor_extension.rs")
        store = "\n".join(
            (
                read("zircon_editor/src/core/extension/store/model.rs"),
                read("zircon_editor/src/core/extension/store/model/snapshot.rs"),
            )
        )
        snapshot = read(
            "zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs"
        )
        inspector_snapshot = read(
            "zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs"
        )
        inspector = read("zircon_editor/src/core/extension/inspector.rs")
        runtime = read("zircon_editor/src/ui/host/editor_event_runtime_reflection.rs")
        adapter = read(
            "zircon_editor/src/ui/template_runtime/component_adapter/component_drawer.rs"
        )

        self.assertIn("InspectorCustomizationDescriptor", registry)
        self.assertIn("register_inspector_customization", registry)
        self.assertIn("snapshot_with_inspector_customizations", snapshot)
        self.assertNotIn("snapshot_with_component_drawers", snapshot)
        self.assertIn("active_inspector_customizations_for_shell", runtime)
        self.assertIn("field_editors", store)
        self.assertIn("active_field_editors_for_shell", runtime)
        self.assertIn("field_editor: FieldEditorInstance", inspector_snapshot)
        self.assertNotIn("global_field_editors", inspector_snapshot)
        self.assertNotIn("RwLock<FieldEditorContainer>", inspector_snapshot)
        self.assertNotIn("global_field_editors", inspector)
        self.assertNotIn("RwLock", inspector)
        self.assertIn("InspectorCustomization", adapter)
        self.assertNotIn("ComponentDrawerDescriptor", adapter)

    def test_snapshot_projection_does_not_retain_drawer_fields(self) -> None:
        projection_sources = (
            read("zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs"),
            read(
                "zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs"
            ),
            read(
                "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs"
            ),
            read(
                "zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/inspector.rs"
            ),
        )
        old_fields = (
            "drawer_available",
            "drawer_ui_document",
            "drawer_controller",
            "drawer_template_id",
            "drawer_data_root",
            "drawer_bindings",
        )

        for source in projection_sources:
            for field in old_fields:
                self.assertNotIn(field, source)

    def test_contribution_snapshot_is_a_separate_read_model_leaf(self) -> None:
        model = read("zircon_editor/src/core/extension/store/model.rs")
        snapshot = read("zircon_editor/src/core/extension/store/model/snapshot.rs")

        self.assertIn("mod snapshot;", model)
        self.assertIn("pub use snapshot::ContributionSnapshot;", model)
        self.assertNotIn("pub struct ContributionSnapshot", model)
        self.assertIn("pub struct ContributionSnapshot", snapshot)
        self.assertIn("pub(super) type IndexedMap", snapshot)

    def test_field_editor_catalog_is_a_separate_inspector_leaf(self) -> None:
        inspector = read("zircon_editor/src/core/extension/inspector.rs")
        field_editor = read(
            "zircon_editor/src/core/extension/inspector/field_editor.rs"
        )

        self.assertIn("mod field_editor;", inspector)
        self.assertIn("pub use field_editor::", inspector)
        self.assertNotIn("pub struct FieldEditorContainer", inspector)
        self.assertNotIn("fn normalize_field_type_name", inspector)
        self.assertIn("pub struct FieldEditorContainer", field_editor)
        self.assertIn("fn normalize_field_type_name", field_editor)
        self.assertNotIn(".expect(", field_editor)

    def test_template_replacement_checks_its_ticket_before_publish(self) -> None:
        model = read("zircon_editor/src/core/extension/store/model.rs")
        typed_ticket_check = "let Some(record) = self.tickets.get_mut(&ticket) else"
        publish = "self.generation = self.generation.saturating_add(1);"
        replacement = model[model.index("pub(crate) fn replace_ui_template_contributions") :]

        self.assertIn(typed_ticket_check, replacement)
        self.assertNotIn("validated contribution ticket disappeared", model)
        self.assertLess(replacement.index(typed_ticket_check), replacement.index(publish))

    def test_module_contract_docs_publish_only_the_customization_api(self) -> None:
        module_docs = (
            read("docs/editor-and-tooling/authoring-plugin-extension-contracts.md"),
            read("docs/editor-and-tooling/editor-command-workflow.md"),
            read("docs/editor-and-tooling/editor-host-minimal-plugin-loading.md"),
            read("docs/editor-and-tooling/ui-binding-reflection-architecture.md"),
            read("docs/zircon_editor/core/editing/command.md"),
            read("docs/zircon_runtime/ui/v2.md"),
            read("docs/zircon_plugins/plugin-sdk-examples-editor.md"),
        )
        retired_api = (
            "ComponentDrawerDescriptor",
            "register_component_drawer",
            "snapshot_with_component_drawers",
            "drawer_available",
        )

        for document in module_docs:
            for token in retired_api:
                self.assertNotIn(token, document)


if __name__ == "__main__":
    unittest.main()
