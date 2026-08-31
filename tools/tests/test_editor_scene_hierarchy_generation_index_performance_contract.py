from pathlib import Path
import unittest

from tools.editor_scene_hierarchy_generation_index_pressure import run


ROOT = Path(__file__).resolve().parents[2]
PROJECTION = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/scene_hierarchy_projection.rs"
)
FRAGMENT = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/scene_hierarchy_fragment.rs"
)
ROW_PATCH = ROOT / (
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/"
    "scene_hierarchy_refresh/hierarchy_row_patch.rs"
)
PROFILE_MANIFEST = ROOT / "tools/profile-capture-manifest.ps1"


class EditorSceneHierarchyGenerationIndexPerformanceContractTests(unittest.TestCase):
    def test_projection_uses_unordered_identity_lookup_without_owned_names(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        row_state = source.split("struct SceneHierarchyRowState", 1)[1].split(
            "impl SceneHierarchyRowState", 1
        )[0]
        replace = source.split("pub(super) fn replace(", 1)[1].split(
            "pub(super) const fn generation", 1
        )[0]

        self.assertIn(
            "rows_by_entity: HashMap<EntityId, SceneHierarchyRowState>", source
        )
        self.assertNotIn("display_name: String", row_state)
        self.assertNotIn("row.display_name.clone()", replace)

    def test_only_changed_rows_carry_replacement_content(self) -> None:
        projection = PROJECTION.read_text(encoding="utf-8")
        fragment = FRAGMENT.read_text(encoding="utf-8")

        self.assertIn("replacement: Option<SceneHierarchyLogicalRowContent>", projection)
        self.assertIn("logical_selection_patch", projection)
        self.assertIn("logical_content_patch", projection)
        selection_patch = projection.split("fn logical_selection_patch", 1)[1].split(
            "fn logical_content_patch", 1
        )[0]
        self.assertNotIn("display_name", selection_patch)
        self.assertIn("logical_content_patch(row)", fragment)

    def test_selection_only_publication_reuses_presented_row_content(self) -> None:
        source = ROW_PATCH.read_text(encoding="utf-8")
        patch = source.split("fn patch_presented_hierarchy_rows", 1)[1].split(
            "fn first_presented_hierarchy_rows", 1
        )[0]

        self.assertIn("rows.get(*row_index).cloned()", patch)
        self.assertRegex(patch, r"patch\s*\.replacement\s*\.clone\(\)")
        self.assertIn("next.selected = patch.selected", patch)

    def test_pressure_model_removes_ordered_work_and_duplicate_name_payload(self) -> None:
        result = run(
            row_count=100_000,
            average_display_name_utf8_bytes=24,
            selection_only_patch_count=65_536,
        )

        self.assertEqual(
            result["retired_ordered_duplicate_projection"][
                "modeled_full_reflow_index_work_units"
            ],
            1_700_000,
        )
        self.assertEqual(
            result["generation_identity_index"][
                "modeled_full_reflow_index_work_units"
            ],
            100_000,
        )
        self.assertEqual(result["delta"]["modeled_index_work_ratio"], 17.0)
        self.assertEqual(
            result["delta"]["avoided_projection_display_name_utf8_payload_bytes"],
            2_400_000,
        )
        self.assertEqual(
            result["delta"]["avoided_selection_only_name_clone_count"], 65_536
        )

    def test_profile_manifest_binds_generation_index_sources(self) -> None:
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")
        for path in (
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/scene_hierarchy_fragment.rs",
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/scene_hierarchy_projection.rs",
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/"
            "scene_hierarchy_refresh.rs",
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/"
            "scene_hierarchy_refresh/hierarchy_row_patch.rs",
            "zircon_editor/src/ui/workbench/snapshot/data/scene_entry/entries.rs",
        ):
            self.assertIn(f'"{path}"', manifest)


if __name__ == "__main__":
    unittest.main()
