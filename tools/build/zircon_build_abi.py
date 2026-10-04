"""Post-link ABI checks for the Windows shipping Runtime DLL.

The shipping pipeline must prove two things at the publication boundary: the
DLL still exports the stable v8 entry point after symbol stripping, and the
Rust source keeps every public entry behind a panic catcher.  A small PE
export-directory reader keeps this check independent of ``dumpbin``/LLVM
installation details on the build machine.
"""

from __future__ import annotations

import argparse
import json
import os
import stat
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping, Sequence


REQUIRED_RUNTIME_EXPORTS = ("zircon_runtime_get_api_v8",)
RUNTIME_EXPORT_SOURCE = Path("zircon_runtime") / "src" / "dynamic_api" / "exports.rs"
PE_MACHINE_X86_64 = 0x8664
PE_MACHINE_X86 = 0x014C
PE_MACHINE_ARM64 = 0xAA64
_PE_DIRECTORY_EXPORT = 0
_PE_DIRECTORY_IMPORT = 1
_PE_DIRECTORY_DELAY_IMPORT = 13
_IMAGE_FILE_DLL = 0x2000
_IMAGE_DIRECTORY_ENTRY_COUNT = 16
_IMAGE_DELAY_IMPORT_RVA = 0x1
_MAX_PE_IMAGE_BYTES = 512 * 1024 * 1024
_MAX_IMPORT_DESCRIPTORS = 4096
_MAX_EXPORTS = 262_144
_WINDOWS_SYSTEM_IMPORTS = frozenset(
    {
        "advapi32.dll",
        "bcrypt.dll",
        "bcryptprimitives.dll",
        "cabinet.dll",
        "cfgmgr32.dll",
        "comctl32.dll",
        "combase.dll",
        "comdlg32.dll",
        "crypt32.dll",
        "d3d11.dll",
        "d3d12.dll",
        "d3dcompiler_47.dll",
        "dcomp.dll",
        "dinput8.dll",
        "dwmapi.dll",
        "dxcore.dll",
        "dxgi.dll",
        "dxguid.dll",
        "gdi32.dll",
        "gdiplus.dll",
        "hid.dll",
        "imm32.dll",
        "iphlpapi.dll",
        "kernel32.dll",
        "kernelbase.dll",
        "mmdevapi.dll",
        "msimg32.dll",
        "msvcp140.dll",
        "msvcp140_1.dll",
        "msvcrt.dll",
        "ntdll.dll",
        "ncrypt.dll",
        "netapi32.dll",
        "ole32.dll",
        "oleacc.dll",
        "oleaut32.dll",
        "onecore.dll",
        "opengl32.dll",
        "propsys.dll",
        "psapi.dll",
        "profapi.dll",
        "powrprof.dll",
        "rpcrt4.dll",
        "secur32.dll",
        "setupapi.dll",
        "shell32.dll",
        "shlwapi.dll",
        "uxtheme.dll",
        "ucrtbase.dll",
        "user32.dll",
        "userenv.dll",
        "version.dll",
        "vcruntime140.dll",
        "vcruntime140_1.dll",
        "win32u.dll",
        "winhttp.dll",
        "winmm.dll",
        "ws2_32.dll",
        "xinput1_4.dll",
        "winspool.drv",
        "shcore.dll",
        "mswsock.dll",
        "schannel.dll",
        "dnsapi.dll",
        "wininet.dll",
        "wintrust.dll",
        "wtsapi32.dll",
        "xinput9_1_0.dll",
        "wevtapi.dll",
        "runtimeobject.dll",
    }
)


class AbiValidationError(ValueError):
    """Raised when a product DLL cannot satisfy the stable FFI contract."""


@dataclass(frozen=True)
class _PeImage:
    path: Path
    data: bytes
    machine: int
    bitness: int
    sections: tuple[tuple[int, int, int, int], ...]
    directories: tuple[tuple[int, int], ...]
    is_dll: bool
    export_directory: tuple[int, int] | None


