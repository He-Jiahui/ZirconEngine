from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.cargo_jobs import CargoCompatibility, CargoJobService, TargetPathPolicy
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class CpuBurstAdmissionTests(unittest.TestCase):
    def test_targeted_library_test_is_automatically_eligible_without_a_target_dir(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            target_root = root / "targets"
            target_root.mkdir()
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(session_id="owner")
            service = CargoJobService(
                database,
                TargetPathPolicy([target_root]),
                repo_root=repo,
                free_space=lambda _path: 200 * 1024**3,
            )

            reservation = service.reserve_cpu(
                "owner",
                compatibility=CargoCompatibility(
                    platform="windows",
                    toolchain="stable-x86_64-pc-windows-msvc",
                    target_architecture="x86_64-pc-windows-msvc",
                    workspace="Cargo.toml",
                    build_config="profile=test",
                ),
                command=("cargo", "test", "-p", "zircon_runtime", "--lib", "project_asset_manager"),
            )

        self.assertTrue(reservation["burstEligible"])


if __name__ == "__main__":
    unittest.main()
