from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORLD_SYNC_RS = ROOT / "zircon_plugins/physics/runtime/src/manager/world_sync.rs"


def source() -> str:
    return WORLD_SYNC_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def build_body() -> str:
    text = source()
    return text.split("pub fn build_world_sync_state", 1)[1].split(
        "pub(super) fn collider_shape_to_physics", 1
    )[0]


class Plugins12PresizedWorldSyncContract(unittest.TestCase):
    def test_projection_captures_the_owned_node_snapshot_once(self) -> None:
        body = compact(build_body())

        self.assertIn("letnodes=world.node_records();", body)
        self.assertIn("fornodeinnodes{", body)

    def test_projection_presizes_each_output_from_component_counts(self) -> None:
        body = compact(build_body())

        self.assertIn("world_sync_projection_capacities(&nodes)", body)
        self.assertIn("bodies:Vec::with_capacity(body_capacity)", body)
        self.assertIn("colliders:Vec::with_capacity(collider_capacity)", body)
        self.assertIn("joints:Vec::with_capacity(joint_capacity)", body)
        self.assertIn("materials:Vec::with_capacity(material_capacity)", body)

    def test_collider_projection_consumes_the_owned_shape(self) -> None:
        body = compact(build_body())

        self.assertIn("ifletSome(collider)=node.collider{", body)
        self.assertIn("shape:collider_shape_into_physics(collider.shape)", body)

    def test_joint_projection_moves_owned_metadata(self) -> None:
        body = compact(build_body())

        self.assertIn("ifletSome(joint)=node.joint{", body)
        self.assertIn("constraint:joint.constraint", body)
        self.assertIn("skeleton_binding:joint.skeleton_binding", body)
        self.assertNotIn("constraint:joint.constraint.clone()", body)
        self.assertNotIn("skeleton_binding:joint.skeleton_binding.clone()", body)

    def test_owned_projection_has_a_direct_rust_contract(self) -> None:
        text = source()

        self.assertIn("owned_projection_preserves_nested_payloads", text)
        self.assertIn("collider_shape_into_physics", text)


if __name__ == "__main__":
    unittest.main()