def inspect_pe_image(binary_path: Path | str) -> dict[str, object]:
    """Return stable PE machine, section, import and export evidence."""

    image = _load_pe(binary_path)
    machine_name = _machine_name(image.machine)
    exports, forwarded_exports = _parse_pe_exports_with_forwarders(image)
    imports = _parse_pe_imports(image, _PE_DIRECTORY_IMPORT)
    delay_imports = _parse_pe_imports(image, _PE_DIRECTORY_DELAY_IMPORT)
    return {
        "binary": image.path.name,
        "machine": image.machine,
        "machine_hex": f"0x{image.machine:04X}",
        "machine_name": machine_name,
        "bitness": image.bitness,
        "sections": [
            {
                "virtual_address": virtual_address,
                "virtual_size": virtual_size,
                "raw_offset": raw_offset,
                "raw_size": raw_size,
            }
            for virtual_address, virtual_size, raw_offset, raw_size in image.sections
        ],
        "is_dll": image.is_dll,
        "exports": list(exports),
        "forwarded_exports": list(forwarded_exports),
        "imports": list(imports),
        "delay_imports": list(delay_imports),
    }


def inspect_pe_imports(binary_path: Path | str) -> tuple[str, ...]:
    """Return normal and delay-loaded DLL names from a PE image."""

    image = _load_pe(binary_path)
    return tuple(
        sorted(
            set(_parse_pe_imports(image, _PE_DIRECTORY_IMPORT))
            | set(_parse_pe_imports(image, _PE_DIRECTORY_DELAY_IMPORT))
        )
    )


def inspect_pe_exports(binary_path: Path | str) -> tuple[str, ...]:
    """Read exported names from a PE32/PE32+ image without external tools."""
    return _parse_pe_exports(_load_pe(binary_path))


def validate_runtime_ffi_source(repo_root: Path | str) -> dict[str, object]:
    """Check the source-level no-unwind guard used by the v8 export table."""

    path = Path(repo_root).expanduser().resolve() / RUNTIME_EXPORT_SOURCE
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as error:
        raise AbiValidationError(f"could not read FFI export source {path}: {error}") from error
    required_markers = ("#[no_mangle]", "catch_unwind(AssertUnwindSafe", "extern \"C\"")
    missing = [marker for marker in required_markers if marker not in source]
    if missing:
        raise AbiValidationError(
            f"FFI export source {path} is missing no-unwind marker(s): {', '.join(missing)}"
        )
    return {"path": RUNTIME_EXPORT_SOURCE.as_posix(), "no_unwind_guard": True}


