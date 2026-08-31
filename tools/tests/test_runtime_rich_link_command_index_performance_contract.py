from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RICH_LINK = ROOT / "zircon_runtime/src/ui/surface/input/rich_link.rs"


def function_body(source: str, name: str) -> str:
    start = source.index(f"fn {name}(")
    boundaries = (
        source.find(marker, start + 1)
        for marker in (
            "\nfn ",
            "\npub(super) fn ",
            "\npub(crate) fn ",
            "\npub fn ",
            "\n#[cfg(",
        )
    )
    next_functions = [boundary for boundary in boundaries if boundary >= 0]
    return source[start:] if not next_functions else source[start : min(next_functions)]


class RuntimeRichLinkCommandIndexPerformanceContractTests(unittest.TestCase):
    def test_rich_link_click_uses_target_node_command_range(self) -> None:
        source = RICH_LINK.read_text(encoding="utf-8")
        body = function_body(source, "dispatch_pointer_rich_link_activation")
        compact = "".join(body.split())

        self.assertIn("commands_for_node(&surface.render_extract, target)", body)
        self.assertIn(".map(|(_, commands)| commands)", body)
        self.assertIn("surface.render_extract.list.commands.as_slice()", body)
        self.assertIn("candidate_commands.iter().rev()", compact)
        self.assertNotIn(".list\n        .commands\n        .iter()", body)

    def test_owned_link_target_is_moved_into_the_dispatch_effect(self) -> None:
        source = RICH_LINK.read_text(encoding="utf-8")
        body = function_body(source, "dispatch_pointer_rich_link_activation")

        self.assertIn("link_target: hit.target", body)
        self.assertNotIn("link_target: hit.target.clone()", body)


if __name__ == "__main__":
    unittest.main()
