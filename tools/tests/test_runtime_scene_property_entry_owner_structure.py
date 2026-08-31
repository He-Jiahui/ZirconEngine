import unittest
from pathlib import Path


class RuntimeScenePropertyEntryOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_08_15_scene_property_entry_component_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_component_property_entry_families_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/scene/world/property_access/entries.rs"
        )
        owner = owner_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(owner.splitlines()), 280)
        for module in ("animation", "camera", "lighting", "mesh", "physics"):
            self.assertIn(f"mod {module};", owner)

        for moved_anchor in (
            "Camera.fov_y_radians",
            "MeshRenderer.model",
            "AmbientLight.color",
            "AnimationSkeleton.skeleton",
            "fn mesh_renderer_morph_weight_path",
            "fn animation_parameter_is_animatable",
        ):
            self.assertNotIn(moved_anchor, owner)

        for retained_anchor in (
            "pub(super) fn property_entries",
            "fn visit_property_entries",
            "fn property_entry_capacity_hint",
            "Name.value",
            "Hierarchy.parent",
            "Transform.translation",
            "dynamic_scene_value_from_json",
            "dynamic_scene_value_is_projectable",
        ):
            self.assertIn(retained_anchor, owner)

        call_order = (
            "visit_camera_property_entries",
            "visit_mesh_property_entries",
            "visit_lighting_property_entries",
            "visit_physics_property_entries",
            "visit_animation_property_entries",
        )
        positions = [owner.index(anchor) for anchor in call_order]
        self.assertEqual(positions, sorted(positions))

        child_contracts = {
            "camera.rs": (
                100,
                "visit_camera_property_entries",
                "camera_property_entry_capacity_hint",
                "Camera.fov_y_radians",
            ),
            "mesh.rs": (
                180,
                "visit_mesh_property_entries",
                "mesh_property_entry_capacity_hint",
                "MeshRenderer.morph_weight_count",
            ),
            "lighting.rs": (
                220,
                "visit_lighting_property_entries",
                "lighting_property_entry_capacity_hint",
                "SpotLight.outer_angle_radians",
            ),
            "animation.rs": (
                220,
                "visit_animation_property_entries",
                "animation_property_entry_capacity_hint",
                "AnimationStateMachinePlayer.active_state",
            ),
        }
        owner_dir = owner_path.with_suffix("")
        for filename, anchors in child_contracts.items():
            budget, *required = anchors
            child = (owner_dir / filename).read_text(encoding="utf-8")
            self.assertLessEqual(len(child.splitlines()), budget, filename)
            self.assertNotIn("pub fn ", child, filename)
            for anchor in required:
                self.assertIn(anchor, child, filename)

    def test_property_entry_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        structure_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/scene/world/property_access/entries.rs",
            "zircon_runtime/src/scene/world/property_access/entries/camera.rs",
            "zircon_runtime/src/scene/world/property_access/entries/mesh.rs",
            "zircon_runtime/src/scene/world/property_access/entries/lighting.rs",
            "zircon_runtime/src/scene/world/property_access/entries/animation.rs",
            "tools/tests/test_runtime_scene_property_entry_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)


if __name__ == "__main__":
    unittest.main()
