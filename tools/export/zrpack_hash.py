"""ZRPack content hashing shared by export report evidence owners."""

from __future__ import annotations

from collections.abc import Iterable


ZRPACK_HASH_SEEDS = (
    0xCBF2_9CE4_8422_2325,
    0x9AE1_6A3B_2F90_404F,
    0x6EED_0E9D_A4D9_4A4F,
    0xACE5_929A_D4D9_8F13,
)
FNV1A64_PRIME = 0x100_0000_01B3
U64_MASK = (1 << 64) - 1


def zrpack_content_hash(bytes_value: Iterable[int]) -> list[int]:
    hash_0, hash_1, hash_2, hash_3 = ZRPACK_HASH_SEEDS
    for byte in bytes_value:
        hash_0 = ((hash_0 ^ byte) * FNV1A64_PRIME) & U64_MASK
        hash_1 = ((hash_1 ^ byte) * FNV1A64_PRIME) & U64_MASK
        hash_2 = ((hash_2 ^ byte) * FNV1A64_PRIME) & U64_MASK
        hash_3 = ((hash_3 ^ byte) * FNV1A64_PRIME) & U64_MASK

    hash_bytes = bytearray()
    for value in (hash_0, hash_1, hash_2, hash_3):
        hash_bytes.extend(value.to_bytes(8, byteorder="little"))
    return list(hash_bytes)
