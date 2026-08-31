from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMPONENT_RS = ROOT / (
    "zircon_runtime/src/core/framework/render/post_process/volume_component.rs"
)


def source() -> str:
    return COMPONENT_RS.read_text(encoding="utf-8")


def apply_defaults_body() -> str:
    text = source()
    return text.split("pub fn apply_defaults", 1)[1].split(
        "pub fn apply_values", 1
    )[0]


class Runtime09H2InlineVolumeDefaultsContract(unittest.TestCase):
    def test_inline_capacity_covers_the_largest_builtin_descriptor(self) -> None:
        self.assertIn(
            "const BUILTIN_VOLUME_PARAM_INLINE_CAPACITY: usize = 9", source()
        )

    def test_builtin_defaults_use_a_fixed_stack_buffer(self) -> None:
        body = re.sub(r"\s+", "", apply_defaults_body())

        self.assertIn(
            "ifself.params.len()<=BUILTIN_VOLUME_PARAM_INLINE_CAPACITY", body
        )
        self.assertIn(
            "[VolumeParamValue::Float(0.0);BUILTIN_VOLUME_PARAM_INLINE_CAPACITY]",
            body,
        )
        self.assertIn(".zip(self.params)", body)
        self.assertIn("&values[..self.params.len()]", body)

    def test_long_plugin_descriptors_retain_the_owned_fallback(self) -> None:
        body = apply_defaults_body()

        self.assertIn("} else {", body)
        inline_body, fallback_body = body.split("} else {", 1)
        self.assertNotIn("default_values", inline_body)
        self.assertIn("let values = self.default_values();", fallback_body)
        self.assertIn("self.apply_values(settings, &values)", fallback_body)

    def test_builtin_bound_and_long_plugin_fallback_have_rust_contracts(self) -> None:
        text = source()

        self.assertIn(
            "render_volume_component_builtin_defaults_fit_inline_capacity", text
        )
        self.assertIn(
            "render_volume_component_long_plugin_defaults_use_complete_fallback", text
        )


if __name__ == "__main__":
    unittest.main()
