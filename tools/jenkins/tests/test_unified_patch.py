from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.source.unified_patch import materialize_patch, parse_unified_patch


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class UnifiedPatchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "src").mkdir()

    def tearDown(self):
        self.temp.cleanup()

    def apply(self, path: str, before: bytes, patch: str):
        target = self.root / path
        if before is not None:
            target.write_bytes(before)
        return materialize_patch(self.root, patch, {path: sha(before) if before is not None else None})

    def test_single_line_preserves_unmodified_lines(self):
        before = b"one\ntwo\nthree\n"
        patch = """diff --git a/src/a.txt b/src/a.txt
index 1111111..2222222 100644
--- a/src/a.txt
+++ b/src/a.txt
@@ -1,3 +1,3 @@
 one
-two
+TWO
 three
"""
        result = self.apply("src/a.txt", before, patch)
        self.assertEqual(result["src/a.txt"], "one\nTWO\nthree\n")

    def test_multiple_hunks_and_new_delete(self):
        before = b"a\nb\nc\nd\ne\n"
        patch = """diff --git a/src/a.txt b/src/a.txt
--- a/src/a.txt
+++ b/src/a.txt
@@ -1,2 +1,2 @@
 a
-b
+B
@@ -4,2 +4,2 @@
 d
-e
+E
"""
        self.assertEqual(self.apply("src/a.txt", before, patch)["src/a.txt"], "a\nB\nc\nd\nE\n")
        new = """diff --git /dev/null b/src/new.txt
new file mode 100644
--- /dev/null
+++ b/src/new.txt
@@ -0,0 +1 @@
+new
"""
        self.assertEqual(self.apply("src/new.txt", None, new)["src/new.txt"], "new\n")
        delete = """diff --git a/src/new.txt /dev/null
deleted file mode 100644
--- a/src/new.txt
+++ /dev/null
@@ -1 +0,0 @@
-new
"""
        (self.root / "src/new.txt").write_bytes(b"new\n")
        self.assertIsNone(materialize_patch(self.root, delete, {"src/new.txt": sha(b"new\n")})["src/new.txt"])

    def test_no_newline_marker_and_crlf_are_supported(self):
        before = b"old\r\nlast"
        patch = """diff --git a/src/a.txt b/src/a.txt
--- a/src/a.txt
+++ b/src/a.txt
@@ -1,2 +1,2 @@
-old
+new
 last
\\ No newline at end of file
"""
        self.assertEqual(self.apply("src/a.txt", before, patch)["src/a.txt"], "new\r\nlast")

    def test_hash_conflict_does_not_touch_file(self):
        target = self.root / "src/a.txt"; target.write_bytes(b"actual\n")
        original = target.read_bytes()
        patch = """diff --git a/src/a.txt b/src/a.txt
--- a/src/a.txt
+++ b/src/a.txt
@@ -1 +1 @@
-actual
+changed
"""
        with self.assertRaisesRegex(JenkinsError, "before hash"):
            materialize_patch(self.root, patch, {"src/a.txt": sha(b"other\n")})
        self.assertEqual(target.read_bytes(), original)

    def test_malformed_rename_path_and_case_duplicate_rejected(self):
        with self.assertRaises(JenkinsError):
            parse_unified_patch("diff --git a/x b/x\nrename from x\nrename to y\n")
        traversal = "diff --git a/../x b/../x\n--- a/../x\n+++ b/../x\n@@ -1 +1 @@\n-x\n+y\n"
        with self.assertRaises(JenkinsError):
            materialize_patch(self.root, traversal, {"../x": None})
        duplicate = """diff --git a/src/a b/src/a
--- a/src/a
+++ b/src/a
@@ -0,0 +1 @@
+a
diff --git a/src/A b/src/A
--- a/src/A
+++ b/src/A
@@ -0,0 +1 @@
+A
"""
        with self.assertRaisesRegex(JenkinsError, "duplicate"):
            parse_unified_patch(duplicate)

    def test_zero_count_insertions_at_middle_and_end(self):
        before = b"a\nb\n"
        middle = """diff --git a/src/a.txt b/src/a.txt
--- a/src/a.txt
+++ b/src/a.txt
@@ -1,0 +2,1 @@
+between
"""
        self.assertEqual(self.apply("src/a.txt", before, middle)["src/a.txt"], "a\nbetween\nb\n")
        end = """diff --git a/src/a.txt b/src/a.txt
--- a/src/a.txt
+++ b/src/a.txt
@@ -2,0 +3,1 @@
+end
"""
        self.assertEqual(self.apply("src/a.txt", before, end)["src/a.txt"], "a\nb\nend\n")

    def test_partial_delete_and_context_newline_mismatch_rejected(self):
        before = b"a\nb\n"
        partial = """diff --git a/src/a.txt /dev/null
deleted file mode 100644
--- a/src/a.txt
+++ /dev/null
@@ -1,1 +0,0 @@
-a
"""
        with self.assertRaisesRegex(JenkinsError, "complete"):
            self.apply("src/a.txt", before, partial)
        mismatch = """diff --git a/src/a.txt b/src/a.txt
--- a/src/a.txt
+++ b/src/a.txt
@@ -1 +1 @@
-a
+z
\\ No newline at end of file
"""
        with self.assertRaisesRegex(JenkinsError, "newline"):
            self.apply("src/a.txt", before, mismatch)

    def test_windows_reserved_names_and_header_rename_rejected(self):
        reserved = """diff --git a/con b/con
--- a/con
+++ b/con
@@ -0,0 +1 @@
+x
"""
        with self.assertRaisesRegex(JenkinsError, "safe"):
            parse_unified_patch(reserved)
        rename = """diff --git a/src/a b/src/b
--- a/src/a
+++ b/src/b
@@ -1 +1 @@
-a
+b
"""
        with self.assertRaisesRegex(JenkinsError, "rename"):
            parse_unified_patch(rename)


if __name__ == "__main__":
    unittest.main()