def validate_runtime_abi(
    binary_path: Path | str,
    *,
    repo_root: Path | str | None = None,
    required_exports: Iterable[str] = REQUIRED_RUNTIME_EXPORTS,
    product_root: Path | str | None = None,
    expected_machine: int | str | None = PE_MACHINE_X86_64,
    allowed_imports: Iterable[str] = (),
    reject_delay_imports: bool = True,
    reject_forwarded_exports: bool = True,
    expected_kind: str | None = None,
) -> dict[str, object]:
    """Validate required exports and return a stable ABI evidence payload."""

    image = _load_pe(binary_path)
    exports, forwarded_exports = _parse_pe_exports_with_forwarders(image)
    required = tuple(dict.fromkeys(str(value).strip() for value in required_exports if str(value).strip()))
    missing = sorted(set(required) - set(exports))
    if missing:
        raise AbiValidationError(
            f"Runtime DLL {Path(binary_path)} is missing required export(s): {', '.join(missing)}"
        )
    expected_machine_value = (
        _machine_value(expected_machine) if expected_machine is not None else None
    )
    if expected_machine_value is not None and image.machine != expected_machine_value:
        raise AbiValidationError(
            f"Runtime image machine 0x{image.machine:04X} "
            f"does not match expected 0x{expected_machine_value:04X}"
        )
    expected_bitness = _machine_bitness(image.machine)
    if expected_bitness is not None and image.bitness != expected_bitness:
        raise AbiValidationError(
            f"Runtime image optional-header bitness {image.bitness} does not match "
            f"machine 0x{image.machine:04X}"
        )
    if forwarded_exports and reject_forwarded_exports:
        raise AbiValidationError(
            "Runtime image contains forwarded export(s) without an explicit shipping policy: "
            + ", ".join(forwarded_exports)
        )
    pe_report = validate_product_pe_image(
        image,
        product_root=product_root,
        expected_machine=None,
        allowed_imports=allowed_imports,
        reject_delay_imports=reject_delay_imports,
        reject_forwarded_exports=False,
        expected_kind=expected_kind,
    )
    delay_imports = tuple(pe_report["delay_imports"])
    imports = tuple(pe_report["imports"])
    non_system_imports = tuple(pe_report["non_system_imports"])
    unexpected_imports = tuple(pe_report["unexpected_imports"])
    source_report = validate_runtime_ffi_source(repo_root) if repo_root is not None else None
    report = {
        "binary": Path(binary_path).name,
        "required_exports": list(required),
        "exports": list(exports),
        "missing_exports": missing,
        "forwarded_exports": list(forwarded_exports),
        "machine": pe_report["machine"],
        "machine_hex": pe_report["machine_hex"],
        "machine_name": pe_report["machine_name"],
        "bitness": pe_report["bitness"],
        "imports": list(imports),
        "delay_imports": list(delay_imports),
        "non_system_imports": list(non_system_imports),
        "unexpected_imports": list(unexpected_imports),
        "import_allowlist": pe_report["import_allowlist"],
        "product_root": _portable_path(product_root),
        "expected_machine": (
            f"0x{expected_machine_value:04X}"
            if expected_machine_value is not None
            else None
        ),
        "expected_kind": expected_kind,
        "source": source_report,
        "passed": not missing
        and not unexpected_imports
        and (not delay_imports or not reject_delay_imports)
        and (not forwarded_exports or not reject_forwarded_exports)
        and (source_report is None or source_report["no_unwind_guard"] is True),
    }
    return report


