from __future__ import annotations

import unittest

from tools import frameworks_01_resource_hard_cut_patch as patch_owner


BASE = """[workspace]
members = [
    "zircon_runtime/crates/zr_math",
]

[workspace.dependencies]
zr_math = { path = "zircon_runtime/crates/zr_math", default-features = false }
"""


class Frameworks01ResourceHardCutPreappliedTests(unittest.TestCase):
    def test_accepts_only_complete_exact_workspace_wiring(self) -> None:
        wired = patch_owner._patch_workspace_manifest(BASE)

        self.assertEqual(wired, patch_owner._patch_workspace_manifest(wired))

    def test_preserves_crlf_for_preapplied_workspace_wiring(self) -> None:
        source = BASE.replace("\n", "\r\n")
        wired = patch_owner._patch_workspace_manifest(source)

        self.assertIn('    "zircon_runtime/crates/zr_resource",\r\n', wired)
        self.assertIn(
            'zr_resource = { path = "zircon_runtime/crates/zr_resource", '
            'default-features = false }\r\n',
            wired,
        )
        self.assertEqual(wired, patch_owner._patch_workspace_manifest(wired))
        self.assertNotIn("\n", wired.replace("\r\n", ""))

    def test_rejects_partial_workspace_wiring(self) -> None:
        partial = BASE.replace(
            '    "zircon_runtime/crates/zr_math",\n',
            '    "zircon_runtime/crates/zr_math",\n'
            '    "zircon_runtime/crates/zr_resource",\n',
        )

        with self.assertRaises(patch_owner.HardCutPatchError) as raised:
            patch_owner._patch_workspace_manifest(partial)

        self.assertIn("partial", str(raised.exception))

    def test_write_set_requires_changed_or_exact_preapplied_path(self) -> None:
        planned = ["Cargo.toml", "zircon_runtime/Cargo.toml"]
        changes = [{"path": "zircon_runtime/Cargo.toml"}]
        before = {"Cargo.toml": "wired"}
        after = {"Cargo.toml": "wired", "zircon_runtime/Cargo.toml": "changed"}

        preapplied = patch_owner._verify_composed_write_set(
            planned, changes, before, after
        )

        self.assertEqual(["Cargo.toml"], preapplied)
        with self.assertRaises(patch_owner.HardCutPatchError):
            patch_owner._verify_composed_write_set(planned, changes, {}, after)


if __name__ == "__main__":
    unittest.main()
