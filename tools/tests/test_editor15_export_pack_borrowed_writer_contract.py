from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
WRITER = ROOT / "zircon_runtime/src/asset/pack/writer.rs"
EXPORT_MANIFEST = ROOT / "zircon_runtime/src/bin/zircon_export_pack/manifest.rs"
EXPORT_RUN = ROOT / "zircon_runtime/src/bin/zircon_export_pack/run.rs"


def _derive_traits(source: str, struct_name: str) -> set[str]:
    match = re.search(
        rf"#\[derive\((?P<traits>[^)]*)\)\]\s*pub struct {struct_name}\b",
        source,
    )
    if match is None:
        raise AssertionError(f"missing derive declaration for {struct_name}")
    return {trait.strip() for trait in match.group("traits").split(",")}


class Editor15ExportPackBorrowedWriterContractTests(unittest.TestCase):
    def test_payload_bearing_input_types_are_not_cloneable(self) -> None:
        writer = WRITER.read_text(encoding="utf-8")
        manifest = EXPORT_MANIFEST.read_text(encoding="utf-8")

        self.assertNotIn("Clone", _derive_traits(writer, "ZrPackInputAsset"))
        self.assertNotIn("Clone", _derive_traits(manifest, "ExportPackInputs"))

    def test_writer_accepts_borrowed_input_assets(self) -> None:
        writer = WRITER.read_text(encoding="utf-8")

        self.assertIn("use std::borrow::Borrow;", writer)
        self.assertIn("I: IntoIterator<Item = A>", writer)
        self.assertIn("A: Borrow<ZrPackInputAsset>", writer)
        self.assertNotIn("asset.bytes.clone()", writer)

    def test_export_pack_reuses_inputs_for_first_and_determinism_writes(self) -> None:
        run = EXPORT_RUN.read_text(encoding="utf-8")

        self.assertIn(
            "ZrPackWriter::write(pack_inputs.pack_assets.iter())",
            run,
        )
        self.assertIn("ZrPackWriter::write(pack_assets.iter())", run)
        self.assertNotIn("pack_inputs.pack_assets.clone()", run)
        self.assertNotIn("ZrPackWriter::write(pack_assets.to_vec())", run)


if __name__ == "__main__":
    unittest.main()
