"""Source files must remain visible to ordinary Git status/add operations."""
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]
# Removed with L4; these exact historical outputs do not allow other caches.
LEGACY_OUTPUTS = {
    "zircon_plugins/rendering/features/contact_shadow/runtime/.zircon-cache/shader_variants/v1/11/1171588ec74b24db4d0fa1713feaa266d70f856d5a50466b4d91de94ae01a58e.meta",
    "zircon_plugins/rendering/features/contact_shadow/runtime/.zircon-cache/shader_variants/v1/11/1171588ec74b24db4d0fa1713feaa266d70f856d5a50466b4d91de94ae01a58e.wgsl.zst",
}


class GitignoreContractTests(unittest.TestCase):
    def test_tracked_files_are_not_ignored(self):
        paths = subprocess.check_output(["git", "ls-files", "-ci", "--exclude-standard", "-z"], cwd=ROOT)
        self.assertEqual(set(paths.decode().split("\0")) - {""} - LEGACY_OUTPUTS, set())

    def test_new_sources_are_visible(self):
        for path in ("tools/tests/new_test.py", ".codex/skills/new/SKILL.md", "examples/woc/new.rs", "examples/vampire/new.txt", "third_party/new/CMakeLists.txt", "docs/layout/new.zui"):
            result = subprocess.run(["git", "check-ignore", "--no-index", "--", path], cwd=ROOT, capture_output=True)
            self.assertEqual(result.returncode, 1, (path, result.stdout))

    def test_runtime_state_stays_ignored(self):
        for path in (".codex/state/new.json", ".codex/outbox/new.json", ".jenkins/state/new.json"):
            self.assertEqual(subprocess.run(["git", "check-ignore", "--no-index", "--", path], cwd=ROOT, capture_output=True).returncode, 0, path)
