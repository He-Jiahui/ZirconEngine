from __future__ import annotations

import hashlib
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.source.unified_patch import render_patch_bytes


def sha(data):
    return None if data is None else hashlib.sha256(data).hexdigest()


def change(path, old, new):
    return f"diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n@@ -1 +1 @@\n-{old}\n+{new}\n"


class PatchBytesTests(unittest.TestCase):
    def test_memory_render_reads_no_checkout_and_preserves_input_bytes(self):
        inputs = {"src/a.rs": b"old\n"}
        with patch.object(Path, "read_bytes", side_effect=AssertionError("filesystem read")), \
                patch.object(Path, "resolve", side_effect=AssertionError("filesystem resolve")):
            result = render_patch_bytes(change("src/a.rs", "old", "new"), inputs,
                                        {"src/a.rs": sha(inputs["src/a.rs"])})
        self.assertEqual({"src/a.rs": b"new\n"}, result)
        self.assertEqual({"src/a.rs": b"old\n"}, inputs)

    def test_lf_crlf_and_final_no_newline_are_exact(self):
        diff = ("diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -1,2 +1,2 @@\n"
                "-old\n+new\n last\n\\ No newline at end of file\n")
        for before, after in [(b"old\nlast", b"new\nlast"), (b"old\r\nlast", b"new\r\nlast")]:
            with self.subTest(before=before):
                self.assertEqual({"a": after}, render_patch_bytes(diff, {"a": before}, {"a": sha(before)}))

    def test_addition_and_complete_deletion_return_bytes_and_null(self):
        diff = ("diff --git /dev/null b/new.rs\nnew file mode 100644\n--- /dev/null\n+++ b/new.rs\n"
                "@@ -0,0 +1 @@\n+new\n"
                "diff --git a/old.rs /dev/null\ndeleted file mode 100644\n--- a/old.rs\n+++ /dev/null\n"
                "@@ -1 +0,0 @@\n-old\n")
        self.assertEqual({"new.rs": b"new\n", "old.rs": None},
                         render_patch_bytes(diff, {"new.rs": None, "old.rs": b"old\n"},
                                            {"new.rs": None, "old.rs": sha(b"old\n")}))

    def test_multi_file_hash_or_context_failure_returns_no_partial_result(self):
        diff = change("a", "old", "new") + change("b", "old", "new")
        inputs = {"a": b"old\n", "b": b"foreign\n"}
        for hashes in [{"a": sha(inputs["a"]), "b": sha(b"old\n")},
                       {path: sha(data) for path, data in inputs.items()}]:
            with self.subTest(hashes=hashes), self.assertRaises(JenkinsError):
                render_patch_bytes(diff, inputs, hashes)
        self.assertEqual({"a": b"old\n", "b": b"foreign\n"}, inputs)

    def test_extra_inputs_case_aliases_or_unsafe_paths_are_rejected(self):
        diff = change("a", "old", "new")
        for inputs, hashes in [({"a": b"old\n", "b": b"other\n"}, {"a": sha(b"old\n")}),
                               ({"a": b"old\n", "A": b"old\n"}, {"a": sha(b"old\n")}),
                               ({"../a": b"old\n"}, {"../a": sha(b"old\n")})]:
            with self.subTest(inputs=inputs), self.assertRaises(JenkinsError):
                render_patch_bytes(diff, inputs, hashes)

    def test_binary_preimage_and_malformed_delete_are_rejected(self):
        with self.assertRaises(JenkinsError) as error:
            render_patch_bytes(change("a", "old", "new"), {"a": b"\xff\n"}, {"a": sha(b"\xff\n")})
        self.assertEqual("patch_binary", error.exception.code)
        deletion = "diff --git a/a /dev/null\n--- a/a\n+++ /dev/null\n@@ -1 +1 @@\n-old\n+new\n"
        with self.assertRaises(JenkinsError):
            render_patch_bytes(deletion, {"a": b"old\n"}, {"a": sha(b"old\n")})

    def test_missing_or_false_no_newline_markers_fail_strict_context(self):
        with self.assertRaises(JenkinsError):
            render_patch_bytes(change("a", "old", "new"), {"a": b"old"}, {"a": sha(b"old")})
        diff = change("a", "old", "new").replace("-old\n", "-old\n\\ No newline at end of file\n")
        with self.assertRaises(JenkinsError):
            render_patch_bytes(diff, {"a": b"old\n"}, {"a": sha(b"old\n")})


if __name__ == "__main__":
    unittest.main()
