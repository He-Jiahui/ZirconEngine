import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_PATH = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "ui"
    / "template_node_conversion.rs"
)


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\b[^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing Rust function: {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function: {name}")
    return source[match.end() : index - 1]


class EditorTemplateNodeOwnedProjectionPerformanceContractTests(unittest.TestCase):
    def test_owned_projection_moves_every_heap_backed_source_field(self):
        source = SOURCE_PATH.read_text(encoding="utf-8")
        body = function_body(source, "to_host_contract_template_node_owned")

        self.assertIn("mut data: ViewTemplateNodeData", source)
        self.assertNotRegex(
            body,
            r"^\s*to_host_contract_template_node\(&data\)\s*$",
            "the owned product path must not clone the complete source node",
        )

        moved_fields = {
            "node_id",
            "control_id",
            "role",
            "text",
            "component_role",
            "component_variant",
            "value_text",
            "options",
            "dispatch_kind",
            "action_id",
            "binding_id",
            "edit_action_id",
            "commit_action_id",
            "surface_variant",
            "text_tone",
            "button_variant",
            "button_style",
            "text_align",
            "overflow",
            "transition_kind",
            "transition_easing",
            "transition_direction",
            "media_source",
            "icon_name",
            "preview_image",
        }
        for field in sorted(moved_fields):
            self.assertRegex(
                body,
                rf"std::mem::take\(&mut data\.{field}\)",
                f"owned projection still clones or drops source field: {field}",
            )

        self.assertRegex(
            body,
            r"to_host_contract_template_node_with_options_text\(\s*&data,\s*options_text,?\s*\)",
        )
        self.assertNotIn(
            "node.options_text =",
            body,
            "owned projection must not allocate and discard a temporary options string",
        )


if __name__ == "__main__":
    unittest.main()
