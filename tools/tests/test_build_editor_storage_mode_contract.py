from pathlib import Path
import re
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_EDITOR = REPO_ROOT / "tools" / "build-editor.ps1"


class BuildEditorStorageModeContractTests(unittest.TestCase):
    def test_storage_mode_is_validated_and_forwarded_to_both_managed_builds(self) -> None:
        source = BUILD_EDITOR.read_text(encoding="utf-8")

        self.assertRegex(
            source,
            re.compile(
                r"\[ValidateSet\(\"reuse\", \"compact\", \"diagnostic\"\)\]\s*"
                r"\[string\]\$StorageMode\s*=\s*\"reuse\"",
                re.MULTILINE,
            ),
        )
        self.assertIn("$arguments += @('-StorageMode', $StorageMode)", source)
        self.assertEqual(source.count("-StorageMode $StorageMode `"), 2)


if __name__ == "__main__":
    unittest.main()