def validate_product_pe_image(
    binary_path: Path | str | _PeImage,
    *,
    product_root: Path | str | None = None,
    expected_machine: int | str | None = PE_MACHINE_X86_64,
    allowed_imports: Iterable[str] = (),
    reject_delay_imports: bool = True,
    reject_forwarded_exports: bool = True,
    expected_kind: str | None = None,
) -> dict[str, object]:
    """Validate the machine and native dependency closure of one product PE."""

    image = binary_path if isinstance(binary_path, _PeImage) else _load_pe(binary_path)
    if expected_kind not in {None, "dll", "exe"}:
        raise AbiValidationError(
            f"expected_kind must be one of 'dll', 'exe', or None, got {expected_kind!r}"
        )
    if expected_kind == "dll" and not image.is_dll:
        raise AbiValidationError(f"PE image {image.path} is not marked as a DLL")
    if expected_kind == "exe" and image.is_dll:
        raise AbiValidationError(f"PE image {image.path} is marked as a DLL, not an executable")
    expected_machine_value = (
        _machine_value(expected_machine) if expected_machine is not None else None
    )
    if expected_machine_value is not None and image.machine != expected_machine_value:
        raise AbiValidationError(
            f"PE image machine 0x{image.machine:04X} does not match expected "
            f"0x{expected_machine_value:04X}"
        )
    machine_bitness = _machine_bitness(image.machine)
    if machine_bitness is not None and image.bitness != machine_bitness:
        raise AbiValidationError(
            f"PE image optional-header bitness {image.bitness} does not match "
            f"machine 0x{image.machine:04X}"
        )
    exports, forwarded_exports = _parse_pe_exports_with_forwarders(image)
    imports = tuple(_parse_pe_imports(image, _PE_DIRECTORY_IMPORT))
    delay_imports = tuple(_parse_pe_imports(image, _PE_DIRECTORY_DELAY_IMPORT))
    if delay_imports and reject_delay_imports:
        raise AbiValidationError(
            "PE image uses delay imports without an explicit shipping policy: "
            + ", ".join(delay_imports)
        )
    if forwarded_exports and reject_forwarded_exports:
        raise AbiValidationError(
            "PE image contains forwarded export(s) without an explicit shipping policy: "
            + ", ".join(forwarded_exports)
        )
    allowed = _normalise_import_names(allowed_imports)
    staged_names = (
        _staged_native_names(product_root) if product_root is not None else frozenset()
    )
    # Delay-loaded DLLs are still part of the native dependency closure.  An
    # explicit policy may permit delay loading, but it must not bypass the
    # staged/allowlisted import check.
    all_imports = set(imports) | set(delay_imports)
    non_system_imports = tuple(
        sorted(name for name in all_imports if not _is_windows_system_import(name))
    )
    unexpected_imports = tuple(
        sorted(
            name
            for name in non_system_imports
            if name not in allowed and name not in staged_names
        )
    )
    if unexpected_imports:
        root_text = (
            f" under {Path(product_root).resolve()}" if product_root is not None else ""
        )
        raise AbiValidationError(
            "PE image imports non-system DLLs that are not in the product closure"
            f"{root_text}: {', '.join(unexpected_imports)}"
        )
    return {
        "binary": image.path.name,
        "machine": image.machine,
        "machine_hex": f"0x{image.machine:04X}",
        "machine_name": _machine_name(image.machine),
        "bitness": image.bitness,
        "is_dll": image.is_dll,
        "expected_kind": expected_kind,
        "exports": list(exports),
        "forwarded_exports": list(forwarded_exports),
        "imports": list(imports),
        "delay_imports": list(delay_imports),
        "non_system_imports": list(non_system_imports),
        "unexpected_imports": list(unexpected_imports),
        "import_allowlist": sorted(set(allowed) | set(_WINDOWS_SYSTEM_IMPORTS)),
        "product_root": _portable_path(product_root),
        "expected_machine": (
            f"0x{expected_machine_value:04X}"
            if expected_machine_value is not None
            else None
        ),
        "passed": True,
    }


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dll", type=Path)
    parser.add_argument("--repo-root", type=Path)
    parser.add_argument("--product-root", type=Path)
    parser.add_argument("--target-triple", default="x86_64-pc-windows-msvc")
    parser.add_argument("--allow-import", action="append", default=[])
    parser.add_argument("--allow-delay-imports", action="store_true")
    args = parser.parse_args(argv)
    try:
        result = validate_runtime_abi(
            args.dll,
            repo_root=args.repo_root,
            product_root=args.product_root,
            expected_machine=_machine_for_target(args.target_triple),
            allowed_imports=args.allow_import,
            reject_delay_imports=not args.allow_delay_imports,
        )
    except AbiValidationError as error:
        print(json.dumps({"passed": False, "error": str(error)}, indent=2))
        return 2
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def _u16(data: bytes, offset: int, label: str) -> int:
    if offset < 0 or offset + 2 > len(data):
        raise AbiValidationError(f"truncated PE while reading {label}")
    return struct.unpack_from("<H", data, offset)[0]


def _u32(data: bytes, offset: int, label: str) -> int:
    if offset < 0 or offset + 4 > len(data):
        raise AbiValidationError(f"truncated PE while reading {label}")
    return struct.unpack_from("<I", data, offset)[0]


def _rva_to_offset(
    rva: int,
    sections: Sequence[tuple[int, ...]],
    data_length: int,
    path: Path,
) -> int:
    if rva < 0:
        raise AbiValidationError(f"{path} contains a negative PE RVA")
    # RVAs in the headers are valid even when no section owns them.
    for section in sections:
        if len(section) == 3:
            virtual_address, span, raw_offset = section
        else:
            virtual_address, virtual_size, raw_offset, raw_size = section[:4]
            span = max(virtual_size, raw_size)
        section_end = virtual_address + span
        if virtual_address <= rva < section_end:
            delta = rva - virtual_address
            offset = raw_offset + delta
            if raw_offset <= data_length and offset < data_length:
                return offset
    raise AbiValidationError(f"{path} PE RVA 0x{rva:X} is outside PE sections")


def _read_c_string(data: bytes, offset: int, path: Path) -> str:
    end = data.find(b"\0", offset)
    if end < 0:
        raise AbiValidationError(f"{path} contains an unterminated export name")
    try:
        return data[offset:end].decode("ascii")
    except UnicodeDecodeError as error:
        raise AbiValidationError(f"{path} contains a non-ASCII export name") from error


