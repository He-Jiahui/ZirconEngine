import unittest
from pathlib import Path


class RuntimeGltfSnapshotContractTests(unittest.TestCase):
    """Static guard for the Runtime glTF immutable-source and decode-budget boundary."""

    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        base = self.repo_root / "zircon_runtime/src/asset/importer/ingest"
        self.decode = base / "gltf_decode.rs"
        self.sources = base / "gltf_decode/sources.rs"
        self.buffers = base / "gltf_decode/buffers.rs"
        self.images = base / "gltf_decode/images.rs"
        self.admission_tests = base / "gltf_decode/tests/admission_tests.rs"

    @staticmethod
    def _production(source: str) -> str:
        return source

    @staticmethod
    def _decode_production(source: str) -> str:
        return source

    def test_decode_starts_from_transaction_bytes_and_prefers_admitted_snapshots(self) -> None:
        decode = self._decode_production(self.decode.read_text(encoding="utf-8"))
        sources = self._production(self.sources.read_text(encoding="utf-8"))

        for anchor in (
            "pub(crate) fn decode_gltf_source(",
            "decode_gltf_source_with_buffer_limit(context, MAX_GLTF_AUXILIARY_BYTES)",
            "gltf::Gltf::from_slice_without_validation(&context.source_bytes)",
            "let mut sources = ExternalSources::new(context);",
            "let mut budget = DecodedBudget::new(limit);",
            "source_file_snapshot(&path)",
            "has_source_file_snapshots()",
            "gltf auxiliary snapshot does not contain",
            "read_gltf_uri_snapshot(uri, self.remaining.min(limit))",
            "ProjectPaths::lexical_identity(&path)",
            "snapshots: BTreeMap<LexicalProjectPathIdentity, Cow<'a, [u8]>>",
        ):
            self.assertIn(anchor, decode + sources)
        self.assertLess(
            decode.index("gltf::Gltf::from_slice_without_validation(&context.source_bytes)"),
            decode.index("let mut sources = ExternalSources::new(context);"),
        )
        self.assertNotIn("std::fs::read", decode)
        self.assertNotIn("fs::read", decode)

    def test_external_snapshot_discovery_deduplicates_and_bounds_files_and_bytes(self) -> None:
        decode = self._decode_production(self.decode.read_text(encoding="utf-8"))
        sources = self._production(self.sources.read_text(encoding="utf-8"))

        for anchor in (
            "pub(crate) fn snapshot_external_gltf_sources(",
            "let existing_keys = existing",
            "let mut references = BTreeMap::new();",
            "existing_keys.contains(&key) || references.contains_key(&key)",
            "AuxiliarySourceResolver::MAX_SNAPSHOT_FILES",
            "existing_keys.len().saturating_add(references.len())",
            "let mut remaining = limit.min(MAX_GLTF_AUXILIARY_BYTES);",
            "remaining = remaining.checked_sub(bytes.len() as u64)",
            "snapshots.insert(lexical, bytes);",
            "self.snapshots.len() >= AuxiliarySourceResolver::MAX_SNAPSHOT_FILES",
            "self.remaining = self.remaining.checked_sub(bytes.len() as u64)",
        ):
            self.assertIn(anchor, decode + sources)
        references = decode[decode.index("let mut references = BTreeMap::new();") :]
        self.assertLess(
            references.index("existing_keys.len().saturating_add(references.len())"),
            references.index("let mut remaining = limit.min(MAX_GLTF_AUXILIARY_BYTES);"),
        )
        self.assertIn("read_gltf_uri_snapshot(uri, remaining)?", references)
        self.assertIn("MAX_GLTF_AUXILIARY_BYTES", decode)

    def test_split_importer_uses_runtime_snapshot_decoder_for_external_sources(self) -> None:
        plugin = (
            self.repo_root / "zircon_plugins/gltf_importer/runtime/src/lib.rs"
        ).read_text(encoding="utf-8")
        decode = self._decode_production(self.decode.read_text(encoding="utf-8"))
        importer = plugin[plugin.index("pub fn import_gltf(") : plugin.index("fn reserve_gltf_mesh_outputs(")]

        self.assertIn("decode_gltf_source_with_required_extensions(", importer)
        self.assertIn("STABLE_IMPORTER_SUPPORTED_REQUIRED_EXTENSIONS", importer)
        self.assertIn("pub fn decode_gltf_source_with_required_extensions(", decode)
        self.assertNotIn("gltf::import_buffers", plugin)
        self.assertNotIn("gltf::import_images", plugin)
        self.assertNotIn("validate_external_gltf_buffers", plugin)

    def test_buffer_and_image_paths_share_budget_and_reject_invalid_ranges(self) -> None:
        buffers = self._production(self.buffers.read_text(encoding="utf-8"))
        images = self._production(self.images.read_text(encoding="utf-8"))

        for anchor in (
            "pub(super) fn load_buffers(",
            "budget: &mut DecodedBudget",
            "document.buffers().len() > AuxiliarySourceResolver::MAX_SNAPSHOT_FILES",
            "budget.check(declared, \"buffer\")?",
            "budget.charge(length as u64, \"buffer\")?",
            "sources.read(uri, remaining)",
            "pub(super) fn decode_images(",
            "decode_external_image(encoded, Path::new(\"\"), Some(mime_type), budget)",
            "checked_add(view.length())",
            "buffer.get(view.offset()..end)",
            "limits.max_alloc = Some(MAX_IMAGE_DECODER_ALLOCATION_BYTES)",
            "budget.check(peak, \"image\")?",
            "budget.charge(pixels.len() as u64, \"image\")?",
        ):
            self.assertIn(anchor, buffers + images)
        self.assertIn("let image_count = document.images().len();", images)
        self.assertIn(
            "image_count > AuxiliarySourceResolver::MAX_SNAPSHOT_FILES", images
        )
        self.assertLess(buffers.index("budget.check(declared, \"buffer\")?"), buffers.index("let mut buffers"))
        self.assertLess(images.index("checked_add(view.length())"), images.index("decode_external_image(encoded"))
        self.assertIn("try_reserve_exact", buffers)
        self.assertIn("try_reserve_exact", images)

    def test_regressions_cover_link_containment_replacement_and_exact_budget_edges(self) -> None:
        decode_tests = "\n".join(
            (self.repo_root / relative).read_text(encoding="utf-8")
            for relative in ('zircon_runtime/src/asset/importer/ingest/tests/gltf_decode_plugins07_deferred_buffer_diagnostic_tests.rs',)
        )
        admission = self.admission_tests.read_text(encoding="utf-8")
        plugin_snapshot = (
            self.repo_root
            / "zircon_plugins/gltf_importer/runtime/src/tests/source_snapshot.rs"
        ).read_text(encoding="utf-8")

        for test_name in (
            "fn gltf_auxiliary_count_limit_is_checked_before_opening_files()",
            "fn gltf_external_source_snapshot_containment_decodes_internal_parent_and_rejects_escape()",
            "fn gltf_external_image_uses_admitted_snapshot_bytes_before_disk()",
            "fn gltf_declared_buffer_budget_rejects_fallback_and_repeated_sources_before_loading()",
            "fn gltf_embedded_buffer_budget_accepts_exact_limit_and_rejects_next_buffer()",
            "fn gltf_external_buffer_budget_rejects_actual_size_during_read_admission()",
            "fn gltf_source_snapshot_charges_windows_case_aliases_once()",
            "fn gltf_direct_source_cache_keeps_the_first_generation_after_file_removal()",
            "fn importer_uses_embedded_source_snapshot_without_reopening_path()",
        ):
            self.assertIn(test_name, decode_tests + admission + plugin_snapshot)
        for anchor in (
            "AssetImportContext::new",
            "missing/embedded.gltf",
            "source_bytes",
            "transaction-owned source bytes",
        ):
            self.assertIn(anchor, plugin_snapshot)


if __name__ == "__main__":
    unittest.main()
