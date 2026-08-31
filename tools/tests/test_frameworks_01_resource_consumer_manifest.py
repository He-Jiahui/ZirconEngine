from __future__ import annotations

import json
import os
import subprocess
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from tools import frameworks_01_resource_consumer_manifest as manifest_owner


def _test_temp_root() -> Path:
    configured = os.environ.get("ZIRCON_TEST_TEMP_ROOT")
    if configured:
        return Path(configured) / "frameworks01-resource-consumer-manifest-tests"
    for drive in ("F:/", "D:/", "E:/"):
        root = Path(drive)
        if root.exists():
            return root / "zircon-profiles/frameworks01-resource-consumer-manifest-tests"
    return Path(tempfile.gettempdir()) / "frameworks01-resource-consumer-manifest-tests"


TEST_TEMP_ROOT = _test_temp_root()


class Frameworks01ResourceConsumerManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        TEST_TEMP_ROOT.mkdir(parents=True, exist_ok=True)
        self.temporary_directory = tempfile.TemporaryDirectory(dir=TEST_TEMP_ROOT)
        self.repo_root = Path(self.temporary_directory.name)

        self._git("init", "--quiet")
        self._write(".gitignore", "ignored.rs\n")
        self._write(
            "zircon_runtime/src/asset/internal.rs",
            "use crate::core::{resource::ResourceLocator};\n",
        )
        self._write(
            "zircon_editor/src/external.rs",
            "use zircon_runtime::core::{resource::ResourceManager};\n",
        )
        self._write(
            "zircon_app/src/both.rs",
            "use zircon_runtime::core::resource::ResourceRuntime;\n",
        )
        self._write(
            "zircon_runtime/src/core/resource/owner.rs",
            "use crate::core::resource::ResourceSnapshot;\n",
        )
        self._write(
            "zircon_runtime/src/asset/comments.rs",
            "// use crate::core::resource::ResourceId;\n"
            'const PATH: &str = "zircon_runtime::core::resource::ResourceId";\n',
        )
        self._write(
            "zircon_runtime/src/asset/unrelated.rs",
            "pub struct UnrelatedAsset;\n",
        )
        deleted_path = self._write(
            "zircon_runtime/src/asset/deleted.rs",
            "use crate::core::resource::ResourceId;\n",
        )
        self._write(
            "zircon_plugins/sample/runtime/src/ignored.rs",
            "type Id = zircon_runtime::core::resource::ResourceId;\n",
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
            "zircon_plugins/sample/runtime/src/untracked.rs",
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

    def test_collects_stable_literal_and_structured_consumer_union(self) -> None:
        report = manifest_owner.build_resource_consumer_manifest(self.repo_root)

        self.assertEqual(5, report["candidate_count"])
        self.assertEqual(4, report["consumer_count"])
        self.assertEqual(
            {
                "both": 1,
                "literal": 2,
                "literal_only": 1,
                "structured": 3,
                "structured_only": 2,
            },
            report["match_counts"],
        )
        self.assertEqual(
            [
                "zircon_app/src/both.rs",
                "zircon_editor/src/external.rs",
                "zircon_plugins/sample/runtime/src/untracked.rs",
                "zircon_runtime/src/asset/internal.rs",
            ],
            [entry["path"] for entry in report["consumers"]],
        )
        self.assertEqual(
            {"candidate_set": True, "source_content": True},
            report["stability"],
        )
        self.assertNotIn("head", report)
        self.assertEqual(
            ["zircon_runtime/src/core/resource"], report["excluded_owner_roots"]
        )
        for field in ("candidate_manifest_sha256", "consumer_manifest_sha256"):
            self.assertEqual(64, len(report[field]))

    def test_excludes_tracked_files_deleted_from_current_worktree(self) -> None:
        candidates = manifest_owner._git_resource_token_candidates(self.repo_root)

        self.assertNotIn(Path("zircon_runtime/src/asset/deleted.rs"), candidates)

    def test_token_prefilter_tracks_dirty_and_untracked_resource_sources(self) -> None:
        unrelated = Path("zircon_runtime/src/asset/unrelated.rs")
        untracked_consumer = Path("zircon_plugins/sample/runtime/src/untracked.rs")

        initial = manifest_owner._git_resource_token_candidates(self.repo_root)

        self.assertNotIn(unrelated, initial)
        self.assertIn(untracked_consumer, initial)

        self._write(
            unrelated.as_posix(),
            "use crate::core::resource::ResourceRegistry;\n",
        )
        untracked_unrelated = Path("zircon_app/src/untracked_unrelated.rs")
        self._write(untracked_unrelated.as_posix(), "pub struct LocalType;\n")

        changed = manifest_owner._git_resource_token_candidates(self.repo_root)

        self.assertIn(unrelated, changed)
        self.assertIn(untracked_consumer, changed)
        self.assertNotIn(untracked_unrelated, changed)

    def test_token_prefilter_reads_only_untracked_matches_in_python(self) -> None:
        tracked = Path("zircon_runtime/src/asset/internal.rs")
        untracked = Path("zircon_plugins/sample/runtime/src/untracked.rs")
        original_read_bytes = Path.read_bytes
        observed: list[Path] = []

        def observed_read_bytes(path: Path) -> bytes:
            try:
                relative = path.relative_to(self.repo_root)
            except ValueError:
                return original_read_bytes(path)
            observed.append(relative)
            return original_read_bytes(path)

        with patch.object(Path, "read_bytes", new=observed_read_bytes):
            candidates = manifest_owner._git_resource_token_candidates(self.repo_root)

        self.assertIn(tracked, candidates)
        self.assertIn(untracked, candidates)
        self.assertNotIn(tracked, observed)
        self.assertIn(untracked, observed)

    def test_combined_reference_inventory_classifies_rust_and_text(self) -> None:
        textual_reference = self._write(
            "docs/resource-owner.md",
            "The facade is `zircon_runtime::core::resource`.\n",
        ).relative_to(self.repo_root)
        self._write("docs/unrelated.md", "No matching owner token.\n")
        inventory = manifest_owner._git_resource_reference_inventory(
            self.repo_root,
            textual_roots=("docs", "examples", "tools"),
            textual_suffixes=frozenset({".md", ".py", ".rs"}),
            textual_tokens=(
                b"core/resource",
                b"core::resource",
                b"zircon_resource",
                b"zr_resource",
            ),
        )

        self.assertIn(
            Path("zircon_runtime/src/asset/internal.rs"), inventory.rust_candidates
        )
        self.assertIn(
            Path("zircon_plugins/sample/runtime/src/untracked.rs"),
            inventory.rust_candidates,
        )
        self.assertNotIn(
            Path("zircon_runtime/src/asset/unrelated.rs"), inventory.rust_candidates
        )
        self.assertEqual(
            (
                Path("zircon_plugins/sample/runtime/src/untracked.rs"),
            ),
            tuple(
                path
                for path in inventory.rust_candidates
                if path.parts[0] == "zircon_plugins"
            ),
        )
        self.assertEqual((textual_reference,), inventory.textual_candidates)

    def test_only_lexes_candidates_with_all_necessary_raw_tokens(self) -> None:
        with (
            patch.object(
                manifest_owner,
                "_rust_code_view",
                wraps=manifest_owner._rust_code_view,
            ) as code_view,
            patch.object(
                manifest_owner,
                "_rust_use_paths",
                wraps=manifest_owner._rust_use_paths,
            ) as use_paths,
        ):
            report = manifest_owner.build_resource_consumer_manifest(self.repo_root)

        self.assertEqual(5, report["candidate_count"])
        self.assertEqual(5, code_view.call_count)
        self.assertEqual(5, use_paths.call_count)

    def test_preserves_structured_raw_identifier_consumers(self) -> None:
        raw_identifier_path = self._write(
            "zircon_runtime/src/asset/raw_identifier.rs",
            "use crate::r#core::{r#resource::ResourceId};\n",
        )

        report = manifest_owner.build_resource_consumer_manifest(self.repo_root)
        consumer = next(
            entry
            for entry in report["consumers"]
            if entry["path"] == raw_identifier_path.relative_to(self.repo_root).as_posix()
        )

        self.assertFalse(consumer["literal"])
        self.assertTrue(consumer["structured"])

    def test_parallel_reads_are_bounded_and_preserve_manifest_order(self) -> None:
        original_read_bytes = Path.read_bytes
        lock = threading.Lock()
        reader_threads: set[int] = set()
        active_readers = 0
        maximum_active_readers = 0

        def observed_read_bytes(path: Path) -> bytes:
            nonlocal active_readers, maximum_active_readers
            try:
                path.relative_to(self.repo_root)
            except ValueError:
                return original_read_bytes(path)
            with lock:
                active_readers += 1
                maximum_active_readers = max(maximum_active_readers, active_readers)
                reader_threads.add(threading.get_ident())
            try:
                time.sleep(0.01)
                return original_read_bytes(path)
            finally:
                with lock:
                    active_readers -= 1

        with patch.object(Path, "read_bytes", new=observed_read_bytes):
            report = manifest_owner.build_resource_consumer_manifest(self.repo_root)

        self.assertGreater(len(reader_threads), 1)
        self.assertLessEqual(maximum_active_readers, 8)
        self.assertEqual(8, manifest_owner.READ_WORKERS)
        self.assertEqual(16, manifest_owner.READ_IN_FLIGHT)
        self.assertEqual(
            sorted(
                (entry["path"] for entry in report["consumers"]),
                key=lambda path: (path.casefold(), path),
            ),
            [entry["path"] for entry in report["consumers"]],
        )

    def test_report_and_explicit_output_are_deterministic(self) -> None:
        first = manifest_owner.build_resource_consumer_manifest(self.repo_root)
        second = manifest_owner.build_resource_consumer_manifest(self.repo_root)

        self.assertEqual(first, second)
        output = Path(self.temporary_directory.name) / "reports" / "resource.json"
        manifest_owner.write_resource_consumer_manifest(first, output)
        self.assertEqual(first, json.loads(output.read_text(encoding="utf-8")))
        self.assertTrue(output.read_bytes().endswith(b"\n"))

    def test_stable_snapshot_preserves_report_and_can_be_revalidated(self) -> None:
        snapshot = manifest_owner.build_resource_consumer_snapshot(self.repo_root)

        self.assertEqual(
            manifest_owner.build_resource_consumer_manifest(self.repo_root),
            snapshot.report,
        )
        self.assertEqual(snapshot.report["candidate_count"], len(snapshot.candidates))
        self.assertEqual(len(snapshot.candidates), len(snapshot.candidate_fingerprints))
        manifest_owner.revalidate_resource_consumer_snapshot(self.repo_root, snapshot)

        unrelated_path = self.repo_root / "zircon_runtime/src/asset/unrelated.rs"
        unrelated_path.write_text("pub struct ChangedAsset;\n", encoding="utf-8")
        self._write(
            "zircon_runtime/src/asset/late_unrelated.rs",
            "pub struct LateUnrelatedAsset;\n",
        )
        manifest_owner.revalidate_resource_consumer_snapshot(self.repo_root, snapshot)

        changed_path = self.repo_root / "zircon_runtime/src/asset/internal.rs"
        changed_path.write_text(
            "use crate::core::{resource::ResourceRegistry};\n",
            encoding="utf-8",
        )
        with self.assertRaises(manifest_owner.ManifestStabilityError) as raised:
            manifest_owner.revalidate_resource_consumer_snapshot(
                self.repo_root, snapshot
            )

        self.assertEqual("source_content_changed", raised.exception.reason)
        self.assertEqual(
            ["zircon_runtime/src/asset/internal.rs"], raised.exception.changed_paths
        )

    def test_capture_finalize_matches_public_snapshot_builder(self) -> None:
        capture = manifest_owner.capture_resource_consumer_snapshot(self.repo_root)
        finalized = manifest_owner.finalize_resource_consumer_snapshot(
            self.repo_root, capture
        )
        public = manifest_owner.build_resource_consumer_snapshot(self.repo_root)

        self.assertEqual(public.report, finalized.report)
        self.assertEqual(public.candidates, finalized.candidates)
        self.assertEqual(
            public.candidate_fingerprints,
            finalized.candidate_fingerprints,
        )

    def test_rejects_candidate_set_drift(self) -> None:
        stable_candidates = manifest_owner._git_resource_token_candidates(self.repo_root)
        self._write(
            "zircon_runtime/src/asset/late.rs",
            "use crate::core::resource::ResourceId;\n",
        )
        changed_candidates = stable_candidates + (
            Path("zircon_runtime/src/asset/late.rs"),
        )

        with patch.object(
            manifest_owner,
            "_git_resource_token_candidates",
            side_effect=(stable_candidates, changed_candidates),
        ):
            with self.assertRaises(manifest_owner.ManifestStabilityError) as raised:
                manifest_owner.build_resource_consumer_manifest(self.repo_root)

        self.assertEqual("candidate_set_changed", raised.exception.reason)

    def test_accepts_unrelated_head_drift(self) -> None:
        capture = manifest_owner.capture_resource_consumer_snapshot(self.repo_root)
        self._write("notes.md", "unrelated documentation\n")
        self._git("add", "notes.md")
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
            "unrelated head",
        )

        snapshot = manifest_owner.finalize_resource_consumer_snapshot(
            self.repo_root, capture
        )

        self.assertEqual(capture.candidate_fingerprints, snapshot.candidate_fingerprints)

    def test_rejects_resource_consumer_drift_across_head(self) -> None:
        capture = manifest_owner.capture_resource_consumer_snapshot(self.repo_root)
        self._write(
            "zircon_runtime/src/asset/internal.rs",
            "use crate::core::resource::ResourceRegistry;\n",
        )
        self._git("add", "zircon_runtime/src/asset/internal.rs")
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
            "resource consumer drift",
        )

        with self.assertRaises(manifest_owner.ManifestStabilityError) as raised:
            manifest_owner.finalize_resource_consumer_snapshot(self.repo_root, capture)

        self.assertEqual("source_content_changed", raised.exception.reason)

    def test_rejects_source_content_drift(self) -> None:
        stable_candidates = manifest_owner._git_resource_token_candidates(self.repo_root)
        changing_path = self.repo_root / "zircon_runtime/src/asset/internal.rs"
        calls = 0

        def candidates_with_mutation(_repo_root: Path) -> tuple[Path, ...]:
            nonlocal calls
            calls += 1
            if calls == 2:
                changing_path.write_text(
                    "use crate::core::{resource::ResourceRegistry};\n",
                    encoding="utf-8",
                )
            return stable_candidates

        with patch.object(
            manifest_owner,
            "_git_resource_token_candidates",
            side_effect=candidates_with_mutation,
        ):
            with self.assertRaises(manifest_owner.ManifestStabilityError) as raised:
                manifest_owner.build_resource_consumer_manifest(self.repo_root)

        self.assertEqual("source_content_changed", raised.exception.reason)
        self.assertEqual(
            ["zircon_runtime/src/asset/internal.rs"], raised.exception.changed_paths
        )

    def test_rejects_source_removed_after_candidate_inventory(self) -> None:
        stable_candidates = manifest_owner._git_resource_token_candidates(self.repo_root)
        removed_path = Path("zircon_runtime/src/asset/internal.rs")

        def candidates_then_remove(_repo_root: Path) -> tuple[Path, ...]:
            (self.repo_root / removed_path).unlink()
            return stable_candidates

        with patch.object(
            manifest_owner,
            "_git_resource_token_candidates",
            side_effect=candidates_then_remove,
        ):
            with self.assertRaises(manifest_owner.ManifestStabilityError) as raised:
                manifest_owner.build_resource_consumer_manifest(self.repo_root)

        self.assertEqual("source_content_changed", raised.exception.reason)
        self.assertEqual([removed_path.as_posix()], raised.exception.changed_paths)


if __name__ == "__main__":
    unittest.main()
