from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Sequence
from unittest.mock import patch

from tools import frameworks_01_resource_hard_cut_manifest as manifest_owner
from tools import frameworks_01_resource_consumer_manifest as consumer_owner


def _test_temp_root() -> Path:
    configured = os.environ.get("ZIRCON_TEST_TEMP_ROOT")
    if configured:
        return Path(configured) / "frameworks01-resource-hard-cut-manifest-tests"
    for drive in ("F:/", "D:/", "E:/"):
        root = Path(drive)
        if root.exists():
            return root / "zircon-profiles/frameworks01-resource-hard-cut-manifest-tests"
    return Path(tempfile.gettempdir()) / "frameworks01-resource-hard-cut-manifest-tests"


TEST_TEMP_ROOT = _test_temp_root()


class Frameworks01ResourceHardCutManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        TEST_TEMP_ROOT.mkdir(parents=True, exist_ok=True)
        self.temporary_directory = tempfile.TemporaryDirectory(dir=TEST_TEMP_ROOT)
        self.repo_root = Path(self.temporary_directory.name)
        self._git("init", "--quiet")
        self._write(".gitignore", "ignored.md\n")

        fixed_sources = {
            "Cargo.toml": "[workspace]\nmembers = []\n",
            "Cargo.lock": "version = 3\n",
            "zircon_runtime/Cargo.toml": "[package]\nname = 'zircon_runtime'\n",
            "zircon_runtime_interface/Cargo.toml": (
                "[package]\nname = 'zircon_runtime_interface'\n"
            ),
            "zircon_runtime/src/lib.rs": "pub mod core;\n",
            "zircon_runtime/src/core/mod.rs": "pub mod resource;\n",
            "zircon_runtime/src/core/resource/mod.rs": "mod manager;\n",
            "zircon_runtime_interface/src/lib.rs": "pub mod resource;\n",
            "zircon_runtime_interface/src/resource/mod.rs": "mod locator;\n",
            "zircon_runtime_interface/src/tests/mod.rs": (
                "mod resource_contracts;\n"
            ),
            "zircon_runtime_interface/src/tests/resource_contracts.rs": (
                "#[test]\nfn resource_contract() {}\n"
            ),
            "zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs": (
                "#[test]\nfn resource_foundation() {}\n"
            ),
        }
        for path, source in fixed_sources.items():
            self._write(path, source)

        self._write(
            "zircon_runtime/src/core/resource/manager.rs",
            "pub struct ResourceManager;\n",
        )
        self._write(
            "zircon_runtime_interface/src/resource/locator.rs",
            "pub struct ResourceLocator;\n",
        )
        self._write(
            "zircon_app/src/consumer.rs",
            "use zircon_runtime::core::resource::ResourceManager;\n",
        )
        self._write(
            "zircon_editor/src/nested.rs",
            "use zircon_runtime::core::{resource::ResourceRuntime};\n",
        )
        self._write(
            "examples/resource_example.rs",
            "use zircon_runtime::core::resource::ResourceRegistry;\n",
        )
        self._write(
            "docs/resource.md",
            "The product facade is `zircon_runtime::core::resource`.\n",
        )
        self._write(
            "tools/tests/resource_guard.py",
            "RESOURCE_ROOT = 'zircon_runtime/src/core/resource'\n",
        )
        self._write("docs/unrelated.md", "No foundation reference.\n")
        self._write(
            "docs/ignored.md",
            "Ignored zircon_runtime::core::resource reference.\n",
        )
        deleted_path = self._write(
            "docs/deleted.md",
            "Deleted zircon_runtime::core::resource reference.\n",
        )
        self._git("add", ".")
        self._git(
            "-c",
            "core.hooksPath=NUL",
            "-c",
            "user.name=Zircon Fixture",
            "-c",
            "user.email=zircon-fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        )
        deleted_path.unlink()
        self._write(
            "examples/untracked_resource.rs",
            "type Id = zircon_runtime::core::resource::ResourceId;\n",
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _git(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", "-C", str(self.repo_root), *arguments],
            check=True,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )

    def _write(self, relative_path: str, source: str) -> Path:
        path = self.repo_root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source, encoding="utf-8")
        return path

    def test_composes_atomic_inputs_with_merged_roles(self) -> None:
        report = manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual(20, report["atomic_input_count"])
        self.assertEqual(4, report["consumer_count"])
        self.assertEqual(
            {
                "fixed_workspace_input": {"bytes": unittest.mock.ANY, "files": 12},
                "interface_resource_dto": {"bytes": unittest.mock.ANY, "files": 2},
                "resource_implementation_owner": {
                    "bytes": unittest.mock.ANY,
                    "files": 2,
                },
                "rust_consumer": {"bytes": unittest.mock.ANY, "files": 4},
                "textual_reference": {"bytes": unittest.mock.ANY, "files": 4},
            },
            report["role_summaries"],
        )
        entries = {entry["path"]: entry for entry in report["inputs"]}
        self.assertEqual(
            ["fixed_workspace_input", "resource_implementation_owner"],
            entries["zircon_runtime/src/core/resource/mod.rs"]["roles"],
        )
        self.assertEqual(
            ["rust_consumer", "textual_reference"],
            entries["examples/resource_example.rs"]["roles"],
        )
        self.assertNotIn("docs/unrelated.md", entries)
        self.assertNotIn("docs/ignored.md", entries)
        self.assertNotIn("docs/deleted.md", entries)
        self.assertEqual(
            list(manifest_owner.FUTURE_CRATE_PATHS), report["future_paths"]
        )
        for field in (
            "atomic_input_manifest_sha256",
            "consumer_manifest_sha256",
            "supplemental_candidate_manifest_sha256",
        ):
            self.assertEqual(64, len(report[field]))
        self.assertEqual(
            {
                "consumer_snapshot": True,
                "future_paths_absent": True,
                "supplemental_candidates": True,
                "supplemental_content": True,
                "supplemental_terminal_snapshot": True,
            },
            report["stability"],
        )

    def test_report_and_explicit_output_are_deterministic(self) -> None:
        first = manifest_owner.build_resource_hard_cut_manifest(self.repo_root)
        second = manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual(first, second)
        output = Path(self.temporary_directory.name) / "reports" / "hard-cut.json"
        manifest_owner.write_resource_hard_cut_manifest(first, output)
        self.assertEqual(first, json.loads(output.read_text(encoding="utf-8")))
        self.assertTrue(output.read_bytes().endswith(b"\n"))

    def test_rejects_supplemental_content_drift(self) -> None:
        original_finalize = manifest_owner.finalize_resource_consumer_snapshot

        def mutate_then_finalize(
            repo_root: Path, capture: object, **arguments: object
        ) -> object:
            self._write(
                "docs/resource.md",
                "Changed `zircon_runtime::core::resource` reference.\n",
            )
            return original_finalize(repo_root, capture, **arguments)

        with patch.object(
            manifest_owner,
            "finalize_resource_consumer_snapshot",
            side_effect=mutate_then_finalize,
        ):
            with self.assertRaises(
                manifest_owner.HardCutManifestStabilityError
            ) as raised:
                manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual("supplemental_content_changed", raised.exception.reason)
        self.assertEqual(["docs/resource.md"], raised.exception.changed_paths)

    def test_rejects_supplemental_candidate_set_drift(self) -> None:
        stable_candidates = manifest_owner._supplemental_candidate_paths(
            self.repo_root
        )
        changed_candidates = stable_candidates + (Path("docs/late.md"),)

        with patch.object(
            manifest_owner,
            "_supplemental_candidate_paths",
            side_effect=(stable_candidates, changed_candidates),
        ):
            with self.assertRaises(
                manifest_owner.HardCutManifestStabilityError
            ) as raised:
                manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual("supplemental_candidate_set_changed", raised.exception.reason)
        self.assertEqual(["docs/late.md"], raised.exception.changed_paths)

    def test_supplemental_candidates_follow_dirty_textual_references(self) -> None:
        unrelated = Path("docs/unrelated.md")

        initial = manifest_owner._supplemental_candidate_paths(self.repo_root)

        self.assertNotIn(unrelated, initial)
        self._write(
            unrelated.as_posix(),
            "The owner is `zircon_runtime::core::resource`.\n",
        )

        changed = manifest_owner._supplemental_candidate_paths(self.repo_root)

        self.assertIn(unrelated, changed)

    def test_supplemental_candidates_include_ignored_physical_owner(self) -> None:
        ignored_owner = Path(
            "zircon_runtime/src/core/resource/local_generated_owner.rs"
        )
        self._write(
            ".gitignore",
            "ignored.md\nzircon_runtime/src/core/resource/local_generated_owner.rs\n",
        )
        self._write(ignored_owner.as_posix(), "pub struct LocalGeneratedOwner;\n")

        candidates = manifest_owner._supplemental_candidate_paths(self.repo_root)

        self.assertIn(ignored_owner, candidates)

    def test_accepts_unrelated_text_drift_during_supplemental_seal(self) -> None:
        original_finalize = manifest_owner.finalize_resource_consumer_snapshot

        def mutate_then_finalize(
            repo_root: Path, capture: object, **arguments: object
        ) -> object:
            self._write("docs/unrelated.md", "Still unrelated after capture.\n")
            return original_finalize(repo_root, capture, **arguments)

        with patch.object(
            manifest_owner,
            "finalize_resource_consumer_snapshot",
            side_effect=mutate_then_finalize,
        ):
            report = manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual(4, report["consumer_count"])

    def test_rejects_late_textual_reference_after_content_seal(self) -> None:
        original_changed_paths = manifest_owner._changed_supplemental_paths

        def seal_then_add_reference(
            repo_root: Path, fingerprints: Sequence[dict[str, object]]
        ) -> list[str]:
            changed_paths = original_changed_paths(repo_root, fingerprints)
            self._write(
                "docs/late-resource-owner.md",
                "The owner is `zircon_runtime::core::resource`.\n",
            )
            return changed_paths

        with patch.object(
            manifest_owner,
            "_changed_supplemental_paths",
            side_effect=seal_then_add_reference,
        ):
            with self.assertRaises(
                manifest_owner.HardCutManifestStabilityError
            ) as raised:
                manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual("supplemental_candidate_set_changed", raised.exception.reason)
        self.assertEqual(
            ["docs/late-resource-owner.md"], raised.exception.changed_paths
        )

    def test_rejects_existing_future_crate_path(self) -> None:
        collision = self.repo_root / manifest_owner.FUTURE_CRATE_PATHS[0]
        collision.parent.mkdir(parents=True, exist_ok=True)
        collision.write_text("[package]\nname = 'zr_resource'\n", encoding="utf-8")

        with self.assertRaises(manifest_owner.HardCutManifestError) as raised:
            manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertIn("future hard-cut path already exists", str(raised.exception))

    def test_accepts_unrelated_head_drift_after_supplemental_content_seal(self) -> None:
        original_changed_paths = manifest_owner._changed_supplemental_paths
        committed = False

        def seal_then_commit(
            repo_root: Path, fingerprints: Sequence[dict[str, object]]
        ) -> list[str]:
            nonlocal committed
            changed_paths = original_changed_paths(repo_root, fingerprints)
            if committed:
                return changed_paths
            self._write("late-head-change.txt", "changes HEAD after both seals\n")
            self._git("add", "late-head-change.txt")
            self._git(
                "-c",
                "core.hooksPath=NUL",
                "-c",
                "user.name=Zircon Fixture",
                "-c",
                "user.email=zircon-fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "late head change",
            )
            committed = True
            return changed_paths

        with patch.object(
            manifest_owner,
            "_changed_supplemental_paths",
            side_effect=seal_then_commit,
        ):
            report = manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual(4, report["consumer_count"])
        self.assertNotIn("head", report)

    def test_rejects_late_resource_consumer_after_supplemental_seal(self) -> None:
        original_changed_paths = manifest_owner._changed_supplemental_paths

        def seal_then_add_consumer(
            repo_root: Path, fingerprints: Sequence[dict[str, object]]
        ) -> list[str]:
            changed_paths = original_changed_paths(repo_root, fingerprints)
            self._write(
                "zircon_app/src/late_consumer.rs",
                "use zircon_runtime::core::resource::ResourceId;\n",
            )
            return changed_paths

        with patch.object(
            manifest_owner,
            "_changed_supplemental_paths",
            side_effect=seal_then_add_consumer,
        ):
            with self.assertRaises(consumer_owner.ManifestStabilityError) as raised:
                manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual("candidate_set_changed", raised.exception.reason)
        self.assertEqual(
            ["zircon_app/src/late_consumer.rs"], raised.exception.changed_paths
        )

    def test_composer_uses_exactly_three_combined_reference_inventory_passes(
        self,
    ) -> None:
        with patch.object(
            manifest_owner,
            "_git_resource_reference_inventory",
            wraps=manifest_owner._git_resource_reference_inventory,
        ) as reference_inventory:
            report = manifest_owner.build_resource_hard_cut_manifest(self.repo_root)

        self.assertEqual(4, report["consumer_count"])
        self.assertEqual(3, reference_inventory.call_count)


if __name__ == "__main__":
    unittest.main()
