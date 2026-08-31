import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_ROOTS = (
    ROOT / "zircon_editor/assets/ui/editor/components/workbench/modules/core",
    ROOT / "zircon_editor/assets/ui/editor/components/workbench/modules/extensions",
)
TOKENS = ROOT / "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
HUD_WORKSPACE = MODULE_ROOTS[0] / "ui/workbench_hud_workspace.zui"
SCAN_ONLY_UI_ASSET_EDITOR = (
    MODULE_ROOTS[1] / "ui/workbench_extension_ui_asset_editor_workspace.zui"
)
TIER_RANK = {"ultra": 0, "narrow": 1, "regular": 2, "wide": 3}


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def density_value(token: str, tokens: dict) -> float:
    prefix = "$editor.density.gap."
    if not token.startswith(prefix):
        raise AssertionError(f"unsupported header gap token: {token}")
    key = f"gap_{token.removeprefix(prefix)}"
    return float(tokens["density"][key])


def dimension_value(value, tokens: dict) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    if not isinstance(value, str) or not value.startswith("$editor."):
        raise AssertionError(f"unsupported layout dimension: {value}")
    if value.startswith("$editor.density.gap."):
        return density_value(value, tokens)

    token_paths = {
        "$editor.control.height.large": ("controls", "large_height"),
        "$editor.control.height.default": ("controls", "default_height"),
        "$editor.control.height.compact": ("controls", "compact_height"),
        "$editor.control.height.dense": ("controls", "dense_height"),
        "$editor.chrome.activity_rail.width": ("chrome", "activity_rail_width"),
        "$editor.chrome.separator.thickness": ("chrome", "separator_thickness"),
        "$editor.chrome.panel_header.height": ("chrome", "panel_header_height"),
    }
    if value not in token_paths:
        raise AssertionError(f"unsupported layout token: {value}")
    group, key = token_paths[value]
    return float(tokens[group][key])


def minimum_width(node: dict, tokens: dict) -> float:
    value = node.get("layout", {}).get("width", {}).get("min", 0.0)
    return dimension_value(value, tokens)


def visible_at_ultra(node: dict) -> bool:
    minimum_tier = node.get("props", {}).get("responsive_min_tier")
    return minimum_tier is None or TIER_RANK[minimum_tier] <= TIER_RANK["ultra"]


