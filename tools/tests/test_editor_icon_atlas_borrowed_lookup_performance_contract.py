from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ATLAS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/"
    "icon_atlas.rs"
)


class EditorIconAtlasBorrowedLookupPerformanceContract(unittest.TestCase):
    def test_slot_index_supports_borrowed_resource_key_lookup(self) -> None:
        source = ATLAS.read_text(encoding="utf-8")

        self.assertIn(
            "slots: BTreeMap<String, BTreeMap<IconSourceVersion, IconAtlasSlot>>",
            source,
        )
        self.assertIn("resource_key: &str", source)
        self.assertIn("fn slot(", source)
        slot = source.split("fn slot(", 1)[1].split("fn next_access", 1)[0]
        self.assertIn("&self", slot)
        self.assertIn("resource_key: &str", slot)

    def test_icon_source_discovery_borrows_key_and_pixels(self) -> None:
        source = ATLAS.read_text(encoding="utf-8")
        discovery = source.split("fn icon_source_from_command", 1)[1]
        discovery = discovery.split("fn is_editor_icon_key", 1)[0]

        self.assertIn("IconSource<'_>", discovery)
        self.assertIn("resource_key: payload.resource_key.as_str()", discovery)
        self.assertIn("rgba", discovery)
        self.assertNotIn("payload.resource_key.clone()", discovery)
        self.assertNotIn("Arc::clone", discovery)

    def test_pack_uses_one_discovery_scan_and_one_rewrite_scan(self) -> None:
        source = ATLAS.read_text(encoding="utf-8")
        pack = source.split("fn pack(&mut self, commands: &mut [ChromeCommand])", 1)[1]
        pack = pack.split("fn allocate(", 1)[0]

        lines = [line.strip() for line in pack.splitlines()]
        self.assertEqual(lines.count("for command in commands.iter() {"), 1)
        self.assertEqual(lines.count("for command in commands {"), 1)
        self.assertNotIn("payload.resource_key.clone()", pack)


if __name__ == "__main__":
    unittest.main()
