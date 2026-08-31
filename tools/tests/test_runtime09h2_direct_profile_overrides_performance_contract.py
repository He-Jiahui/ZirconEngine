from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXTRACT_RS = ROOT / (
    "zircon_runtime/src/core/framework/render/post_process/volume_extract.rs"
)


def source() -> str:
    return EXTRACT_RS.read_text(encoding="utf-8")


def from_profile_body() -> str:
    text = source()
    return text.split("pub fn from_profile", 1)[1].split(
        "pub struct PostProcessVolumeExtract", 1
    )[0]


def push_effect_stack_body() -> str:
    text = source()
    return text.split("fn push_effect_stack_overrides", 1)[1].split(
        "struct EffectStackOverride", 1
    )[0]


def profile_override_count_body() -> str:
    text = source()
    return text.split("fn profile_override_count", 1)[1].split(
        "fn push_effect_stack_overrides", 1
    )[0]


def effect_stack_builder_body() -> str:
    text = source()
    return text.split("struct EffectStackOverride", 1)[1].split(
        "fn tonemap_operator_id", 1
    )[0]


class Runtime09H2DirectProfileOverridesContract(unittest.TestCase):
    def test_profile_projection_reserves_its_derived_override_count(self) -> None:
        body = re.sub(r"\s+", "", from_profile_body())

        self.assertIn("letoverride_count=profile_override_count(profile)", body)
        self.assertIn("Vec::with_capacity(override_count)", body)

    def test_effect_stack_override_count_matches_all_direct_builders(self) -> None:
        text = source()
        push_body = push_effect_stack_body()

        self.assertIn("const EFFECT_STACK_PROFILE_OVERRIDE_COUNT: usize = 11", text)
        self.assertEqual(push_body.count("overrides.push("), 11)
        self.assertIn(
            "EFFECT_STACK_PROFILE_OVERRIDE_COUNT", profile_override_count_body()
        )

    def test_effect_stack_builders_construct_final_overrides_without_staging(self) -> None:
        body = effect_stack_builder_body()

        self.assertIn("struct EffectStackOverride;", source())
        self.assertNotIn("values: Vec<VolumeParamValue>", body)
        self.assertNotIn("into_override", body)
        self.assertEqual(body.count("VolumeComponentOverride::from_values("), 11)

    def test_profile_override_order_has_a_direct_rust_contract(self) -> None:
        self.assertIn(
            "render_volume_extract_profile_override_order_is_stable", source()
        )


if __name__ == "__main__":
    unittest.main()
