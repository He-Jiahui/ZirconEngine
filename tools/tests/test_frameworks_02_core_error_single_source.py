import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class Frameworks02CoreErrorSingleSourceTests(unittest.TestCase):
    def test_runtime_core_exposes_only_core_error(self) -> None:
        error_source = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/error.rs"
        ).read_text(encoding="utf-8")
        core_root_source = (REPO_ROOT / "zircon_runtime/src/core/mod.rs").read_text(
            encoding="utf-8"
        )
        task_source = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/tasks/mod.rs"
        ).read_text(encoding="utf-8")
        asset_worker_source = (
            REPO_ROOT / "zircon_runtime/src/asset/pipeline/worker_pool.rs"
        ).read_text(encoding="utf-8")
        prelude_source = (REPO_ROOT / "zircon_runtime/src/prelude.rs").read_text(
            encoding="utf-8"
        )

        self.assertNotIn("pub enum ZirconError", error_source)
        self.assertIn("pub enum CoreError", error_source)
        self.assertIn("ChannelSend(String)", error_source)
        self.assertIn("ThreadSpawn(String)", error_source)
        self.assertIn(
            "pub use runtime::error::{CoreError, CoreResult};", core_root_source
        )
        self.assertNotIn("ZirconError", core_root_source)

        self.assertIn("use crate::core::{CoreError, CoreResult};", task_source)
        self.assertIn("-> CoreResult<JoinHandle<T>>", task_source)
        self.assertIn("CoreError::ThreadSpawn", task_source)

        self.assertIn("use crate::core::{CoreError, CoreResult};", asset_worker_source)
        self.assertIn(
            "pub fn request(&self, request: AssetRequest) -> CoreResult<AssetWorkerCompletionTicket>",
            asset_worker_source,
        )
        self.assertIn("Err(CoreError::ChannelSend", asset_worker_source)

        prelude_core_exports = prelude_source.split("pub use crate::core::{", 1)[1].split(
            "};", 1
        )[0]
        self.assertIn("CoreError", prelude_core_exports)
        self.assertIn("CoreResult", prelude_core_exports)
        self.assertNotIn("ZirconError", prelude_core_exports)

        stale_consumers = []
        for path in (REPO_ROOT / "zircon_runtime/src").rglob("*.rs"):
            if "ZirconError" in path.read_text(encoding="utf-8"):
                stale_consumers.append(path.relative_to(REPO_ROOT).as_posix())

        self.assertEqual([], stale_consumers)


if __name__ == "__main__":
    unittest.main()
