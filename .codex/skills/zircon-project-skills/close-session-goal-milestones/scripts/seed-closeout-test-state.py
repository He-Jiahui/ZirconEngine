from __future__ import annotations

import argparse
import os
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(REPO_ROOT))

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.models import SessionStatus, utc_text
from tools.session_coordinator.sessions import SessionService


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True)
    parser.add_argument(
        "--action", choices=("init", "lease", "release", "attribute", "status", "failure"), required=True
    )
    parser.add_argument("--session-id", default="session-m2")
    parser.add_argument("--plan-path", default="docs/plans/feature/02-feature.md")
    parser.add_argument("--path", action="append", default=[])
    parser.add_argument("--status", choices=tuple(item.value for item in SessionStatus))
    arguments = parser.parse_args()

    repo = Path(arguments.repo_root).resolve()
    temporary_root = Path(tempfile.gettempdir()).resolve()
    marker = repo / ".codex/state/session-coordinator/closeout-test-fixture"
    if os.environ.get("ZIRCON_CLOSEOUT_TEST_FIXTURE") != "1" or not repo.is_relative_to(temporary_root):
        raise RuntimeError("Test state seeding is restricted to marked temporary repositories")
    if arguments.action == "init":
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_text("test-only\n", encoding="utf-8")
    elif not marker.is_file():
        raise RuntimeError("Temporary repository is missing the closeout test marker")
    config = CoordinatorConfig.for_repo(repo)
    database = Database(config.database_path)
    migrate(database)
    baselines = BaselineService(database, repo)
    sessions = SessionService(database, repo)
    leases = LeaseService(
        database,
        PathPolicy(repo),
        ttl_seconds=config.lease_ttl_seconds,
        grace_seconds=config.lease_grace_seconds,
    )
    if arguments.action == "init":
        baselines.initialize()
        sessions.register(session_id=arguments.session_id, plan_path=arguments.plan_path)
        sessions.set_status(arguments.session_id, SessionStatus.ACTIVE)
    elif arguments.action == "lease":
        acquisition = leases.acquire(arguments.session_id, arguments.path)
        if not acquisition.acquired:
            raise RuntimeError(f"Cannot seed leases: {acquisition.conflicts}")
    elif arguments.action == "release":
        leases.release(arguments.session_id, arguments.path)
    elif arguments.action == "attribute":
        baselines.attribute(arguments.session_id, arguments.path)
    elif arguments.action == "status":
        if not arguments.status:
            parser.error("--status is required for action=status")
        sessions.set_status(arguments.session_id, SessionStatus(arguments.status))
    else:
        now = utc_text()
        with database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO failure_nodes(
                    lifecycle_key, artifact_path, kind, status, created_at,
                    resolved_at, summary_slug, origin_plan, fixing_plan,
                    origin_child_dir, fixing_child_dir, priority, imported_at
                ) VALUES (?, ?, 'failure', 'open', ?, NULL, ?, ?, ?, ?, ?, 10, ?)
                """,
                (
                    "test-open-failure",
                    "docs/plans/origin/01/failure-2026-07-11-test.md",
                    now,
                    "test-open-failure",
                    "docs/plans/origin/01-origin.md",
                    arguments.plan_path,
                    "docs/plans/origin/01",
                    str(Path(arguments.plan_path).with_suffix("")),
                    now,
                ),
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
