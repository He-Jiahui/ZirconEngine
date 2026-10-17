"""Boundary cases for the Windows engine path budget."""
import unittest

from tools.audits.path_names import audit, owned_path, path_units, RELATIVE_LIMIT


class PathNamesTests(unittest.TestCase):
    def test_utf16_budget_counts_surrogate_pairs(self):
        self.assertEqual(path_units("a\U0001f600"), 3)

    def test_relative_and_checkout_limits(self):
        path = "a" * RELATIVE_LIMIT
        self.assertTrue(audit([path])["passed"])
        self.assertFalse(audit([path + "a"])["passed"])
        self.assertFalse(audit([path], "E:/" + "x" * 49)["passed"])

    def test_generated_outputs_are_excluded_but_source_target_modules_are_owned(self):
        self.assertFalse(owned_path("zircon_runtime/target/debug/a"))
        self.assertFalse(owned_path("zircon_plugins/a/.zircon-cache/a"))
        self.assertTrue(owned_path("zircon_runtime/src/export/target/mod.rs"))


if __name__ == "__main__":
    unittest.main()
