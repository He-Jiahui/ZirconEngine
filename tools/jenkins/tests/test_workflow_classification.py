from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.state import State
from tools.jenkins.workflow.classification import classify_change, classify_rust_source
from tools.jenkins.workflow.handler import _sealed_classification


class WorkflowClassificationTests(unittest.TestCase):
    def test_plain_comment_is_comments_only(self):
        result = classify_rust_source("// explanatory note\nlet value = 1;\n", changed_lines=[1])
        self.assertEqual(result["kind"], "comments_only")

    def _sealed_record(self, before: str, after: str, *, manifest_before: str | None = None,
                       journal: bool = True, base_head: str | None = None):
        temp = tempfile.TemporaryDirectory()
        root = Path(temp.name)
        object_root = root / "objects"
        objects = object_root / "inputs" / "objects"
        objects.mkdir(parents=True)
        before_hash = hashlib.sha256(before.encode()).hexdigest()
        after_hash = hashlib.sha256(after.encode()).hexdigest()
        (objects / after_hash).write_bytes(after.encode())
        if journal:
            (objects / before_hash).write_bytes(before.encode())
        entry = {"path": "src/lib.rs", "status": "present",
                 "beforeHash": manifest_before or (after_hash if journal else before_hash),
                 "sha256": after_hash, "objectDigest": after_hash}
        record = {"payload": {"sourceManifest": {"entries": [entry], "baseHead": base_head},
                              "objectRoot": str(object_root), "repoRoot": str(root)}}
        state = None
        if journal:
            state = State(root / "state.sqlite3")
            state.put_operation("source_patch", "patch-1", {
                "objectRoot": str(object_root),
                "beforeObjects": {"src/lib.rs": before_hash},
                "expectedAfterHashes": {"src/lib.rs": after_hash},
            }, status="complete")
            record["payload"]["patchOperationRefs"] = ["patch-1"]
        return temp, record, state

    def test_sealed_unchanged_rust_skips_git_base(self):
        temp, record, _ = self._sealed_record("fn main() {}\n", "fn main() {}\n",
                                              journal=False, base_head="deadbeef")
        try:
            with patch("tools.jenkins.workflow.handler.subprocess.run") as run:
                result = _sealed_classification(record)
            self.assertEqual({"kind": "blocked", "reason": "no_changed_rust_source"}, result)
            run.assert_not_called()
        finally:
            temp.cleanup()

    def test_sealed_journal_preimage_proves_comment_vs_semantic_change(self):
        comment_temp, comment_record, comment_state = self._sealed_record(
            "fn main() {}\n", "fn main() {}\n// note\n")
        semantic_temp, semantic_record, semantic_state = self._sealed_record(
            "fn main() {}\n", "fn changed() {}\n")
        try:
            self.assertEqual("comments_only",
                             _sealed_classification(comment_record, comment_state)["kind"])
            self.assertEqual("semantic",
                             _sealed_classification(semantic_record, semantic_state)["kind"])
        finally:
            comment_temp.cleanup()
            semantic_temp.cleanup()

    def test_sealed_manifest_equal_hash_uses_journal_preimage(self):
        temp, record, state = self._sealed_record(
            "fn main() {}\n", "fn changed() {}\n",
            manifest_before=hashlib.sha256(b"fn changed() {}\n").hexdigest())
        try:
            result = _sealed_classification(record, state)
            self.assertEqual("semantic", result["kind"])
            self.assertEqual("non_comment_token_change", result["reason"])
        finally:
            temp.cleanup()

    def test_sealed_missing_or_dirty_before_proof_is_blocked(self):
        temp, record, _ = self._sealed_record(
            "fn main() {}\n", "fn main() {}\n// note\n",
            journal=False, base_head="deadbeef")
        try:
            with patch("tools.jenkins.workflow.handler.subprocess.run") as run:
                run.return_value.stdout = b"fn dirty() {}\n"
                result = _sealed_classification(record)
            self.assertNotEqual("comments_only", result["kind"])
            self.assertEqual("sealed_base_mismatch", result["reason"])
        finally:
            temp.cleanup()

    def test_sealed_mixed_cargo_change_is_semantic(self):
        temp, record, state = self._sealed_record(
            "fn main() {}\n", "fn main() {}\n// note\n")
        try:
            record["payload"]["sourceManifest"]["entries"].append({
                "path": "Cargo.toml", "status": "present",
                "beforeHash": "a" * 64, "sha256": "b" * 64,
                "objectDigest": "b" * 64})
            result = _sealed_classification(record, state)
            self.assertEqual("semantic", result["kind"])
            self.assertEqual("non_rust_source_change", result["reason"])
        finally:
            temp.cleanup()

    def test_sealed_deleted_rust_with_comment_change_is_blocked(self):
        temp, record, state = self._sealed_record(
            "fn main() {}\n", "fn main() {}\n// note\n")
        try:
            old = b"fn removed() {}\n"
            old_hash = hashlib.sha256(old).hexdigest()
            objects = Path(record["payload"]["objectRoot"]) / "inputs" / "objects"
            (objects / old_hash).write_bytes(old)
            state.put_operation("source_patch", "patch-2", {
                "objectRoot": record["payload"]["objectRoot"],
                "beforeObjects": {"src/removed.rs": old_hash},
                "expectedAfterHashes": {"src/removed.rs": None},
            }, status="complete")
            record["payload"]["patchOperationRefs"].append("patch-2")
            record["payload"]["sourceManifest"]["entries"].append({
                "path": "src/removed.rs", "status": "deleted",
                "beforeHash": old_hash, "sha256": None})
            result = _sealed_classification(record, state)
            self.assertEqual("blocked", result["kind"])
            self.assertEqual("sealed_before_unavailable", result["reason"])
        finally:
            temp.cleanup()

    def test_sealed_doctest_marker_is_semantic(self):
        temp, record, state = self._sealed_record(
            "fn main() {}\n", "fn main() {}\n/// " + chr(96) * 3 + "rust\n/// fn example() {}\n")
        try:
            result = _sealed_classification(record, state)
            self.assertEqual("semantic", result["kind"])
            self.assertEqual("documentation_comment", result["reason"])
        finally:
            temp.cleanup()

    def test_doc_comment_is_not_comments_only(self):
        result = classify_rust_source("/// changes generated documentation\n", changed_lines=[1])
        self.assertEqual(result["kind"], "semantic")

    def test_include_and_cfg_are_semantic(self):
        for text in ["include!(\"generated.rs\");\n", "#[cfg(feature = \"x\")]\n"]:
            self.assertEqual(classify_rust_source(text)["kind"], "semantic")

    def test_unterminated_comment_blocks(self):
        self.assertEqual(classify_rust_source("/* unfinished")["kind"], "blocked")

    def test_unknown_file_blocks_and_mixed_change_is_not_comment_only(self):
        result = classify_change({"a.rs": "// note\n", "a.bin": "bytes"})
        self.assertEqual(result["kind"], "blocked")


if __name__ == "__main__":
    unittest.main()
