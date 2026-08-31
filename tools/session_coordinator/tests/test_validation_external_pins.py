from __future__ import annotations

import hashlib
import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.snapshots import ObjectStore
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.validation_external_pins import (
    discover_pinned_external_sources,
    external_sources_from_coverage,
    merge_external_sources_into_coverage,
)
from tools.session_coordinator.validation_tickets import ValidationTicketService


class ValidationExternalPinTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.root = root
        self.repo = init_repo(root / "zircon")
        self.external = init_repo(root / "zr_vm")
        (self.external / "binding/src").mkdir(parents=True)
        (self.external / "Cargo.toml").write_text(
            "[workspace]\nmembers=['binding']\nresolver='2'\n",
            encoding="utf-8",
        )
        (self.external / "binding/Cargo.toml").write_text(
            "[package]\nname='binding'\nversion='0.1.0'\n\n[lib]\npath='src/lib.rs'\n",
            encoding="utf-8",
        )
        (self.external / "binding/src/lib.rs").write_text(
            "pub fn binding() {}\n", encoding="utf-8"
        )
        self._commit(self.external, "external binding")
        self.external_commit = self._git(self.external, "rev-parse", "HEAD")

        (self.repo / "app/src").mkdir(parents=True)
        (self.repo / "Cargo.toml").write_text(
            "[workspace]\nmembers=['app']\nresolver='2'\n", encoding="utf-8"
        )
        (self.repo / "app/Cargo.toml").write_text(
            "[package]\nname='app'\nversion='0.1.0'\n\n[dependencies]\nbinding={path='../../zr_vm/binding'}\n",
            encoding="utf-8",
        )
        (self.repo / "app/src/lib.rs").write_text(
            "pub fn app() {}\n", encoding="utf-8"
        )
        self._commit(self.repo, "main workspace")
        self.baseline = self._git(self.repo, "rev-parse", "HEAD")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    @staticmethod
    def _git(repo: Path, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    def _commit(self, repo: Path, message: str) -> None:
        subprocess.run(["git", "add", "--all"], cwd=repo, check=True)
        subprocess.run(["git", "commit", "-q", "-m", message], cwd=repo, check=True)

    def test_discovers_sibling_commit_without_running_cargo(self) -> None:
        descriptors = discover_pinned_external_sources(
            self.repo,
            baseline_commit=self.baseline,
        )
        self.assertEqual(1, len(descriptors))
        self.assertEqual(str(self.external.resolve()), descriptors[0]["repoRoot"])
        self.assertEqual(self.external_commit, descriptors[0]["commit"])
        self.assertEqual(["binding"], descriptors[0]["includeRoots"])

    def test_sealed_overlay_manifest_is_scanned(self) -> None:
        overlay = {
            "app/Cargo.toml": (
                "[package]\nname='app'\nversion='0.1.0'\n\n"
                "[dependencies]\nbinding={path='../../zr_vm/binding'}\n"
            ).encode("utf-8")
        }
        descriptors = discover_pinned_external_sources(
            self.repo,
            baseline_commit=self.baseline,
            overlay_files=overlay,
        )
        self.assertEqual(self.external_commit, descriptors[0]["commit"])
        deleted = discover_pinned_external_sources(
            self.repo,
            baseline_commit=self.baseline,
            overlay_files={"app/Cargo.toml": None},
        )
        self.assertEqual((), deleted)

    def test_new_sealed_overlay_manifest_is_scanned(self) -> None:
        descriptors = discover_pinned_external_sources(
            self.repo,
            baseline_commit=self.baseline,
            overlay_files={
                "app/Cargo.toml": (
                    "[package]\nname='app'\nversion='0.1.0'\n"
                ).encode("utf-8"),
                "new_plugin/Cargo.toml": (
                    "[package]\nname='new_plugin'\nversion='0.1.0'\n\n"
                    "[dependencies]\nbinding={path='../../zr_vm/binding'}\n"
                ).encode("utf-8"),
            },
        )

        self.assertEqual(1, len(descriptors))
        self.assertEqual(self.external_commit, descriptors[0]["commit"])
        self.assertEqual(["binding"], descriptors[0]["includeRoots"])

    def test_coverage_merge_rejects_conflicting_identity(self) -> None:
        descriptors = discover_pinned_external_sources(
            self.repo,
            baseline_commit=self.baseline,
        )
        merged = merge_external_sources_into_coverage({"kind": "compile"}, descriptors)
        self.assertEqual(descriptors, external_sources_from_coverage(merged))
        conflict = dict(descriptors[0])
        conflict["commit"] = "0" * 40
        with self.assertRaises(CoordinatorError) as raised:
            merge_external_sources_into_coverage(
                {"externalSources": [conflict]}, descriptors
            )
        self.assertEqual("validation_ticket_external_source_conflict", raised.exception.code)

    def test_empty_scan_still_marks_the_ticket_as_pinned(self) -> None:
        merged = merge_external_sources_into_coverage({"kind": "compile"}, ())
        self.assertEqual([], merged["externalSources"])
        self.assertEqual((), external_sources_from_coverage(merged))

    def test_manifest_patch_path_is_pinned(self) -> None:
        (self.repo / "app/Cargo.toml").write_text(
            "[package]\nname='app'\nversion='0.1.0'\n\n"
            "[patch.crates-io]\nbinding={path='../../zr_vm/binding'}\n",
            encoding="utf-8",
        )
        self._commit(self.repo, "patch external binding")
        baseline = self._git(self.repo, "rev-parse", "HEAD")

        descriptors = discover_pinned_external_sources(
            self.repo,
            baseline_commit=baseline,
        )

        self.assertEqual(1, len(descriptors))
        self.assertEqual(self.external_commit, descriptors[0]["commit"])

    def test_config_paths_are_rejected_before_ticket_insertion(self) -> None:
        config = self.repo / ".cargo/config.toml"
        config.parent.mkdir(parents=True)
        config.write_text("paths = ['../unsealed-registry']\n", encoding="utf-8")
        self._commit(self.repo, "add unsupported cargo paths config")
        baseline = self._git(self.repo, "rev-parse", "HEAD")

        with self.assertRaises(CoordinatorError) as raised:
            discover_pinned_external_sources(
                self.repo,
                baseline_commit=baseline,
            )

        self.assertEqual(
            "validation_ticket_external_config_unsupported", raised.exception.code
        )

    def test_config_source_replacement_is_rejected(self) -> None:
        config = self.repo / ".cargo/config.toml"
        config.parent.mkdir(parents=True)
        config.write_text(
            "[source.crates-io]\nreplace-with='private'\n"
            "[source.private]\ndirectory='../private-registry'\n",
            encoding="utf-8",
        )
        self._commit(self.repo, "add unsupported source replacement")
        baseline = self._git(self.repo, "rev-parse", "HEAD")

        with self.assertRaises(CoordinatorError) as raised:
            discover_pinned_external_sources(self.repo, baseline_commit=baseline)

        self.assertEqual(
            "validation_ticket_external_config_unsupported", raised.exception.code
        )

    def test_package_selection_does_not_scan_unrelated_workspace_manifest(self) -> None:
        unrelated = self.repo / "unrelated/Cargo.toml"
        unrelated.parent.mkdir(parents=True)
        unrelated.write_text(
            "[package]\nname='unrelated'\nversion='0.1.0'\n"
            "[dependencies]\nmissing={path='../../missing-sibling/crate'}\n",
            encoding="utf-8",
        )
        (self.repo / "Cargo.toml").write_text(
            "[workspace]\nmembers=['app','unrelated']\nresolver='2'\n",
            encoding="utf-8",
        )
        self._commit(self.repo, "add unrelated broken package")
        baseline = self._git(self.repo, "rev-parse", "HEAD")

        descriptors = discover_pinned_external_sources(
            self.repo,
            baseline_commit=baseline,
            command=("cargo", "test", "-p", "app"),
        )

        self.assertEqual(1, len(descriptors))
        self.assertEqual(self.external_commit, descriptors[0]["commit"])

    def test_selected_package_inherits_external_workspace_dependency(self) -> None:
        (self.repo / "Cargo.toml").write_text(
            "[workspace]\nmembers=['app']\nresolver='2'\n"
            "[workspace.dependencies]\nbinding={path='../zr_vm/binding'}\n",
            encoding="utf-8",
        )
        (self.repo / "app/Cargo.toml").write_text(
            "[package]\nname='app'\nversion='0.1.0'\n"
            "[dependencies]\nbinding.workspace=true\n",
            encoding="utf-8",
        )
        self._commit(self.repo, "inherit external workspace dependency")
        baseline = self._git(self.repo, "rev-parse", "HEAD")

        descriptors = discover_pinned_external_sources(
            self.repo,
            baseline_commit=baseline,
            command=("cargo", "test", "-p", "app"),
        )

        self.assertEqual(1, len(descriptors))
        self.assertEqual(self.external_commit, descriptors[0]["commit"])

    def test_ticket_persists_the_submit_time_sibling_commit(self) -> None:
        database = Database(self.root / "coordinator.sqlite3")
        migrate(database)
        with database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, plan_path, status, base_head,
                    created_at, updated_at, last_heartbeat_at
                ) VALUES (
                    'session-a', 'docs/plans/tooling/01.md', 'active', ?,
                    '2026-08-31T00:00:00+00:00',
                    '2026-08-31T00:00:00+00:00',
                    '2026-08-31T00:00:00+00:00'
                )
                """,
                (self.baseline,),
            )
        service = ValidationTicketService(
            database,
            repo_root=self.repo,
            object_store=ObjectStore(database, self.root / "objects"),
        )
        source = self.repo / "app/src/lib.rs"
        receipt = service.submit(
            session_id="session-a",
            request_id="pin-sibling-at-submit",
            source_manifest={
                "app/src/lib.rs": hashlib.sha256(source.read_bytes()).hexdigest()
            },
            command=("cargo", "check", "-p", "app"),
            toolchain={"rust": "test"},
            coverage={"kind": "compile"},
        )

        (self.external / "binding/src/lib.rs").write_text(
            "pub fn binding() { println!(\"new\"); }\n", encoding="utf-8"
        )
        self._commit(self.external, "advance external after submit")
        current_external = self._git(self.external, "rev-parse", "HEAD")
        ticket = service.get(receipt.ticket.ticket_id)

        self.assertNotEqual(self.external_commit, current_external)
        self.assertEqual(
            self.external_commit,
            ticket.coverage["externalSources"][0]["commit"],
        )


if __name__ == "__main__":
    unittest.main()
