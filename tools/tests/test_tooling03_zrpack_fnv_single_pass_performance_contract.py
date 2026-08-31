from __future__ import annotations

import unittest
from unittest import mock

from tools.zircon_export import pipeline_report_cook_assets_source_bytes
from tools.zircon_export import pipeline_report_pack_file_evidence


ZRPACK_HASH_SEEDS = (
    0xCBF2_9CE4_8422_2325,
    0x9AE1_6A3B_2F90_404F,
    0x6EED_0E9D_A4D9_4A4F,
    0xACE5_929A_D4D9_8F13,
)
FNV1A64_PRIME = 0x100_0000_01B3
U64_MASK = (1 << 64) - 1


class CountingBytes:
    def __init__(self, value: bytes) -> None:
        self.value = value
        self.iterations = 0

    def __iter__(self):
        self.iterations += 1
        return iter(self.value)


def legacy_zrpack_content_hash(value: bytes) -> list[int]:
    digest = bytearray()
    for seed in ZRPACK_HASH_SEEDS:
        state = seed
        for byte in value:
            state ^= byte
            state = (state * FNV1A64_PRIME) & U64_MASK
        digest.extend(state.to_bytes(8, byteorder="little"))
    return list(digest)


class Tooling03ZrpackFnvSinglePassPerformanceContractTests(unittest.TestCase):
    HASH_FUNCTIONS = (
        pipeline_report_cook_assets_source_bytes.zrpack_content_hash,
        pipeline_report_pack_file_evidence.zrpack_content_hash,
    )

    def test_hashes_preserve_the_four_seed_zrpack_digest(self) -> None:
        payloads = (
            b"",
            b"zircon-pack",
            bytes(range(256)) * 3,
        )
        for hash_function in self.HASH_FUNCTIONS:
            for payload in payloads:
                with self.subTest(function=hash_function.__module__, size=len(payload)):
                    self.assertEqual(
                        hash_function(payload),
                        legacy_zrpack_content_hash(payload),
                    )

    def test_each_hash_entrypoint_walks_the_payload_once(self) -> None:
        payload = bytes(range(256)) * 16
        expected = legacy_zrpack_content_hash(payload)
        for hash_function in self.HASH_FUNCTIONS:
            counted_payload = CountingBytes(payload)
            with self.subTest(function=hash_function.__module__):
                self.assertEqual(hash_function(counted_payload), expected)
                self.assertEqual(
                    counted_payload.iterations,
                    1,
                    "ZRPack hashing must update all four FNV states in one pass",
                )

    def test_pack_chunk_hashing_borrows_the_pack_payload(self) -> None:
        payload = bytes(range(256)) * 16
        manifest_offset = 24 + len(payload)
        header = (
            b"ZRPK"
            + bytes(4)
            + manifest_offset.to_bytes(8, byteorder="little")
            + bytes(8)
        )
        pack_bytes = header + payload + b"{}"
        expected_hash = legacy_zrpack_content_hash(payload)
        manifest = {
            "chunks": [
                {
                    "hash": expected_hash,
                    "offset": 24,
                    "size": len(payload),
                }
            ]
        }
        observed_payloads: list[object] = []

        def capture_payload(value: object) -> list[int]:
            observed_payloads.append(value)
            return expected_hash

        with mock.patch.object(
            pipeline_report_pack_file_evidence,
            "zrpack_content_hash",
            side_effect=capture_payload,
        ):
            diagnostics = (
                pipeline_report_pack_file_evidence
                .pack_report_chunk_payload_hash_diagnostics(
                    "pack report pack",
                    "synthetic.zrpack",
                    pack_bytes,
                    manifest,
                    "chunks",
                )
            )

        self.assertEqual(diagnostics, [])
        self.assertEqual(len(observed_payloads), 1)
        self.assertIsInstance(observed_payloads[0], memoryview)
        self.assertIs(observed_payloads[0].obj, pack_bytes)


if __name__ == "__main__":
    unittest.main()
