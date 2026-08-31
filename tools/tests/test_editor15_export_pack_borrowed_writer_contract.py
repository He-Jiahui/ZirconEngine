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

    def test_export_pack_streams_sources_for_first_and_determinism_writes(self) -> None:
        run = EXPORT_RUN.read_text(encoding="utf-8")

        self.assertEqual(run.count("ZrPackWriter::write_files"), 2)
        self.assertNotIn("pack_inputs.pack_assets", run)
        self.assertNotIn("ZrPackWriter::write(pack_assets", run)

    def test_export_manifest_keeps_source_paths_instead_of_all_payload_bytes(self) -> None:
        manifest = EXPORT_MANIFEST.read_text(encoding="utf-8")

        self.assertIn("pub pack_sources: Vec<ExportPackInputSource>", manifest)
        self.assertIn("source: source_path(manifest_dir, source)", manifest)
        self.assertNotIn("std::fs::read(&source)", manifest)
        self.assertNotIn("pub pack_assets: Vec<ZrPackInputAsset>", manifest)

    def test_export_writer_streams_file_sources_through_a_fixed_buffer(self) -> None:
        writer = WRITER.read_text(encoding="utf-8")
        run = EXPORT_RUN.read_text(encoding="utf-8")

        self.assertIn("const FILE_READ_BUFFER_SIZE: usize = 64 * 1024;", writer)
        self.assertIn("pub(crate) fn write_files", writer)
        self.assertIn("let mut read_buffer = [0_u8; FILE_READ_BUFFER_SIZE];", writer)
        self.assertIn("ZrPackWriter::write_files", run)
        self.assertNotIn("pack_inputs.pack_assets", run)

    def test_duplicate_file_sources_are_identified_before_payload_append(self) -> None:
        writer = WRITER.read_text(encoding="utf-8")

        self.assertIn("file.seek(SeekFrom::Start(0))", writer)
        self.assertIn("ZrPackFileWriteError::SourceChanged", writer)
        self.assertNotIn("self.bytes.truncate(payload_offset)", writer)


if __name__ == "__main__":
    unittest.main()