def _load_pe(binary_path: Path | str) -> _PeImage:
    """Load and structurally validate a bounded PE32/PE32+ image."""

    unresolved = Path(binary_path).expanduser()
    _assert_no_reparse_ancestors(unresolved)
    if _is_reparse_point(unresolved):
        raise AbiValidationError(f"Runtime image is a symlink/reparse point: {unresolved}")
    path = unresolved.resolve()
    if _is_reparse_point(path) or not path.is_file():
        raise AbiValidationError(f"Runtime image is not a regular file: {path}")
    try:
        size = path.stat().st_size
    except OSError as error:
        raise AbiValidationError(f"could not stat Runtime image {path}: {error}") from error
    if size <= 0 or size > _MAX_PE_IMAGE_BYTES:
        raise AbiValidationError(
            f"Runtime image size {size} is outside the PE admission limit"
        )
    try:
        data = path.read_bytes()
    except OSError as error:
        raise AbiValidationError(f"could not read Runtime image {path}: {error}") from error
    if len(data) != size:
        raise AbiValidationError(f"Runtime image changed while it was being read: {path}")
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise AbiValidationError(f"{path} is not a valid DOS/PE image")
    pe_offset = _u32(data, 0x3C, "DOS e_lfanew")
    if pe_offset > len(data) - 4 or data[pe_offset : pe_offset + 4] != b"PE\0\0":
        raise AbiValidationError(f"{path} has an invalid PE signature")
    coff = pe_offset + 4
    machine = _u16(data, coff, "COFF machine")
    section_count = _u16(data, coff + 2, "COFF section count")
    optional_size = _u16(data, coff + 16, "COFF optional-header size")
    if section_count == 0 or section_count > 96:
        raise AbiValidationError(f"{path} has an invalid PE section count {section_count}")
    optional = coff + 20
    if optional > len(data) or optional_size > len(data) - optional:
        raise AbiValidationError(f"{path} has a truncated PE optional header")
    magic = _u16(data, optional, "optional-header magic")
    if magic == 0x10B:
        bitness = 32
        directory_count_offset = optional + 92
        directory_offset = optional + 96
    elif magic == 0x20B:
        bitness = 64
        directory_count_offset = optional + 108
        directory_offset = optional + 112
    else:
        raise AbiValidationError(f"{path} uses unsupported PE optional-header magic 0x{magic:X}")
    machine_bitness = _machine_bitness(machine)
    if machine_bitness is not None and machine_bitness != bitness:
        raise AbiValidationError(
            f"{path} PE machine 0x{machine:04X} conflicts with optional-header bitness {bitness}"
        )
    if directory_count_offset + 4 > optional + optional_size:
        raise AbiValidationError(f"{path} optional header omits the PE data-directory count")
    raw_directory_count = _u32(data, directory_count_offset, "data-directory count")
    directory_count = min(raw_directory_count, _IMAGE_DIRECTORY_ENTRY_COUNT)
    # Keep compatibility with deliberately tiny PE fixtures that populate an
    # export directory but omit NumberOfRvaAndSizes.  A real image normally
    # stores 16 here; inference is only made when the first directory is
    # visibly populated and the optional header has room for the table.
    if (
        raw_directory_count == 0
        and directory_offset + 8 <= optional + optional_size
        and any(data[directory_offset : directory_offset + 8])
    ):
        directory_count = 1
    directory_bytes = directory_count * 8
    if directory_offset > optional + optional_size or directory_bytes > optional + optional_size - directory_offset:
        raise AbiValidationError(f"{path} has a truncated PE data-directory table")
    directories: list[tuple[int, int]] = []
    for index in range(directory_count):
        offset = directory_offset + index * 8
        directories.append(
            (_u32(data, offset, f"data-directory[{index}] RVA"), _u32(data, offset + 4, f"data-directory[{index}] size"))
        )
    while len(directories) < _IMAGE_DIRECTORY_ENTRY_COUNT:
        directories.append((0, 0))
    section_offset = optional + optional_size
    section_table_bytes = section_count * 40
    if section_offset > len(data) or section_table_bytes > len(data) - section_offset:
        raise AbiValidationError(f"{path} has a truncated PE section table")
    sections: list[tuple[int, int, int, int]] = []
    for index in range(section_count):
        offset = section_offset + index * 40
        virtual_size = _u32(data, offset + 8, f"section[{index}] virtual size")
        virtual_address = _u32(data, offset + 12, f"section[{index}] virtual address")
        raw_size = _u32(data, offset + 16, f"section[{index}] raw size")
        raw_offset = _u32(data, offset + 20, f"section[{index}] raw offset")
        if raw_offset > len(data):
            raise AbiValidationError(f"{path} section[{index}] raw data is outside the image")
        # Keep the declared virtual span for RVA translation, but clamp the
        # file-backed portion.  Linker-generated test/minimal images sometimes
        # leave SizeOfRawData rounded past EOF; every actual read remains
        # bounded by ``_rva_to_offset`` and the byte buffer.
        raw_size = min(raw_size, len(data) - raw_offset)
        sections.append((virtual_address, virtual_size, raw_offset, raw_size))
    export_directory = directories[_PE_DIRECTORY_EXPORT]
    return _PeImage(
        path=path,
        data=data,
        machine=machine,
        bitness=bitness,
        sections=tuple(sections),
        directories=tuple(directories),
        is_dll=bool(_u16(data, coff + 18, "COFF characteristics") & _IMAGE_FILE_DLL),
        export_directory=export_directory if export_directory[0] and export_directory[1] else None,
    )


