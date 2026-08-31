from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/gameplay_host/combat.rs"


class InPlaceAnimationBoolParameterPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        helper_parts = cls.source.split("fn set_animation_bool_parameter", maxsplit=1)
        cls.helper = (
            helper_parts[1].split("pub(super) fn set_animation_bool", maxsplit=1)[0]
            if len(helper_parts) == 2
            else ""
        )
        cls.host_call = cls.source.split(
            "pub(super) fn set_animation_bool", maxsplit=1
        )[1].split("pub(super) fn damage_entity", maxsplit=1)[0]

    def test_existing_animation_parameter_is_replaced_in_place(self) -> None:
        self.assertIn("parameters.get_mut(parameter)", self.helper)
        self.assertIn("*current = next;", self.helper)
        self.assertIn("return false;", self.helper)

    def test_parameter_key_is_copied_only_for_first_insert(self) -> None:
        self.assertIn("parameters.insert(parameter.to_owned(), next);", self.helper)
        self.assertIn(
            "if set_animation_bool_parameter(&mut player.parameters, parameter, value)",
            self.host_call,
        )
        self.assertIn(
            "ScriptHostHotPathMetrics::record_guest_string_copy(parameter.len());",
            self.host_call,
        )
        self.assertNotIn(".insert(parameter.to_owned()", self.host_call)

    def test_rust_guard_covers_existing_and_missing_parameters(self) -> None:
        self.assertIn(
            "animation_bool_parameter_only_copies_a_missing_key",
            self.source,
        )
        self.assertIn('parameters.get("moving")', self.source)
        self.assertIn('parameters.get("grounded")', self.source)


if __name__ == "__main__":
    unittest.main()
