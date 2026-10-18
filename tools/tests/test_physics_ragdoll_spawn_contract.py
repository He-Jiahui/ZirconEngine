from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PROFILE_RS = ROOT / "zircon_plugins/physics/runtime/src/skeletal/profile.rs"


class PhysicsRagdollSpawnContractTests(unittest.TestCase):
    def test_spawn_node_scene_error_is_propagated_before_configuration(self) -> None:
        source = PROFILE_RS.read_text(encoding="utf-8")

        self.assertIn(
            "let body = world\n                .spawn_node(NodeKind::Empty)\n                .map_err(|error| RagdollProfileError::SceneMutation(error.to_string()))?;",
            source,
        )
        self.assertNotIn(
            "let body = world.spawn_node(NodeKind::Empty);",
            source,
        )


if __name__ == "__main__":
    unittest.main()
