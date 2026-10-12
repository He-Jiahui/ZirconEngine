"""Regression cases for sealed before/after Rust comment admission."""
import hashlib
from pathlib import Path
import unittest

from tools.jenkins.tests import test_workflow_classification as fixtures
from tools.jenkins.workflow.handler import _sealed_classification


class SealedLexicalEvidenceTests(unittest.TestCase):
    def fixture(self, before, after):
        temp, record, state = fixtures.WorkflowClassificationTests()._sealed_record(before, after)
        self.addCleanup(temp.cleanup)
        return record, state

    def test_ordinary_inline_comment_is_proved_by_equal_tokens(self):
        record, state = self.fixture("fn value() -> i32 { 1 } // before\n",
                                     "fn value() -> i32 { 1 } // after\n")
        self.assertEqual("comments_only", _sealed_classification(record, state)["kind"])

    def test_raw_literal_internal_quote_does_not_hide_semantic_change(self):
        record, state = self.fixture('const TEXT: &str = r#"before"//one"#;\n',
                                     'const TEXT: &str = r#"before"//two"#;\n')
        result = _sealed_classification(record, state)
        self.assertEqual("semantic", result["kind"])
        self.assertEqual("non_comment_token_change", result["reason"])

    def test_comment_line_change_with_line_macro_requires_compilation(self):
        record, state = self.fixture("fn value() -> u32 { line!() }\n",
                                     "// moves source location\nfn value() -> u32 { line!() }\n")
        result = _sealed_classification(record, state)
        self.assertEqual("semantic", result["kind"])
        self.assertEqual("line_sensitive_macro", result["reason"])

    def test_unchanged_module_can_embed_the_changed_file(self):
        record, state = self.fixture("fn value() {} // before\n", "fn value() {} // after\n")
        caller = b'const TEXT: &str = include_str!("lib.rs");\n'
        caller_hash = hashlib.sha256(caller).hexdigest()
        objects = Path(record["payload"]["objectRoot"]) / "inputs" / "objects"
        (objects / caller_hash).write_bytes(caller)
        record["payload"]["sourceManifest"]["entries"].append({
            "path": "src/caller.rs", "status": "present", "beforeHash": caller_hash,
            "sha256": caller_hash, "objectDigest": caller_hash})
        result = _sealed_classification(record, state)
        self.assertEqual("semantic", result["kind"])
        self.assertEqual("source_embedding_in_input_closure", result["reason"])

    def test_macro_lookalike_in_raw_literal_does_not_trigger_macro_guard(self):
        source = 'const TEXT: &str = r#"include_str!(\"x\") line!()"#;\n'
        record, state = self.fixture(source + "// before\n", source + "// after\n")
        self.assertEqual("comments_only", _sealed_classification(record, state)["kind"])


if __name__ == "__main__":
    unittest.main()
