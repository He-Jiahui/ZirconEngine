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


class ReviewStageTests(unittest.TestCase):
    """Review must cover each registered language and leave history intact."""

    def test_each_registered_language_reports_its_own_tool(self):
        from tools.jenkins.workflow.syntax import review_receipt
        cases = (("a.py", "def f():\n    return 1\n", "ast.parse"),
                 ("a.json", '{"k": 1}', "json.loads"),
                 ("a.toml", 'k = "v"\n', "tomllib.loads"))
        for path, text, tool in cases:
            with self.subTest(path=path):
                receipt = review_receipt(path, text)
                self.assertEqual("passed", receipt["status"])
                self.assertEqual("review", receipt["kind"])
                self.assertEqual(tool, receipt["tool"])
                self.assertEqual(path, receipt["path"])

    def test_a_broken_file_is_rejected_rather_than_reported_as_reviewed(self):
        from tools.jenkins.workflow.syntax import review_receipt
        from tools.jenkins.contracts import JenkinsError
        for path, text, code in (("a.py", "def f(:\n", "python_syntax_error"),
                                 ("a.json", '{"k": }', "json_syntax_error"),
                                 ("a.toml", "k = \n", "toml_syntax_error")):
            with self.subTest(path=path):
                with self.assertRaises(JenkinsError) as caught:
                    review_receipt(path, text)
                self.assertEqual(code, caught.exception.code)

    def test_unregistered_language_is_refused(self):
        from tools.jenkins.workflow.syntax import review_receipt
        from tools.jenkins.contracts import JenkinsError
        with self.assertRaises(JenkinsError) as caught:
            review_receipt("a.bin", "bytes")
        self.assertEqual("review_language_unsupported", caught.exception.code)

    def test_review_is_a_light_stage(self):
        from tools.jenkins.validation.receipts import HEAVY_STAGES, LIGHT_STAGES
        self.assertIn("review", LIGHT_STAGES)
        self.assertNotIn("review", HEAVY_STAGES)

    def test_existing_templates_keep_their_recorded_digest(self):
        """A workflow's dagDigest is its stage list, so history must not move."""
        from tools.jenkins.contracts import digest
        from tools.jenkins.workflow.orchestrator import TEMPLATES
        recorded = {"comments_only": "b08e67e4e9a1", "module_unit": "b67ee7bc9572",
                    "cross_module": "72f14d5afaf9", "failure_repair": "28c149dd0dba"}
        for template, prefix in recorded.items():
            with self.subTest(template=template):
                self.assertTrue(digest(list(TEMPLATES[template])).startswith(prefix),
                                "accepted history depends on this exact stage list")

    def test_review_templates_are_new_names_not_edited_ones(self):
        from tools.jenkins.workflow.orchestrator import TEMPLATES
        for name in ("comments_only_review", "module_unit_review",
                     "cross_module_review", "failure_repair_review"):
            self.assertIn(name, TEMPLATES)
            self.assertIn("review", TEMPLATES[name])
            self.assertLess(TEMPLATES[name].index("review"), TEMPLATES[name].index("acceptance"))


if __name__ == "__main__":
    unittest.main()
