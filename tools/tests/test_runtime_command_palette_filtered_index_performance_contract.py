from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMMAND_PALETTE = ROOT / "zircon_runtime/src/ui/surface/render/command_palette.rs"


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


class RuntimeCommandPaletteFilteredIndexPerformanceContractTests(unittest.TestCase):
    def test_filtered_rows_use_one_borrowed_command_index(self) -> None:
        source = COMMAND_PALETTE.read_text(encoding="utf-8")
        body = function_body(source, "command_rows")
        compact = "".join(body.split())

        self.assertIn("let command_index = command_entry_index(&commands);", body)
        self.assertIn("command_index.get(id.as_str())", compact)
        self.assertNotIn("commands.iter().find(|entry|entry.matches_id(&id))", compact)

    def test_index_preserves_first_command_match_for_id_and_label(self) -> None:
        source = COMMAND_PALETTE.read_text(encoding="utf-8")
        body = function_body(source, "command_entry_index")
        compact = "".join(body.split())

        self.assertIn("commands.iter().enumerate()", body)
        self.assertIn("index.entry(entry.id.as_str()).or_insert(entry_index)", compact)
        self.assertIn("index.entry(entry.label.as_str()).or_insert(entry_index)", compact)


if __name__ == "__main__":
    unittest.main()
