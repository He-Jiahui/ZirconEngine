from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCENE = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs"
)
APPLY = ROOT / "zircon_editor/src/ui/retained_host/ui/apply_presentation.rs"


class EditorHostSceneProjectionPerformanceContractTests(unittest.TestCase):
    def test_full_apply_projects_floating_windows_once(self) -> None:
        scene = SCENE.read_text(encoding="utf-8")

        self.assertEqual(scene.count("floating_windows_with_pane_shell_layouts("), 2)

    def test_native_surface_reuses_main_scene_floating_projection(self) -> None:
        scene = "".join(SCENE.read_text(encoding="utf-8").split())
        apply = "".join(APPLY.read_text(encoding="utf-8").split())

        self.assertIn(
            "host_scene_data.floating_layer.floating_windows.clone()", scene
        )
        self.assertIn(
            "build_native_floating_surface_data(&host_scene_data,"
            "&presentation.host_shell)",
            apply,
        )


if __name__ == "__main__":
    unittest.main()