class EditorZuiModuleHeaderBudgetContract(unittest.TestCase):
    def test_center_headers_fit_the_ultra_workspace_budget(self) -> None:
        tokens = load_document(TOKENS)
        ultra_width = float(tokens["density"]["ultra_minimum_window_width"])
        headers = []
        narrow_secondary_actions = []

        for module_root in MODULE_ROOTS:
            for path in sorted(module_root.rglob("*_workspace.zui")):
                if path == SCAN_ONLY_UI_ASSET_EDITOR:
                    continue
                document = load_document(path)
                nodes = document.get("nodes", {})
                for node_id, header in nodes.items():
                    if not node_id.endswith("_center_header"):
                        continue

                    children = [nodes[entry["node"]] for entry in header["children"]]
                    visible = [child for child in children if visible_at_ultra(child)]
                    gap = header.get("layout", {}).get("container", {}).get("gap", 0.0)
                    if isinstance(gap, str):
                        gap = density_value(gap, tokens)
                    visible_minimum = sum(
                        minimum_width(child, tokens) for child in visible
                    )
                    visible_minimum += max(0, len(visible) - 1) * float(gap)
                    location = f"{path.relative_to(ROOT).as_posix()}:{node_id}"
                    headers.append(location)
                    self.assertLessEqual(
                        visible_minimum,
                        ultra_width,
                        f"{location} needs {visible_minimum}px at Ultra",
                    )

                    for child in children:
                        tier = child.get("props", {}).get("responsive_min_tier")
                        if child.get("component") != "WorkbenchButton" or tier is None:
                            continue
                        self.assertIn(tier, {"narrow", "regular", "wide"}, location)
                        self.assertEqual(
                            "outlined",
                            child.get("props", {}).get("button_variant"),
                            f"{location} must keep the filled primary action at Ultra",
                        )
                        if tier == "narrow":
                            narrow_secondary_actions.append(location)

        self.assertEqual(52, len(headers), "all writable module headers must be covered")
        self.assertEqual(
            28,
            len(narrow_secondary_actions),
            "only over-budget dual-action headers need responsive secondary actions",
        )

    def test_hud_extension_shortcuts_use_one_bounded_tools_menu_trigger(self) -> None:
        document = load_document(HUD_WORKSPACE)
        nodes = document["nodes"]

        self.assertEqual(
            [
                "hud_center_title",
                "hud_header_fill",
                "hud_tools_button",
                "hud_preview_button",
            ],
            [entry["node"] for entry in nodes["hud_center_header"]["children"]],
        )
        center_children = [entry["node"] for entry in nodes["hud_center"]["children"]]
        self.assertEqual(
            [
                "hud_center_header",
                "hud_canvas",
                "hud_validation_row",
            ],
            center_children,
        )
        self.assertNotIn("hud_extension_shortcut_scroll", nodes)

        trigger = nodes["hud_tools_button"]
        self.assertEqual("WorkbenchButton", trigger["component"])
        self.assertEqual("WorkbenchHudTools", trigger["control_id"])
        self.assertEqual("HUD Tools", trigger["props"]["text"])
        self.assertEqual(
            "workbench.module.hud.tools.open",
            trigger["events"][0]["route"],
        )
        self.assertLessEqual(trigger["layout"]["width"]["max"], 104.0)

    def test_all_ultra_visible_horizontal_groups_fit_the_workspace(self) -> None:
        tokens = load_document(TOKENS)
        ultra_width = float(tokens["density"]["ultra_minimum_window_width"])
        covered_workspaces = []
        horizontal_groups = []

        for module_root in MODULE_ROOTS:
            for path in sorted(module_root.rglob("*_workspace.zui")):
                if path == SCAN_ONLY_UI_ASSET_EDITOR:
                    continue
                document = load_document(path)
                nodes = document.get("nodes", {})
                reachable = set()
                pending = [
                    component["root"]
                    for component in document.get("components", {}).values()
                ]
                while pending:
                    node_id = pending.pop()
                    if node_id in reachable or node_id not in nodes:
                        continue
                    node = nodes[node_id]
                    if not visible_at_ultra(node):
                        continue
                    reachable.add(node_id)
                    pending.extend(entry["node"] for entry in node.get("children", []))

                covered_workspaces.append(path.relative_to(ROOT).as_posix())
                for node_id in sorted(reachable):
                    node = nodes[node_id]
                    container = node.get("layout", {}).get("container", {})
                    if container.get("kind") != "HorizontalBox":
                        continue
                    child_ids = [
                        entry["node"]
                        for entry in node.get("children", [])
                        if entry["node"] in reachable
                    ]
                    if not child_ids:
                        continue

                    minimum = sum(
                        minimum_width(nodes[child_id], tokens)
                        for child_id in child_ids
                    )
                    gap = dimension_value(container.get("gap", 0.0), tokens)
                    minimum += max(0, len(child_ids) - 1) * gap
                    location = f"{path.relative_to(ROOT).as_posix()}:{node_id}"
                    horizontal_groups.append(location)
                    self.assertLessEqual(
                        minimum,
                        ultra_width,
                        f"{location} needs {minimum}px at Ultra",
                    )

        self.assertEqual(54, len(covered_workspaces))
        self.assertGreater(len(horizontal_groups), len(covered_workspaces))

    def test_ultra_visible_vertical_groups_fit_or_scroll_in_the_main_band(self) -> None:
        tokens = load_document(TOKENS)
        available_height = (
            float(tokens["density"]["ultra_minimum_window_height"])
            - float(tokens["chrome"]["workbench_toolbar_height"])
            - float(tokens["controls"]["default_height"])
            - float(tokens["chrome"]["status_bar_height"])
        )
        covered_groups = []
        scrolling_groups = []

        for module_root in MODULE_ROOTS:
            for path in sorted(module_root.rglob("*_workspace.zui")):
                if path == SCAN_ONLY_UI_ASSET_EDITOR:
                    continue
                document = load_document(path)
                nodes = document.get("nodes", {})
                reachable = set()
                pending = [
                    component["root"]
                    for component in document.get("components", {}).values()
                ]
                while pending:
                    node_id = pending.pop()
                    if node_id in reachable or node_id not in nodes:
                        continue
                    node = nodes[node_id]
                    if not visible_at_ultra(node):
                        continue
                    reachable.add(node_id)
                    pending.extend(entry["node"] for entry in node.get("children", []))

                for node_id in sorted(reachable):
                    node = nodes[node_id]
                    container = node.get("layout", {}).get("container", {})
                    container_kind = container.get("kind")
                    if container_kind not in {"VerticalBox", "ScrollableBox"}:
                        continue
                    child_ids = [
                        entry["node"]
                        for entry in node.get("children", [])
                        if entry["node"] in reachable
                    ]
                    if not child_ids:
                        continue

                    minimum = sum(
                        dimension_value(
                            nodes[child_id]
                            .get("layout", {})
                            .get("height", {})
                            .get("min", 0.0),
                            tokens,
                        )
                        for child_id in child_ids
                    )
                    gap = dimension_value(container.get("gap", 0.0), tokens)
                    minimum += max(0, len(child_ids) - 1) * gap
                    location = f"{path.relative_to(ROOT).as_posix()}:{node_id}"
                    covered_groups.append(location)
                    if container_kind == "VerticalBox":
                        self.assertLessEqual(
                            minimum,
                            available_height,
                            f"{location} needs {minimum}px in the Ultra main band",
                        )
                    else:
                        scrolling_groups.append(location)
                        self.assertEqual("ScrollableBox", node["component"], location)
                        self.assertTrue(node["layout"]["clip"], location)
                        self.assertEqual("Receive", node["layout"]["input_policy"], location)
                        self.assertEqual(
                            "Auto", container["scrollbar_visibility"], location
                        )

        self.assertGreater(len(covered_groups), 100)
        self.assertGreater(len(scrolling_groups), 50)

    def test_effect_canvas_scrolls_instead_of_clipping_fixed_rows(self) -> None:
        effect_workspace = (
            MODULE_ROOTS[0] / "gameplay/workbench_effect_workspace.zui"
        )
        canvas = load_document(effect_workspace)["nodes"]["effect_canvas"]

        self.assertEqual("ScrollableBox", canvas["component"])
        self.assertTrue(canvas["props"]["input_hoverable"])
        self.assertNotIn("input_interactive", canvas["props"])
        self.assertTrue(canvas["layout"]["clip"])
        self.assertEqual("Receive", canvas["layout"]["input_policy"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Vertical",
                "gap": "$editor.density.gap.regular",
                "scrollbar_visibility": "Auto",
            },
            canvas["layout"]["container"],
        )


if __name__ == "__main__":
    unittest.main()
