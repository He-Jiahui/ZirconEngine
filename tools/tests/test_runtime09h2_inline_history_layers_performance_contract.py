from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
KEY_RS = ROOT / (
    "zircon_runtime/src/graphics/runtime/render_framework/viewport_record/"
    "camera_history_key.rs"
)


def source() -> str:
    return KEY_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def conversion_body() -> str:
    text = source()
    return text.split("impl From<&RenderLayerSet> for ViewportCameraHistoryLayerKey", 1)[
        1
    ].split("struct ViewportCameraHistoryRectKey", 1)[0]


class Runtime09H2InlineHistoryLayersContract(unittest.TestCase):
    def test_layer_key_has_four_inline_slots_and_a_shared_fallback(self) -> None:
        text = compact(source())

        self.assertIn("constINLINE_HISTORY_LAYER_CAPACITY:usize=4", text)
        self.assertIn("enumViewportCameraHistoryLayerKey", text)
        self.assertIn("Inline{layers:[RenderLayer;INLINE_HISTORY_LAYER_CAPACITY],len:u8", text)
        self.assertIn("Shared(Arc<[RenderLayer]>)", text)

    def test_common_layer_sets_fill_the_inline_array_without_collecting(self) -> None:
        body = compact(conversion_body())
        inline_body = body.split("iflen==INLINE_HISTORY_LAYER_CAPACITY", 1)[0]

        self.assertIn("letmutlayers=[0;INLINE_HISTORY_LAYER_CAPACITY]", inline_body)
        self.assertIn("forlayerinvalue.iter()", inline_body)
        self.assertNotIn("collect", inline_body)

    def test_only_overflowing_layer_sets_use_shared_storage(self) -> None:
        body = compact(conversion_body())

        self.assertIn("iflen==INLINE_HISTORY_LAYER_CAPACITY", body)
        self.assertIn("returnSelf::Shared(value.iter().collect())", body)

    def test_inline_and_shared_paths_have_direct_rust_contracts(self) -> None:
        text = source()

        self.assertIn("camera_history_key_common_layers_are_inline", text)
        self.assertIn("camera_history_key_wide_clones_share_layer_storage", text)


if __name__ == "__main__":
    unittest.main()
