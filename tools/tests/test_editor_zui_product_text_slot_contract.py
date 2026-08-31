import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSET_ROOT = REPO_ROOT / "zircon_editor/assets/ui/editor"
PRODUCT_ROOTS = (
    "windows/workbench_window.zui",
    "asset_browser.zui",
    "assets_activity.zui",
    "project_overview.zui",
    "welcome.zui",
    "hierarchy.zui",
    "inspector.zui",
    "console.zui",
)
PROTECTED_SCAN_ONLY_ASSET = (
    "components/workbench/modules/extensions/ui/"
    "workbench_extension_ui_asset_editor_workspace.zui"
)
FONT_SIZES = {
    "$editor.typography.caption.size": 8.0 * 96.0 / 72.0,
    "$editor.typography.overlay.size": 9.0 * 96.0 / 72.0,
    "$editor.typography.body.size": 10.0 * 96.0 / 72.0,
    "$editor.typography.title.size": 14.0 * 96.0 / 72.0,
}
MUTED_TEXT_TONES = {"muted", "subtle", "secondary"}
LINE_HEIGHT_RATIO = 1.2


def load_document(asset_name: str) -> dict:
    with (ASSET_ROOT / asset_name).open("rb") as handle:
        return tomllib.load(handle)


def reachable_product_assets() -> list[str]:
    pending = list(PRODUCT_ROOTS)
    reachable = set()
    while pending:
        asset_name = pending.pop()
        if asset_name in reachable or asset_name == PROTECTED_SCAN_ONLY_ASSET:
            continue
        path = ASSET_ROOT / asset_name
        if not path.is_file():
            continue
        reachable.add(asset_name)
        imports = load_document(asset_name).get("imports", {}).get("widgets", [])
        for imported in imports:
            resource = imported.split("#", 1)[0]
            prefix = "res://ui/editor/"
            if resource.startswith(prefix):
                pending.append(resource.removeprefix(prefix))
    return sorted(reachable)


def label_font_size(node: dict) -> float | None:
    props = node.get("props", {})
    authored = props.get("font_size")
    if isinstance(authored, str):
        return FONT_SIZES.get(authored)
    if isinstance(authored, (int, float)):
        return float(authored)
    if props.get("text_tone", "") in MUTED_TEXT_TONES:
        return FONT_SIZES["$editor.typography.caption.size"]
    return FONT_SIZES["$editor.typography.body.size"]


class EditorZuiProductTextSlotContractTests(unittest.TestCase):
    def test_fixed_parent_slots_preserve_child_label_line_height(self):
        violations = []

        for asset_name in reachable_product_assets():
            nodes = load_document(asset_name)["nodes"]
            parents = {}
            for parent_name, parent in nodes.items():
                for child in parent.get("children", []):
                    parents.setdefault(child["node"], []).append((parent_name, child))

            for node_name, node in nodes.items():
                if node.get("component") != "Label":
                    continue
                font_size = label_font_size(node)
                if font_size is None:
                    continue
                required_height = font_size * LINE_HEIGHT_RATIO

                for parent_name, child in parents.get(node_name, []):
                    parent_height = nodes[parent_name].get("layout", {}).get("height", {})
                    if parent_height.get("stretch") != "Fixed":
                        continue
                    preferred = parent_height.get("preferred")
                    if not isinstance(preferred, (int, float)):
                        continue

                    padding = (
                        child.get("slot", {}).get("layout", {}).get("padding", {})
                    )
                    top = padding.get("top", 0.0)
                    bottom = padding.get("bottom", 0.0)
                    if not isinstance(top, (int, float)) or not isinstance(
                        bottom, (int, float)
                    ):
                        continue
                    available_height = float(preferred) - float(top) - float(bottom)
                    if available_height + 1.0e-6 < required_height:
                        violations.append(
                            f"{asset_name}:{parent_name}>{node_name}:"
                            f"{available_height:g}<{required_height:g}"
                        )

        self.assertEqual([], violations)


if __name__ == "__main__":
    unittest.main()