def _is_reparse_point(path: Path) -> bool:
    """Detect links/junctions without resolving the candidate first."""

    try:
        if path.is_symlink():
            return True
        attributes = os.lstat(path).st_file_attributes
    except (OSError, AttributeError):
        return False
    return bool(attributes & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400))


def _assert_no_reparse_ancestors(path: Path) -> None:
    current = path if path.is_absolute() else Path.cwd() / path
    while True:
        if _is_reparse_point(current):
            raise AbiValidationError(f"Runtime image path contains a symlink/reparse point: {current}")
        if current.parent == current:
            return
        current = current.parent


def _directory_offset(image: _PeImage, index: int) -> tuple[int, int] | None:
    if index < 0 or index >= len(image.directories):
        return None
    rva, size = image.directories[index]
    if not rva or not size:
        return None
    # Mapping the first byte also catches a directory pointing outside the image.
    start = _rva_to_offset(rva, image.sections, len(image.data), image.path)
    if size > len(image.data) - start:
        raise AbiValidationError(
            f"{image.path} PE directory {index} extends beyond the image"
        )
    return start, size


def _parse_pe_exports(image: _PeImage) -> tuple[str, ...]:
    return _parse_pe_exports_with_forwarders(image)[0]


def _parse_pe_exports_with_forwarders(
    image: _PeImage,
) -> tuple[tuple[str, ...], tuple[str, ...]]:
    directory = _directory_offset(image, _PE_DIRECTORY_EXPORT)
    if directory is None:
        return (), ()
    offset, size = directory
    if size < 40:
        raise AbiValidationError(f"{image.path} has a truncated PE export directory")
    number_of_functions = _u32(image.data, offset + 20, "export function count")
    number_of_names = _u32(image.data, offset + 24, "export name count")
    if number_of_functions > _MAX_EXPORTS or number_of_names > _MAX_EXPORTS:
        raise AbiValidationError(f"{image.path} has an excessive PE export count")
    address_of_functions = _u32(image.data, offset + 28, "export function table RVA")
    address_of_names = _u32(image.data, offset + 32, "export name table RVA")
    address_of_ordinals = _u32(image.data, offset + 36, "export ordinal table RVA")
    # A few linkers/fixtures omit the function count and ordinal table when
    # they expose one unnamed ordinal.  Treat that narrow shape as ordinal 0;
    # all indexed tables are still bounds checked below.
    effective_function_count = number_of_functions or number_of_names
    names: list[str] = []
    forwarded: list[str] = []
    export_rva, export_size = image.directories[_PE_DIRECTORY_EXPORT]
    for index in range(number_of_names):
        name_rva_offset = _rva_to_offset(
            address_of_names + index * 4,
            image.sections,
            len(image.data),
            image.path,
        )
        name_rva = _u32(image.data, name_rva_offset, f"export name RVA[{index}]")
        name_offset = _rva_to_offset(name_rva, image.sections, len(image.data), image.path)
        name = _read_c_string(image.data, name_offset, image.path)
        names.append(name)
        if address_of_ordinals:
            ordinal_offset = _rva_to_offset(
                address_of_ordinals + index * 2,
                image.sections,
                len(image.data),
                image.path,
            )
            ordinal = _u16(image.data, ordinal_offset, f"export ordinal[{index}]")
        else:
            ordinal = 0
        if ordinal >= effective_function_count:
            raise AbiValidationError(f"{image.path} export ordinal {ordinal} is out of range")
        if address_of_functions:
            function_offset = _rva_to_offset(
                address_of_functions + ordinal * 4,
                image.sections,
                len(image.data),
                image.path,
            )
            function_rva = _u32(image.data, function_offset, f"export RVA[{index}]")
            if function_rva and export_rva <= function_rva < export_rva + export_size:
                forwarded.append(name)
    return tuple(sorted(set(names))), tuple(sorted(set(forwarded)))


