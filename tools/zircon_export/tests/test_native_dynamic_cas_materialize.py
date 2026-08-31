from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from unittest import mock

from tools.zircon_export import native_dynamic_materialize_io as materialize_io
from tools.zircon_export.native_build_execution import (
    execute_native_dynamic_package_build,
)
from tools.zircon_export.native_dynamic_materialize_io import (
    copy_native_dynamic_file,
    copy_native_dynamic_tree,
    native_dynamic_cas_scope,
    prune_native_dynamic_cas,
    reset_native_dynamic_plugins_dir,
    resolve_native_dynamic_cas_root,
)


class NativeDynamicCasMaterializeTests(unittest.TestCase):
    def test_windows_process_probe_does_not_terminate_the_process(self) -> None:
        child = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(30)"]
        )
        try:
            self.assertTrue(materialize_io._native_dynamic_process_is_alive(child.pid))
            time.sleep(0.2)
            self.assertIsNone(child.poll())
        finally:
            if child.poll() is None:
                child.terminate()
            child.wait(timeout=5)

    def test_prune_removes_a_dead_publisher_temporary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            digest = "a" * 64
            temporary = (
                cas_root
                / "sha256"
                / digest[:2]
                / f".{digest[2:]}.2147483647.123.{'b' * 32}.tmp"
            )
            temporary.parent.mkdir(parents=True)
            temporary.write_bytes(b"crash residue")

            receipt = prune_native_dynamic_cas(cas_root, max_bytes=0)

            self.assertFalse(temporary.exists())
            self.assertEqual(1, receipt["removedTemporaryFiles"])

    def test_cas_hit_rejects_a_source_that_changes_after_hashing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            source.write_bytes(b"first-payload")
            self.assertTrue(
                copy_native_dynamic_file(
                    source,
                    root / "first/plugin.dll",
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )
            original = materialize_io._sha256_file

            def mutate_after_hash(path: Path) -> str:
                digest = original(path)
                if path == source:
                    source.write_bytes(b"other-payload")
                return digest

            diagnostics: list[str] = []
            with mock.patch.object(
                materialize_io, "_sha256_file", side_effect=mutate_after_hash
            ):
                copied = copy_native_dynamic_file(
                    source,
                    root / "second/plugin.dll",
                    diagnostics,
                    "artifact",
                    cas_root=cas_root,
                )

            self.assertFalse(copied)
            self.assertFalse((root / "second/plugin.dll").exists())
            self.assertTrue(any("source changed" in item for item in diagnostics))

    @unittest.skipIf(os.name == "nt", "POSIX executable mode regression")
    def test_stage_copy_preserves_source_executable_bits(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "helper"
            source.write_bytes(b"executable helper")
            source.chmod(0o755)
            destination = root / "stage/helper"

            self.assertTrue(
                copy_native_dynamic_file(
                    source,
                    destination,
                    [],
                    "artifact",
                    cas_root=root / "cas",
                )
            )

            self.assertEqual(0o755, destination.stat().st_mode & 0o777)

    def test_reset_deletes_readonly_stage_copy_without_unsealing_cas(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            stage = root / "stage"
            destination = stage / "plugins" / "demo" / "plugin.dll"
            source.write_bytes(b"shared readonly payload")
            self.assertTrue(
                copy_native_dynamic_file(
                    source, destination, [], "artifact", cas_root=cas_root
                )
            )
            blob = next(
                path for path in (cas_root / "sha256").rglob("*") if path.is_file()
            )

            diagnostics: list[str] = []
            self.assertTrue(reset_native_dynamic_plugins_dir(stage, diagnostics))

            self.assertEqual([], diagnostics)
            self.assertFalse(destination.exists())
            self.assertEqual(b"shared readonly payload", blob.read_bytes())
            self.assertEqual(0, blob.stat().st_mode & 0o222)

    def test_fast_path_blob_validation_serializes_with_prune(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            source.write_bytes(b"prune race payload")
            self.assertTrue(
                copy_native_dynamic_file(
                    source, root / "first" / "plugin.dll", [], "artifact", cas_root=cas_root
                )
            )
            (root / "first" / "plugin.dll").chmod(0o644)
            (root / "first" / "plugin.dll").unlink()
            entered = materialize_io.threading.Event()
            release = materialize_io.threading.Event()
            prune_done = materialize_io.threading.Event()
            original = materialize_io._validated_native_dynamic_blob_identity

            def blocking_validation(*args, **kwargs):
                result = original(*args, **kwargs)
                if result is not None and not entered.is_set():
                    entered.set()
                    release.wait(5)
                return result

            def prune() -> None:
                prune_native_dynamic_cas(cas_root, max_bytes=0)
                prune_done.set()

            with mock.patch.object(
                materialize_io,
                "_validated_native_dynamic_blob_identity",
                side_effect=blocking_validation,
            ):
                with ThreadPoolExecutor(max_workers=2) as executor:
                    publisher = executor.submit(
                        copy_native_dynamic_file,
                        source,
                        root / "second" / "plugin.dll",
                        [],
                        "artifact",
                        cas_root=cas_root,
                    )
                    self.assertTrue(entered.wait(5))
                    pruner = executor.submit(prune)
                    self.assertFalse(prune_done.wait(0.1))
                    release.set()
                    self.assertTrue(publisher.result(timeout=5))
                    pruner.result(timeout=5)

            self.assertEqual(
                source.read_bytes(), (root / "second" / "plugin.dll").read_bytes()
            )

    def test_source_symlink_is_rejected_instead_of_followed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "outside.dll"
            source.write_bytes(b"outside payload")
            link = root / "linked.dll"
            try:
                link.symlink_to(source)
            except OSError as error:
                self.skipTest(f"symbolic links are unavailable: {error}")
            diagnostics: list[str] = []

            self.assertFalse(
                copy_native_dynamic_file(
                    link,
                    root / "stage" / "plugin.dll",
                    diagnostics,
                    "artifact",
                    cas_root=root / "cas",
                )
            )
            self.assertTrue(any("reparse point" in item for item in diagnostics))

    def test_successful_native_build_stages_through_the_shared_cas(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo_root = root / "repo"
            repo_root.mkdir()
            source = root / "target" / "release" / "plugin.dll"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"freshly built native payload")
            package_dir = root / "stage" / "plugins" / "demo"
            package_dir.mkdir(parents=True)
            cas_root = root / "cas"
            plan = {
                "package_id": "demo",
                "crate_name": "demo_native",
                "command": ["cargo", "build"],
                "expected_loadable_artifact": str(source),
            }
            completed = mock.Mock(returncode=0, stdout="", stderr="")

            with mock.patch.dict(
                os.environ,
                {"ZIRCON_NATIVE_DYNAMIC_CAS_ROOT": str(cas_root)},
                clear=False,
            ), mock.patch(
                "tools.zircon_export.native_build_execution.subprocess.run",
                return_value=completed,
            ):
                result = execute_native_dynamic_package_build(
                    plan,
                    repo_root,
                    {"demo": package_dir},
                    [],
                )

            destination = package_dir / "native" / "plugin.dll"
            blobs = [
                path
                for path in (cas_root / "sha256").rglob("*")
                if path.is_file()
            ]
            self.assertEqual(str(destination), result["copied_loadable_artifact"])
            self.assertEqual(1, len(blobs))
            self.assertFalse(os.path.samefile(destination, blobs[0]))

    def test_same_content_files_share_one_cas_blob_with_isolated_stage_copies(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source_a = root / "source-a.dll"
            source_b = root / "source-b.dll"
            destination_a = root / "stage-a" / "plugin.dll"
            destination_b = root / "stage-b" / "plugin.dll"
            source_a.write_bytes(b"stable native payload\0" * 128)
            source_b.write_bytes(source_a.read_bytes())

            self.assertTrue(
                copy_native_dynamic_file(
                    source_a,
                    destination_a,
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )
            self.assertTrue(
                copy_native_dynamic_file(
                    source_b,
                    destination_b,
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )

            self.assertFalse(os.path.samefile(destination_a, destination_b))
            blobs = [
                path
                for path in (cas_root / "sha256").rglob("*")
                if path.is_file()
            ]
            self.assertEqual(1, len(blobs))
            self.assertFalse(os.path.samefile(destination_a, blobs[0]))

    def test_stage_mutation_cannot_change_another_stage_or_the_shared_blob(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            destination_a = root / "stage-a" / "plugin.dll"
            destination_b = root / "stage-b" / "plugin.dll"
            payload = b"immutable shared native payload"
            source.write_bytes(payload)
            self.assertTrue(
                copy_native_dynamic_file(
                    source, destination_a, [], "artifact", cas_root=cas_root
                )
            )
            self.assertTrue(
                copy_native_dynamic_file(
                    source, destination_b, [], "artifact", cas_root=cas_root
                )
            )
            blob = next(
                path for path in (cas_root / "sha256").rglob("*") if path.is_file()
            )

            destination_a.chmod(0o644)
            destination_a.write_bytes(b"mutated")

            self.assertEqual(b"mutated", destination_a.read_bytes())
            self.assertEqual(payload, destination_b.read_bytes())
            self.assertEqual(payload, blob.read_bytes())

    def test_corrupt_legacy_link_is_detached_and_repaired(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            source.write_bytes(b"expected payload")
            destination = root / "stage" / "plugin.dll"
            self.assertTrue(
                copy_native_dynamic_file(
                    source, destination, [], "artifact", cas_root=cas_root
                )
            )
            blob = next(
                path for path in (cas_root / "sha256").rglob("*") if path.is_file()
            )
            legacy = root / "legacy" / "plugin.dll"
            legacy.parent.mkdir()
            destination.chmod(0o644)
            destination.unlink()
            blob.chmod(0o644)
            os.link(blob, legacy)
            legacy.write_bytes(b"corrupt")

            repaired = root / "repaired" / "plugin.dll"
            self.assertTrue(
                copy_native_dynamic_file(
                    source, repaired, [], "artifact", cas_root=cas_root
                )
            )

            self.assertEqual(b"corrupt", legacy.read_bytes())
            self.assertEqual(b"expected payload", blob.read_bytes())
            self.assertEqual(b"expected payload", repaired.read_bytes())
            self.assertFalse(os.path.samefile(blob, legacy))

    def test_existing_blob_is_content_checked_only_once_per_stage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            source.write_bytes(b"stable payload" * 4096)
            self.assertTrue(
                copy_native_dynamic_file(
                    source,
                    root / "first" / "plugin.dll",
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )

            with mock.patch.object(
                materialize_io,
                "_sha256_file",
                wraps=materialize_io._sha256_file,
            ) as sha256_file:
                self.assertTrue(
                    copy_native_dynamic_file(
                        source,
                        root / "second" / "plugin.dll",
                        [],
                        "artifact",
                        cas_root=cas_root,
                    )
                )

            # One source hash establishes the content key and one blob hash
            # validates an existing cache entry before it is copied.
            self.assertEqual(2, sha256_file.call_count)

    def test_concurrent_publishers_leave_one_complete_blob(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            source.write_bytes(b"concurrent payload\0" * 4096)

            def publish(index: int) -> bool:
                return copy_native_dynamic_file(
                    source,
                    root / f"stage-{index}" / "plugin.dll",
                    [],
                    "artifact",
                    cas_root=cas_root,
                )

            with ThreadPoolExecutor(max_workers=6) as executor:
                results = list(executor.map(publish, range(6)))

            self.assertEqual([True] * 6, results)
            destinations = [
                root / f"stage-{index}" / "plugin.dll" for index in range(6)
            ]
            self.assertTrue(all(destination.read_bytes() == source.read_bytes() for destination in destinations))
            blobs = [
                path
                for path in (cas_root / "sha256").rglob("*")
                if path.is_file()
            ]
            self.assertEqual(1, len(blobs))
            self.assertFalse(
                any(path.suffix == ".tmp" for path in (cas_root / "sha256").rglob("*") if path.exists())
            )

    def test_stage_always_receives_a_regular_copy(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            destination = root / "stage" / "plugin.dll"
            source.write_bytes(b"cross-volume payload")

            self.assertTrue(
                copy_native_dynamic_file(
                    source,
                    destination,
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )

            self.assertEqual(source.read_bytes(), destination.read_bytes())
            blobs = [
                path
                for path in (cas_root / "sha256").rglob("*")
                if path.is_file()
            ]
            self.assertEqual(1, len(blobs))
            self.assertFalse(os.path.samefile(destination, blobs[0]))

    def test_corrupt_same_size_blob_is_repaired_before_reuse(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            first_destination = root / "first" / "plugin.dll"
            second_destination = root / "second" / "plugin.dll"
            source.write_bytes(b"correct payload")
            self.assertTrue(
                copy_native_dynamic_file(
                    source,
                    first_destination,
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )
            blob = next(
                path
                for path in (cas_root / "sha256").rglob("*")
                if path.is_file()
            )
            first_destination.chmod(0o644)
            first_destination.unlink()
            blob.chmod(0o644)
            blob.write_bytes(b"x" * len(source.read_bytes()))

            self.assertTrue(
                copy_native_dynamic_file(
                    source,
                    second_destination,
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )

            self.assertEqual(source.read_bytes(), second_destination.read_bytes())
            self.assertFalse(os.path.samefile(blob, second_destination))

    def test_prune_removes_cas_blobs_without_affecting_stage_copies(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            linked_source = root / "linked.dll"
            retired_source = root / "retired.dll"
            linked_destination = root / "stage" / "linked.dll"
            retired_destination = root / "temporary-stage" / "retired.dll"
            linked_source.write_bytes(b"linked-payload")
            retired_source.write_bytes(b"retired-payload")
            self.assertTrue(
                copy_native_dynamic_file(
                    linked_source,
                    linked_destination,
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )
            self.assertTrue(
                copy_native_dynamic_file(
                    retired_source,
                    retired_destination,
                    [],
                    "artifact",
                    cas_root=cas_root,
                )
            )
            retired_destination.chmod(0o644)
            retired_destination.unlink()

            result = prune_native_dynamic_cas(cas_root, max_bytes=0)
            blobs = [
                path for path in (cas_root / "sha256").rglob("*") if path.is_file()
            ]

            self.assertEqual(2, result["removedBlobs"])
            self.assertEqual(0, len(blobs))
            self.assertEqual(b"linked-payload", linked_destination.read_bytes())

    def test_cas_scope_prunes_to_the_configured_limit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            destination = root / "stage" / "plugin.dll"
            source.write_bytes(b"scope-prune-payload")
            with mock.patch.dict(
                os.environ,
                {
                    "ZIRCON_NATIVE_DYNAMIC_CAS_ROOT": str(cas_root),
                    "ZIRCON_NATIVE_DYNAMIC_CAS_MAX_BYTES": "0",
                },
                clear=False,
            ):
                with native_dynamic_cas_scope(allow_hardlinks=True):
                    self.assertTrue(
                        copy_native_dynamic_file(source, destination, [], "artifact")
                    )
                    destination.chmod(0o644)
                    destination.unlink()

            self.assertEqual(
                [], [path for path in (cas_root / "sha256").rglob("*") if path.is_file()]
            )

    def test_tree_materialization_reuses_nested_files_and_empty_directories(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            destination = root / "destination"
            second_destination = root / "destination-second"
            cas_root = root / "cas"
            (source / "nested").mkdir(parents=True)
            (source / "empty").mkdir()
            (source / "plugin.toml").write_text("id = 'demo'\n", encoding="utf-8")
            (source / "nested" / "plugin.dll").write_bytes(b"nested native payload")

            self.assertTrue(
                copy_native_dynamic_tree(
                    source,
                    destination,
                    [],
                    "tree",
                    cas_root=cas_root,
                )
            )
            self.assertTrue(
                copy_native_dynamic_tree(
                    source,
                    second_destination,
                    [],
                    "tree",
                    cas_root=cas_root,
                )
            )

            self.assertTrue((destination / "empty").is_dir())
            self.assertFalse(
                os.path.samefile(
                    destination / "plugin.toml",
                    second_destination / "plugin.toml",
                )
            )
            self.assertFalse(
                os.path.samefile(
                    destination / "nested" / "plugin.dll",
                    second_destination / "nested" / "plugin.dll",
                )
            )
            self.assertEqual(
                (source / "plugin.toml").read_bytes(),
                (destination / "plugin.toml").read_bytes(),
            )
            self.assertEqual(
                (source / "nested" / "plugin.dll").read_bytes(),
                (destination / "nested" / "plugin.dll").read_bytes(),
            )

    def test_environment_is_opt_in(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=True):
            self.assertIsNone(resolve_native_dynamic_cas_root())
        self.assertEqual(
            Path("cas").resolve(),
            resolve_native_dynamic_cas_root("cas"),
        )

    def test_mutating_stage_scope_disables_hardlinks_and_restores_policy(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cas_root = root / "cas"
            source = root / "plugin.dll"
            destination = root / "signed-stage" / "plugin.dll"
            source.write_bytes(b"payload that will be signed")
            with mock.patch.dict(
                os.environ,
                {"ZIRCON_NATIVE_DYNAMIC_CAS_ROOT": str(cas_root)},
                clear=False,
            ):
                with native_dynamic_cas_scope(allow_hardlinks=False):
                    self.assertTrue(
                        copy_native_dynamic_file(
                            source,
                            destination,
                            [],
                            "artifact",
                        )
                    )
                self.assertEqual(cas_root, resolve_native_dynamic_cas_root())

            self.assertEqual(source.read_bytes(), destination.read_bytes())
            self.assertFalse(cas_root.exists())
            self.assertFalse(os.path.samefile(source, destination))


if __name__ == "__main__":
    unittest.main()
