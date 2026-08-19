from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from contextlib import contextmanager
from pathlib import Path
from unittest import mock

from tools.session_coordinator.cli import _parser
from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.database import Database
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.isolated_patch_finalize import (
    IsolatedPatchFinalizeService,
)
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


_TARGET = "tools/construct.rs"
_LONG_CHECKOUT_PATHS = (
    "docs/tests/runtime/shader/reflection_probe_capture_product_20260711/"
    "render/ibl-derived/v0000002f2c526421/"
    "d0cb4901c81d11ac77301bba77ee9169f9fba9712ec9c314ac26d73f6f8724c9/"
    "face_0064_mips_07.zribl",
    "docs/tests/runtime/shader/reflection_probe_capture_product_20260711/"
    "render/ibl-source/v0000002f2c526421/"
    "d0cb4901c81d11ac77301bba77ee9169f9fba9712ec9c314ac26d73f6f8724c9/"
    "face_0064_mips_07.zcube",
)
_BASE_SOURCE = """fn construct() {
    State {
        degrade_ladder: Default::default(),
        graphics_debugger,
    };
}
"""
_DERIVED_SOURCE = """fn construct() {
    State {
        degrade_ladder: Default::default(),
        graphics_debugger,
        viewport_products: Default::default(),
    };
}
"""
_MIXED_SOURCE = """fn construct() {
    State {
        environment_ibl_hydration_cache: Default::default(),
        degrade_ladder: Default::default(),
        graphics_debugger,
        viewport_products: Default::default(),
    };
}
"""
_PATCH = f"""diff --git a/{_TARGET} b/{_TARGET}
--- a/{_TARGET}
+++ b/{_TARGET}
@@ -1,6 +1,7 @@
 fn construct() {{
     State {{
         degrade_ladder: Default::default(),
         graphics_debugger,
+        viewport_products: Default::default(),
     }};
 }}
""".encode("utf-8")


class IsolatedPatchFinalizeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.target = self.repo / _TARGET
        self.target.parent.mkdir(parents=True)
        self.target.write_text(_BASE_SOURCE, encoding="utf-8")
        subprocess.run(["git", "add", _TARGET], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "test: add target"],
            cwd=self.repo,
            check=True,
        )
        self.base_head = self._git("rev-parse", "HEAD")
        self.base_blob = self._git("rev-parse", f"HEAD:{_TARGET}")

        self.database = Database(root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, plan_path, status, created_at, updated_at,
                    last_heartbeat_at
                ) VALUES (
                    'render-owner', 'docs/plans/render/17-render.md', 'active',
                    '2026-08-15T00:00:00+00:00', '2026-08-15T00:00:00+00:00',
                    '2026-08-15T00:00:00+00:00'
                )
                """
            )
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=900,
            grace_seconds=120,
        )
        self.assertTrue(self.leases.acquire("render-owner", [_TARGET]).acquired)
        self.service = IsolatedPatchFinalizeService(
            self.database,
            self.repo,
            self.baselines,
            self.leases,
        )

    def test_finalize_derives_one_blob_and_preserves_283_path_index_and_worktree(self) -> None:
        self._stage_foreign_paths(283)
        self.target.write_text(_MIXED_SOURCE, encoding="utf-8")
        worktree_before = self.target.read_bytes()
        staged_before = self._git_bytes(
            "diff", "--cached", "--binary", "--no-ext-diff", "--no-renames", "HEAD", "--"
        )
        staged_paths_before = self._staged_paths()

        result = self.service.finalize(
            session_id="render-owner",
            target=_TARGET,
            patch=_PATCH,
            expected_head=self.base_head,
            expected_blob=self.base_blob,
            message="fix(coordinator): isolate target patch",
            validation_commands=(
                (
                    sys.executable,
                    "-c",
                    (
                        "from pathlib import Path; "
                        f"value=Path(r'{_TARGET}').read_text(encoding='utf-8'); "
                        "assert 'viewport_products' in value; "
                        "assert 'environment_ibl_hydration_cache' not in value"
                    ),
                ),
            ),
        )

        self.assertEqual(self.base_head, result.base_head)
        self.assertEqual(self.base_blob, result.base_blob)
        self.assertEqual(hashlib.sha256(_PATCH).hexdigest(), result.patch_hash)
        self.assertEqual(
            self._git("rev-parse", f"{result.commit_sha}:{_TARGET}"),
            result.derived_blob,
        )
        self.assertEqual(
            _DERIVED_SOURCE.rstrip(),
            self._git("show", f"{result.commit_sha}:{_TARGET}"),
        )
        self.assertEqual(worktree_before, self.target.read_bytes())
        self.assertEqual(staged_paths_before, self._staged_paths())

        self.assertEqual(283, result.staged_path_count)
        self.assertEqual(
            staged_before,
            self._git_bytes(
                "diff",
                "--cached",
                "--binary",
                "--no-ext-diff",
                "--no-renames",
                "HEAD",
                "--",
            ),
        )
        self.assertEqual(
            hashlib.sha256(staged_before).hexdigest(),
            result.staged_projection_fingerprint,
        )
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT payload_json FROM events
                WHERE event_type='maintenance.isolated_patch_finalized'
                ORDER BY event_id DESC LIMIT 1
                """
            ).fetchone()
            request = connection.execute(
                "SELECT status, commit_sha, maintenance FROM finalize_requests WHERE request_id=?",
                (result.request_id,),
            ).fetchone()
        self.assertIsNotNone(row)
        payload = json.loads(row["payload_json"])
        self.assertEqual(self.base_head, payload["baseHead"])
        self.assertEqual(self.base_blob, payload["baseBlob"])
        self.assertEqual(result.patch_hash, payload["patchHash"])
        self.assertEqual(result.derived_blob, payload["derivedBlob"])
        self.assertEqual(283, payload["stagedPathCount"])
        self.assertEqual("committed", request["status"])
        self.assertEqual(result.commit_sha, request["commit_sha"])
        self.assertEqual(1, request["maintenance"])

    @unittest.skipUnless(os.name == "nt", "Windows path-length regression")
    def test_validation_checkout_uses_short_root_for_tracked_long_paths(self) -> None:
        subprocess.run(
            ["git", "config", "core.longpaths", "true"],
            cwd=self.repo,
            check=True,
        )
        for relative in _LONG_CHECKOUT_PATHS:
            resource = self.repo / relative
            resource.parent.mkdir(parents=True, exist_ok=True)
            resource.write_bytes(b"fixture")
        subprocess.run(
            ["git", "add", "--", *_LONG_CHECKOUT_PATHS],
            cwd=self.repo,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-qm", "test: add long checkout resources"],
            cwd=self.repo,
            check=True,
        )
        self.baselines.refresh_for_head_change()
        long_base = self.repo.parent / ("long-checkout-base-" + "x" * 96)
        long_base.mkdir()
        short_base = Path.home().resolve()
        short_children_before = set(short_base.glob(".zr-ip-*"))
        service = IsolatedPatchFinalizeService(
            self.database,
            self.repo,
            self.baselines,
            self.leases,
            checkout_base_candidates=(long_base, short_base),
        )
        validation = (
            sys.executable,
            "-c",
            "import os; from pathlib import Path; "
            "root=Path(os.environ['ZR_ISOLATED_PATCH_ROOT']); "
            f"assert root.parent == Path({str(short_base)!r}); "
            "assert max(len(str(root / Path(value))) for value in "
            f"{_LONG_CHECKOUT_PATHS!r}) <= 248; "
            + "; ".join(
                f"assert Path({relative!r}).is_file()"
                for relative in _LONG_CHECKOUT_PATHS
            ),
        )

        result = service.finalize(
            session_id="render-owner",
            target=_TARGET,
            patch=_PATCH,
            expected_head=self.base_head,
            expected_blob=self.base_blob,
            message="fix(coordinator): validate long checkout paths",
            validation_commands=(validation,),
        )

        self.assertEqual(result.commit_sha, self._git("rev-parse", "HEAD"))
        for relative in _LONG_CHECKOUT_PATHS:
            self.assertEqual("fixture", self._git("show", f"HEAD:{relative}"))
        self.assertEqual([], list(long_base.glob(".zr-ip-*")))
        self.assertEqual(short_children_before, set(short_base.glob(".zr-ip-*")))

    def test_validation_checkout_rejects_reparse_root(self) -> None:
        root = mock.Mock(spec=Path)
        root.is_dir.return_value = True
        root.is_symlink.return_value = False
        root.is_junction.return_value = True

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.checkout_roots.require_private_root(root)

        self.assertEqual(
            "isolated_patch_checkout_root_unsafe", rejected.exception.code
        )

    def test_expected_head_may_be_ancestor_when_target_blob_is_unchanged(self) -> None:
        unrelated = self.repo / "tools" / "unrelated.rs"
        unrelated.write_text("const VALUE: usize = 1;\n", encoding="utf-8")
        subprocess.run(["git", "add", "tools/unrelated.rs"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "test: advance unrelated head"],
            cwd=self.repo,
            check=True,
        )
        parent_head = self._git("rev-parse", "HEAD")
        self.baselines.refresh_for_head_change()
        self.target.write_text(_MIXED_SOURCE, encoding="utf-8")

        result = self._finalize()

        self.assertEqual(parent_head, result.parent_head)
        self.assertEqual(parent_head, self._git("rev-parse", f"{result.commit_sha}^"))

    def test_target_blob_drift_rejects_without_changing_shared_state(self) -> None:
        self.target.write_text(_BASE_SOURCE.replace("State", "ChangedState"), encoding="utf-8")
        subprocess.run(["git", "add", _TARGET], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-qm", "test: drift target"], cwd=self.repo, check=True)
        self.baselines.refresh_for_head_change()
        self.target.write_text(_MIXED_SOURCE, encoding="utf-8")
        head_before = self._git("rev-parse", "HEAD")
        index_before = self._git_bytes("diff", "--cached", "--binary", "HEAD", "--")
        worktree_before = self.target.read_bytes()

        with self.assertRaises(CoordinatorError) as rejected:
            self._finalize()

        self.assertEqual("isolated_patch_target_blob_changed", rejected.exception.code)
        self.assertEqual(head_before, self._git("rev-parse", "HEAD"))
        self.assertEqual(
            index_before,
            self._git_bytes("diff", "--cached", "--binary", "HEAD", "--"),
        )
        self.assertEqual(worktree_before, self.target.read_bytes())

    def test_patch_touching_another_path_is_rejected(self) -> None:
        patch = b"""diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-baseline
+changed
"""
        head_before = self._git("rev-parse", "HEAD")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                session_id="render-owner",
                target=_TARGET,
                patch=patch,
                expected_head=self.base_head,
                expected_blob=self.base_blob,
                message="fix(coordinator): reject cross target patch",
                validation_commands=((sys.executable, "-c", "pass"),),
            )

        self.assertEqual("isolated_patch_scope_mismatch", rejected.exception.code)
        self.assertEqual(head_before, self._git("rev-parse", "HEAD"))

    def test_patch_with_credential_assignment_is_rejected_before_validation(self) -> None:
        credential_name = "pass" + "word"
        secret_patch = f"""diff --git a/{_TARGET} b/{_TARGET}
--- a/{_TARGET}
+++ b/{_TARGET}
@@ -1,6 +1,7 @@
 fn construct() {{
     State {{
         degrade_ladder: Default::default(),
         graphics_debugger,
+        {credential_name}: \"not-a-real-secret\",
     }};
 }}
""".encode("utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                session_id="render-owner",
                target=_TARGET,
                patch=secret_patch,
                expected_head=self.base_head,
                expected_blob=self.base_blob,
                message="fix(coordinator): reject credential patch",
                validation_commands=((sys.executable, "-c", "pass"),),
            )

        self.assertEqual("isolated_patch_secret_detected", rejected.exception.code)

    def test_mode_change_is_rejected(self) -> None:
        mode_patch = f"""diff --git a/{_TARGET} b/{_TARGET}
old mode 100644
new mode 100755
""".encode("utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                session_id="render-owner",
                target=_TARGET,
                patch=mode_patch,
                expected_head=self.base_head,
                expected_blob=self.base_blob,
                message="fix(coordinator): reject target mode change",
                validation_commands=((sys.executable, "-c", "pass"),),
            )

        self.assertEqual("isolated_patch_target_mode_changed", rejected.exception.code)

    def test_index_cas_rejects_stage_after_precheck_without_overwriting_it(self) -> None:
        self._stage_foreign_paths(1)
        foreign = self.repo / "foreign" / "path_000.txt"
        main_head = self._git("rev-parse", "HEAD")
        original_aligned_index = self.service._aligned_index

        @contextmanager
        def stage_during_publication(*args, **kwargs):
            with original_aligned_index(*args, **kwargs) as aligned:
                foreign.write_text("concurrent-stage\n", encoding="utf-8")
                subprocess.run(
                    ["git", "add", "foreign/path_000.txt"],
                    cwd=self.repo,
                    check=True,
                )
                yield aligned

        self.service._aligned_index = stage_during_publication  # type: ignore[method-assign]
        with self.assertRaises(CoordinatorError) as rejected:
            self._finalize()

        self.assertEqual("isolated_patch_shared_index_changed", rejected.exception.code)
        self.assertEqual(main_head, self._git("rev-parse", "refs/heads/main"))
        staged = self._git("show", ":foreign/path_000.txt")
        self.assertEqual("concurrent-stage", staged)

    def test_worktree_drift_after_precheck_rejects_before_main_publication(self) -> None:
        main_head = self._git("rev-parse", "HEAD")
        original_aligned_index = self.service._aligned_index

        @contextmanager
        def edit_during_publication(*args, **kwargs):
            with original_aligned_index(*args, **kwargs) as aligned:
                self.target.write_text("concurrent edit\n", encoding="utf-8")
                yield aligned

        self.service._aligned_index = edit_during_publication  # type: ignore[method-assign]
        with self.assertRaises(CoordinatorError) as rejected:
            self._finalize()

        self.assertEqual("isolated_patch_worktree_changed", rejected.exception.code)
        self.assertEqual(main_head, self._git("rev-parse", "refs/heads/main"))
        self.assertEqual("concurrent edit\n", self.target.read_text(encoding="utf-8"))

    def test_worktree_drift_after_index_lock_event_rejects_before_ref_cas(self) -> None:
        main_head = self._git("rev-parse", "refs/heads/main")
        original_record_event = self.service._record_event

        def edit_after_index_lock(session_id, event_type, payload):
            original_record_event(session_id, event_type, payload)
            if event_type == "maintenance.isolated_patch_index_locked":
                self.target.write_text("last-window edit\n", encoding="utf-8")

        self.service._record_event = edit_after_index_lock  # type: ignore[method-assign]
        with self.assertRaises(CoordinatorError) as rejected:
            self._finalize()

        self.assertEqual("isolated_patch_worktree_changed", rejected.exception.code)
        self.assertEqual(main_head, self._git("rev-parse", "refs/heads/main"))
        self.assertEqual("last-window edit\n", self.target.read_text(encoding="utf-8"))

    def test_same_oid_branch_switch_rejects_without_advancing_either_branch(self) -> None:
        main_head = self._git("rev-parse", "refs/heads/main")
        original_aligned_index = self.service._aligned_index

        @contextmanager
        def switch_branch_during_publication(*args, **kwargs):
            with original_aligned_index(*args, **kwargs) as aligned:
                subprocess.run(
                    ["git", "switch", "-q", "-c", "other"],
                    cwd=self.repo,
                    check=True,
                )
                yield aligned

        self.service._aligned_index = (  # type: ignore[method-assign]
            switch_branch_during_publication
        )
        with self.assertRaises(CoordinatorError) as rejected:
            self._finalize()

        self.assertEqual("isolated_patch_branch_changed", rejected.exception.code)
        self.assertEqual(main_head, self._git("rev-parse", "refs/heads/main"))
        self.assertEqual(main_head, self._git("rev-parse", "refs/heads/other"))

    def test_validation_environment_does_not_inherit_daemon_secrets(self) -> None:
        maintenance_name = "ZIRCON_COORDINATOR_" + "MAINTENANCE_TOKEN"
        cloud_name = "AWS_" + "SECRET_ACCESS_KEY"
        command = (
            sys.executable,
            "-c",
            (
                "import os; "
                "assert ('ZIRCON_COORDINATOR_' + 'MAINTENANCE_TOKEN') "
                "not in os.environ; "
                "assert ('AWS_' + 'SECRET_ACCESS_KEY') not in os.environ; "
                "assert os.environ['ZR_ISOLATED_PATCH_DERIVED_BLOB']"
            ),
        )

        with mock.patch.dict(
            os.environ,
            dict.fromkeys((maintenance_name, cloud_name), "test-only-value"),
        ):
            result = self.service.finalize(
                session_id="render-owner",
                target=_TARGET,
                patch=_PATCH,
                expected_head=self.base_head,
                expected_blob=self.base_blob,
                message="fix(coordinator): sanitize validation environment",
                validation_commands=(command,),
            )

        self.assertEqual(result.commit_sha, self._git("rev-parse", "refs/heads/main"))

    def test_validation_failure_records_failed_request_without_publishing(self) -> None:
        self.target.write_text(_MIXED_SOURCE, encoding="utf-8")
        head_before = self._git("rev-parse", "HEAD")
        worktree_before = self.target.read_bytes()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                session_id="render-owner",
                target=_TARGET,
                patch=_PATCH,
                expected_head=self.base_head,
                expected_blob=self.base_blob,
                message="fix(coordinator): reject failed validation",
                validation_commands=((sys.executable, "-c", "raise SystemExit(7)"),),
            )

        self.assertEqual("isolated_patch_validation_failed", rejected.exception.code)
        self.assertEqual(head_before, self._git("rev-parse", "HEAD"))
        self.assertEqual(worktree_before, self.target.read_bytes())
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT status, error_text, index_snapshot
                FROM finalize_requests ORDER BY created_at DESC LIMIT 1
                """
            ).fetchone()
        self.assertEqual("failed", row["status"])
        self.assertIn("exit code 7", row["error_text"])
        self.assertIsNone(row["index_snapshot"])

    def test_existing_finalizer_recovers_interruption_after_head_publication(self) -> None:
        self._stage_foreign_paths(3)
        self.target.write_text(_MIXED_SOURCE, encoding="utf-8")
        worktree_before = self.target.read_bytes()
        staged_before = self._git_bytes(
            "diff", "--cached", "--binary", "--no-renames", "HEAD", "--"
        )
        original_replace_index = self.service._replace_index

        def fail_shared_index_alignment(source: Path, target: Path) -> None:
            raise CoordinatorError(
                "injected_index_alignment_failure",
                "Injected failure after HEAD publication",
            )

        self.service._replace_index = fail_shared_index_alignment  # type: ignore[method-assign]
        with self.assertRaises(CoordinatorError) as interrupted:
            self._finalize()
        self.assertEqual("injected_index_alignment_failure", interrupted.exception.code)
        self.service._replace_index = original_replace_index  # type: ignore[method-assign]

        published_head = self._git("rev-parse", "HEAD")
        self.assertNotEqual(self.base_head, published_head)
        with self.database.connect() as connection:
            pending = connection.execute(
                """
                SELECT request_id, status, ref_updated_sha, index_snapshot
                FROM finalize_requests ORDER BY created_at DESC LIMIT 1
                """
            ).fetchone()
            mutex = connection.execute(
                "SELECT owner_id FROM git_mutex WHERE lock_name='index'"
            ).fetchone()
        self.assertEqual("finalizing", pending["status"])
        self.assertEqual(published_head, pending["ref_updated_sha"])
        self.assertIsNotNone(pending["index_snapshot"])
        self.assertIsNotNone(mutex)

        index_path = Path(self._git("rev-parse", "--git-path", "index"))
        if not index_path.is_absolute():
            index_path = self.repo / index_path
        recovery_lock = index_path.with_name(index_path.name + ".lock")
        recovery_lock.write_bytes(b"")
        stale_timestamp = time.time() - 60.0
        os.utime(recovery_lock, (stale_timestamp, stale_timestamp))
        lock_path = self.database.path.parent / "coordinator.lock"
        lock_path.write_text(json.dumps({"pid": os.getpid()}), encoding="utf-8")
        finalizer = GitFinalizeService(
            self.database,
            self.repo,
            self.baselines,
            SessionService(self.database, self.repo),
        )

        recovered = finalizer.recover_stale_mutex()

        self.assertEqual(1, recovered)
        self.assertEqual(worktree_before, self.target.read_bytes())
        self.assertEqual(
            staged_before,
            self._git_bytes(
                "diff", "--cached", "--binary", "--no-renames", "HEAD", "--"
            ),
        )
        with self.database.connect() as connection:
            request = connection.execute(
                """
                SELECT status, commit_sha, index_snapshot
                FROM finalize_requests WHERE request_id=?
                """,
                (pending["request_id"],),
            ).fetchone()
            mutex_count = connection.execute(
                "SELECT COUNT(*) FROM git_mutex"
            ).fetchone()[0]
        self.assertEqual("committed", request["status"])
        self.assertEqual(published_head, request["commit_sha"])
        self.assertIsNone(request["index_snapshot"])
        self.assertEqual(0, mutex_count)
        with self.database.connect() as connection:
            finalized = connection.execute(
                """
                SELECT payload_json FROM events
                WHERE event_type='maintenance.isolated_patch_finalized'
                ORDER BY event_id DESC LIMIT 1
                """
            ).fetchone()
        recovered_payload = json.loads(finalized["payload_json"])
        self.assertTrue(recovered_payload["recovered"])
        self.assertEqual(published_head, recovered_payload["commitSha"])
        self.assertEqual("passed", recovered_payload["validationStatus"])
        self.assertFalse(recovery_lock.exists())

    def test_live_target_lease_is_required(self) -> None:
        self.leases.release("render-owner", [_TARGET])

        with self.assertRaises(CoordinatorError) as rejected:
            self._finalize()

        self.assertEqual("isolated_patch_lease_missing", rejected.exception.code)
        with self.database.connect() as connection:
            count = connection.execute(
                "SELECT COUNT(*) FROM finalize_requests"
            ).fetchone()[0]
            self.assertEqual(0, count)

    def test_cli_names_the_operation_as_maintenance_and_requires_patch_identity(self) -> None:
        arguments = _parser().parse_args(
            [
                "maintenance",
                "finalize-patch",
                "--session-id",
                "render-owner",
                "--target",
                _TARGET,
                "--expected-head",
                self.base_head,
                "--expected-blob",
                self.base_blob,
                "--message",
                "fix(coordinator): isolate target patch",
                "--patch-stdin",
                "--validation-command",
                "python -m unittest focused.case",
            ]
        )

        self.assertEqual("maintenance", arguments.command)
        self.assertEqual("finalize-patch", arguments.maintenance_command)
        self.assertTrue(arguments.patch_stdin)
        self.assertFalse(hasattr(arguments, "compile_ticket_id"))

    def _finalize(self):
        return self.service.finalize(
            session_id="render-owner",
            target=_TARGET,
            patch=_PATCH,
            expected_head=self.base_head,
            expected_blob=self.base_blob,
            message="fix(coordinator): isolate target patch",
            validation_commands=((sys.executable, "-c", "pass"),),
        )

    def _stage_foreign_paths(self, count: int) -> None:
        root = self.repo / "foreign"
        root.mkdir()
        paths = []
        for index in range(count):
            path = root / f"path_{index:03}.txt"
            path.write_text("base\n", encoding="utf-8")
            paths.append(path.relative_to(self.repo).as_posix())
        subprocess.run(["git", "add", "foreign"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "test: add foreign paths"],
            cwd=self.repo,
            check=True,
        )
        self.baselines.refresh_for_head_change()
        for path in paths:
            (self.repo / path).write_text("staged\n", encoding="utf-8")
        subprocess.run(["git", "add", "foreign"], cwd=self.repo, check=True)

    def _staged_paths(self) -> tuple[str, ...]:
        output = self._git_bytes(
            "diff", "--cached", "--name-only", "-z", "--no-renames", "HEAD", "--"
        )
        return tuple(item.decode("utf-8") for item in output.split(b"\0") if item)

    def _git(self, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
            encoding="utf-8",
        ).stdout.strip()

    def _git_bytes(self, *arguments: str) -> bytes:
        return subprocess.run(
            ["git", *arguments], cwd=self.repo, check=True, capture_output=True
        ).stdout


if __name__ == "__main__":
    unittest.main()
