"""Bounded Jenkins cleanup of the explicitly authorized shared Rust cache.

Rust's session lock is the authority for live cache use. No source-tree inventory
or content hashing is needed to discard this regenerable cache.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import ctypes
import hashlib
from ctypes import wintypes
import json
import os
from pathlib import Path
import re
import shutil
import stat
import sys
import time
import urllib.parse
import xml.etree.ElementTree as ET

from checkout_commit import physical, write_json, transport

REPO = Path("E:/Git/ZirconEngine")
CACHE = Path("E:/cargo-targets/zircon-local/win32/codex-default/build/debug/incremental")
REPORTS = Path("E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/incremental")
BEGIN = "// BEGIN ZIRCON INCREMENTAL CACHE CLEANUP"
END = "// END ZIRCON INCREMENTAL CACHE CLEANUP"
GIB = 2 ** 30
# Exact namespaces audited for the user-requested D/E/F cleanup on 2026-10-18.
# Only Rust incremental sessions are eligible; these are not whole build/source roots.
CACHE_POLICIES = {
    'shared-e': (Path('E:/cargo-targets/zircon-local/win32/codex-default/build/debug/incremental'), 80 * GIB),
    'legacy-d-integration': (Path('D:/cargo-targets/zircon-local/win32/2e35cdd9399f5e89889e/build/debug/incremental'), 80 * GIB),
    'legacy-d-editor': (Path('D:/cargo-targets/zircon-local/win32/engine-audit-editor102-testbuild-20261007-r4/build/debug/incremental'), 80 * GIB),
    'audited-d-1': (Path('D:/cargo-targets/zircon-local/dependency-fix/wasm/build/wasm32-unknown-unknown/debug/incremental'), 80 * GIB),
    'audited-d-2': (Path('D:/cargo-targets/zircon-local/dependency-fix/wasm/build/debug/incremental'), 80 * GIB),
    'audited-d-5': (Path('D:/cargo-targets/zircon-local/dependency-fix/font-probe/build/debug/incremental'), 80 * GIB),
    'audited-d-6': (Path('D:/cargo-targets/zircon-local/dependency-fix/codec-probe/build/debug/incremental'), 80 * GIB),
    'audited-d-7': (Path('D:/cargo-targets/zircon-local/windows/runtime-editor-app-20261004-v1/debug/build/debug/incremental'), 80 * GIB),
    'audited-d-8': (Path('D:/cargo-targets/zircon-local/windows/m1-runtime-tests-20261004-r4/build/debug/incremental'), 80 * GIB),
    'audited-d-9': (Path('D:/cargo-targets/zircon-local/windows/build/debug/incremental'), 80 * GIB),
    'dependency-d-native': (Path('D:/cargo-targets/zircon-local/dependency-fix/native/build/debug/incremental'), 80 * GIB),
    'dependency-d-plugins': (Path('D:/cargo-targets/zircon-local/dependency-fix/plugins/build/debug/incremental'), 80 * GIB),
    'audited-e-15': (Path('E:/cargo-targets/zircon-local/m/x86_64-unknown-linux-gnu/debug/incremental'), 80 * GIB),
    'audited-e-16': (Path('E:/cargo-targets/zircon-local/m/x86_64-pc-windows-msvc/debug/incremental'), 80 * GIB),
    'audited-e-17': (Path('E:/cargo-targets/zircon-local/m/debug/incremental'), 80 * GIB),
    'audited-e-19': (Path('E:/cargo-targets/zircon-local/win32/miri-windows-20261010/build/debug/incremental'), 80 * GIB),
    'audited-e-20': (Path('E:/cargo-targets/zircon-local/win32/codex-default-wasm/build/wasm32-unknown-unknown/debug/incremental'), 80 * GIB),
    'audited-e-21': (Path('E:/cargo-targets/zircon-local/win32/codex-default-wasm/build/debug/incremental'), 80 * GIB),
    'audited-e-22': (Path('E:/cargo-targets/zircon-local/win32/codex-default/build/x86_64-unknown-linux-gnu/debug/incremental'), 80 * GIB),
    'audited-f-12': (Path('F:/cargo-targets/zircon-local/penpot-workbench/01a0f04d-current/retained-native-runtime-incremental-r273/build/debug/incremental'), 80 * GIB),
    'audited-f-13': (Path('F:/cargo-targets/zircon-local/penpot-workbench/01a0f04d-current/retained-finished-native-runtime-and-old-product-deps-r290/build/debug/incremental'), 80 * GIB),
    'audited-f-14': (Path('F:/cargo-targets/zircon-local/penpot-workbench/01a0f04d-current/retained-finished-native-runtime-and-old-editor-incremental-r342/build/debug/incremental'), 80 * GIB),
    'audited-f-10': (Path('F:/cargo-targets/zircon-local/penpot-workbench/01a0f04d-current/unreal-current-native-product-r392/product/targets/runtime/build/debug/incremental'), 80 * GIB),
    'audited-f-11': (Path('F:/cargo-targets/zircon-local/penpot-workbench/01a0f04d-current/unreal-current-native-product-r392/product/targets/build/debug/incremental'), 80 * GIB),
}


class CacheBusy(OSError):
    pass


def kernel():
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
                               ctypes.c_void_p, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p]
    api.CreateFileW.restype = wintypes.HANDLE
    api.CloseHandle.argtypes = [wintypes.HANDLE]
    api.CloseHandle.restype = wintypes.BOOL
    api.GetFileInformationByHandleEx.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    api.GetFileInformationByHandleEx.restype = wintypes.BOOL
    api.SetFileInformationByHandle.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    api.SetFileInformationByHandle.restype = wintypes.BOOL
    return api


@contextmanager
def session_lock(path):
    """Hold the same nonblocking exclusive byte-range lock used by rustc."""
    import msvcrt
    path = physical(path)
    api = kernel()
    # Refuse deletion/replacement of the lock inode while it protects the session.
    handle = api.CreateFileW(str(path), 0xC0000000, 3, None, 3, 0x02200000, None)
    if handle == wintypes.HANDLE(-1).value:
        raise CacheBusy("session lock is unavailable")
    descriptor = None
    try:
        descriptor = msvcrt.open_osfhandle(handle, os.O_RDWR | os.O_BINARY)
        info = os.fstat(descriptor)
        if not stat.S_ISREG(info.st_mode) or info.st_size or info.st_nlink != 1:
            raise ValueError("unrecognized Rust session lock")
        class Overlapped(ctypes.Structure):
            _fields_ = [("internal", ctypes.c_size_t), ("internal_high", ctypes.c_size_t),
                        ("offset", wintypes.DWORD), ("offset_high", wintypes.DWORD), ("event", wintypes.HANDLE)]
        overlapped = Overlapped()
        api.LockFileEx.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.DWORD,
                                  wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p]
        api.LockFileEx.restype = wintypes.BOOL
        if not api.LockFileEx(handle, 3, 0, 0xFFFFFFFF, 0xFFFFFFFF, ctypes.byref(overlapped)):
            raise CacheBusy("compiler session is locked")
        yield
    finally:
        if descriptor is not None:
            os.close(descriptor)
        else:
            api.CloseHandle(handle)


def cache_files(session):
    """Recognize flat rustc session files; unknown content is retained as a unit."""
    facts = []
    for path in physical(session).iterdir():
        physical(path)
        info = path.lstat()
        if not stat.S_ISREG(info.st_mode) or path.suffix not in {".o", ".obj", ".bin", ".rmeta", ".bc"}:
            raise ValueError("unknown incremental session content retained")
        facts.append({"path": str(path), "identity": [info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns],
                      "logicalBytes": info.st_size})
    return facts


def remove_cache_file(fact):
    """Unlink only the observed file handle; preserve every other hard link."""
    import msvcrt
    if str(REPO) not in sys.path:
        sys.path.insert(0, str(REPO))
    from tools.jenkins.governance.guarded_delete import locked_parents
    path = physical(fact["path"])
    with locked_parents(path) as api:
        handle = api.CreateFileW(str(path), 0x80000000 | 0x10000, 1, None, 3, 0x02200000, None)
        if handle == wintypes.HANDLE(-1).value:
            raise ctypes.WinError(ctypes.get_last_error())
        descriptor = None
        try:
            descriptor = msvcrt.open_osfhandle(handle, os.O_RDONLY | os.O_BINARY)
            info = os.fstat(descriptor)
            if [info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns] != fact["identity"]:
                raise ValueError("cache file changed during cleanup")
            class Standard(ctypes.Structure):
                _fields_ = [("allocation", ctypes.c_longlong), ("end", ctypes.c_longlong),
                            ("links", wintypes.DWORD), ("pending", ctypes.c_ubyte), ("directory", ctypes.c_ubyte)]
            standard = Standard()
            if not api.GetFileInformationByHandleEx(handle, 1, ctypes.byref(standard), ctypes.sizeof(standard)):
                raise ctypes.WinError(ctypes.get_last_error())
            disposition = ctypes.c_ubyte(1)
            api.SetFileInformationByHandle.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
            api.SetFileInformationByHandle.restype = wintypes.BOOL
            if not api.SetFileInformationByHandle(handle, 4, ctypes.byref(disposition), 1):
                raise ctypes.WinError(ctypes.get_last_error())
            return {**fact, "status": "deleted", "hardLinksBefore": standard.links,
                    "unlinkedAllocationBytes": standard.allocation if standard.links == 1 else 0}
        finally:
            if descriptor is not None:
                os.close(descriptor)
            else:
                api.CloseHandle(handle)


def clean(cache, output, *, dry_run=False, max_bytes=32 * GIB, minimum_age=600, trigger_free=80 * GIB):
    cache, output = physical(cache), physical(output)
    if output.exists():
        raise ValueError("retain the original cleanup receipt; use a new run identity")
    output.mkdir(parents=True)
    manifest = {"taskId": "jenkins-scheduled-cleanup", "owner": "Jenkins compiler-cache maintenance",
                "status": "running", "cacheRoot": str(cache), "authorization": "user requested recurring artifact cleanup",
                "artifacts": [{"path": str(output), "retention": "retain deletion receipts and unknown/active protection evidence"}],
                "preserved": ["source", "Git data", "downloaded dependencies", "locked compiler sessions", "unknown content"]}
    write_json(output / "artifacts.json", manifest)
    result = {"schemaVersion": 1, "status": "running", "cacheRoot": str(cache), "dryRun": dry_run,
              "maxBytes": max_bytes, "triggerFreeBytes": trigger_free, "minimumAgeSeconds": minimum_age,
              "logicalDeletedBytes": 0, "unlinkedAllocationBytes": 0, "deletedFiles": 0,
              "selectedBytes": 0, "sessions": [], "errors": [], "startedAt": time.time(),
              "scope": "allowlisted-rust-incremental-sessions-only",
              "selectionAttempted": False, "discoveredSessions": None, "globalCleanupComplete": False,
              "freeBefore": shutil.disk_usage(cache.anchor).free, "actualFreedBytes": None}
    write_json(output / "receipt.json", result)
    try:
        if not cache.exists():
            result["status"] = "cache-absent"
        elif result["freeBefore"] >= trigger_free:
            result["status"] = "enough-free-space"
        else:
            sessions = sorted((p for unit in cache.iterdir() if unit.is_dir() and not unit.is_junction()
                               for p in unit.iterdir() if p.is_dir() and not p.is_junction()),
                              key=lambda p: p.stat().st_mtime)
            result.update(selectionAttempted=True, discoveredSessions=len(sessions))
            with (output / "deletions.jsonl").open("a", encoding="utf-8") as journal:
                for session in sessions:
                    row = {"path": str(session), "status": "protected"}
                    result["sessions"].append(row)
                    try:
                        match = re.fullmatch(r"(s-[a-z0-9]+-[a-z0-9]+)-[a-z0-9]+", session.name)
                        if not match or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*-[a-z0-9]+", session.parent.name):
                            raise ValueError("unrecognized Rust session path")
                        lock = physical(session.parent / (match[1] + ".lock"))
                        if time.time() - max(session.stat().st_mtime, lock.stat().st_mtime) < minimum_age:
                            raise CacheBusy("new compiler session retained")
                        with session_lock(lock):
                            files = cache_files(session)
                            size = sum(fact["logicalBytes"] for fact in files)
                            if result["selectedBytes"] + size > max_bytes:
                                row.update(reason="run-byte-budget", logicalBytes=size)
                                continue
                            row.update(status="planned", logicalBytes=size, files=len(files))
                            result["selectedBytes"] += size
                            write_json(output / ("plan-" + session.parent.name + "-" + session.name + ".json"),
                                       {"session": str(session), "lock": str(lock), "files": files})
                            if dry_run:
                                row["status"] = "dry-run"
                                continue
                            for fact in files:
                                # Persist intent before native deletion and the acknowledgement afterwards.
                                journal.write(json.dumps({**fact, "status": "delete-intent"}) + "\n")
                                journal.flush()
                                deleted = remove_cache_file(fact)
                                result["logicalDeletedBytes"] += deleted["logicalBytes"]
                                result["unlinkedAllocationBytes"] += deleted["unlinkedAllocationBytes"]
                                result["deletedFiles"] += 1
                                journal.write(json.dumps(deleted) + "\n")
                                journal.flush()
                            if str(REPO) not in sys.path:
                                sys.path.insert(0, str(REPO))
                            from tools.jenkins.governance.guarded_delete import remove_empty
                            remove_empty(session)
                            row["status"] = "deleted"
                    except (OSError, ValueError) as error:
                        row.update(status="protected", reason=str(error))
                        if not isinstance(error, CacheBusy):
                            result["errors"].append({"path": str(session), "reason": str(error)})
                    write_json(output / "receipt.json", result)
            result["status"] = "partial" if result["errors"] else ("complete" if sessions else "cache-empty")
    except BaseException as error:
        result.update(status="failed", failure=str(error))
        raise
    finally:
        result["retainedSessions"] = sum(row["status"] != "deleted" for row in result["sessions"])
        result["namespaceCleanupComplete"] = (result["status"] in {"complete", "cache-empty", "cache-absent"}
                                               and not result["retainedSessions"])
        result.update(freeAfter=shutil.disk_usage(cache.anchor).free, finishedAt=time.time())
        result["observedFreeDelta"] = result["freeAfter"] - result["freeBefore"]
        result["spaceAccounting"] = "native unlink allocation is separate from volume delta, which includes concurrent writes"
        write_json(output / "receipt.json", result)
        manifest.update(status=result["status"], activeOwnedProcesses=[], logicalDeletedBytes=result["logicalDeletedBytes"])
        write_json(output / "artifacts.json", manifest)
    return result


def install(repo, client):
    route = "/job/zircon-scheduled-cleanup/config.xml"
    code, original, _ = client.request(route)
    if code != 200:
        raise ValueError("Cannot read the existing scheduled cleanup")
    tree = ET.fromstring(original)
    script = tree.find("definition/script")
    source_directory = physical(Path(__file__).parent)
    addon = (source_directory / "incremental-cache-cleanup.groovy").read_text(encoding="utf-8")
    pins = {"entry": physical(Path(__file__)), "helper": physical(source_directory / "checkout_commit.py"),
            "runtime": physical(repo / "tools/dev/jenkins_cleanup_runtime.py"),
            "python": physical(repo / ".jenkins/runtime/python/python.exe")}
    replacements = {"CACHE_KEYS": "[" + ", ".join(repr(key) for key in CACHE_POLICIES) + "]",
                    "DRIVER": json.loads((repo / ".jenkins/state/deployment/driver.json").read_bytes())["driverDigest"]}
    for key, path in pins.items():
        replacements[key.upper()] = path.as_posix()
        replacements[key.upper() + "_SHA"] = hashlib.sha256(path.read_bytes()).hexdigest()
    for key, value in replacements.items():
        addon = addon.replace("@" + key + "@", value)
    if re.search(r"@[A-Z_]+@", addon):
        raise ValueError("Unresolved cleanup implementation pin")
    previous = script.text or ""
    if BEGIN in previous:
        first, tail = previous.split(BEGIN, 1)
        _, tail = tail.split(END, 1)
        previous = first + tail.lstrip("\r\n")
    previous = previous.replace("currentBuild.description = report.status +",
        "currentBuild.description = 'compiler cache ' + (env.ZIRCON_INCREMENTAL_DELETED_BYTES ?: '0') + ' logical bytes; ' + report.status +")
    script.text = addon.rstrip() + "\n" + previous
    if client.request(route)[1] != original:
        raise ValueError("Scheduled cleanup changed during installation")
    code, _, _ = client.request(route, method="POST", data=ET.tostring(tree, encoding="utf-8"), content_type="application/xml")
    if code != 200 or ET.fromstring(client.request(route)[1]).findtext("definition/script") != script.text:
        raise ValueError("Scheduled cache-cleanup installation failed")
    return {"status": "installed", "job": "zircon-scheduled-cleanup", "schedule": "H/30 * * * *",
            "scope": [{"key": key, "path": str(path), "triggerFreeBytes": threshold}
                      for key, (path, threshold) in CACHE_POLICIES.items()], "maxBytesPerNamespacePerRun": 32 * GIB,
            "existingWorkflowPreserved": True, "implementationSource": str(source_directory),
            "sourcePins": {key: {"path": str(path), "sha256": replacements[key.upper() + "_SHA"]}
                           for key, path in pins.items()}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("install", "run"))
    parser.add_argument("--run-id")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--cache-key", choices=tuple(CACHE_POLICIES), default="shared-e")
    args = parser.parse_args()
    if args.command == "install":
        result = install(REPO, transport(REPO))
    else:
        if os.environ.get("JOB_NAME") != "zircon-scheduled-cleanup" or os.environ.get("NODE_NAME") != "zircon-windows-agent":
            raise ValueError("Cache deletion requires the existing Jenkins scheduled job and fixed agent")
        if not args.run_id or not re.fullmatch(r"incremental-[0-9]+", args.run_id):
            raise ValueError("Invalid scheduled cleanup identity")
        cache, threshold = CACHE_POLICIES[args.cache_key]
        suffix = "" if args.cache_key == "shared-e" else "-" + args.cache_key
        result = clean(cache, REPORTS / (args.run_id + suffix), dry_run=args.dry_run, trigger_free=threshold)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
