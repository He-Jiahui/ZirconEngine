from __future__ import annotations

import unittest

from tools.jenkins.workflow.classification import classify_change, classify_rust_source


class WorkflowClassificationTests(unittest.TestCase):
    def test_plain_comment_is_comments_only(self):
        result = classify_rust_source("// explanatory note\nlet value = 1;\n", changed_lines=[1])
        self.assertEqual(result["kind"], "comments_only")

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
