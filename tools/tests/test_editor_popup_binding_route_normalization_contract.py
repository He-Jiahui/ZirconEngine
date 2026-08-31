from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMPONENT_PROJECTION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection"
)


class EditorPopupBindingRouteNormalizationContractTests(unittest.TestCase):
    def test_binding_path_normalization_uses_one_output_buffer(self) -> None:
        source = (COMPONENT_PROJECTION / "binding_actions/path.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("String::with_capacity(binding_id.len())", source)
        self.assertIn("push_binding_path_action_id", source)
        self.assertNotIn("collect::<Vec<_>>()", source)
        self.assertNotIn('.join(".")', source)
        self.assertNotIn("trim_matches", source)

    def test_showcase_routes_share_the_binding_path_normalizer(self) -> None:
        source = (COMPONENT_PROJECTION / "showcase_actions/binding_ids.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("binding_path_action_id", source)
        self.assertIn("push_binding_path_action_id", source)
        self.assertNotIn("fn camel_to_snake", source)
        self.assertNotIn("fn binding_path_action_id", source)
        self.assertNotIn("format!(", source)


if __name__ == "__main__":
    unittest.main()