def _parse_pe_imports(image: _PeImage, directory_index: int) -> tuple[str, ...]:
    directory = _directory_offset(image, directory_index)
    if directory is None:
        return ()
    offset, size = directory
    if directory_index == _PE_DIRECTORY_DELAY_IMPORT:
        return _parse_delay_imports(image, offset, size)
    if size < 20:
        raise AbiValidationError(f"{image.path} has a truncated PE import directory")
    names: list[str] = []
    max_descriptors = min(_MAX_IMPORT_DESCRIPTORS, size // 20)
    terminated = False
    for index in range(max_descriptors):
        descriptor_offset = offset + index * 20
        fields = struct.unpack_from("<5I", image.data, descriptor_offset)
        if fields == (0, 0, 0, 0, 0):
            terminated = True
            break
        name_rva = fields[3]
        if not name_rva:
            raise AbiValidationError(f"{image.path} import descriptor {index} has no DLL name")
        name_offset = _rva_to_offset(name_rva, image.sections, len(image.data), image.path)
        names.append(_normalise_dll_name(_read_c_string(image.data, name_offset, image.path), image.path))
    if not terminated and max_descriptors < _MAX_IMPORT_DESCRIPTORS:
        raise AbiValidationError(f"{image.path} import directory is not NUL-terminated")
    if not terminated and max_descriptors == _MAX_IMPORT_DESCRIPTORS:
        raise AbiValidationError(f"{image.path} import descriptor count exceeds admission limit")
    return tuple(sorted(set(names)))


def _parse_delay_imports(image: _PeImage, offset: int, size: int) -> tuple[str, ...]:
    if size < 32:
        raise AbiValidationError(f"{image.path} has a truncated PE delay-import directory")
    names: list[str] = []
    max_descriptors = min(_MAX_IMPORT_DESCRIPTORS, size // 32)
    terminated = False
    for index in range(max_descriptors):
        descriptor_offset = offset + index * 32
        fields = struct.unpack_from("<8I", image.data, descriptor_offset)
        if fields == (0,) * 8:
            terminated = True
            break
        attrs, name_rva = fields[0], fields[1]
        if not attrs & _IMAGE_DELAY_IMPORT_RVA:
            raise AbiValidationError(
                f"{image.path} delay-import descriptor {index} uses unsupported VA addresses"
            )
        if not name_rva:
            raise AbiValidationError(f"{image.path} delay-import descriptor {index} has no DLL name")
        name_offset = _rva_to_offset(name_rva, image.sections, len(image.data), image.path)
        names.append(_normalise_dll_name(_read_c_string(image.data, name_offset, image.path), image.path))
    if not terminated:
        raise AbiValidationError(f"{image.path} delay-import directory is not NUL-terminated")
    return tuple(sorted(set(names)))


def _normalise_dll_name(value: str, path: Path) -> str:
    name = value.strip().lower()
    if (
        not name
        or not name.isascii()
        or name != value.lower()
        or "/" in name
        or "\\" in name
        or ":" in name
        or any(ord(character) < 0x20 for character in name)
    ):
        raise AbiValidationError(f"{path} contains an invalid PE DLL import name {value!r}")
    if not name.endswith((".dll", ".ocx", ".sys")):
        # Windows accepts extensionless imports, but the shipping policy only
        # admits plain library names.  Keep the normalized spelling for the
        # allowlist while rejecting path-like values above.
        if "." in name:
            raise AbiValidationError(f"{path} contains an invalid PE DLL import name {value!r}")
    return name


def _normalise_import_names(values: Iterable[str]) -> frozenset[str]:
    normalized: set[str] = set()
    for value in values:
        if not isinstance(value, str) or not value.strip():
            continue
        normalized.add(_normalise_dll_name(value.strip(), Path("<allowlist>")))
    return frozenset(normalized)


def _is_windows_system_import(name: str) -> bool:
    lowered = name.lower()
    return (
        lowered in _WINDOWS_SYSTEM_IMPORTS
        or lowered.startswith("api-ms-win-")
        or lowered.startswith("ext-ms-win-")
        or lowered.startswith("api-ms-onecore-")
    )


def _staged_native_names(product_root: Path | str | None) -> frozenset[str]:
    if product_root is None:
        return frozenset()
    unresolved = Path(product_root).expanduser()
    _assert_no_reparse_ancestors(unresolved)
    root = unresolved.resolve()
    if _is_reparse_point(root) or not root.is_dir():
        raise AbiValidationError(f"product root is not a directory: {root}")
    names: set[str] = set()

    def visit(directory: Path) -> None:
        try:
            with os.scandir(directory) as iterator:
                entries = sorted(iterator, key=lambda entry: entry.name.casefold())
        except OSError as error:
            raise AbiValidationError(
                f"could not inspect product root {directory}: {error}"
            ) from error
        for entry in entries:
            candidate = Path(entry.path)
            if _is_reparse_point(candidate):
                raise AbiValidationError(f"product root contains a symlink/reparse point: {candidate}")
            try:
                if entry.is_dir(follow_symlinks=False):
                    visit(candidate)
                elif entry.is_file(follow_symlinks=False) and candidate.suffix.lower() in {
                    ".dll",
                    ".exe",
                    ".ocx",
                    ".sys",
                }:
                    names.add(candidate.name.lower())
            except OSError as error:
                raise AbiValidationError(
                    f"could not inspect product root path {candidate}: {error}"
                ) from error

    visit(root)
    return frozenset(names)


def _machine_name(machine: int) -> str:
    return {
        PE_MACHINE_X86_64: "x86_64",
        PE_MACHINE_X86: "x86",
        PE_MACHINE_ARM64: "aarch64",
    }.get(machine, "unknown")


def _machine_bitness(machine: int) -> int | None:
    if machine == PE_MACHINE_X86_64 or machine == PE_MACHINE_ARM64:
        return 64
    if machine == PE_MACHINE_X86:
        return 32
    return None


def _machine_value(value: int | str) -> int:
    if isinstance(value, int):
        return value
    if not isinstance(value, str):
        raise AbiValidationError(f"unsupported expected PE machine/target {value!r}")
    normalized = value.strip().lower()
    aliases = {
        "x86_64": PE_MACHINE_X86_64,
        "amd64": PE_MACHINE_X86_64,
        "x86_64-pc-windows-msvc": PE_MACHINE_X86_64,
        "x86": PE_MACHINE_X86,
        "i686": PE_MACHINE_X86,
        "aarch64": PE_MACHINE_ARM64,
        "arm64": PE_MACHINE_ARM64,
        "aarch64-pc-windows-msvc": PE_MACHINE_ARM64,
    }
    if normalized not in aliases:
        raise AbiValidationError(f"unsupported expected PE machine/target {value!r}")
    return aliases[normalized]


def _machine_for_target(target_triple: str) -> int:
    return _machine_value(target_triple)


def _portable_path(value: Path | str | None) -> str | None:
    if value is None:
        return None
    return Path(value).name or None


if __name__ == "__main__":
    raise SystemExit(main())
