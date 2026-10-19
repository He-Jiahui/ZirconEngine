import ast
import hashlib
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits.dynamic_runtime_api_boundary import (  # noqa: E402
    _missing_anchors,
    dynamic_runtime_api_boundary_audit,
)
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import (  # noqa: E402
    ARCHIVE_ROOT_ENV,
    HISTORICAL_IDENTITIES,
    HistoricalArchiveReader,
    RUNTIME_09_UI_OUTPUT_ARCHIVE,
    RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE,
    RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
)

# 调用动态 API 边界审计器核对 runtime15 退役证据已归档，并保持 runtime09/10 的现行契约来源。
class RuntimeDynamicApiBoundaryArchiveOwnershipTests(unittest.TestCase):
    def test_dynamic_api_inventories_hard_cut_runtime_15_evidence_to_archive(self) -> None:
        inventory_root = AUDIT_SCRIPTS / "runtime_structure_audits"
        canonical = (
            "docs/plans/_archive/zircon_runtime/runtime/15/"
            "2026-07-09-runtime-index-output-records.md"
        )
        retired = (
            "docs/plans/zircon_runtime/runtime/15/"
            "2026-07-09-runtime-index-output-records.md"
        )
        owner_path = inventory_root / "dynamic_runtime_api_archive_inventory.py"
        owner_tree = ast.parse(
            owner_path.read_text(encoding="utf-8"), filename=str(owner_path)
        )
        owner_literals = [
            node.value
            for node in ast.walk(owner_tree)
            if isinstance(node, ast.Constant) and isinstance(node.value, str)
        ]

        self.assertEqual(canonical, RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE)
        self.assertEqual(1, owner_literals.count(canonical))
        self.assertNotIn(retired, owner_literals)

        for name in (
            "dynamic_runtime_api_ui_contract_inventory.py",
            "dynamic_runtime_api_validation_inventory.py",
        ):
            consumer_path = inventory_root / name
            consumer_tree = ast.parse(
                consumer_path.read_text(encoding="utf-8"), filename=str(consumer_path)
            )
            consumer_literals = [
                node.value
                for node in ast.walk(consumer_tree)
                if isinstance(node, ast.Constant) and isinstance(node.value, str)
            ]
            imports = [
                node
                for node in ast.walk(consumer_tree)
                if isinstance(node, ast.ImportFrom)
                and node.module
                == "runtime_structure_audits.dynamic_runtime_api_archive_inventory"
            ]

            self.assertEqual(1, len(imports), name)
            self.assertEqual(
                (["RUNTIME_09_UI_OUTPUT_ARCHIVE", "RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE"]
                 if name == "dynamic_runtime_api_ui_contract_inventory.py" else
                 ["RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE"])
                + ["RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE"],
                [alias.name for alias in imports[0].names],
                name,
            )
            self.assertNotIn(canonical, consumer_literals, name)
            self.assertNotIn(retired, consumer_literals, name)
            self.assertNotIn(RUNTIME_09_UI_OUTPUT_ARCHIVE, consumer_literals, name)
            self.assertNotIn(RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE, consumer_literals, name)

    def test_numbered_runtime_09_10_archives_supply_concrete_contract_evidence(self) -> None:
        report = dynamic_runtime_api_boundary_audit(REPO_ROOT)

        self.assertEqual([], report["missing_ui_pending_gate_anchors"])
        self.assertEqual([], report["missing_ui_contract_single_source_anchors"])
        self.assertEqual([], report["missing_ui_v2_contract_sync_anchors"])
        self.assertEqual([], report["missing_doc_anchors"])
        self.assertEqual([], report["risks"])
        readbacks = report["historical_archive_readback"]
        self.assertEqual(set(HISTORICAL_IDENTITIES), {row["path"] for row in readbacks})
        self.assertTrue(all(row["historical_only"] for row in readbacks))
        self.assertTrue(all(row["status"] == "verified" for row in readbacks))

    def test_history_reader_verifies_short_fixture_without_checkout_fallback(self) -> None:
        relative = RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE
        payload = b"historical pending marker\n"
        identities = {relative: (len(payload), hashlib.sha256(payload).hexdigest())}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "archive"
            archive.mkdir()
            target = archive / relative
            target.parent.mkdir(parents=True)
            target.write_bytes(payload)
            reader = HistoricalArchiveReader(archive, identities=identities)
            self.assertEqual(payload.decode(), reader.read(relative))
            self.assertEqual("verified", reader.records[relative]["status"])
            self.assertTrue(reader.records[relative]["historical_only"])
            self.assertEqual([], _missing_anchors(
                root, ((relative, "pending marker"),), history=reader,
            ))
            current = root / "current.rs"
            current.write_text("current marker", encoding="utf-8")
            self.assertEqual([], _missing_anchors(
                root, (("current.rs", "current marker"),), history=reader,
            ))

    def test_history_reader_rejects_missing_root_and_checkout_fallback(self) -> None:
        relative = RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE
        payload = b"historical pending marker\n"
        identities = {relative: (len(payload), hashlib.sha256(payload).hexdigest())}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            checkout = root / relative
            checkout.parent.mkdir(parents=True)
            checkout.write_bytes(payload)
            with patch.dict(os.environ, {}, clear=True):
                reader = HistoricalArchiveReader(identities=identities)
            self.assertEqual([{"path": relative, "snippet": "pending marker"}],
                             _missing_anchors(root, ((relative, "pending marker"),), history=reader))
            self.assertEqual("archive-root-not-configured", reader.records[relative]["reason"])
            archive = root / "archive"
            archive.mkdir()
            missing = HistoricalArchiveReader(archive, identities=identities)
            self.assertIsNone(missing.read(relative))
            self.assertEqual("archive-read-error", missing.records[relative]["reason"])

    def test_history_reader_rejects_wrong_length_and_sha256(self) -> None:
        relative = RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE
        payload = b"historical pending marker\n"
        identities = {relative: (len(payload), hashlib.sha256(payload).hexdigest())}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / relative
            target.parent.mkdir(parents=True)
            for invalid, reason in ((payload + b"x", "length-mismatch"),
                                    (b"X" + payload[1:], "sha256-mismatch")):
                with self.subTest(reason=reason):
                    target.write_bytes(invalid)
                    reader = HistoricalArchiveReader(root, identities=identities)
                    self.assertIsNone(reader.read(relative))
                    self.assertEqual(reason, reader.records[relative]["reason"])

    def test_history_reader_uses_explicit_root_before_environment(self) -> None:
        with patch.dict(os.environ, {ARCHIVE_ROOT_ENV: "environment-root"}):
            self.assertEqual(Path("environment-root"), HistoricalArchiveReader().archive_root)
            self.assertEqual(Path("explicit-root"), HistoricalArchiveReader(Path("explicit-root")).archive_root)

    def test_history_reader_rejects_relative_roots_and_escaping_paths(self) -> None:
        reader = HistoricalArchiveReader(Path("relative"), identities={"record.md": (0, "")})
        self.assertIsNone(reader.read("record.md"))
        self.assertEqual("archive-root-not-absolute", reader.records["record.md"]["reason"])
        for relative in ("../escape.md", "/absolute.md", "C:/record.md", "dir\\record.md"):
            with self.subTest(path=relative):
                reader = HistoricalArchiveReader(identities={relative: (0, "")})
                self.assertIsNone(reader.read(relative))
                self.assertEqual("invalid-relative-path", reader.records[relative]["reason"])


if __name__ == "__main__":
    unittest.main()
