from collections import defaultdict
import unittest

from tools.audits.repair_doc_paths import repair_text, replacement


class RepairDocPathsTests(unittest.TestCase):
    def test_file_to_module_move(self):
        self.assertEqual(replacement("runtime/src/owner.rs", {"runtime/src/owner/mod.rs"}, {}), "runtime/src/owner/mod.rs")

    def test_ambiguous_and_external_paths_are_not_guessed(self):
        suffixes = {("runtime", ("tests", "cases.rs")): {"runtime/src/a/tests/cases.rs", "runtime/src/b/tests/cases.rs"}}
        self.assertIsNone(replacement("runtime/src/old/tests/cases.rs", set(), suffixes))
        self.assertIsNone(replacement("../runtime/src/mod.rs", set(), suffixes))

    def test_retires_only_stale_declarations_and_preserves_prose_commands(self):
        source = "---\nrelated_code:\n  - runtime/src/owner.rs\n  - gone.rs\ntests:\n  - cargo test -p runtime\n---\nHistorical gone.rs remains in prose.\n"
        invalid = {("related_code", "runtime/src/owner.rs"): "missing path", ("related_code", "gone.rs"): "missing path"}
        updated, counts = repair_text(source, invalid, {"runtime/src/owner/mod.rs"}, defaultdict(set))
        self.assertIn("  - runtime/src/owner/mod.rs", updated)
        self.assertNotIn("  - gone.rs", updated)
        self.assertIn("cargo test -p runtime", updated)
        self.assertIn("Historical gone.rs remains in prose.", updated)
        self.assertEqual(counts, {"repaired": 1, "retired": 1})
