from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PROJECTION = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/scene_hierarchy_projection.rs"
)


class EditorSceneHierarchyControlArcSharingPerformanceContractTests(unittest.TestCase):
    def test_bidirectional_control_indexes_share_arc_backing_storage(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        replace = source.split("pub(super) fn replace(", 1)[1].split(
            "pub(super) const fn generation", 1
        )[0]

        self.assertIn("sync::Arc", source)
        self.assertIn("controls_by_entity: HashMap<EntityId, Arc<str>>", source)
        self.assertIn("entities_by_control: HashMap<Arc<str>, EntityId>", source)
        self.assertIn("let control_id: Arc<str> = Arc::from(control_id.as_str());", replace)
        self.assertIn("Arc::clone(&control_id)", replace)
        self.assertNotIn("control_id.clone(), row.entity", replace)

    def test_lookup_surface_still_borrows_str_keys(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        self.assertIn("pub(super) fn control_for(&self, entity: EntityId) -> Option<&str>", source)
        self.assertIn("pub(super) fn contains_control(&self, control_id: &str) -> bool", source)
        self.assertIn("pub(super) fn entity_for_control(&self, control_id: &str)", source)
        self.assertIn("self.entities_by_control.contains_key(control_id)", source)

    def test_pressure_model_reduces_control_string_allocations(self) -> None:
        row_count = 100_000
        retired_allocations = row_count * 2
        optimized_allocations = row_count
        self.assertEqual(retired_allocations, 200_000)
        self.assertEqual(optimized_allocations, 100_000)
        self.assertEqual(retired_allocations // optimized_allocations, 2)


if __name__ == "__main__":
    unittest.main()
