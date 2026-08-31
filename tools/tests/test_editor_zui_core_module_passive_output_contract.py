import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE_MODULE_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core"
)
RUNTIME_SOURCES = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs",
    REPO_ROOT
    / (
        "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
        "workbench/module_navigation.rs"
    ),
    REPO_ROOT
    / (
        "zircon_editor/src/ui/template_runtime/builtin/"
        "workbench_module_template_bindings.rs"
    ),
    REPO_ROOT
    / (
        "zircon_editor/src/ui/template_runtime/builtin/"
        "workbench_module_template_bindings/assets_workspace_routes.rs"
    ),
)
PASSIVE_OUTPUTS = (
    (
        "ai/workbench_behavior_workspace.zui",
        "behavior_output",
        "workbench.module.behavior.output.select",
    ),
    (
        "ai/workbench_perception_workspace.zui",
        "perception_event_row",
        "workbench.module.perception.event.select",
    ),
    (
        "assets/workbench_assets_workspace.zui",
        "assets_output",
        "workbench.module.assets.output.select",
    ),
    (
        "gameplay/workbench_ability_workspace.zui",
        "ability_output",
        "workbench.module.ability.output.select",
    ),
    (
        "gameplay/workbench_effect_workspace.zui",
        "effect_output",
        "workbench.module.effect.output.select",
    ),
    (
        "gameplay/workbench_tags_workspace.zui",
        "tags_validation_row",
        "workbench.module.tags.validation_select.select",
    ),
    (
        "rendering/workbench_material_workspace.zui",
        "material_output",
        "workbench.module.material.output.select",
    ),
    (
        "rendering/workbench_render_workspace.zui",
        "render_capture_row",
        "workbench.module.render.capture.select",
    ),
    (
        "rendering/workbench_vfx_workspace.zui",
        "vfx_output",
        "workbench.module.vfx.output.select",
    ),
    (
        "ui/workbench_hud_workspace.zui",
        "hud_validation_row",
        "workbench.module.hud.validation_row.select",
    ),
)


class EditorZuiCoreModulePassiveOutputContractTests(unittest.TestCase):
    def test_output_and_diagnostic_rows_are_passive_readouts(self):
        for relative_path, node_name, _ in PASSIVE_OUTPUTS:
            with self.subTest(module=relative_path, node=node_name):
                with (CORE_MODULE_ROOT / relative_path).open("rb") as source:
                    node = tomllib.load(source)["nodes"][node_name]
                self.assertNotIn("events", node)
                props = node["props"]
                for property_name in (
                    "input_interactive",
                    "input_clickable",
                    "input_hoverable",
                    "input_focusable",
                ):
                    self.assertIs(False, props[property_name])

    def test_retired_output_selection_actions_have_no_runtime_registration(self):
        runtime_source = "\n".join(
            path.read_text(encoding="utf-8") for path in RUNTIME_SOURCES
        )
        for _, _, action_id in PASSIVE_OUTPUTS:
            with self.subTest(action=action_id):
                self.assertNotIn(action_id, runtime_source)


if __name__ == "__main__":
    unittest.main()
