from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CONTRACTS = ROOT / "examples/woc/native/crates/woc_protocol/src/contracts.rs"
LIB = ROOT / "examples/woc/native/crates/woc_protocol/src/lib.rs"
PAYLOAD = ROOT / "examples/woc/native/crates/woc_protocol/src/payload.rs"
RUNTIME = ROOT / "examples/woc/native/plugins/woc_runtime/src/transaction.rs"
RUNTIME_TESTS = ROOT / "examples/woc/native/plugins/woc_runtime/tests/transaction.rs"


def braced_body(source: str, start: int) -> str:
    opening = source.find("{", start)
    if opening < 0:
        raise AssertionError("missing opening brace")
    depth = 1
    index = opening + 1
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError("unterminated braced body")
    return source[opening + 1 : index - 1]


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\([^)]*\)[^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing function {name}")
    return braced_body(source, match.start())


def impl_body(source: str, type_name: str) -> str:
    match = re.search(rf"\bimpl(?:\s*<[^>]+>)?\s+{re.escape(type_name)}(?:<[^>]+>)?\s*{{", source)
    if match is None:
        raise AssertionError(f"missing impl {type_name}")
    return braced_body(source, match.start())


class WocRuntimeBorrowedTickStatePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contracts = CONTRACTS.read_text(encoding="utf-8")
        cls.lib = LIB.read_text(encoding="utf-8")
        cls.payload = PAYLOAD.read_text(encoding="utf-8")
        cls.runtime = RUNTIME.read_text(encoding="utf-8")
        cls.runtime_tests = RUNTIME_TESTS.read_text(encoding="utf-8")

    def test_protocol_exposes_a_copyable_borrowed_tick_input(self) -> None:
        self.assertRegex(
            self.contracts,
            r"pub struct FixedTickInputRef<'a>\s*{[^}]*commands:\s*&'a \[Command\]"
            r"[^}]*committed_state:\s*&'a \[u8\]"
            r"[^}]*movement_frames:\s*&'a \[MovementFrame\]"
            r"[^}]*offline_bootstrap:\s*Option<&'a OfflineSessionBootstrap>",
        )
        self.assertIn("FixedTickInputRef", self.lib)

    def test_owned_encoder_delegates_to_the_borrowed_encoder(self) -> None:
        owned_impl = impl_body(self.payload, "FixedTickInput")
        owned_encode = function_body(owned_impl, "encode_payload")
        self.assertIn("FixedTickInputRef::from(self).encode_payload()", owned_encode)
        self.assertNotIn("self.movement_frames.clone()", owned_encode)

    def test_borrowed_encoder_preserves_the_existing_wire_contract(self) -> None:
        borrowed_impl = impl_body(self.payload, "FixedTickInputRef")
        borrowed_encode = function_body(borrowed_impl, "encode_payload")
        self.assertIn('"FixedTickInput.committed_state"', borrowed_encode)
        self.assertIn("self.committed_state", borrowed_encode)
        self.assertIn("MovementFrameBatch::new(self.movement_frames.to_vec())", borrowed_encode)
        self.assertIn("OfflineSessionBootstrap::encode_payload", borrowed_encode)

    def test_runtime_encodes_the_committed_state_without_a_clone(self) -> None:
        prepare_tick = function_body(self.runtime, "prepare_tick")
        self.assertIn("FixedTickInputRef {", prepare_tick)
        self.assertIn("commands: &commands", prepare_tick)
        self.assertIn("committed_state: &self.committed.state", prepare_tick)
        self.assertIn("movement_frames: &movement_frames", prepare_tick)
        self.assertIn("offline_bootstrap: self.bootstrap_for_next_tick()", prepare_tick)
        self.assertNotIn("self.committed.state.clone()", prepare_tick)
        self.assertNotIn("self.bootstrap_for_next_tick().cloned()", prepare_tick)

    def test_release_gate_emits_raw_paired_samples(self) -> None:
        self.assertIn("WOC_APP03_BORROWED_TICK_STATE_PERF", self.runtime_tests)
        self.assertIn("sample_pairs=21", self.runtime_tests)
        self.assertIn("percentile_method=nearest_rank", self.runtime_tests)
        self.assertIn("threshold_percent=35", self.runtime_tests)


if __name__ == "__main__":
    unittest.main()
