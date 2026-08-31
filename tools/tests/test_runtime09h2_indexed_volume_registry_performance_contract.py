from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
REGISTRY_RS = ROOT / (
    "zircon_runtime/src/core/framework/render/post_process/volume_registry.rs"
)


def source() -> str:
    return REGISTRY_RS.read_text(encoding="utf-8")


def register_body() -> str:
    text = source()
    return text.split("pub fn register(", 1)[1].split("pub fn len(", 1)[0]


def get_body() -> str:
    text = source()
    return text.split("pub fn get(", 1)[1].split(
        "pub fn default_resolved_post_process_settings", 1
    )[0]


class Runtime09H2IndexedVolumeRegistryContract(unittest.TestCase):
    def test_registry_retains_ordered_descriptors_and_a_static_id_index(self) -> None:
        text = source()

        self.assertIn("descriptors: Vec<VolumeComponentDescriptor>", text)
        self.assertIn("descriptor_indices: HashMap<&'static str, usize>", text)

    def test_register_uses_the_side_index_for_duplicate_detection(self) -> None:
        body = register_body()
        compact = re.sub(r"\s+", "", body)

        self.assertNotIn(".descriptors\n            .iter()", body)
        self.assertIn(
            "self.descriptor_indices.contains_key(descriptor.component_id)", compact
        )
        self.assertIn("letdescriptor_index=self.descriptors.len()", compact)
        self.assertIn(
            "self.descriptor_indices.insert(descriptor.component_id,descriptor_index)",
            compact,
        )

    def test_get_projects_the_index_without_a_linear_find(self) -> None:
        body = get_body()
        compact = re.sub(r"\s+", "", body)

        self.assertNotIn(".iter()", body)
        self.assertNotIn(".find(", body)
        self.assertIn("self.descriptor_indices.get(component_id)", compact)
        self.assertIn("self.descriptors.get(*descriptor_index)", compact)

    def test_ordered_iteration_has_a_direct_rust_contract(self) -> None:
        text = source()

        self.assertIn("self.descriptors.iter()", text)
        self.assertIn(
            "render_volume_registry_index_preserves_registration_order", text
        )


if __name__ == "__main__":
    unittest.main()
