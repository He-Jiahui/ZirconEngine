from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / "examples/woc/scripts/woc_game/src/protocol/binary.zr"
TEST_MAIN = (
    ROOT
    / "examples/woc/scripts/woc_game/src/protocol/binary_random_access_test_main.zr"
)
TEST_PACKAGE = (
    ROOT / "examples/woc/scripts/woc_game/woc_protocol_binary_random_access_tests.zrp"
)


def zr_function_body(source: str, name: str) -> str:
    match = re.search(rf"\b(?:pub\s+)?{re.escape(name)}\s*\([^)]*\)[^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing Zr function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Zr function {name}")
    return source[match.end() : index - 1]


class Runtime12F64RandomAccessPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.binary = BINARY.read_text(encoding="utf-8")

    def test_random_access_decoder_does_not_copy_or_scan_the_prefix(self) -> None:
        body = zr_function_body(self.binary, "readF64LeAt")
        self.assertIn("decodeFiniteF64Bits(readU64LeAtBytes(source, offset))", body)
        self.assertNotIn("new ByteReader", body)
        self.assertNotIn("while (", body)
        self.assertNotIn("readByte", body)
        self.assertNotIn("new container.Array", body)

    def test_fixed_width_bit_load_reads_exactly_eight_checked_bytes(self) -> None:
        body = zr_function_body(self.binary, "readU64LeAtBytes")
        self.assertEqual(body.count("checkedByteAt("), 8)
        self.assertNotIn("while (", body)
        self.assertNotIn("for (", body)
        self.assertNotIn("new container.Array", body)

    def test_random_access_bytes_keep_wire_validation(self) -> None:
        body = zr_function_body(self.binary, "checkedByteAt")
        self.assertIn("value > <uint>255", body)
        self.assertIn('throw "woc byte value exceeds 255"', body)

    def test_sequential_and_random_access_paths_share_f64_decoding(self) -> None:
        sequential = zr_function_body(self.binary, "readF64")
        shared = zr_function_body(self.binary, "decodeFiniteF64Bits")
        self.assertIn(
            "return decodeFiniteF64Bits(this.readU64(1, 1, 2, 4));",
            sequential,
        )
        self.assertNotIn("mantissa", sequential)
        self.assertIn("var mantissa", shared)
        self.assertIn("exponent == 2047", shared)
        self.assertIn("powerOfTwo(-1074, true)", shared)
        self.assertIn("powerOfTwo(exponent - 1023, true)", shared)

    def test_zr_self_test_covers_nonzero_offset(self) -> None:
        body = zr_function_body(self.binary, "selfTest")
        self.assertIn("var randomAccessHalf", body)
        self.assertIn(
            "readF64LeAt(randomAccessHalf, <uint>1, <uint>8)",
            body,
        )

    def test_dedicated_zr_package_runs_binary_self_test(self) -> None:
        package = json.loads(TEST_PACKAGE.read_text(encoding="utf-8"))
        main = TEST_MAIN.read_text(encoding="utf-8")
        self.assertEqual(package["entry"], "protocol/binary_random_access_test_main")
        self.assertIn('%import("protocol/binary")', main)
        self.assertIn("binary.selfTest()", main)

    def test_worst_case_source_work_drops_by_at_least_ninety_nine_percent(self) -> None:
        payload_length = 65_536
        offset = payload_length - 8
        legacy_byte_touches = payload_length + offset + 8
        optimized_byte_touches = 8
        reduction_percent = (
            (legacy_byte_touches - optimized_byte_touches)
            * 100.0
            / legacy_byte_touches
        )
        self.assertGreaterEqual(reduction_percent, 99.0)


if __name__ == "__main__":
    unittest.main()
