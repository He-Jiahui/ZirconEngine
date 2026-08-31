import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MATERIAL_LAB = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/material_component_lab.zui"
)
EDITOR_TOKENS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
)


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def minimum_width(node: dict) -> float:
    return float(node.get("layout", {}).get("width", {}).get("min", 0.0))


def visible_at_ultra(node: dict) -> bool:
    return node.get("props", {}).get("responsive_min_tier", "ultra") == "ultra"


def density_gap(value, tokens: dict) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    prefix = "$editor.density.gap."
    if not isinstance(value, str) or not value.startswith(prefix):
        raise AssertionError(f"unsupported density gap {value!r}")
    return float(tokens["density"][f"gap_{value.removeprefix(prefix)}"])


def string_values(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for child in value.values():
            yield from string_values(child)
    elif isinstance(value, list):
        for child in value:
            yield from string_values(child)


def numeric_visual_overrides(value, owner: str = "document"):
    if isinstance(value, dict):
        for key, child in value.items():
            child_owner = f"{owner}.{key}"
            if (
                key in {"radius", "corner_radius", "font_size", "font_weight"}
                or key == "gap"
                or key.endswith("_gap")
            ) and isinstance(child, (int, float)):
                yield child_owner, child
            yield from numeric_visual_overrides(child, child_owner)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from numeric_visual_overrides(child, f"{owner}[{index}]")


class EditorZuiMaterialLabResponsiveContractTests(unittest.TestCase):
    def test_lab_metadata_and_state_labels_use_readable_caption_token(self) -> None:
        document = load_document(MATERIAL_LAB)
        rules = {
            rule["selector"]: rule["set"]["self"]
            for rule in document["stylesheets"][0]["rules"]
        }

        caption_token = "$editor.typography.caption.size"
        for selector in (".material-lab-meta-chip", ".material-lab-state-pill"):
            self.assertEqual(caption_token, rules[selector]["font_size"], selector)

        self.assertGreater(
            float(load_document(EDITOR_TOKENS)["typography"]["caption_size"]),
            8.0,
        )

    def test_lab_stylesheet_projects_editor_tokens_instead_of_private_hex_palette(
        self,
    ) -> None:
        document = load_document(MATERIAL_LAB)
        rules = {
            rule["selector"]: rule["set"]["self"]
            for rule in document["stylesheets"][0]["rules"]
        }

        self.assertNotRegex(repr(document["stylesheets"]), r"#[0-9a-fA-F]{6,8}")
        self.assertEqual(
            {
                "background_color": "$editor.surface.0",
                "foreground_color": "$editor.text.primary",
            },
            rules[".material-lab-shell"],
        )
        self.assertEqual("$editor.surface.1", rules[".material-lab-card"]["background_color"])
        self.assertEqual("$editor.border", rules[".material-lab-card"]["border_color"])
        self.assertEqual(
            "$editor.accent.soft",
            rules[".material-lab-nav-active"]["background_color"],
        )
        self.assertEqual(
            "$editor.focus.ring",
            rules[".material-lab-state-focus"]["border_color"],
        )
        self.assertEqual(
            "$editor.semantic.error.container",
            rules[".material-lab-state-error"]["background_color"],
        )

    def test_lab_shell_uses_editor_shape_typography_and_spacing_tokens(self) -> None:
        document = load_document(MATERIAL_LAB)

        self.assertEqual(
            [
                "res://ui/editor/theme/editor_tokens.zui",
                "res://ui/theme/editor_material.zui",
            ],
            document["imports"]["styles"],
        )
        self.assertEqual([], list(numeric_visual_overrides(document)))
        self.assertEqual(
            "$editor.control.radius.panel",
            document["stylesheets"][0]["rules"][1]["set"]["self"]["radius"],
        )
        self.assertEqual(
            "$editor.typography.title.size",
            document["nodes"]["appbar_title"]["props"]["font_size"],
        )
        self.assertEqual(
            "$editor.density.gap.large",
            document["nodes"]["material_lab_root"]["layout"]["container"]["gap"],
        )

        public_tokens = {
            f"${token_name}"
            for token_name in string_values(load_document(EDITOR_TOKENS)["names"])
        }
        referenced_tokens = {
            value for value in string_values(document) if value.startswith("$editor.")
        }
        self.assertEqual(set(), referenced_tokens - public_tokens)

    def test_lab_shell_preserves_content_with_bounded_ultra_budgets(self) -> None:
        nodes = load_document(MATERIAL_LAB)["nodes"]
        tokens = load_document(EDITOR_TOKENS)
        ultra_width = float(tokens["density"]["ultra_minimum_window_width"])
        narrow_width = float(tokens["density"]["minimum_window_width"])

        for node_name in ("appbar_status", "appbar_capture", "drawer", "side_panel"):
            self.assertEqual(
                "narrow",
                nodes[node_name]["props"]["responsive_min_tier"],
                node_name,
            )

        appbar = nodes["appbar"]
        appbar_names = [child["node"] for child in appbar["children"]]
        appbar_gap = density_gap(appbar["layout"]["container"]["gap"], tokens)
        appbar_authored = sum(minimum_width(nodes[name]) for name in appbar_names)
        appbar_authored += appbar_gap * (len(appbar_names) - 1)
        appbar_ultra_names = [
            name for name in appbar_names if visible_at_ultra(nodes[name])
        ]
        self.assertEqual(
            ["appbar_title", "appbar_scope", "appbar_count"],
            appbar_ultra_names,
        )
        appbar_ultra = sum(
            minimum_width(nodes[name]) for name in appbar_ultra_names
        ) + appbar_gap * (len(appbar_ultra_names) - 1)
        self.assertLessEqual(appbar_authored, narrow_width)
        self.assertLessEqual(appbar_ultra, ultra_width)

        body = nodes["body"]
        body_names = [child["node"] for child in body["children"]]
        body_gap = density_gap(body["layout"]["container"]["gap"], tokens)
        body_authored = sum(minimum_width(nodes[name]) for name in body_names)
        body_authored += body_gap * (len(body_names) - 1)
        body_ultra_names = [
            name for name in body_names if visible_at_ultra(nodes[name])
        ]
        self.assertEqual(["content"], body_ultra_names)
        body_ultra = sum(minimum_width(nodes[name]) for name in body_ultra_names)
        self.assertLessEqual(body_authored, narrow_width)
        self.assertLessEqual(body_ultra, ultra_width)

    def test_lab_prototype_content_owns_vertical_wheel_input(self) -> None:
        content = load_document(MATERIAL_LAB)["nodes"]["content"]

        self.assertEqual("ScrollableBox", content["component"])
        self.assertEqual({"input_hoverable": True}, content["props"])
        self.assertTrue(content["layout"]["clip"])
        self.assertEqual("Receive", content["layout"]["input_policy"])
        self.assertEqual("ScrollableBox", content["layout"]["container"]["kind"])
        self.assertEqual("Vertical", content["layout"]["container"]["axis"])
        self.assertEqual(
            "Auto", content["layout"]["container"]["scrollbar_visibility"]
        )


if __name__ == "__main__":
    unittest.main()
