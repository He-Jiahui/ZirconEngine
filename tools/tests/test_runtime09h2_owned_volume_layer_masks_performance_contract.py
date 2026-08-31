from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORLD_RS = ROOT / "zircon_runtime/src/scene/world/render_post_process.rs"


def source() -> str:
    return WORLD_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def collect_body() -> str:
    text = source()
    return text.split("pub(super) fn collect_post_process_volumes(", 1)[1].split(
        "fn post_process_volume_extract(", 1
    )[0]


def fog_helper_body() -> str:
    text = source()
    return text.split("fn fog_volume_from_extract(", 1)[1].split(
        "fn render_layers_for_view(", 1
    )[0]


class Runtime09H2OwnedVolumeLayerMasksContract(unittest.TestCase):
    def test_initial_layer_mask_is_moved_into_the_extract(self) -> None:
        body = compact(collect_body())

        self.assertIn(
            "self.post_process_volume_extract(entity,volume,volume_mask)", body
        )
        self.assertNotIn(
            "self.post_process_volume_extract(entity,volume,volume_mask.clone())",
            body,
        )

    def test_only_dual_output_volumes_clone_the_extract_mask(self) -> None:
        body = compact(collect_body())

        self.assertIn(
            "letfog_layer_mask=ifaffects_post_process{extract.volume_mask.clone()}",
            body,
        )
        self.assertIn(
            "else{std::mem::replace(&mutextract.volume_mask,RenderLayerSet::none(),)}",
            body,
        )

    def test_fog_helper_consumes_the_selected_layer_mask(self) -> None:
        body = compact(fog_helper_body())

        self.assertIn("layer_mask:RenderLayerSet", body)
        self.assertIn("layer_mask,", body)
        self.assertNotIn("extract.volume_mask.clone()", body)

    def test_supplied_mask_behavior_has_a_direct_rust_contract(self) -> None:
        self.assertIn(
            "fog_volume_from_extract_uses_supplied_layer_mask", source()
        )


if __name__ == "__main__":
    unittest.main()
