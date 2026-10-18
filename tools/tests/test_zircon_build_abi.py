import struct
import tempfile
import unittest
from pathlib import Path

from tools.build.zircon_build_abi import (
    AbiValidationError,
    inspect_pe_exports,
    validate_product_pe_image,
    validate_runtime_abi,
    validate_runtime_ffi_source,
)


class RuntimeAbiTests(unittest.TestCase):
    def test_pe_export_reader_finds_required_v8_symbol(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            dll = Path(temporary) / "zircon_runtime.dll"
            dll.write_bytes(_synthetic_pe_with_export("zircon_runtime_get_api_v8"))
            self.assertEqual(
                inspect_pe_exports(dll),
                ("zircon_runtime_get_api_v8",),
            )

    def test_abi_validation_rejects_missing_export(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            dll = Path(temporary) / "runtime.dll"
            dll.write_bytes(_synthetic_pe_with_export("other_export"))
            with self.assertRaisesRegex(AbiValidationError, "missing required"):
                validate_runtime_abi(dll)

    def test_source_guard_contains_no_unwind_boundary(self) -> None:
        report = validate_runtime_ffi_source(Path(__file__).resolve().parents[2])
        self.assertTrue(report["no_unwind_guard"])

    def test_product_pe_rejects_wrong_machine_and_dll_host_shape(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            image = root / "host.exe"
            image.write_bytes(_synthetic_product_pe(dll=True))
            with self.assertRaisesRegex(AbiValidationError, "marked as a DLL"):
                validate_product_pe_image(
                    image,
                    product_root=root,
                    expected_machine="x86_64",
                    expected_kind="exe",
                )
            image.write_bytes(_synthetic_product_pe(dll=False, machine=0xAA64))
            with self.assertRaisesRegex(AbiValidationError, "does not match expected"):
                validate_product_pe_image(
                    image,
                    product_root=root,
                    expected_machine="x86_64",
                    expected_kind="exe",
                )

    def test_product_pe_import_allowlist_and_delay_policy_are_explicit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            dll = root / "zircon_runtime.dll"
            dll.write_bytes(
                _synthetic_product_pe(
                    imports=("third_party.dll",),
                    delay_imports=("delay.dll",),
                )
            )
            with self.assertRaisesRegex(AbiValidationError, "delay imports"):
                validate_runtime_abi(
                    dll,
                    product_root=root,
                    required_exports=("zircon_runtime_get_api_v8",),
                )
            with self.assertRaisesRegex(AbiValidationError, "not in the product closure"):
                validate_runtime_abi(
                    dll,
                    product_root=root,
                    required_exports=("zircon_runtime_get_api_v8",),
                    reject_delay_imports=False,
                )
            report = validate_runtime_abi(
                dll,
                product_root=root,
                required_exports=("zircon_runtime_get_api_v8",),
                allowed_imports=("third_party.dll", "delay.dll"),
                reject_delay_imports=False,
            )
            self.assertTrue(report["passed"])
            self.assertEqual(report["delay_imports"], ["delay.dll"])

    def test_product_pe_rejects_forwarded_exports_by_default(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            dll = Path(temporary) / "zircon_runtime.dll"
            dll.write_bytes(_synthetic_product_pe(forward=True))
            with self.assertRaisesRegex(AbiValidationError, "forwarded export"):
                validate_runtime_abi(
                    dll,
                    required_exports=("zircon_runtime_get_api_v8",),
                )


def _synthetic_pe_with_export(name: str) -> bytes:
    data = bytearray(0x500)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 0x3C, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    coff = 0x84
    struct.pack_into("<H", data, coff, 0x8664)
    struct.pack_into("<H", data, coff + 2, 1)
    struct.pack_into("<H", data, coff + 16, 0xF0)
    optional = coff + 20
    struct.pack_into("<H", data, optional, 0x20B)
    # PE32+ data-directory[0] (export table).
    struct.pack_into("<II", data, optional + 112, 0x200, 0x100)
    section = optional + 0xF0
    data[section : section + 8] = b".edata\0\0"
    struct.pack_into("<I", data, section + 8, 0x300)
    struct.pack_into("<I", data, section + 12, 0x200)
    struct.pack_into("<I", data, section + 16, 0x300)
    struct.pack_into("<I", data, section + 20, 0x300)
    export = 0x300
    struct.pack_into("<I", data, export + 24, 1)  # NumberOfNames
    struct.pack_into("<I", data, export + 32, 0x250)  # AddressOfNames
    struct.pack_into("<I", data, 0x350, 0x260)
    data[0x360 : 0x360 + len(name) + 1] = name.encode("ascii") + b"\0"
    return bytes(data)


def _synthetic_product_pe(
    *,
    dll: bool = True,
    machine: int = 0x8664,
    imports: tuple[str, ...] = (),
    delay_imports: tuple[str, ...] = (),
    forward: bool = False,
) -> bytes:
    """Build a bounded PE fixture with normal/delay imports and one export."""

    data = bytearray(0x1800)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 0x3C, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    coff = 0x84
    struct.pack_into("<H", data, coff, machine)
    struct.pack_into("<H", data, coff + 2, 1)
    struct.pack_into("<H", data, coff + 16, 0xF0)
    struct.pack_into("<H", data, coff + 18, 0x2000 if dll else 0)
    optional = coff + 20
    struct.pack_into("<H", data, optional, 0x20B)
    struct.pack_into("<I", data, optional + 108, 16)
    struct.pack_into("<II", data, optional + 112, 0x1000, 0x100)
    if imports:
        struct.pack_into("<II", data, optional + 120, 0x1100, (len(imports) + 1) * 20)
    if delay_imports:
        struct.pack_into(
            "<II",
            data,
            optional + 112 + 13 * 8,
            0x1180,
            (len(delay_imports) + 1) * 32,
        )
    section = optional + 0xF0
    data[section : section + 8] = b".all\0\0\0\0"
    struct.pack_into("<IIII", data, section + 8, 0x1600, 0x1000, 0x1600, 0x200)

    export = 0x200
    struct.pack_into("<IIIII", data, export + 20, 1, 1, 0x1040, 0x1044, 0x1048)
    struct.pack_into("<I", data, 0x240, 0x1080 if forward else 0x1300)
    struct.pack_into("<I", data, 0x244, 0x1200)
    struct.pack_into("<H", data, 0x248, 0)
    export_name = b"zircon_runtime_get_api_v8\0"
    data[0x400 : 0x400 + len(export_name)] = export_name
    if forward:
        forwarded = b"KERNEL32.Sleep\0"
        data[0x280 : 0x280 + len(forwarded)] = forwarded

    for index, name in enumerate(imports):
        descriptor = 0x300 + index * 20
        name_rva = 0x1250 + index * 0x20
        struct.pack_into("<5I", data, descriptor, 0, 0, 0, name_rva, 0)
        encoded = name.encode("ascii") + b"\0"
        offset = 0x450 + index * 0x20
        data[offset : offset + len(encoded)] = encoded
    for index, name in enumerate(delay_imports):
        descriptor = 0x380 + index * 32
        name_rva = 0x1260 + index * 0x20
        struct.pack_into("<8I", data, descriptor, 1, name_rva, 0, 0, 0, 0, 0, 0)
        encoded = name.encode("ascii") + b"\0"
        offset = 0x460 + index * 0x20
        data[offset : offset + len(encoded)] = encoded
    return bytes(data)


if __name__ == "__main__":
    unittest.main()
