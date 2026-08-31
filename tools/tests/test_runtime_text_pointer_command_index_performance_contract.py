from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TEXT_POINTER = ROOT / "zircon_runtime/src/ui/surface/input/text_pointer.rs"


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


class RuntimeTextPointerCommandIndexPerformanceContractTests(unittest.TestCase):
    def test_text_hit_uses_target_node_command_range(self) -> None:
        source = TEXT_POINTER.read_text(encoding="utf-8")
        body = function_body(source, "text_pointer_hit")

        self.assertIn("commands_for_node(&surface.render_extract, target)", body)
        self.assertIn(".map(|(_, commands)| commands)", body)
        self.assertIn("surface.render_extract.list.commands.as_slice()", body)
        compact = "".join(body.split())
        self.assertIn("candidate_commands.iter().find", compact)
        self.assertIn("command.text_layout.is_some()", compact)
        self.assertIn("command.text_layout.as_ref()?", compact)
        self.assertNotIn(".list\n        .commands\n        .iter()", body)


if __name__ == "__main__":
    unittest.main()
