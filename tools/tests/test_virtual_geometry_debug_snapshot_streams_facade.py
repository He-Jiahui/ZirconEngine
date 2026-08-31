"""Guard the public debug-readback stream contract against glob re-exports."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STREAMS_MODULE = REPO_ROOT / (
    "zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams.rs"
)
TYPES_MODULE = REPO_ROOT / (
    "zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams/types.rs"
)


def _owned_public_symbols(source: str) -> set[str]:
    return set(
        re.findall(
            r"^pub (?:struct|enum|type|const) (RenderVirtualGeometry\\w+)",
            source,
            flags=re.MULTILINE,
        )
    )


def _reexported_type_symbols(source: str) -> set[str]:
    match = re.search(r"pub use types::\\{(?P<symbols>.*?)\\};", source, flags=re.DOTALL)
    if match is None:
        return set()
    return set(re.findall(r"\\b(RenderVirtualGeometry\\w+)", match.group("symbols")))


class VirtualGeometryDebugSnapshotStreamsFacadeTests(unittest.TestCase):
    def test_types_owner_is_explicitly_reexported(self) -> None:
        streams_source = STREAMS_MODULE.read_text(encoding="utf-8")
        type_owner_source = TYPES_MODULE.read_text(encoding="utf-8")

        self.assertNotIn("pub use types::*;", streams_source)
        self.assertEqual(
            _owned_public_symbols(type_owner_source),
            _reexported_type_symbols(streams_source),
        )


if __name__ == "__main__":
    unittest.main()
