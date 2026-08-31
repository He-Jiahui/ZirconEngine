from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
COMMAND_PAYLOAD = ROOT / "examples/woc/native/crates/woc_protocol/src/command_payload.rs"
MARKET_PAYLOAD = ROOT / "examples/woc/native/crates/woc_protocol/src/market_payload.rs"
GENERATED = ROOT / "examples/woc/native/crates/woc_protocol/src/generated_command_payloads.rs"

CRATE_GROUP_IMPORT = re.compile(r"use\s+crate::\{(?P<body>.*?)\};", re.DOTALL)
COMMAND_ID = re.compile(r"\b[A-Z][A-Z0-9_]*_COMMAND_ID\b")


def missing_command_id_imports(source: str) -> set[str]:
    imported: set[str] = set()
    for match in CRATE_GROUP_IMPORT.finditer(source):
        imported.update(COMMAND_ID.findall(match.group("body")))
    implementation = CRATE_GROUP_IMPORT.sub("", source)
    referenced = set(COMMAND_ID.findall(implementation))
    return referenced - imported


class Runtime19CommandPayloadImportContract(unittest.TestCase):
    def test_shared_command_payload_imports_every_referenced_command_id(self) -> None:
        source = COMMAND_PAYLOAD.read_text(encoding="utf-8")
        self.assertEqual(missing_command_id_imports(source), set())

    def test_market_payload_imports_every_referenced_command_id(self) -> None:
        source = MARKET_PAYLOAD.read_text(encoding="utf-8")
        self.assertEqual(missing_command_id_imports(source), set())

    def test_missing_ids_are_generated_catalog_authority(self) -> None:
        source = GENERATED.read_text(encoding="utf-8")
        self.assertIn("pub const ENTER_DELVE_COMMAND_ID: u16 = 115;", source)
        self.assertIn("pub const MARKET_SEARCH_COMMAND_ID: u16 = 101;", source)


if __name__ == "__main__":
    unittest.main()
