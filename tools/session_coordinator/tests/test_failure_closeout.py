from __future__ import annotations

import json
import os
import io
import subprocess
import tempfile
import unittest
from datetime import date
from pathlib import Path
from unittest import mock
from contextlib import redirect_stdout

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.cli import _parser, main
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.failures import FailureGraphService, FailureResolution
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus, utc_text
from tools.session_coordinator.notifications import WeComNotificationService
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.snapshots import ObjectStore, SnapshotService
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.failure_closeouts import FailureCloseoutWorkflowService


class FailureCloseoutWorkflowTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        fixture = FailureGraphFixture(self.repo)
        self.origin = fixture.add_plan("docs/plans/editor/15-layout.md")
        self.fixing = fixture.add_plan("docs/plans/plugins/01-core.md")
        self.other_failure = fixture.add_handoff(
            self.origin,
            self.fixing,
            "bridge-runtime-call-lock",
        )
        self.other_paths = tuple(f"src/bridge/path_{index}.rs" for index in range(11))
        self._add_child_scope(self.other_failure, self.other_paths)
        self.paths = (
            "Cargo.toml",
            "Cargo.lock",
            "editor/Cargo.toml",
            "runtime/Cargo.toml",
        )
        for path in (*self.paths, *self.other_paths):
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"base {path}\n", encoding="utf-8")

        self.fixed_path = (
            "docs/plans/editor/15/fixed-2026-07-22-bridge-arc-swap-lock.md"
        )
        self.source_path = (
            "docs/plans/plugins/01/failure-2026-07-22-bridge-arc-swap-lock.md"
        )
        self.second_source_path = (
            "docs/plans/plugins/01/failure-2026-07-22-bridge-second-atomic-target.md"
        )
        for path in (self.source_path, self.second_source_path):
            source = self.repo / path
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text("tracked prior Failure artifact\n", encoding="utf-8")
        for plan in (self.origin.path, self.fixing.path):
            link = Path(
                os.path.relpath(self.repo / self.fixed_path, plan.parent)
            ).as_posix()
            with plan.open("a", encoding="utf-8") as stream:
                stream.write(
                    "\n- fixed 已修复：[bridge-arc-swap-lock]"
                    f"({link})\n"
                )
        subprocess.run(["git", "add", "-A"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add failure graph"],
            cwd=self.repo,
            check=True,
        )

        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.config = config
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.session_id = "plugins01-closeout"
        self.sessions.register(
            session_id=self.session_id,
            plan_path=self.fixing.path.relative_to(self.repo).as_posix(),
            write_scope=list(self.paths),
        )
        self.sessions.register(session_id="reviewer-b")
        self.sessions.register(session_id="executor-thread")
        self.sessions.set_status("reviewer-b", SessionStatus.ACTIVE)
        self.sessions.set_status("executor-thread", SessionStatus.ACTIVE)
        self.sessions.set_status(self.session_id, SessionStatus.ACTIVE)
        self.sessions.set_status(self.session_id, SessionStatus.RESOLVING_FAILURE)
        self._bind_reviewer_thread("reviewer-b")
        self._bind_reviewer_thread("executor-thread")
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=config.lease_ttl_seconds,
            grace_seconds=config.lease_grace_seconds,
        )
        objects = ObjectStore(self.database, config.object_root)
        self.snapshots = SnapshotService(self.database, self.repo, objects)
        self.failures = FailureGraphService(self.database, self.repo)
        self.finalize = GitFinalizeService(
            self.database,
            self.repo,
            self.baselines,
            self.sessions,
            failures=self.failures,
        )
        self.messages: list[str] = []

        def notify(command: list[str]) -> subprocess.CompletedProcess[str]:
            self.messages.append(command[command.index("-Message") + 1])
            return subprocess.CompletedProcess(command, 0, '{"errcode":0}', "")

        notifications = WeComNotificationService(
            self.database, script_path="send.ps1", runner=notify
        )
        self.service = FailureCloseoutWorkflowService(
            self.database,
            self.repo,
            self.baselines,
            self.finalize,
            self.snapshots,
            self.failures,
            notifications,
            sessions=self.sessions,
            leases=self.leases,
        )
        self.return_path = (
            "docs/plans/plugins/01/2026-07-22-bridge-arc-swap-lock-return.md"
        )
        self._write_candidate()
        self.failures.import_repository()
        target = next(
            node
            for node in self.failures.audit().nodes
            if node.summary_slug == "bridge-arc-swap-lock"
        )
        self.lifecycle_key = target.lifecycle_key
        self.snapshot = self.snapshots.create(
            session_id=self.session_id,
            paths=[*self.paths, self.source_path, self.fixed_path, self.return_path],
            baseline_epoch=self.baselines.current().epoch_id,
            purpose="failure-lifecycle-exact6-closeout",
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    @staticmethod
    def _add_child_scope(path: Path, related: tuple[str, ...]) -> None:
        text = path.read_text(encoding="utf-8")
        marker = "---\n\n# Cross-plan handoff"
        scope = "plan_link_mode: child_record_only\nrelated_code:\n" + "".join(
            f"  - {item}\n" for item in related
        )
        path.write_text(text.replace(marker, scope + marker), encoding="utf-8")

    def _write_candidate(self) -> None:
        (self.repo / self.source_path).unlink()
        (self.repo / self.second_source_path).unlink()
        for path in self.paths:
            target = self.repo / path
            target.write_text(target.read_text(encoding="utf-8") + "arc-swap = true\n")
        fixed = self.repo / self.fixed_path
        fixed.parent.mkdir(parents=True, exist_ok=True)
        fixed.write_text(
            "---\n"
            "handoff_kind: fixed\n"
            "status: fixed\n"
            "created_at: 2026-07-22\n"
            "resolved_at: 2026-07-22\n"
            "summary_slug: bridge-arc-swap-lock\n"
            f"origin_plan: {self.origin.path.relative_to(self.repo).as_posix()}\n"
            f"fixing_plan: {self.fixing.path.relative_to(self.repo).as_posix()}\n"
            f"origin_child_dir: {self.origin.child.relative_to(self.repo).as_posix()}\n"
            f"fixing_child_dir: {self.fixing.child.relative_to(self.repo).as_posix()}\n"
            "plan_link_mode: child_record_only\n"
            "related_code:\n"
            + "".join(f"  - {path}\n" for path in self.paths)
            + "---\n\n# Fixed\n\n"
            "## 来源执行者\n\n"
            f"- 来源计划：`{self.origin.path.relative_to(self.repo).as_posix()}`\n"
            "- 来源执行切片：M3 fixture\n"
            f"- 修复责任计划：`{self.fixing.path.relative_to(self.repo).as_posix()}`\n"
            "- 交接原因：lowest shared owner\n\n"
            "## 失败现象与复现证据\n\nThe locked graph drifted.\n\n"
            "## 最低共享层根因\n\nThe root edge was incomplete.\n\n"
            "## 架构修复验收\n\n- Managed metadata passes.\n\n"
            "## 禁止临时方案\n\n- No unlocked fallback.\n\n"
            "## 修复结果与回传\n\n"
            "- 根因：workspace lock edge drifted.\n"
            "- 架构修复：the canonical lock edge was restored.\n"
            "- 验证：managed metadata passed.\n"
            "- 回传：the origin gate may resume.\n",
            encoding="utf-8",
        )
        returned = self.repo / self.return_path
        returned.parent.mkdir(parents=True, exist_ok=True)
        returned.write_text(
            "---\n"
            "record_kind: failure_return_status\n"
            "status: fixed\n"
            "resolved_at: 2026-07-22\n"
            "summary_slug: bridge-arc-swap-lock\n"
            f"origin_plan: {self.origin.path.relative_to(self.repo).as_posix()}\n"
            f"fixing_plan: {self.fixing.path.relative_to(self.repo).as_posix()}\n"
            "plan_link_mode: child_record_only\n"
            f"source_artifact: {self.source_path}\n"
            "---\n\n# Return\n",
            encoding="utf-8",
        )

    def _combined_candidate(self, *, include_noop_provenance: bool = False):
        slug = "bridge-second-atomic-target"
        state_provenance = (
            ".codex/state/session-coordinator/cargo-runs/proof/stderr.log"
        )
        related = (
            "src/combined/second.rs",
            *(("README.md", state_provenance) if include_noop_provenance else ()),
        )
        related_path = self.repo / "src/combined/second.rs"
        related_path.parent.mkdir(parents=True, exist_ok=True)
        related_path.write_text("second target\n", encoding="utf-8")
        if include_noop_provenance:
            state_path = self.repo / state_provenance
            state_path.parent.mkdir(parents=True, exist_ok=True)
            state_path.write_text("managed validation stderr\n", encoding="utf-8")
        fixture = FailureGraphFixture(self.repo)
        fixed = fixture.add_handoff(
            self.origin,
            self.fixing,
            slug,
            kind="fixed",
            created_at="2026-07-21" if include_noop_provenance else "2026-07-22",
            resolved_at="2026-07-23",
        )
        self._add_child_scope(fixed, related)
        fixed_path = fixed.relative_to(self.repo).as_posix()
        return_path = (
            "docs/plans/plugins/01/2026-07-23-bridge-second-atomic-target-return.md"
        )
        returned = self.repo / return_path
        source_artifact = (
            "docs/plans/plugins/01/failure-2026-07-21-bridge-second-atomic-target.md"
            if include_noop_provenance
            else self.second_source_path
        )
        returned.write_text(
            "---\n"
            "record_kind: failure_return_status\n"
            "status: fixed\n"
            "resolved_at: 2026-07-23\n"
            f"summary_slug: {slug}\n"
            f"origin_plan: {self.origin.path.relative_to(self.repo).as_posix()}\n"
            f"fixing_plan: {self.fixing.path.relative_to(self.repo).as_posix()}\n"
            "plan_link_mode: child_record_only\n"
            f"source_artifact: {source_artifact}\n"
            "---\n\n# Return\n",
            encoding="utf-8",
        )
        self.failures.import_repository()
        second = next(
            node for node in self.failures.audit().nodes if node.summary_slug == slug
        )
        output_path = "docs/plans/plugins/01/2026-07-23-combined-closeout-output.md"
        delivery_paths: tuple[str, ...] = ()
        lifecycle_keys = tuple(
            sorted({self.lifecycle_key, second.lifecycle_key}, key=str.casefold)
        )
        (self.repo / output_path).write_text(
            "---\n"
            "record_kind: failure_closeout_delivery\n"
            "status: accepted\n"
            f"lifecycle_keys_json: {json.dumps(lifecycle_keys)}\n"
            f"delivery_paths_json: {json.dumps(delivery_paths)}\n"
            "---\n\n# Combined closeout output\n",
            encoding="utf-8",
        )
        additional_paths = (output_path,)
        paths = tuple(
            sorted(
                {
                    *self.paths,
                    self.source_path,
                    self.fixed_path,
                    self.return_path,
                    *related,
                    fixed_path,
                    return_path,
                    source_artifact,
                    *additional_paths,
                },
                key=str.casefold,
            )
        )
        snapshot = self.snapshots.create(
            session_id=self.session_id,
            paths=list(paths),
            baseline_epoch=self.baselines.current().epoch_id,
            purpose="failure-lifecycle-combined-closeout",
        )
        source_manifest = {
            path: digest.upper()
            for path, digest in snapshot.manifest.items()
            if not path.casefold().startswith("docs/plans/")
        }
        self._insert_green_validation(source_manifest=source_manifest)
        return snapshot, second.lifecycle_key, output_path, additional_paths

    def _prepare(self):
        self._insert_green_validation()
        return self.service.prepare(
            session_id=self.session_id,
            snapshot_id=self.snapshot.snapshot_id,
            lifecycle_key=self.lifecycle_key,
            validation_command=[
                "cargo",
                "+1.94.1",
                "metadata",
                "--format-version",
                "1",
                "--locked",
                "--offline",
                "--no-deps",
            ],
            validation_job_id="job-green",
            validation_run_id="run-green",
            executor_thread_id="executor-thread",
            actor=self.session_id,
        )

    def _bind_reviewer_thread(self, thread_id: str) -> None:
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO codex_sessions(
                       thread_id, rollout_path, source_location, state, cwd,
                       originator, cli_version, thread_source, last_event,
                       last_turn_id, bound_session_id, diagnostic_code,
                       first_seen_at, last_activity_at, last_synced_at,
                       source_mtime_ns, source_size, missing_scan_count
                   ) VALUES (?, ?, 'active', 'active', ?, 'Codex Desktop',
                             'test', 'active', 'task_started', 'turn-review',
                             ?, NULL, ?, ?, ?, 1, 1, 0)""",
                (
                    thread_id,
                    str(self.repo / f"{thread_id}.jsonl"),
                    str(self.repo),
                    thread_id,
                    now,
                    now,
                    now,
                ),
            )

    def _insert_green_validation(self, *, source_manifest: dict[str, str] | None = None):
        command = [
            "cargo",
            "+1.94.1",
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--offline",
            "--no-deps",
        ]
        compatibility = {
            "build_config": "profile=metadata;locked=true;offline=true;no_deps=true",
            "platform": "windows",
            "toolchain": "1.94.1@x86_64-pc-windows-msvc",
            "workspace": "Cargo.toml",
            "source_manifest": source_manifest
            or {path: self.snapshot.manifest[path].upper() for path in self.paths},
        }
        environment = {
            "CARGO_NET_OFFLINE": "true",
            "RUSTFLAGS": "-C debuginfo=0 -C codegen-units=16",
        }
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute("DELETE FROM cargo_job_runs WHERE run_id='run-green'")
            connection.execute("DELETE FROM cargo_jobs WHERE job_id='job-green'")
            connection.execute(
                """INSERT INTO cargo_jobs(
                       job_id, session_id, lane_kind, target_dir, status, dry_run,
                       command_json, exit_code, created_at, last_heartbeat_at,
                       started_at, finished_at, released_at, target_key,
                       compatibility_json, compatibility_key,
                       process_tree_live_pids_json,
                       process_tree_exited_at
                   ) VALUES ('job-green', ?, 'test', 'target', 'released', 0,
                              ?, 0, ?, ?, ?, ?, ?, 'target', ?,
                              'compatibility-green', '[]', ?)""",
                (
                    self.session_id,
                    json.dumps(command),
                    now,
                    now,
                    now,
                    now,
                    now,
                    json.dumps(compatibility),
                    now,
                ),
            )
            connection.execute(
                """INSERT INTO cargo_job_runs(
                       run_id, job_id, session_id, command_json, status, exit_code,
                       stdout_path, stderr_path, stdout_tail, stderr_tail,
                       started_at, completed_at, environment_json
                    ) VALUES ('run-green', 'job-green', ?, ?, 'completed', 0,
                              'stdout.log', 'stderr.log', 'ok', '', ?, ?, ?)""",
                (
                    self.session_id,
                    json.dumps(command),
                    now,
                    now,
                    json.dumps(environment),
                ),
            )
        return command

    def test_prepare_binds_exact_snapshot_and_preserves_other_open_lifecycle(self) -> None:
        prepared = self._prepare()

        self.assertEqual(7, len(prepared.paths))
        self.assertEqual(self.lifecycle_key, prepared.lifecycle_key)
        self.assertEqual((self.return_path,), prepared.return_records)
        self.assertEqual(1, len(prepared.preserved_open_failures))
        self.assertEqual(
            self.other_paths,
            prepared.preserved_open_failures[0].related_code,
        )

    def test_combined_closeout_sorts_targets_and_commits_one_exact_manifest(self) -> None:
        snapshot, second_key, delivery_record, additional_paths = self._combined_candidate(
            include_noop_provenance=True
        )
        expected_keys = tuple(sorted({self.lifecycle_key, second_key}, key=str.casefold))
        prepared = self.service.prepare_combined(
            session_id=self.session_id,
            snapshot_id=snapshot.snapshot_id,
            lifecycle_keys=[second_key, self.lifecycle_key, second_key],
            delivery_records=[delivery_record],
            validation_command=[
                "cargo",
                "+1.94.1",
                "metadata",
                "--format-version",
                "1",
                "--locked",
                "--offline",
                "--no-deps",
            ],
            validation_job_id="job-green",
            validation_run_id="run-green",
            executor_thread_id="executor-thread",
            actor=self.session_id,
        )

        self.assertEqual(expected_keys, prepared.lifecycle_keys)
        self.assertIn("README.md", prepared.paths)
        self.assertIn(
            ".codex/state/session-coordinator/cargo-runs/proof/stderr.log",
            prepared.paths,
        )
        self.assertIn(
            "docs/plans/plugins/01/failure-2026-07-21-bridge-second-atomic-target.md",
            prepared.paths,
        )
        self.assertEqual(tuple(sorted(additional_paths, key=str.casefold)), prepared.additional_paths)
        self.assertEqual(2, len(prepared.target_artifacts))
        self.assertEqual(2, len(prepared.return_records))
        self.assertEqual(
            {"bridge-runtime-call-lock"},
            {
                failure.lifecycle_key.rsplit("|", 1)[-1]
                for failure in prepared.preserved_open_failures
            },
        )

        self.service.bind_validation(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            job_id="job-green",
            cargo_run_id="run-green",
            actor=self.session_id,
        )
        self.service.record_review(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            reviewer_session_id="reviewer-b",
            reviewer_thread_id="reviewer-b",
            critical_count=0,
            important_count=0,
            moderate_count=0,
            summary="independent combined C0/I0/M0 review",
        )
        proof_only = {
            ".codex/state/session-coordinator/cargo-runs/proof/stderr.log",
            "README.md",
            "docs/plans/plugins/01/failure-2026-07-21-bridge-second-atomic-target.md",
        }
        expected_commit_paths = tuple(
            path for path in prepared.paths if path not in proof_only
        )
        self.leases.acquire(self.session_id, list(expected_commit_paths))
        self.baselines.attribute(self.session_id, list(expected_commit_paths))
        acceptance = self.finalize._require_failure_closeouts_acceptance
        differing = self.finalize._worktree_paths_differing_from_head
        with (
            mock.patch.object(
                self.finalize,
                "_require_failure_closeouts_acceptance",
                wraps=acceptance,
            ) as revalidated,
            mock.patch.object(
                self.finalize,
                "_worktree_paths_differing_from_head",
                wraps=differing,
            ) as scanned,
        ):
            result = self.service.commit(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                summary="fix(plugins): close two failure lifecycles atomically",
                actor=self.session_id,
            )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.finalize.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual(sorted(expected_commit_paths), sorted(committed))
        self.assertTrue(revalidated.called)
        self.assertEqual(2, scanned.call_count)
        self.assertEqual(expected_keys, revalidated.call_args.args[1])
        self.assertEqual(prepared.paths, revalidated.call_args.args[2])
        with self.database.connect() as connection:
            request = connection.execute(
                "SELECT paths_json FROM finalize_requests WHERE request_id=?",
                (result.finalize.request_id,),
            ).fetchone()
        self.assertEqual(list(expected_commit_paths), json.loads(request["paths_json"]))

    def test_combined_closeout_rejects_omitted_owned_dirty_path(self) -> None:
        snapshot, second_key, delivery_record, _additional_paths = (
            self._combined_candidate(include_noop_provenance=True)
        )
        prepared = self.service.prepare_combined(
            session_id=self.session_id,
            snapshot_id=snapshot.snapshot_id,
            lifecycle_keys=[self.lifecycle_key, second_key],
            delivery_records=[delivery_record],
            validation_command=[
                "cargo",
                "+1.94.1",
                "metadata",
                "--format-version",
                "1",
                "--locked",
                "--offline",
                "--no-deps",
            ],
            validation_job_id="job-green",
            validation_run_id="run-green",
            executor_thread_id="executor-thread",
            actor=self.session_id,
        )
        self.service.bind_validation(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            job_id="job-green",
            cargo_run_id="run-green",
            actor=self.session_id,
        )
        self.service.record_review(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            reviewer_session_id="reviewer-b",
            reviewer_thread_id="reviewer-b",
            critical_count=0,
            important_count=0,
            moderate_count=0,
            summary="independent combined C0/I0/M0 review",
        )
        coordinator_state = ".codex/state/session-coordinator/"
        committable_proof = [
            path
            for path in prepared.paths
            if not path.casefold().startswith(coordinator_state)
        ]
        self.leases.acquire(self.session_id, committable_proof)
        self.baselines.attribute(self.session_id, committable_proof)
        omitted = "src/owned-but-omitted.rs"
        target = self.repo / omitted
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("owned but omitted\n", encoding="utf-8")
        self.baselines.attribute(self.session_id, [omitted])

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.commit(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                summary="fix(plugins): reject omitted owned path",
                actor=self.session_id,
            )

        self.assertEqual("finalize_owned_path_omitted", rejected.exception.code)
        self.assertEqual([omitted], rejected.exception.details["paths"])

    def test_combined_closeout_rejects_unattributed_material_path(self) -> None:
        snapshot, second_key, delivery_record, _additional_paths = (
            self._combined_candidate(include_noop_provenance=True)
        )
        prepared = self.service.prepare_combined(
            session_id=self.session_id,
            snapshot_id=snapshot.snapshot_id,
            lifecycle_keys=[self.lifecycle_key, second_key],
            delivery_records=[delivery_record],
            validation_command=[
                "cargo",
                "+1.94.1",
                "metadata",
                "--format-version",
                "1",
                "--locked",
                "--offline",
                "--no-deps",
            ],
            validation_job_id="job-green",
            validation_run_id="run-green",
            executor_thread_id="executor-thread",
            actor=self.session_id,
        )
        self.service.bind_validation(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            job_id="job-green",
            cargo_run_id="run-green",
            actor=self.session_id,
        )
        self.service.record_review(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            reviewer_session_id="reviewer-b",
            reviewer_thread_id="reviewer-b",
            critical_count=0,
            important_count=0,
            moderate_count=0,
            summary="independent combined C0/I0/M0 review",
        )
        unattributed = "src/combined/second.rs"
        owned = [
            path
            for path in prepared.paths
            if not path.casefold().startswith(".codex/state/")
            and path != unattributed
            and path not in {
                "README.md",
                "docs/plans/plugins/01/failure-2026-07-21-bridge-second-atomic-target.md",
            }
        ]
        self.leases.acquire(self.session_id, owned)
        self.baselines.attribute(self.session_id, owned)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.commit(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                summary="fix(plugins): reject unattributed material path",
                actor=self.session_id,
            )

        self.assertEqual("finalize_unattributed_path", rejected.exception.code)
        self.assertIn(unattributed, str(rejected.exception))

    def test_combined_closeout_rejects_proof_only_tamper(self) -> None:
        snapshot, second_key, delivery_record, _additional_paths = (
            self._combined_candidate(include_noop_provenance=True)
        )
        prepared = self.service.prepare_combined(
            session_id=self.session_id,
            snapshot_id=snapshot.snapshot_id,
            lifecycle_keys=[self.lifecycle_key, second_key],
            delivery_records=[delivery_record],
            validation_command=[
                "cargo",
                "+1.94.1",
                "metadata",
                "--format-version",
                "1",
                "--locked",
                "--offline",
                "--no-deps",
            ],
            validation_job_id="job-green",
            validation_run_id="run-green",
            executor_thread_id="executor-thread",
            actor=self.session_id,
        )
        state_provenance = (
            self.repo
            / ".codex/state/session-coordinator/cargo-runs/proof/stderr.log"
        )
        state_provenance.write_text(
            "tampered proof-only path\n", encoding="utf-8"
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.bind_validation(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                job_id="job-green",
                cargo_run_id="run-green",
                actor=self.session_id,
            )

        self.assertEqual("failure_closeout_snapshot_drift", rejected.exception.code)
        self.assertEqual(
            [".codex/state/session-coordinator/cargo-runs/proof/stderr.log"],
            rejected.exception.details["paths"],
        )

    def test_combined_closeout_rejects_untyped_extra_path(self) -> None:
        snapshot, second_key, delivery_record, _additional_paths = self._combined_candidate()
        unrelated = "unrelated.txt"
        (self.repo / unrelated).write_text("not owned by delivery record\n", encoding="utf-8")
        expanded = self.snapshots.create(
            session_id=self.session_id,
            paths=[*snapshot.manifest, unrelated],
            baseline_epoch=self.baselines.current().epoch_id,
            purpose="failure-lifecycle-combined-untyped-extra",
        )
        source_manifest = {
            path: digest.upper()
            for path, digest in expanded.manifest.items()
            if not path.casefold().startswith("docs/plans/")
        }
        self._insert_green_validation(source_manifest=source_manifest)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.prepare_combined(
                session_id=self.session_id,
                snapshot_id=expanded.snapshot_id,
                lifecycle_keys=[self.lifecycle_key, second_key],
                delivery_records=[delivery_record],
                validation_command=self._insert_green_validation(
                    source_manifest=source_manifest
                ),
                validation_job_id="job-green",
                validation_run_id="run-green",
                executor_thread_id="executor-thread",
                actor=self.session_id,
            )

        self.assertEqual("failure_closeout_manifest_not_exact", rejected.exception.code)
        self.assertIn(unrelated, rejected.exception.details["extra"])

    def test_combined_closeout_rejects_preserved_failure_overlap(self) -> None:
        snapshot, second_key, delivery_record, additional_paths = self._combined_candidate()
        overlap = self.other_paths[0]
        record = self.repo / delivery_record
        metadata = record.read_text(encoding="utf-8").replace(
            f"delivery_paths_json: {json.dumps(additional_paths[:-1])}",
            f"delivery_paths_json: {json.dumps((*additional_paths[:-1], overlap))}",
        )
        record.write_text(metadata, encoding="utf-8")
        expanded = self.snapshots.create(
            session_id=self.session_id,
            paths=[*snapshot.manifest, overlap],
            baseline_epoch=self.baselines.current().epoch_id,
            purpose="failure-lifecycle-combined-preserved-overlap",
        )
        source_manifest = {
            path: digest.upper()
            for path, digest in expanded.manifest.items()
            if not path.casefold().startswith("docs/plans/")
        }
        command = self._insert_green_validation(source_manifest=source_manifest)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.prepare_combined(
                session_id=self.session_id,
                snapshot_id=expanded.snapshot_id,
                lifecycle_keys=[self.lifecycle_key, second_key],
                delivery_records=[delivery_record],
                validation_command=command,
                validation_job_id="job-green",
                validation_run_id="run-green",
                executor_thread_id="executor-thread",
                actor=self.session_id,
            )

        self.assertEqual("failure_closeout_preserved_scope_overlap", rejected.exception.code)
        self.assertEqual([overlap], rejected.exception.details["paths"])

    def test_prepare_requires_prior_failure_deletion_from_real_child_return(self) -> None:
        fixture = FailureGraphFixture(self.repo)
        source = fixture.add_handoff(
            self.origin,
            self.fixing,
            "real-child-return-deletion",
            created_at="2026-07-22",
        )
        self._add_child_scope(source, ("Cargo.toml",))
        self.failures.import_repository()
        lifecycle_key = next(
            node.lifecycle_key
            for node in self.failures.audit().nodes
            if node.summary_slug == "real-child-return-deletion"
        )
        fixed = self.failures.return_fixed(
            lifecycle_key,
            FailureResolution(
                root_cause="The source Failure was not part of the closeout manifest.",
                architecture_fix="The closeout now binds the prior artifact deletion.",
                validation="The exact delete/add snapshot contract is covered.",
                return_summary="The origin can consume one atomic lifecycle move.",
            ),
            resolved_at=date(2026, 7, 23),
        )
        source_path = source.relative_to(self.repo).as_posix()
        fixed_path = fixed.relative_to(self.repo).as_posix()
        return_path = (
            self.fixing.child.relative_to(self.repo).as_posix()
            + "/2026-07-23-real-child-return-deletion-return.md"
        )
        incomplete = self.snapshots.create(
            session_id=self.session_id,
            paths=["Cargo.toml", fixed_path, return_path],
            baseline_epoch=self.baselines.current().epoch_id,
            purpose="failure-lifecycle-missing-source-deletion",
        )
        source_manifest = {"Cargo.toml": incomplete.manifest["Cargo.toml"].upper()}
        command = self._insert_green_validation(source_manifest=source_manifest)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.prepare(
                session_id=self.session_id,
                snapshot_id=incomplete.snapshot_id,
                lifecycle_key=lifecycle_key,
                validation_command=command,
                validation_job_id="job-green",
                validation_run_id="run-green",
                executor_thread_id="executor-thread",
                actor=self.session_id,
            )
        self.assertEqual("failure_closeout_manifest_not_exact", rejected.exception.code)
        self.assertIn(source_path, rejected.exception.details["missing"])

        complete = self.snapshots.create(
            session_id=self.session_id,
            paths=["Cargo.toml", source_path, fixed_path, return_path],
            baseline_epoch=self.baselines.current().epoch_id,
            purpose="failure-lifecycle-exact-source-deletion",
        )
        self.assertIsNone(complete.manifest[source_path])
        prepared = self.service.prepare(
            session_id=self.session_id,
            snapshot_id=complete.snapshot_id,
            lifecycle_key=lifecycle_key,
            validation_command=command,
            validation_job_id="job-green",
            validation_run_id="run-green",
            executor_thread_id="executor-thread",
            actor=self.session_id,
        )
        self.assertIn(source_path, prepared.paths)

    def test_validation_rejects_source_manifest_drift(self) -> None:
        prepared = self._prepare()
        source = {path: self.snapshot.manifest[path].upper() for path in self.paths}
        source["Cargo.toml"] = "0" * 64
        self._insert_green_validation(source_manifest=source)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.bind_validation(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                job_id="job-green",
                cargo_run_id="run-green",
                actor=self.session_id,
            )

        self.assertEqual("failure_closeout_validation_source_drift", rejected.exception.code)

    def test_validation_rejects_prepare_bound_environment_drift(self) -> None:
        prepared = self._prepare()
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_job_runs SET environment_json=? WHERE run_id='run-green'",
                (json.dumps({"RUSTFLAGS": "-C opt-level=3"}),),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.bind_validation(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                job_id="job-green",
                cargo_run_id="run-green",
                actor=self.session_id,
            )

        self.assertEqual(
            "failure_closeout_validation_contract_drift", rejected.exception.code
        )

    def test_prepare_rejects_overlap_with_an_open_failure_scope(self) -> None:
        content = self.other_failure.read_text(encoding="utf-8")
        self.other_failure.write_text(
            content.replace(self.other_paths[0], "Cargo.toml"),
            encoding="utf-8",
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self._prepare()

        self.assertEqual(
            "failure_closeout_preserved_scope_overlap",
            rejected.exception.code,
        )

    def test_review_requires_independent_zero_finding_evidence(self) -> None:
        prepared = self._prepare()
        with self.assertRaises(CoordinatorError) as same_executor:
            self.service.record_review(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                reviewer_session_id=self.session_id,
                reviewer_thread_id=self.session_id,
                critical_count=0,
                important_count=0,
                moderate_count=0,
                summary="clean",
            )
        self.assertEqual("failure_closeout_review_not_independent", same_executor.exception.code)

        review = self.service.record_review(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            reviewer_session_id="reviewer-b",
            reviewer_thread_id="reviewer-b",
            critical_count=0,
            important_count=0,
            moderate_count=1,
            summary="one moderate finding",
        )
        self.assertEqual("rejected", review.verdict)

    def test_review_rejects_borrowed_reviewer_session_identity(self) -> None:
        prepared = self._prepare()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.record_review(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                reviewer_session_id="reviewer-b",
                reviewer_thread_id="other-thread",
                critical_count=0,
                important_count=0,
                moderate_count=0,
                summary="spoofed clean review",
            )

        self.assertEqual(
            "failure_closeout_reviewer_provenance_invalid", rejected.exception.code
        )

    def test_review_rejects_executor_task_using_an_alias_session(self) -> None:
        prepared = self._prepare()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.record_review(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                reviewer_session_id="executor-thread",
                reviewer_thread_id="executor-thread",
                critical_count=0,
                important_count=0,
                moderate_count=0,
                summary="same task under another semantic session",
            )

        self.assertEqual("failure_closeout_review_not_independent", rejected.exception.code)

    def test_commit_revalidates_terminal_validation_under_git_mutex(self) -> None:
        prepared = self._prepare()
        self._insert_green_validation()
        self.service.bind_validation(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            job_id="job-green",
            cargo_run_id="run-green",
            actor=self.session_id,
        )
        self.service.record_review(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            reviewer_session_id="reviewer-b",
            reviewer_thread_id="reviewer-b",
            critical_count=0,
            important_count=0,
            moderate_count=0,
            summary="independent C0/I0/M0 review",
        )
        self.leases.acquire(self.session_id, list(prepared.paths))
        self.baselines.attribute(self.session_id, list(prepared.paths))
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_job_runs SET exit_code=101 WHERE run_id='run-green'"
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.commit(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                summary="fix(plugins): reject stale validation",
                actor=self.session_id,
            )

        self.assertEqual("failure_closeout_validation_not_green", rejected.exception.code)

    def test_post_cas_crash_is_reconciled_and_notified(self) -> None:
        prepared = self._prepare()
        self._insert_green_validation()
        self.service.bind_validation(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            job_id="job-green",
            cargo_run_id="run-green",
            actor=self.session_id,
        )
        self.service.record_review(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            reviewer_session_id="reviewer-b",
            reviewer_thread_id="reviewer-b",
            critical_count=0,
            important_count=0,
            moderate_count=0,
            summary="independent C0/I0/M0 review",
        )
        self.leases.acquire(self.session_id, list(prepared.paths))
        self.baselines.attribute(self.session_id, list(prepared.paths))

        with mock.patch.object(
            self.service,
            "_complete_committed",
            side_effect=RuntimeError("simulated post-CAS process loss"),
        ):
            with self.assertRaisesRegex(RuntimeError, "post-CAS"):
                self.service.commit(
                    session_id=self.session_id,
                    closeout_id=prepared.closeout_id,
                    summary="fix(plugins): recover committed lifecycle",
                    actor=self.session_id,
                )

        recovered = self.service.recover_pending_commits()

        self.assertEqual(1, len(recovered))
        self.assertEqual(1, len(self.messages))
        with self.database.connect() as connection:
            committed = connection.execute(
                """SELECT COUNT(*) FROM events
                   WHERE session_id=? AND event_type=?""",
                (self.session_id, self.service.COMMITTED_EVENT),
            ).fetchone()[0]
        self.assertEqual(1, committed)

    def test_startup_recovery_accepts_exact_commit_after_head_advances(self) -> None:
        prepared = self._prepare()
        self.service.bind_validation(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            job_id="job-green",
            cargo_run_id="run-green",
            actor=self.session_id,
        )
        self.service.record_review(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            reviewer_session_id="reviewer-b",
            reviewer_thread_id="reviewer-b",
            critical_count=0,
            important_count=0,
            moderate_count=0,
            summary="independent C0/I0/M0 review",
        )
        self.leases.acquire(self.session_id, list(prepared.paths))
        self.baselines.attribute(self.session_id, list(prepared.paths))
        original_accept = self.baselines.accept_commit
        self.baselines.accept_commit = mock.Mock(
            side_effect=RuntimeError("simulated post-CAS baseline loss")
        )

        with self.assertRaisesRegex(RuntimeError, "post-CAS"):
            self.service.commit(
                session_id=self.session_id,
                closeout_id=prepared.closeout_id,
                summary="fix(plugins): recover historical exact closeout",
                actor=self.session_id,
            )

        self.baselines.accept_commit = original_accept
        exact_commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        followup = self.repo / "followup.txt"
        followup.write_text("later managed work\n", encoding="utf-8")
        subprocess.run(["git", "add", "followup.txt"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: later managed work"],
            cwd=self.repo,
            check=True,
        )
        advanced_head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        self.assertNotEqual(exact_commit, advanced_head)
        (self.database.path.parent / "coordinator.lock").write_text(
            json.dumps({"pid": os.getpid()}), encoding="utf-8"
        )

        self.finalize.recover_stale_mutex()
        recovered = self.service.recover_pending_commits()

        self.assertEqual(1, len(recovered))
        self.assertEqual(
            advanced_head,
            subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=self.repo,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip(),
        )
        with self.database.connect() as connection:
            request = connection.execute(
                "SELECT status, commit_sha, index_snapshot FROM finalize_requests "
                "WHERE ref_updated_sha=?",
                (exact_commit,),
            ).fetchone()
        self.assertEqual("committed", request["status"])
        self.assertEqual(exact_commit, request["commit_sha"])
        self.assertIsNone(request["index_snapshot"])

    def test_commit_is_exact_notifies_and_keeps_resolving_failure_open(self) -> None:
        prepared = self._prepare()
        self._insert_green_validation()
        validation = self.service.bind_validation(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            job_id="job-green",
            cargo_run_id="run-green",
            actor=self.session_id,
        )
        self.assertEqual("accepted", validation.verdict)
        review = self.service.record_review(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            reviewer_session_id="reviewer-b",
            reviewer_thread_id="reviewer-b",
            critical_count=0,
            important_count=0,
            moderate_count=0,
            summary="independent C0/I0/M0 review",
        )
        self.assertEqual("accepted", review.verdict)
        self.leases.acquire(self.session_id, list(prepared.paths))
        self.baselines.attribute(self.session_id, list(prepared.paths))
        foreign = self.repo / "foreign.txt"
        foreign.write_text("foreign staged\n", encoding="utf-8")
        subprocess.run(["git", "add", "foreign.txt"], cwd=self.repo, check=True)

        result = self.service.commit(
            session_id=self.session_id,
            closeout_id=prepared.closeout_id,
            summary="fix(plugins): close arc swap lock lifecycle",
            actor=self.session_id,
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.finalize.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        staged = subprocess.run(
            ["git", "diff", "--cached", "--name-only"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual(sorted(prepared.paths), sorted(committed))
        self.assertEqual(["foreign.txt"], staged)
        self.assertEqual(
            SessionStatus.RESOLVING_FAILURE,
            self.sessions.get(self.session_id).status,
        )
        self.failures.import_repository()
        remaining = self.failures.open_for_plan(
            self.fixing.path.relative_to(self.repo).as_posix()
        )
        other = next(node for node in remaining if node.summary_slug == "bridge-runtime-call-lock")
        self.assertEqual(self.other_paths, other.related_code)
        self.assertEqual("succeeded", result.notification.status)
        self.assertEqual(1, len(self.messages))
        self.assertIn(result.finalize.commit_sha, self.messages[0])
        self.assertTrue(result.shortstat)

    def test_cli_exposes_four_explicit_closeout_phases(self) -> None:
        parser = _parser()
        prepared = parser.parse_args(
            [
                "failure",
                "closeout-prepare",
                self.lifecycle_key,
                "--session-id",
                self.session_id,
                "--snapshot-id",
                str(self.snapshot.snapshot_id),
                "--validation-command-json",
                '["cargo","metadata"]',
                "--job-id",
                "job-green",
                "--cargo-run-id",
                "run-green",
            ]
        )
        validation = parser.parse_args(
            [
                "failure",
                "closeout-validate",
                "closeout-id",
                "--session-id",
                self.session_id,
                "--job-id",
                "job-id",
                "--cargo-run-id",
                "run-id",
            ]
        )
        review = parser.parse_args(
            [
                "failure",
                "closeout-review",
                "closeout-id",
                "--executor-session-id",
                self.session_id,
                "--critical-count",
                "0",
                "--important-count",
                "0",
                "--moderate-count",
                "0",
                "--summary",
                "clean",
            ]
        )
        committed = parser.parse_args(
            [
                "failure",
                "closeout-commit",
                "closeout-id",
                "--session-id",
                self.session_id,
                "--summary",
                "fix(plugins): close exact lifecycle",
            ]
        )

        self.assertEqual("closeout-prepare", prepared.failure_command)
        self.assertEqual('["cargo","metadata"]', prepared.validation_command_json)
        self.assertEqual(0, review.moderate_count)
        self.assertEqual("closeout-commit", committed.failure_command)

        combined = parser.parse_args(
            [
                "failure",
                "closeout-prepare-combined",
                "lifecycle-b",
                "lifecycle-a",
                "--delivery-record",
                "docs/plans/plugins/01/output.md",
                "--session-id",
                self.session_id,
                "--snapshot-id",
                str(self.snapshot.snapshot_id),
                "--validation-command-json",
                '["cargo","metadata"]',
                "--job-id",
                "job-id",
                "--cargo-run-id",
                "run-id",
            ]
        )
        self.assertEqual("closeout-prepare-combined", combined.failure_command)
        self.assertEqual(["lifecycle-b", "lifecycle-a"], combined.lifecycle_keys)

    def test_cli_argument_errors_are_wrapper_safe_json(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            exit_code = main(
                [
                    "--json",
                    "failure",
                    "closeout-validate",
                    "closeout-id",
                ]
            )

        payload = json.loads(output.getvalue())
        self.assertEqual(2, exit_code)
        self.assertEqual("error", payload["status"])
        self.assertEqual("cli_arguments_invalid", payload["error"]["code"])


if __name__ == "__main__":
    unittest.main()
