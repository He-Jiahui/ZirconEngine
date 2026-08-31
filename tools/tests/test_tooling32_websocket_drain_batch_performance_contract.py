from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_plugins"
    / "net"
    / "features"
    / "websocket"
    / "runtime"
    / "src"
    / "backend"
    / "connection.rs"
)


class WebSocketDrainBatchPerformanceContractTests(unittest.TestCase):
    def test_drain_preallocates_the_bounded_batch(self) -> None:
        function = self._drain_function()

        self.assertIn("let drain_count = max_frames.min(inbound.len());", function)
        self.assertIn("Vec::with_capacity(drain_count)", function)
        self.assertNotIn("let mut frames = Vec::new()", function)

    def test_drain_releases_inbound_before_locking_connection_state(self) -> None:
        function = self._drain_function()
        release = function.index("drop(inbound);")
        state_lock = function.index(".state", release)

        self.assertLess(release, state_lock)
        self.assertNotIn(".state", function[:release])

    @staticmethod
    def _drain_function() -> str:
        source = SOURCE.read_text(encoding="utf-8")
        return source.split("fn drain_frames", 1)[1].split(
            "impl TungsteniteWebSocketConnection", 1
        )[0]


if __name__ == "__main__":
    unittest.main()
