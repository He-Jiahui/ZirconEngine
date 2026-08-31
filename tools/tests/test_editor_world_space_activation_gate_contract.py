from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORLD_SPACE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "pane_component_projection/world_space"
)


class EditorWorldSpaceActivationGateContractTests(unittest.TestCase):
    def test_activation_gates_world_only_field_projection(self) -> None:
        source = (WORLD_SPACE / "mod.rs").read_text(encoding="utf-8")
        function = source.split("pub(super) fn projected_world_space", 1)[1]

        activation = function.index("projected_world_space_enabled")
        disabled_return = function.index("if !enabled")
        transform = function.index("projected_world_transform")
        surface = function.index("projected_world_surface")
        rendering = function.index("projected_world_rendering")

        self.assertLess(activation, disabled_return)
        self.assertLess(disabled_return, transform)
        self.assertLess(disabled_return, surface)
        self.assertLess(disabled_return, rendering)
        self.assertIn("return ProjectedWorldSpace::default()", function)

    def test_disabled_world_defaults_preserve_unit_scale(self) -> None:
        source = (WORLD_SPACE / "model.rs").read_text(encoding="utf-8")
        default_impl = source.split("impl Default for ProjectedWorldSpace", 1)[1]

        self.assertIn("scale_x: 1.0", default_impl)
        self.assertIn("scale_y: 1.0", default_impl)
        self.assertIn("scale_z: 1.0", default_impl)


if __name__ == "__main__":
    unittest.main()
