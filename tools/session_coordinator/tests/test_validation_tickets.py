from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.validation_ticket_worker import ValidationTicketWorker
from tools.session_coordinator.validation_tickets import ValidationTicketService


class _FakeWorkspaceCopy:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.records: dict[str, SimpleNamespace] = {}
        self.materializations: list[tuple[str, tuple[str, ...], tuple[str, ...]]] = []
        self.starts: list[tuple[str, str, tuple[str, ...], str]] = []
        self.run_results: dict[str, dict[str, object]] = {}

    def materialize_cargo_async(
        self,
        session_id: str,
        *,
        command: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        discover_external_sources: bool,
    ) -> SimpleNamespace:
        self.assert_external_discovery(discover_external_sources)
        job_id = f"copy-{len(self.records) + 1}"
        source_root = self.root / job_id / "source"
        record = SimpleNamespace(
            job_id=job_id,
            source_root=source_root,
            status="materializing",
            error_code=None,
            error_stage=None,
            error_path=None,
        )
        self.records[job_id] = record
        self.materializations.append((session_id, command, overlay_paths))
        return record

    @staticmethod
    def assert_external_discovery(discover_external_sources: bool) -> None:
        if not discover_external_sources:
            raise AssertionError("validation worker must pin discovered sibling sources")

    def status(self, session_id: str, job_id: str) -> SimpleNamespace:
        del session_id
        return self.records[job_id]

    def start(
        self,
        session_id: str,
        job_id: str,
        *,
        command: tuple[str, ...],
        run_id: str,
    ) -> dict[str, object]:
        self.starts.append((session_id, job_id, command, run_id))
        self.records[job_id].status = "running"
        return {"jobId": job_id, "runId": run_id, "status": "running"}

    def run_result(self, run_id: str) -> dict[str, object] | None:
        return self.run_results.get(run_id)


class ValidationTicketTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.database = Database(Path(self.temporary.name) / "coordinator.sqlite3")
        self.repo = Path(self.temporary.name) / "repo"
        self.repo.mkdir()
        migrate(self.database)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, plan_path, status, created_at, updated_at, last_heartbeat_at
                ) VALUES (
                    'primary', 'docs/plans/tooling/01-tooling.md', 'active',
                    '2026-07-31T00:00:00+00:00', '2026-07-31T00:00:00+00:00',
                    '2026-07-31T00:00:00+00:00'
                )
                """
            )
        self.service = ValidationTicketService(self.database)
        self.workspace_copy = _FakeWorkspaceCopy(Path(self.temporary.name) / "copies")
        self.worker = ValidationTicketWorker(
            self.database,
            self.repo,
            self.service,
            self.workspace_copy,
            run_result_lookup=self.workspace_copy.run_result,
        )

    def test_submit_returns_a_durable_receipt_without_blocking_the_owner_session(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )
        replay = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )
        merged = self.service.submit(
            session_id="primary",
            request_id="request-b",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )

        self.assertEqual("queued", first.ticket.status)
        self.assertFalse(first.reused)
        self.assertEqual(first, replay)
        self.assertEqual(first.ticket.ticket_id, merged.ticket.ticket_id)
        self.assertTrue(merged.reused)
        with self.database.connect() as connection:
            owner_status = connection.execute(
                "SELECT status FROM sessions WHERE session_id='primary'"
            ).fetchone()["status"]
            request_count = connection.execute(
                "SELECT COUNT(*) FROM validation_ticket_requests"
            ).fetchone()[0]
        self.assertEqual("active", owner_status)
        self.assertEqual(2, request_count)

    def test_different_source_manifest_is_not_merged(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )
        second = self.service.submit(
            session_id="primary",
            request_id="request-b",
            source_manifest={"tools/session_coordinator/governance.py": "b" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )

        self.assertNotEqual(first.ticket.ticket_id, second.ticket.ticket_id)
        self.assertFalse(second.reused)

    def test_terminal_result_can_be_recorded_without_polling_for_running(self) -> None:
        receipt = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )

        completed = self.service.record_result(
            receipt.ticket.ticket_id,
            "failed",
            evidence={"exitCode": 1},
        )

        self.assertEqual("failed", completed.status)
        with self.database.connect() as connection:
            owner_status = connection.execute(
                "SELECT status FROM sessions WHERE session_id='primary'"
            ).fetchone()["status"]
        self.assertEqual("active", owner_status)

    def test_worker_terminalizes_a_stale_snapshot_without_materializing_it(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        receipt = self.service.submit(
            session_id="primary",
            request_id="stale-request",
            source_manifest={"tools/owned.py": "a" * 64},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )

        result = self.worker.tick()

        self.assertEqual(1, result["snapshot_stale"])
        self.assertEqual(
            "snapshot_stale", self.service.get(receipt.ticket.ticket_id).status
        )
        self.assertEqual([], self.workspace_copy.materializations)

    def test_worker_consumes_an_exact_snapshot_through_copy_and_run_links(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="exact-request",
            source_manifest={"tools/owned.py": digest},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        ticket_id = receipt.ticket.ticket_id

        claimed = self.worker.tick()
        self.assertEqual(1, claimed["materializing"])
        copy = next(iter(self.workspace_copy.records.values()))
        copied_source = copy.source_root / "tools" / "owned.py"
        copied_source.parent.mkdir(parents=True)
        copied_source.write_bytes(source.read_bytes())
        copy.status = "materialized"

        started = self.worker.tick()
        self.assertEqual(1, started["running"])
        self.assertEqual("running", self.service.get(ticket_id).status)
        self.workspace_copy.run_results[ticket_id] = {
            "runId": ticket_id,
            "jobId": copy.job_id,
            "exitCode": 0,
            "stdout": "ok",
            "stderr": "",
        }

        completed = self.worker.tick()

        self.assertEqual(1, completed["passed"])
        self.assertEqual("passed", self.service.get(ticket_id).status)
        self.assertEqual(ticket_id, self.workspace_copy.starts[0][3])

    def test_worker_terminalizes_an_interrupted_claim_without_a_copy_link(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        receipt = self.service.submit(
            session_id="primary",
            request_id="interrupted-request",
            source_manifest={
                "tools/owned.py": hashlib.sha256(source.read_bytes()).hexdigest()
            },
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        self.service.claim_next()

        result = self.worker.tick()

        self.assertEqual(1, result["failed"])
        self.assertEqual("failed", self.service.get(receipt.ticket.ticket_id).status)


if __name__ == "__main__":
    unittest.main()
