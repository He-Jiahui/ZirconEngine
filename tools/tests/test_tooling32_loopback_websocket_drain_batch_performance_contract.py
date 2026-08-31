from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_plugins"
    / "net"
    / "runtime"
    / "src"
    / "service_types"
    / "websocket"
    / "frames.rs"
)


class LoopbackWebSocketDrainBatchPerformanceContractTests(unittest.TestCase):
    def test_loopback_drain_preallocates_inside_the_bounded_batch(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        function = source.split("fn poll_websocket_frames_impl", 1)[1].split(
            "fn websocket_frame_bytes", 1
        )[0]

        self.assertIn(
            "let drain_count = max_frames.min(entry.inbound.len());", function
        )
        self.assertIn("Vec::with_capacity(drain_count)", function)
        self.assertNotIn("let mut frames = Vec::new()", function)


if __name__ == "__main__":
    unittest.main()
