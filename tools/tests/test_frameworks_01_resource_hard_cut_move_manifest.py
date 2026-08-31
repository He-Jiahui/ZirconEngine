from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from tools import frameworks_01_resource_hard_cut_move_manifest as manifest_owner


def _test_temp_root() -> Path:
    configured = os.environ.get("ZIRCON_TEST_TEMP_ROOT")
    if configured:
        return Path(configured) / "frameworks01-resource-move-manifest-tests"
    for drive in ("F:/", "D:/", "E:/"):
        root = Path(drive)
        if root.exists():
            return root / "zircon-profiles/frameworks01-resource-move-manifest-tests"
    return Path(tempfile.gettempdir()) / "frameworks01-resource-move-manifest-tests"


TEST_TEMP_ROOT = _test_temp_root()


def _sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _manifest_sha256(value: object) -> str:
    payload = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return _sha256(payload)


class Frameworks01ResourceHardCutMoveManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        TEST_TEMP_ROOT.mkdir(parents=True, exist_ok=True)
        self.temporary_directory = tempfile.TemporaryDirectory(dir=TEST_TEMP_ROOT)
        self.repo_root = Path(self.temporary_directory.name)
        self._git("init", "--quiet")

        sources = {
            "Cargo.toml": "[workspace]\nmembers = []\n",
            "Cargo.lock": "version = 3\n",
            "zircon_runtime/Cargo.toml": "[package]\nname = 'zircon_runtime'\n",
            "zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs": (
                "mod runtime_surface;\n"
            ),
            "zircon_runtime/src/core/resource/mod.rs": "mod data;\npub mod io;\n",
            "zircon_runtime/src/core/resource/io/mod.rs": (
                "mod atomic_file;\npub use atomic_file::atomic_write;\n"
            ),
            "zircon_runtime/src/core/resource/data.rs": (
                "use crate::core::resource::ResourceId;\n"
                "pub struct ResourceData(ResourceId);\n"
            ),
            "zircon_runtime/src/core/resource/comment_only.rs": (
                "// crate::core::resource::Ignored\n"
                "const EXAMPLE: &str = \"crate::core::resource::Ignored\";\n"
            ),
            "zircon_runtime/src/core/resource/management_generation/tests/mod.rs": (
                "mod hard_cut;\nmod projection;\nmod support;\n"
            ),
            "zircon_runtime/src/core/resource/management_generation/tests/hard_cut.rs": (
                "use super::support::*;\n#[test]\nfn owner_boundary() {}\n"
            ),
            "zircon_runtime/src/core/resource/management_generation/tests/support.rs": (
                "pub(super) fn rust_code_view() {}\n"
            ),
            "zircon_runtime/src/core/resource/management_generation/tests/projection.rs": (
                "#[test]\nfn projection() {}\n"
            ),
        }
        for path, source in sources.items():
            self._write(path, source)
        for index, path in enumerate(manifest_owner.REQUIRED_CONSUMER_PATCHES):
            self._write(path, f"pub fn resource_consumer_{index}() {{}}\n")
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

    def _entry(self, path: str, *roles: str) -> dict[str, object]:
        source = (self.repo_root / path).read_bytes()
        return {
            "bytes": len(source),
            "path": path,
            "roles": sorted(roles),
            "sha256": _sha256(source),
        }

    def _source_report(self) -> dict[str, object]:
        resource_root = self.repo_root / manifest_owner.RESOURCE_IMPLEMENTATION_ROOT
        entries = [
            self._entry(
                path.relative_to(self.repo_root).as_posix(),
                "resource_implementation_owner",
            )
            for path in sorted(resource_root.rglob("*.rs"))
        ]
        entries.extend(
            self._entry(path, "fixed_workspace_input")
            for path in manifest_owner.REQUIRED_PATCH_INPUTS
        )
        entries.extend(
            self._entry(path, "rust_consumer")
            for path in manifest_owner.REQUIRED_CONSUMER_PATCHES
        )
        entries.sort(key=lambda entry: str(entry["path"]))
        return {
            "atomic_input_count": len(entries),
            "atomic_input_manifest_sha256": _manifest_sha256(entries),
            "future_paths": list(manifest_owner.SOURCE_REPORT_FUTURE_PATHS),
            "inputs": entries,
            "schema_version": manifest_owner.SOURCE_REPORT_SCHEMA_VERSION,
            "stability": {
                "consumer_snapshot": True,
                "future_paths_absent": True,
                "supplemental_candidates": True,
                "supplemental_content": True,
                "supplemental_terminal_snapshot": True,
            },
        }

    def _reseal(self, report: dict[str, object]) -> None:
        report["inputs"].sort(key=lambda entry: str(entry["path"]))
        report["atomic_input_count"] = len(report["inputs"])
        report["atomic_input_manifest_sha256"] = _manifest_sha256(report["inputs"])

    def test_partitions_every_owner_and_classifies_only_code_rewrites(self) -> None:
        report = manifest_owner.build_resource_hard_cut_move_manifest(
            self.repo_root, self._source_report()
        )
        operations = {operation["source"]: operation for operation in report["operations"]}

        self.assertEqual(8, report["resource_owner_input_count"])
        self.assertEqual(
            {
                "generate_crate_surface": 4,
                "move_rewrite_crate_root": 1,
                "move_rewrite_module_set": 1,
                "move_verbatim": 2,
                "patch_consumer": 8,
                "patch_required": 4,
                "relocate_runtime_guard": 2,
                "replace_runtime_facade": 2,
            },
            report["operation_counts"],
        )
        self.assertEqual(
            "move_rewrite_crate_root",
            operations["zircon_runtime/src/core/resource/data.rs"]["kind"],
        )
        self.assertEqual(
            "move_verbatim",
            operations["zircon_runtime/src/core/resource/comment_only.rs"]["kind"],
        )
        self.assertEqual(
            "move_rewrite_module_set",
            operations[
                "zircon_runtime/src/core/resource/management_generation/tests/mod.rs"
            ]["kind"],
        )
        self.assertEqual(
            "relocate_runtime_guard",
            operations[
                "zircon_runtime/src/core/resource/management_generation/tests/hard_cut.rs"
            ]["kind"],
        )
        destinations = [
            operation["destination"]
            for operation in report["operations"]
            if operation["destination"] is not None
        ]
        self.assertEqual(len(destinations), len(set(destinations)))
        self.assertEqual(30, report["write_path_count"])
        self.assertEqual(
            _manifest_sha256(report["write_paths"]),
            report["write_path_manifest_sha256"],
        )
        write_paths = {entry["path"]: entry for entry in report["write_paths"]}
        self.assertEqual(
            ["delete_source"],
            write_paths["zircon_runtime/src/core/resource/data.rs"]["roles"],
        )
        self.assertEqual(
            ["write_destination"],
            write_paths["zircon_runtime/crates/zr_resource/src/data.rs"]["roles"],
        )
        self.assertEqual(
            ["write_destination"],
            write_paths[manifest_owner.REQUIRED_CONSUMER_PATCHES[0]]["roles"],
        )
        self.assertIsNone(
            write_paths["zircon_runtime/crates/zr_resource/src/lib.rs"][
                "current_sha256"
            ]
        )
        self.assertEqual(
            _sha256(
                (
                    self.repo_root / manifest_owner.REQUIRED_CONSUMER_PATCHES[0]
                ).read_bytes()
            ),
            write_paths[manifest_owner.REQUIRED_CONSUMER_PATCHES[0]][
                "current_sha256"
            ],
        )
        self.assertTrue(all(report["stability"].values()))

    def test_report_and_explicit_output_are_deterministic(self) -> None:
        source_report = self._source_report()
        first = manifest_owner.build_resource_hard_cut_move_manifest(
            self.repo_root, source_report
        )
        second = manifest_owner.build_resource_hard_cut_move_manifest(
            self.repo_root, source_report
        )
        output = self.repo_root / "artifacts/move-manifest.json"

        manifest_owner.write_resource_hard_cut_move_manifest(first, output)

        self.assertEqual(first, second)
        self.assertEqual(first, json.loads(output.read_text(encoding="utf-8")))
        self.assertTrue(output.read_bytes().endswith(b"\n"))

    def test_rejects_owner_content_drift(self) -> None:
        source_report = self._source_report()
        self._write(
            "zircon_runtime/src/core/resource/data.rs",
            "pub struct Changed;\n",
        )

        with self.assertRaises(manifest_owner.MoveManifestStabilityError) as raised:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, source_report
            )

        self.assertEqual("resource_owner_content_changed", raised.exception.reason)
        self.assertEqual(
            ["zircon_runtime/src/core/resource/data.rs"],
            raised.exception.changed_paths,
        )

    def test_rejects_higher_layer_runtime_dependency_in_moved_code(self) -> None:
        self._write(
            "zircon_runtime/src/core/resource/comment_only.rs",
            "use crate::asset::AssetUri;\n",
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
            "higher layer dependency",
        )
        source_report = self._source_report()

        with self.assertRaises(manifest_owner.MoveManifestError) as raised:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, source_report
            )

        self.assertIn("higher-layer Runtime dependency", str(raised.exception))

    def test_rejects_missing_or_extra_owner_input(self) -> None:
        missing_report = self._source_report()
        missing_report["inputs"] = [
            entry
            for entry in missing_report["inputs"]
            if entry["path"] != "zircon_runtime/src/core/resource/data.rs"
        ]
        self._reseal(missing_report)
        with self.assertRaises(manifest_owner.MoveManifestStabilityError) as missing:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, missing_report
            )
        self.assertEqual("resource_owner_membership_changed", missing.exception.reason)

        extra_report = self._source_report()
        extra_report["inputs"].append(
            {
                "bytes": 0,
                "path": "zircon_runtime/src/core/resource/removed.rs",
                "roles": ["resource_implementation_owner"],
                "sha256": _sha256(b""),
            }
        )
        self._reseal(extra_report)
        with self.assertRaises(manifest_owner.MoveManifestStabilityError) as extra:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, extra_report
            )
        self.assertEqual("resource_owner_membership_changed", extra.exception.reason)

    def test_rejects_tampered_source_input_manifest_hash(self) -> None:
        source_report = self._source_report()
        source_report["atomic_input_manifest_sha256"] = "0" * 64

        with self.assertRaises(manifest_owner.MoveManifestError) as raised:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, source_report
            )

        self.assertIn("manifest hash does not match", str(raised.exception))

    def test_rejects_patch_input_without_fixed_workspace_role(self) -> None:
        source_report = self._source_report()
        cargo_lock = next(
            entry for entry in source_report["inputs"] if entry["path"] == "Cargo.lock"
        )
        cargo_lock["roles"] = ["textual_reference"]
        self._reseal(source_report)

        with self.assertRaises(manifest_owner.MoveManifestError) as raised:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, source_report
            )

        self.assertIn("required patch input is missing", str(raised.exception))

    def test_rejects_missing_or_drifted_consumer_patch_input(self) -> None:
        missing = self._source_report()
        missing["inputs"] = [
            entry
            for entry in missing["inputs"]
            if entry["path"] != manifest_owner.REQUIRED_CONSUMER_PATCHES[0]
        ]
        self._reseal(missing)

        with self.assertRaises(manifest_owner.MoveManifestError) as raised:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, missing
            )

        self.assertIn("required consumer patch input is missing", str(raised.exception))

        drifted = self._source_report()
        self._write(
            manifest_owner.REQUIRED_CONSUMER_PATCHES[0],
            "pub fn changed_consumer() {}\n",
        )
        with self.assertRaises(manifest_owner.MoveManifestStabilityError) as raised:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, drifted
            )

        self.assertEqual("consumer_patch_content_changed", raised.exception.reason)

    def test_rejects_generated_or_relocated_destination_collision(self) -> None:
        source_report = self._source_report()
        self._write(
            "zircon_runtime/crates/zr_resource/src/lib.rs",
            "pub struct Collision;\n",
        )

        with self.assertRaises(manifest_owner.MoveManifestError) as raised:
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, source_report
            )

        self.assertIn("destination already exists", str(raised.exception))

    def test_rejects_unstable_or_wrong_schema_source_report(self) -> None:
        unstable = self._source_report()
        unstable["stability"]["supplemental_content"] = False
        with self.assertRaises(manifest_owner.MoveManifestError):
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, unstable
            )

        wrong_schema = self._source_report()
        wrong_schema["schema_version"] = 999
        with self.assertRaises(manifest_owner.MoveManifestError):
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, wrong_schema
            )

        truthy_but_invalid = self._source_report()
        truthy_but_invalid["stability"]["supplemental_content"] = "true"
        with self.assertRaises(manifest_owner.MoveManifestError):
            manifest_owner.build_resource_hard_cut_move_manifest(
                self.repo_root, truthy_but_invalid
            )

    def test_accepts_unrelated_source_head_drift(self) -> None:
        source_report = self._source_report()
        self._write("head-change.txt", "new head\n")
        self._git("add", "head-change.txt")
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
            "head change",
        )

        report = manifest_owner.build_resource_hard_cut_move_manifest(
            self.repo_root, source_report
        )

        self.assertNotIn("source_head", report)
        self.assertEqual(
            source_report["atomic_input_manifest_sha256"],
            report["source_atomic_input_manifest_sha256"],
        )


if __name__ == "__main__":
    unittest.main()
