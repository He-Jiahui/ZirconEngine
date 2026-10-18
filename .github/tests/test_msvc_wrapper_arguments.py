"""Execute the real Windows/MSVC wrapper without invoking a compiler."""

import os
from pathlib import Path
import subprocess
import sys
import unittest


ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(os.name == "nt", "requires Windows with VS x64 tools and pwsh")
class MsvcWrapperArgumentsTests(unittest.TestCase):
    def invoke(self, arguments):
        return subprocess.run(
            ["pwsh", "-NoProfile", "-NonInteractive", "-File",
             str(ROOT / ".github/scripts/invoke-cargo-msvc.ps1"), *arguments],
            cwd=ROOT, text=True, capture_output=True, timeout=60, check=False,
        )

    def test_native_options_are_forwarded_without_powershell_parameter_binding(self):
        expected = ["check", "-p", "zircon_app", "--locked", "--", "--list", "path with spaces"]
        code = (
            "import sys; expected=" + repr(expected) + "; "
            "assert sys.argv[1:]==expected, repr(sys.argv[1:]); "
            "print('native argument forwarding verified')"
        )
        result = self.invoke([sys.executable, "-c", code, *expected])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("native argument forwarding verified", result.stdout)

    def test_filtered_test_wrapper_lists_and_requires_a_passed_summary(self):
        code = (
            "import sys; "
            "assert sys.argv[1:] in (['test', '-p', 'example'], ['test', '-p', 'example', '--', '--list']); "
            "print('example: test' if '--list' in sys.argv[1:] else 'test result: ok. 1 passed; 0 failed;')"
        )
        result = self.invoke(["-RequireMatchingTests", sys.executable, "-c", code, "test", "-p", "example"])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("Verified 1 matching test(s)", result.stdout)

    def test_filtered_test_wrapper_rejects_zero_matches(self):
        result = self.invoke(["-RequireMatchingTests", sys.executable, "-c", "print('0 tests')"])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("matched zero tests", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
