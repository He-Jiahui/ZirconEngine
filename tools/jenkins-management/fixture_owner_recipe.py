"""Run the original S86 continuation or prepare a current S90 first-suite measurement.

S90 mode uses the reviewed current eight frozen inputs, an exact fresh namespace,
one unconditional suite start and the remaining 180-second total execution budget.
The outer owned Job must enforce that same deadline across producer preflight;
terminal EOF/Job cleanup evidence is drained separately after an execution timeout.
The other eight invocations stay NOTRUN even when all 21 first-suite cases pass.

The plan is an immutable JSON input, passed with --plan/--plan-sha; --self-sha
pins this entry. It supplies taskId/requestId/attemptId, output, ownerThreadId,
message/sourceClaim/artifactRegistration/component pins, runtime, powershell,
pester, controlPins, pinnedInputs, contractTestsBudgetSeconds and nine ordered
invocations (invocationId, commandId, suite, keep, budgetSeconds, argv=[]).
All paths and commands below are derived, never accepted as command templates.
The continuation summary has its own directory. The unchanged wrapper still
uses the original TaskId report namespace and nine new, exact invocation leaves.

summary.json keeps every requested invocation, including those not run after the
first failure. Native termination, suite result, journal lifecycle, compilation
evidence and post-process physical observations are separate facts. This reader
has no fixture deletion authority; only each original live wrapper owns fixtures.
"""
from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PureWindowsPath
import re
import stat
import sys
import time
import xml.etree.ElementTree as ET

VERSION = 1
ROOT = Path("E:/Git/ZirconEngine")
RUN = "engine-s78-fixture-owner-validation-20261018-r1"
ORIGINAL_OUTPUT = Path("E:/cargo-targets/zircon-local/reports") / RUN
OUTPUT = ORIGINAL_OUTPUT / "s86-continuation"
WRAPPER_OUTPUT = ORIGINAL_OUTPUT / "s78-fixture-owner"
COMPONENT_ID = "a976ebe8151834c0ebb2e4f7b30f93a52b662d1c02f14c47de0e434e20bff56f"
COMPONENT = ROOT / ".jenkins/runtime/branch-integration" / COMPONENT_ID
RECIPE = ROOT / "tools/jenkins-management/fixture_owner_recipe.py"
TESTS = ROOT / "tools/jenkins-management/tests/test_fixture_owner_recipe.py"
WRAPPER = ROOT / "tools/mvp/Invoke-MvpTestSuite.ps1"
POWERSHELL = Path("C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe")
PYTHON = ROOT / ".jenkins/runtime/python/python.exe"
MESSAGE = ROOT / ".jenkins/records/engine-audit-20261009-01a12097-s86-source-ready-handoff.md"
MESSAGE_SHA = "986896071f7f30fbef6963e73b5c75a52d84cf5159490f14001dba633be4542e"
REGISTRATION = Path("E:/cargo-targets/zircon-local/reports/jenkins-records-monitor-20261012/heartbeat-20261018T162910/s86-fixture-artifact-registration.json")
REGISTRATION_SHA = "84d18188e980c26c8ce6710d8b1eb2c88a72b68284912bb2703925bee75d9539"
SUITES = {
    "build-editor": "build-editor.Tests.ps1",
    "terminal-receipt": "mvp-staging-terminal-receipt.Tests.ps1",
    "storage-policy": "mvp-artifact-storage-policy.Tests.ps1",
    "staging": "mvp-staging.Tests.ps1",
    "acceptance": "mvp-acceptance.Tests.ps1",
    "snapshot": "mvp-acceptance-staging-snapshot.Tests.ps1",
    "release": "mvp-staging-release.Tests.ps1",
    "fixture-contract": "mvp-test-fixture-paths.Tests.ps1",
}
INVOCATION_IDS = tuple(f"s86-f{i:02d}-{suite}" for i, suite in enumerate(SUITES, 1)) + ("s86-f09-fixture-contract-keep",)
FIRST_SUITE_MODE = "first-suite-only"
FIRST_SUITE_OUTPUT = ORIGINAL_OUTPUT / "s90-first-suite"
FIRST_SUITE_MESSAGE = ROOT / ".jenkins/records/engine-audit-20261009-01a12097-s91-fixture-workspace-source-handoff.md"
FIRST_SUITE_MESSAGE_SHA = "f9c73e91e4b11543d15e90f80953225a158e37d088789abb57e39df5997cd2f5"
FIRST_SUITE_REGISTRATION = Path("E:/cargo-targets/zircon-local/reports/jenkins-records-monitor-20261012/heartbeat-20261018T202044/s91-first-suite-artifact-registration.json")
FIRST_SUITE_IDS = tuple(name.replace("s86-", "s90-", 1) for name in INVOCATION_IDS)
PESTER_SUITES = frozenset(tuple(SUITES)[:3])
FROZEN = {
    "tools/mvp/MvpTestFixtureContext.psm1": ("579cb948d17cd8b3746d034d379ffb7ea52f309b29d91d4daca49efbef9a1d73", 39985, 672, 1792341162827360600),
    "tools/mvp/Invoke-MvpTestSuite.ps1": ("0c30b22048eee7326146c41d1652c73c0e71d6cf562b74680a66859a2fb2dccd", 11690, 165, 1792340195274401400),
    "tools/mvp/MvpTestFixturePaths.psm1": ("d6eec6adbba8774a26d13cc0792702065a6e2736c2e45da84556556e45c15995", 1044, 33, 1792329013891501300),
    "tools/tests/mvp-test-fixture-paths.Tests.ps1": ("fdccbdf56b97da6c74e522aef38ea30c45a722d7fde39dc2edefcaa19507b731", 15469, 232, 1792341162830408000),
    "tools/tests/mvp-artifact-storage-policy.Tests.ps1": ("181d29b5e743ea528bfb53a0aaab9ebb73b4196482d5b924c9d29a03420bc31a", 25215, 436, 1792329013893625500),
    "tools/tests/mvp-acceptance.Tests.ps1": ("433234c87e26ce1b399a1f7f2871bd68c931a5fa4512223b739f85f82716e76b", 188941, 3380, 1792329013896135800),
    "tools/tests/windows-path-resolver.Tests.ps1": ("edc34164cbd3cb06279e279960816792af8b1718da69c737c52dee937c58c01b", 4273, 94, 1792334729809249100),
    "tools/tests/build-editor.Tests.ps1": ("44c3c3bec0df20cf91e438beb33d77473823dda9eedb8fdac69dc3bbec06ad15", 35134, 606, 1792341162833957400),
}
FIRST_SUITE_FROZEN = {**FROZEN,
    "tools/mvp/Invoke-MvpTestSuite.ps1": ("35e3c5603b86d275444cd4d923bff30664bf04b0ee6552ddca098c7248e70b1b", 12325, 179, 1792346125132578300),
    "tools/tests/build-editor.Tests.ps1": ("f2ac6e22dd25c75e69b38e7291aa75c058083092a9d005b792009ddaa91efee7", 35585, 615, 1792352911532851300),
}
IDENTIFIER = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}\Z", re.ASCII)
SHA256 = re.compile(r"[0-9a-f]{64}\Z", re.ASCII)
CONSUMER_INPUTS = (
    "tools/tests/Invoke-MvpAcceptanceTestDriver.ps1",
    ".github/workflows/mvp-editor-windows.yml",
    "tools/build/zircon_build_runtime_manifest.py",
    "tools/build/build_editor.py",
    "tools/build/zircon_build_product_profile.py",
    "tools/build/zircon_build_size_report.py",
    "tools/dev/local_cargo.py",
    "tools/jenkins/contracts/__init__.py",
    "tools/jenkins/frontend.py",
    "tools/jenkins/resources/paths.py",
    "tools/jenkins/validation/receipts.py",
    "tools/export/product_build_profiles.toml",
    "zircon_runtime_interface/src/runtime_build_set/interface_spec_v1.json",
    "zircon_runtime_interface/src/runtime_build_set/payload_schema_set_v1.json",
    "tools/mvp/Invoke-MvpAcceptance.ps1",
    "tools/mvp/MvpBuildSummaryEvidence.psm1",
    "tools/mvp/MvpBuildGateRegistry.psm1",
    "tools/mvp/MvpDatePreservingJson.psm1",
)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def windows_path(value):
    """Validate a literal DOS path without normalizing away traversal or aliases."""
    raw = str(value)
    require(re.match(r"^[A-Za-z]:[\\/]", raw) is not None, "Absolute DOS path required: " + raw)
    parts = re.split(r"[\\/]", raw[3:]) if raw[3:] else []
    require(all(p and p not in (".", "..") and not p.endswith((".", " "))
                and not re.search(r"~[0-9]", p) and ":" not in p and not re.search(r'[\x00-\x1f<>"|?*]', p)
                for p in parts), "Noncanonical path: " + raw)
    return PureWindowsPath(raw)


def same_path(left, right):
    return windows_path(left) == windows_path(right)


def is_under(path, root):
    value, parent = windows_path(path), windows_path(root)
    return value != parent and value.is_relative_to(parent)


def invocation_directory(item):
    require(item.get("invocationId") in (*INVOCATION_IDS, *FIRST_SUITE_IDS), "Exact registered invocation identity required")
    return WRAPPER_OUTPUT / item["invocationId"]


def registered_output(path):
    return any(is_under(path, root) for root in (OUTPUT, FIRST_SUITE_OUTPUT)) or any(
        is_under(path, WRAPPER_OUTPUT / name) for name in (*INVOCATION_IDS, *FIRST_SUITE_IDS))


def first_suite_mode(plan):
    return plan.get("mode") == FIRST_SUITE_MODE


def output_directory(plan):
    return FIRST_SUITE_OUTPUT if first_suite_mode(plan) else OUTPUT


def selected_invocations(plan):
    """The diagnostic dispatch never yields a second invocation, even on success."""
    return iter(plan["invocations"][:1] if first_suite_mode(plan) else plan["invocations"])


def require_fresh_outputs(plan):
    require(not physical(output_directory(plan)).exists(), "Continuation output already exists; reconcile it without replay")
    require(physical(WRAPPER_OUTPUT).is_dir(), "Original wrapper report namespace required")
    for item in plan["invocations"]:
        require(not physical(invocation_directory(item)).exists(), "Invocation output already exists; preserve it without replay")


def remaining_budget(started, maximum, clock=time.monotonic):
    remaining = 180 - (clock() - started)
    if remaining <= 0:
        raise TimeoutError("Overall first-suite deadline exhausted, including preflight")
    return min(maximum, remaining)


def dispatch_after_persistence(run_owned, command, cwd, environment, directory,
                               started, persist_launch, launch_record, clock=time.monotonic):
    """Charge launch-record persistence before calculating the dispatch timeout."""
    persist_launch({**launch_record, "budgetAtRecordSeconds": remaining_budget(started, 180, clock)})
    return run_owned(command, cwd, environment, directory, remaining_budget(started, 180, clock))


def identity(value, name):
    require(isinstance(value, str) and IDENTIFIER.fullmatch(value), "Invalid " + name)
    return value


def pin_index(items):
    require(isinstance(items, list), "Pin list required")
    result = {}
    for item in items:
        require(isinstance(item, dict) and SHA256.fullmatch(item.get("sha256", "")), "SHA256 pin required")
        key = windows_path(item["path"])
        require(key not in result, "Duplicate input pin: " + str(key))
        length = item.get("bytes", item.get("size"))
        require(type(length) is int and length >= 0, "Input byte length required")
        if "bytes" in item and "size" in item:
            require(item["bytes"] == item["size"], "Conflicting byte lengths")
        result[key] = item
    return result


def validate_plan(plan):
    require(isinstance(plan, dict) and plan.get("schemaVersion") == VERSION, "Unsupported plan schema")
    for key in ("taskId", "requestId", "attemptId"):
        require(identity(plan.get(key), key) == RUN, "Original S78 " + key + " required")
    require(plan.get("mode") in (None, FIRST_SUITE_MODE), "Unsupported invocation mode")
    first_only = first_suite_mode(plan)
    require(same_path(plan["output"], output_directory(plan)), "Exact registered output required")
    require(isinstance(plan.get("ownerThreadId"), str) and bool(plan["ownerThreadId"]), "Owner thread required")
    message, message_sha = (FIRST_SUITE_MESSAGE, FIRST_SUITE_MESSAGE_SHA) if first_only else (MESSAGE, MESSAGE_SHA)
    registration_path = FIRST_SUITE_REGISTRATION if first_only else REGISTRATION
    require(same_path(plan["message"]["path"], message) and plan["message"]["sha256"] == message_sha,
            "Exact S83 source-ready handoff required")
    require(same_path(plan["artifactRegistration"]["path"], registration_path)
            and (SHA256.fullmatch(plan["artifactRegistration"].get("sha256", "")) if first_only
                 else plan["artifactRegistration"]["sha256"] == REGISTRATION_SHA), "Exact artifact registration required")
    if first_only:
        require(type(plan.get("overallBudgetSeconds")) is int and plan["overallBudgetSeconds"] == 180,
                "First-suite overall budget must be 180 seconds, including preflight")
        require(type(plan.get("maximumSuiteStarts")) is int and plan["maximumSuiteStarts"] == 1,
                "Unconditional one-start cap required")
        require(plan.get("expectedOriginalCases") == 21, "All original build-editor cases required")
        for name in ("producerRegressionEvidence", "capabilityEvidence"):
            evidence = plan.get(name, {})
            require(is_under(evidence.get("path", ""), "E:/cargo-targets/zircon-local/reports")
                    and SHA256.fullmatch(evidence.get("sha256", "")), "Pinned reusable evidence required: " + name)
    require(same_path(plan["component"]["path"], COMPONENT / "component.json")
            and plan["component"]["sha256"] == COMPONENT_ID, "Original sealed component required")
    require(same_path(plan["powershell"], POWERSHELL), "Native Desktop PowerShell executable required")
    require(is_under(plan["pester"]["path"], "C:/Program Files/WindowsPowerShell/Modules/Pester")
            and windows_path(plan["pester"]["path"]).name.casefold() == "pester.psd1"
            and re.fullmatch(r"[0-9]+(?:\.[0-9]+){1,3}", plan["pester"]["version"]), "Installed Pester identity required")
    runtime = plan["runtime"]
    require(SHA256.fullmatch(runtime.get("driverDigest", "")), "Driver digest required")
    identity(runtime.get("runtimeOperationId"), "runtime operation")
    require(type(plan.get("contractTestsBudgetSeconds")) is int
            and 30 <= plan["contractTestsBudgetSeconds"] <= 180, "Bounded contract-test budget required")
    inputs = pin_index(plan["pinnedInputs"])
    required = {RECIPE, TESTS, PYTHON, POWERSHELL,
                ROOT / "tools/maintenance/WindowsPathResolver.psm1",
                ROOT / "tools/mvp/MvpArtifactStoragePolicy.psm1",
                ROOT / "tools/mvp/mvp-artifact-storage-policy.json",
                ROOT / "tools/tests/MvpTestCleanup.psm1",
                ROOT / "tools/build/build-editor.ps1",
                ROOT / ".jenkins/runtime/drivers" / runtime["driverDigest"] / "driver-manifest.json",
                Path(plan["pester"]["path"]),
                *(ROOT / name for name in CONSUMER_INPUTS),
                *(ROOT / "tools/tests" / name for name in SUITES.values())}
    require({windows_path(x) for x in required} <= inputs.keys(), "Pinned input coverage is incomplete")
    for name, (digest, size, lines, mtime) in (FIRST_SUITE_FROZEN if first_only else FROZEN).items():
        item = inputs.get(windows_path(ROOT / name), {})
        require(item.get("sha256") == digest and item.get("bytes", item.get("size")) == size
                and item.get("lines") == lines and item.get("mtime_ns") == mtime,
                "Frozen source pin differs: " + name)
    controls = pin_index(plan["controlPins"])
    for name in (".jenkins/state/deployment/driver.json", ".jenkins/state/deployment-operation.json",
                 ".jenkins/state/management/installed.json"):
        require(windows_path(ROOT / name) in controls, "Missing selected-runtime control pin: " + name)
    require(any(is_under(item["path"], ROOT / ".jenkins/runtime/management")
                and windows_path(item["path"]).name == "capsule.json" for item in controls.values()),
            "Selected management capsule pin required")
    invocations = plan["invocations"]
    require(isinstance(invocations, list) and len(invocations) == 9, "All nine independent invocations required")
    expected = [(name, False) for name in SUITES] + [("fixture-contract", True)]
    seen_invocations, seen_commands = set(), set()
    for item, (suite, keep), invocation_id in zip(invocations, expected, FIRST_SUITE_IDS if first_only else INVOCATION_IDS):
        for key, seen in (("invocationId", seen_invocations), ("commandId", seen_commands)):
            value = identity(item.get(key), key)
            require(value.casefold() not in seen, "Independent unique " + key + " required")
            seen.add(value.casefold())
        require(item.get("suite") == suite and item.get("keep") is keep, "Original ordered suite/KEEP matrix required")
        require(item["invocationId"] == invocation_id, "Exact ordered S83 invocation identities required")
        require(type(item.get("budgetSeconds")) is int and item["budgetSeconds"] == 180,
                "Original S83 invocation budget must be 180 seconds")
        require(type(item.get("argv")) is list and item["argv"] == [], "Complete original suite requires JSON argv=[]")
    return plan


def native_identity(path):
    """Read physical final path/file ID through a no-follow native handle."""
    require(os.name == "nt", "Windows physical evidence required")
    api = ctypes.WinDLL("kernel32", use_last_error=True)

    class FileInfo(ctypes.Structure):
        _fields_ = [("attributes", wintypes.DWORD), ("created", wintypes.FILETIME),
                    ("accessed", wintypes.FILETIME), ("written", wintypes.FILETIME),
                    ("volume", wintypes.DWORD), ("sizeHigh", wintypes.DWORD),
                    ("sizeLow", wintypes.DWORD), ("links", wintypes.DWORD),
                    ("indexHigh", wintypes.DWORD), ("indexLow", wintypes.DWORD)]

    api.CreateFileW.argtypes = (wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p,
                                wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE)
    api.CreateFileW.restype = wintypes.HANDLE
    api.GetFileInformationByHandle.argtypes = (wintypes.HANDLE, ctypes.POINTER(FileInfo))
    api.GetFileInformationByHandle.restype = wintypes.BOOL
    api.GetFinalPathNameByHandleW.argtypes = (wintypes.HANDLE, wintypes.LPWSTR, wintypes.DWORD, wintypes.DWORD)
    api.GetFinalPathNameByHandleW.restype = wintypes.DWORD
    api.CloseHandle.argtypes = (wintypes.HANDLE,)
    api.CloseHandle.restype = wintypes.BOOL
    handle = api.CreateFileW(str(path), 0x80, 7, None, 3, 0x02200000, None)
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        info = FileInfo()
        if not api.GetFileInformationByHandle(handle, ctypes.byref(info)):
            raise ctypes.WinError(ctypes.get_last_error())
        require(not info.attributes & 0x400, "Reparse input is forbidden: " + str(path))
        buffer = ctypes.create_unicode_buffer(32768)
        count = api.GetFinalPathNameByHandleW(handle, buffer, len(buffer), 0)
        require(0 < count < len(buffer), "Physical final path unavailable")
        final = buffer.value
        if final.startswith("\\\\?\\"):
            final = final[4:]
        require(same_path(final, path), "Physical path alias: " + str(path))
        return {"finalPath": final, "fileIdentity": f"{info.volume:08X}:{info.indexHigh:08X}{info.indexLow:08X}",
                "volumeSerial": info.volume, "fileIndex": (info.indexHigh << 32) | info.indexLow,
                "hardLinkCount": info.links, "attributes": info.attributes, "reparsePoint": False}
    finally:
        if not api.CloseHandle(handle):
            raise ctypes.WinError(ctypes.get_last_error())


def physical(path):
    windows_path(path)
    value = Path(path)
    for entry in reversed((value, *value.parents)):
        if entry.exists() or entry.is_symlink():
            info = entry.lstat()
            require(not stat.S_ISLNK(info.st_mode) and not getattr(info, "st_file_attributes", 0) & 0x400,
                    "Reparse ancestor: " + str(entry))
            native_identity(entry)
    return value


def pin(path):
    value = physical(path)
    with value.open("rb") as stream:
        before = os.fstat(stream.fileno())
        data = stream.read()
        after = os.fstat(stream.fileno())
    keys = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_nlink")
    require(stat.S_ISREG(after.st_mode) and all(getattr(before, k) == getattr(after, k) for k in keys)
            and all(getattr(value.stat(), k) == getattr(after, k) for k in keys), "Input changed while reading: " + str(value))
    installed = windows_path(value).drive.casefold() == "c:"
    require(installed or after.st_nlink == 1, "Non-installed hardlinked input: " + str(value))
    lines = None
    if value.suffix.casefold() in {".py", ".ps1", ".psm1", ".psd1", ".json", ".md", ".toml", ".txt", ".groovy", ".xml", ".csv"}:
        try:
            lines = len(data.decode("utf-16" if data.startswith((b"\xff\xfe", b"\xfe\xff")) else "utf-8-sig").splitlines())
        except UnicodeError:
            pass
    return {"path": str(value), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data),
            "lines": lines, "mtime_ns": after.st_mtime_ns, "fileId": after.st_ino,
            "device": after.st_dev, "hardLinkCount": after.st_nlink,
            "readOnlyInstalledSystemInput": installed, "physical": native_identity(value)}, data


def matches(actual, expected):
    return all(actual.get(k) == expected[k] for k in
               ("sha256", "bytes", "lines", "mtime_ns", "fileId", "device", "hardLinkCount") if k in expected) \
        and ("size" not in expected or actual["bytes"] == expected["size"])


def checked(expected):
    actual, raw = pin(expected["path"])
    require(matches(actual, expected), "Pinned input drift: " + expected["path"])
    return raw


def write(path, value, *, raw=False, replace=False):
    require(registered_output(path), "Unregistered output: " + str(path))
    value_path = physical(path)
    data = value if raw else (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode("utf-8")
    with value_path.open("wb" if replace else "xb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def load(name, path, expected):
    raw = checked({"path": str(path), "sha256": expected})
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    exec(compile(raw, str(path), "exec"), module.__dict__)
    return module


def pester_members(plan):
    """Match the pinned installed module's two actual function-import globs."""
    root = physical(Path(plan["pester"]["path"]).parent)
    members = [root / "Pester.psd1", root / "Pester.psm1"]
    for directory in (root / "Functions", root / "Functions/Assertions"):
        physical(directory)
        members.extend(path for path in directory.glob("*.ps1") if ".tests." not in path.name.casefold())
    return sorted(str(physical(path)) for path in members)


def source_members():
    directory = physical(ROOT / "tools/mvp")
    return sorted(str(path) for path in directory.iterdir() if path.suffix.casefold() in (".ps1", ".psm1", ".json"))


def membership_observation(plan, original):
    try:
        actual = {"mvp": source_members(), "pester": pester_members(plan)}
        return {"before": original, "after": actual, "unchanged": actual == original, "error": None}
    except Exception as error:
        return {"before": original, "after": None, "unchanged": False, "error": str(error)}


def expand_inputs(plan):
    """Include every sealed implementation byte and imported Pester input."""
    items = dict(pin_index(plan["pinnedInputs"] + plan["controlPins"]))
    for key in ("message", "sourceClaim", "artifactRegistration", "component"):
        item = plan[key]
        checked(item)
        items[windows_path(item["path"])] = item
    for optional in ("inputClosurePreparation", "producerRegressionEvidence", "capabilityEvidence"):
        if optional in plan:
            checked(plan[optional])
            items[windows_path(plan[optional]["path"])] = plan[optional]
    component = json.loads(checked(plan["component"]))
    require(component.get("schemaVersion") == 1 and same_path(component["repositoryRoot"], ROOT), "Component repository differs")
    for name, fact in component["files"].items():
        require(name in {"branch_integration.py", "checkout_commit.py", "delivery.py", "local_cargo.py", "run_checks.py"},
                "Unexpected sealed component member")
        item = {"path": str(COMPONENT / name), **fact}
        items[windows_path(item["path"])] = item
    driver = ROOT / ".jenkins/runtime/drivers" / plan["runtime"]["driverDigest"]
    manifest = json.loads(checked(items[windows_path(driver / "driver-manifest.json")]))
    for name, fact in manifest["files"].items():
        require(name and not PureWindowsPath(name).is_absolute() and ".." not in name.split("/")
                and "\\" not in name and ":" not in name, "Unsafe immutable driver member")
        item = {"path": str(driver / name), **fact}
        items.setdefault(windows_path(item["path"]), item)
    for value in pester_members(plan):
        require(windows_path(value) in items, "Imported Pester file is not plan-pinned: " + str(value))
    for value in source_members():
        require(windows_path(value) in items, "MVP consumer source is not plan-pinned: " + str(value))
    return list(items.values()), component["files"]


def observe_inputs(expected, before=None):
    result = []
    for index, item in enumerate(expected):
        try:
            actual, _ = pin(item["path"])
            result.append({"path": item["path"], "actual": actual,
                           "matchesPlan": matches(actual, item),
                           "unchanged": before is None or actual == before[index].get("actual")})
        except Exception as error:
            result.append({"path": item["path"], "actual": None, "matchesPlan": False,
                           "unchanged": False, "error": str(error)})
    return result


def devices():
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.QueryDosDeviceW.argtypes = (wintypes.LPCWSTR, wintypes.LPWSTR, wintypes.DWORD)
    api.QueryDosDeviceW.restype = wintypes.DWORD
    result = {}
    for letter in "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
        buffer = ctypes.create_unicode_buffer(32768)
        count = api.QueryDosDeviceW(letter + ":", buffer, len(buffer))
        if count:
            result[letter + ":"] = buffer[:count].split("\x00")
    return result


def output_encoding(raw, acp):
    encodings = []
    for name in dict.fromkeys(("cp" + str(acp), "utf-8", "utf-16-le")):
        try:
            if raw.decode(name).encode(name) == raw:
                encodings.append(name)
        except (UnicodeError, LookupError):
            pass
    return {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(),
            "strictRoundtripEncodings": encodings, "selected": None,
            "systemAnsiCodePage": acp, "rawBytesPreserved": True}


def xml_result(raw):
    result = {"present": raw is not None, "attributes": None, "cases": [], "parseError": None}
    if raw is None:
        return result
    try:
        require(b"<!DOCTYPE" not in raw.upper() and b"<!ENTITY" not in raw.upper(), "XML declarations are forbidden")
        root = ET.fromstring(raw)
        result["attributes"] = dict(root.attrib)
        result["sha256"] = hashlib.sha256(raw).hexdigest()
        result["cases"] = [{"name": node.get("name"), "description": node.get("description"),
                            "executed": node.get("executed"), "result": node.get("result"),
                            "success": node.get("success"), "message": node.findtext("failure/message"),
                            "stackTrace": node.findtext("failure/stack-trace")} for node in root.iter("test-case")]
    except Exception as error:
        result["parseError"] = str(error)
    return result


def read_journal(raw):
    result = {"present": raw is not None, "events": [], "errors": []}
    if raw is None:
        return result
    for number, line in enumerate(raw.splitlines(), 1):
        try:
            item = json.loads(line.decode("utf-8-sig"))
            require(isinstance(item, dict) and item.get("schemaVersion") == 1, "Unsupported journal record")
            result["events"].append(item)
        except Exception as error:
            result["errors"].append({"line": number, "error": str(error)})
    return result


def interpret_journal(journal, item, native, suite_sha, xml):
    """Interpret original records without fabricating handle or compile evidence."""
    events = []
    errors = list(journal["errors"])
    for index, event in enumerate(journal["events"]):
        valid = (isinstance(event.get("ownerId"), str) and isinstance(event.get("kind"), str)
                 and isinstance(event.get("state"), str)
                 and (event.get("kind") != "fixture" or isinstance(event.get("instanceId"), str)))
        if valid:
            events.append(event)
        else:
            errors.append({"record": index + 1, "error": "Malformed original journal identity; raw record retained"})
    owners = [e for e in events if e.get("kind") == "owner" and e.get("state") == "Active"]
    roots = [e for e in owners if e.get("testOwner") is False]
    main = roots[0] if len(roots) == 1 else None
    expected = {"taskId": RUN, "requestId": RUN, "attemptId": RUN, "commandId": item["commandId"],
                "invocationId": item["invocationId"], "suite": item["suite"]}
    if main is None:
        errors.append({"error": "Exactly one original wrapper owner activation is required"})
    for owner in owners:
        try:
            metadata = owner.get("metadata", {})
            valid = all(metadata.get(k) == v for k, v in expected.items())
            valid = valid and metadata.get("driverId") == item["driverId"] and metadata.get("component") == COMPONENT_ID
            valid = valid and metadata.get("suiteArgv") == [] and metadata.get("suiteSha256") == suite_sha
            valid = valid and same_path(metadata.get("suitePath", "C:/invalid"), ROOT / "tools/tests" / SUITES[item["suite"]])
            valid = valid and owner.get("processId") == native.get("pid") and owner.get("processStartFileTimeUtc") == native.get("creationTime")
            valid = valid and owner.get("keep") is item["keep"] and re.fullmatch(r"[0-9a-f]{32}", owner.get("ownerId", "")) is not None
        except (ValueError, TypeError, AttributeError):
            valid = False
        if not valid:
            errors.append({"error": "Journal owner identity differs from native command", "ownerId": owner.get("ownerId")})
    owner_id = main.get("ownerId") if main else None
    suite_events = [e for e in events if e.get("kind") == "suite" and e.get("ownerId") == owner_id]
    states = [e.get("state") for e in suite_events]
    suite_passed = states == ["Started", "Passed"]
    finals, histories = {}, {}
    historical_failures = []
    for event in events:
        if event.get("state") in ("Failed", "CleanupFailed"):
            historical_failures.append(event)
        if event.get("kind") == "fixture":
            key = (event.get("ownerId"), event.get("instanceId"))
            finals[key] = event
            histories.setdefault(key, []).append(event)
    owner_ids = {owner.get("ownerId") for owner in owners}
    for key, history in histories.items():
        if (key[0] not in owner_ids or not isinstance(key[1], str)
                or re.fullmatch(r"[0-9a-f]{32}", key[1]) is None
                or history[0].get("state") != "Reserved"
                or sum(e.get("state") == "Reserved" for e in history) != 1):
            errors.append({"error": "Fixture lifecycle lacks its original owner/reservation", "instanceId": key[1]})
            continue
        active = [e for e in history if e.get("state") == "Active"]
        if len(active) > 1 or any(e.get("path") != history[0].get("path") for e in history):
            errors.append({"error": "Fixture lifecycle changed its instance path/activation", "instanceId": key[1]})
        if active:
            original_identity = active[0].get("fileIdentity", "")
            if (not isinstance(original_identity, str) or re.fullmatch(r"[0-9A-F]{8}:[0-9A-F]{16}", original_identity) is None
                    or any(e.get("fileIdentity") != original_identity for e in history
                           if e.get("state") in ("Released", "Retained"))):
                errors.append({"error": "Fixture file identity changed", "instanceId": key[1]})
    terminals = []
    for owner in owners:
        terminal = [e for e in events if e.get("kind") == "owner" and e.get("ownerId") == owner.get("ownerId")
                    and e.get("state") in ("Released", "Retained", "CleanupFailed")]
        terminals.append(terminal[-1] if terminal else {"ownerId": owner.get("ownerId"), "state": "Nonterminal"})
    lifecycle_terminal = bool(owners) and all(e.get("state") in ("Released", "Retained") for e in terminals)
    pester = [e for e in events if e.get("kind") == "pester" and e.get("state") == "Completed" and e.get("ownerId") == owner_id]
    counts = pester[-1] if pester else None
    counts_ok = item["suite"] not in PESTER_SUITES
    if not counts_ok and len(pester) == 1 and xml["present"] and not xml["parseError"]:
        keys = ("total", "passed", "failed", "skipped", "pending", "xmlCases", "xmlFailures")
        counts_ok = all(type(counts.get(k)) is int and counts[k] >= 0 for k in keys)
        if counts_ok:
            failures = sum(str(c.get("success") or "").casefold() == "false" or c.get("result") in ("Failure", "Error") for c in xml["cases"])
            counts_ok = (counts["total"] > 0 and counts["passed"] > 0 and counts["total"] == len(xml["cases"]) == counts["xmlCases"]
                         and counts["failed"] == failures == counts["xmlFailures"] == 0
                         and sum(counts[k] for k in ("passed", "failed", "skipped", "pending")) == counts["total"]
                         and counts.get("resultSha256") == xml.get("sha256"))
    return {"present": journal["present"], "eventCount": len(events), "ownerActivation": main,
            "errors": errors, "suiteStates": states, "suitePassed": suite_passed,
            "owners": owners, "ownerTerminals": terminals, "lifecycleTerminal": lifecycle_terminal,
            "instanceFinals": list(finals.values()), "historicalFailures": historical_failures,
            "pesterCounts": counts, "pesterCountsAgree": bool(counts_ok),
            "compileStatus": "confirmed-by-original-owner-activation" if main and not errors else "unknown",
            "directCompilerReceipt": False, "liveHandleDisposalObservedByProducer": False,
            "fixtureDeletionAuthority": False}


def fixture_satisfied(event, keep, exists, same_original_instance, is_directory):
    """KEEP needs the actual Retained instance, observed after native closure."""
    if keep:
        return bool(event.get("state") == "Retained" and event.get("reason") == "KEEP"
                    and exists and is_directory and same_original_instance)
    return bool(event.get("state") == "Released" and event.get("physicalAbsent") is True and not exists)


def native_closed(native):
    """Extra close evidence, beyond the sealed normal-completion verifier."""
    tree = native.get("processTree", {})
    return bool(tree.get("nativeWaiterJoined") is True and tree.get("readersFinished") is True
                and tree.get("pipeEOF") is True and tree.get("jobHandleRetainedUntilCliExit") is False
                and all(tree.get("pipes", {}).get(n, {}).get("eof") is True
                        and tree.get("pipes", {}).get(n, {}).get("closed") is True for n in ("stdout", "stderr"))
                and native.get("nativeJobEvidence", {}).get("records", {}).get("afterClose", {}).get("activeProcesses") == 0)


def native_stream_observation(directory, command, cwd, checks, acp, returned_native=None):
    """Read full available bytes; only this process's own joined EOF proves completeness."""
    result = {"nativeComplete": False, "identityMatchesStart": False, "streams": {}, "errors": []}

    def optional(name):
        path = directory / name
        try:
            if not physical(path).exists():
                return None, None
            return pin(path)
        except Exception as error:
            result["errors"].append({"path": str(path), "type": type(error).__name__, "message": str(error)})
            return None, None

    terminal_pin, terminal_raw = optional("native-terminal.json")
    start_pin, start_raw = optional("started.json")
    result.update(terminalEvidence=terminal_pin, startEvidence=start_pin)
    try:
        require(terminal_raw is not None and start_raw is not None, "Own terminal and start evidence required")
        native, start = json.loads(terminal_raw), json.loads(start_raw)
        identity_ok = all(native.get(key) == start.get(key) for key in ("pid", "creationTime", "command", "cwd"))
        identity_ok = bool(identity_ok and native.get("command") == command and native.get("cwd") == str(cwd)
                           and (returned_native is None or returned_native == native))
        result.update(identityMatchesStart=identity_ok, nativePid=native.get("pid"),
                      nativeBirth=native.get("creationTime"), nativeExitCode=native.get("rootExitCode"))
        result["nativeComplete"] = bool(identity_ok and checks.native_complete(native, command, cwd) and native_closed(native))
    except Exception as error:
        result["errors"].append({"type": type(error).__name__, "message": str(error)})
    for channel in ("stdout", "stderr"):
        stream_pin, raw = optional(channel + ".bin")
        complete = bool(raw is not None and result["nativeComplete"])
        item = {"present": raw is not None, "path": str(directory / (channel + ".bin")),
                "completeBytesRead": complete, "completeness": "complete" if complete else "partial-or-unknown" if raw is not None else "missing",
                "truncated": False if complete else None, "originalEncodingProved": False}
        if raw is not None:
            encoding = output_encoding(raw, acp)
            item.update(pin=stream_pin, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest(),
                        encodingObservation=encoding,
                        candidateDecodedFullText={name: raw.decode(name) for name in encoding["strictRoundtripEncodings"]})
        result["streams"][channel] = item
    result["bothStreamsComplete"] = all(item["completeBytesRead"] for item in result["streams"].values())
    return result


def invocation_status(native, native_complete, journal, physical_ok):
    # A genuine nonzero exit stays failed even when suite Passed preceded a
    # finally/Dispose failure. A timeout or missing native proof stays pending.
    if not native_complete:
        return "pending"
    if native.get("rootExitCode") != 0:
        return "failed"
    if journal["errors"] or not journal["suitePassed"] or not journal["pesterCountsAgree"]:
        return "failed"
    if not journal["lifecycleTerminal"] or not physical_ok:
        return "pending"
    return "passed"


def observe_fixtures(journal, keep):
    results = []
    for event in journal["instanceFinals"]:
        row = {"ownerId": event.get("ownerId"), "instanceId": event.get("instanceId"),
               "path": event.get("path"), "journalState": event.get("state"),
               "journalFileIdentity": event.get("fileIdentity"), "deletionAuthorized": False}
        try:
            require(re.fullmatch(r"E:\\cargo-targets\\zircon-local\\mvp-test-fixtures-[1-9][0-9]*(?:-[0-9a-f]{32})?\\[A-Za-z0-9][A-Za-z0-9._-]*-[0-9a-f]{32}",
                                 str(windows_path(event["path"])), re.ASCII) is not None, "Unregistered fixture namespace")
            path = physical(event["path"])
            row["existsAfterNativeTermination"] = path.exists()
            row["physical"] = native_identity(path) if path.exists() else None
            row["sameOriginalInstance"] = bool(row["physical"] and event.get("fileIdentity")
                                                == row["physical"]["fileIdentity"])
            row["satisfied"] = fixture_satisfied(event, keep, path.exists(), row["sameOriginalInstance"], path.is_dir())
        except Exception as error:
            row.update(satisfied=False, error=str(error))
        results.append(row)
    parents = []
    for owner, terminal in zip(journal["owners"], journal["ownerTerminals"]):
        row = {"path": owner.get("parentPath"), "ownerId": owner.get("ownerId"), "deletionAuthorized": False}
        try:
            path = physical(owner["parentPath"])
            require(is_under(path, "E:/cargo-targets") and re.fullmatch(r"mvp-test-fixtures-[1-9][0-9]*(?:-[0-9a-f]{32})?", path.name),
                    "Unregistered fixture parent")
            row["existsAfterNativeTermination"] = path.exists()
            row["physical"] = native_identity(path) if path.exists() else None
            row["satisfied"] = bool(terminal.get("state") == "Retained" and path.is_dir()) if keep else (
                terminal.get("state") == "Released" and not path.exists())
        except Exception as error:
            row.update(satisfied=False, error=str(error))
        parents.append(row)
    ok = bool(journal["owners"]) and all(r["satisfied"] for r in results + parents)
    if keep:
        ok = ok and bool(results) and any(r.get("sameOriginalInstance") for r in results)
    return {"instances": results, "parents": parents, "satisfied": bool(ok),
            "scope": "Exact original journal paths after native process termination; no recursive fixture cleanup"}


def wrapper_command(plan, item):
    directory = invocation_directory(item)
    command = [plan["powershell"], "-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
               "-File", str(WRAPPER), "-TaskId", RUN, "-RequestId", RUN, "-AttemptId", RUN,
               "-DriverId", plan["runtime"]["driverDigest"], "-Component", COMPONENT_ID,
               "-CommandId", item["commandId"], "-InvocationId", item["invocationId"], "-Suite", item["suite"],
               "-JournalPath", str(directory / ("fixture-owner-" + item["invocationId"] + ".jsonl")),
               "-SuiteArgumentsJsonPath", str(directory / "argv.json")]
    if item["suite"] in PESTER_SUITES:
        command.extend(("-PesterResultPath", str(directory / ("fixture-owner-" + item["invocationId"] + ".pester.xml"))))
    return command


PROBE_SCRIPT = r"""$ErrorActionPreference='Stop'; Import-Module Pester -ErrorAction Stop; $p=Get-Command Invoke-Pester -CommandType Function -ErrorAction Stop; [ordered]@{kind='fixture-owner-environment'; powershell=[string]$PSVersionTable.PSVersion; edition=[string]$PSVersionTable.PSEdition; pesterVersion=[string]$p.Module.Version; pesterPath=[IO.Path]::Combine($p.Module.ModuleBase,'Pester.psd1'); pesterModulePath=[string]$p.Module.Path; processId=$PID; birthFileTimeUtc=[string]([Diagnostics.Process]::GetCurrentProcess().StartTime.ToUniversalTime().ToFileTimeUtc()); temp=$env:TEMP; tmp=$env:TMP; physicalTemp=[IO.Path]::GetTempPath(); testDrivePresent=[bool](Get-PSDrive -Name TestDrive -ErrorAction SilentlyContinue)} | ConvertTo-Json -Compress"""


def prepare_step(directory, delivery, runtime, keep=False):
    physical(directory).mkdir()
    (directory / "temp").mkdir()
    environment = delivery.environment(directory, runtime)
    environment.update(RUN_ID=RUN, JOB_NAME=os.environ["JOB_NAME"], NODE_NAME=os.environ["NODE_NAME"],
                       ZIRCON_KEEP_MVP_ACCEPTANCE_FIXTURE="1" if keep else "0",
                       PYTHONPATH=os.pathsep.join((str(ROOT), str(ROOT / ".jenkins/runtime/test-deps"))))
    facts = {}
    for name in ("TEMP", "TMP", "TMPDIR"):
        require(same_path(environment[name], directory / "temp") and is_under(environment[name], "E:/cargo-targets"),
                "Compiler temporary path differs from registration")
        facts[name] = native_identity(physical(environment[name]))
    write(directory / "environment-before.json", {"temporaryPaths": facts, "keep": keep,
          "pythonPath": environment["PYTHONPATH"], "canonicalPythonSourceRoot": str(ROOT),
          "selectedRuntime": runtime,
          "dosDevices": devices(), "proofRecordedBeforeProcessLaunch": True,
          "testDrive": {"scope": "Fresh original wrapper process; no producer PSDrive observation"}})
    return environment


def run_native(cargo, checks, command, directory, environment, budget):
    native = cargo._run_owned_command(command, ROOT, environment, directory, budget)
    write(directory / "native-terminal.json", native)
    complete = bool(checks.native_complete(native, command, ROOT) and native_closed(native))
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    api.GetACP.restype = wintypes.UINT
    encodings = {}
    for name in ("stdout", "stderr"):
        path = directory / (name + ".bin")
        encodings[name] = output_encoding(path.read_bytes(), int(api.GetACP())) if path.exists() else {"present": False}
    write(directory / "raw-encoding.json", encodings)
    return native, complete, encodings


def first_failure(directory, journal, native):
    original = [e for e in journal.get("historicalFailures", []) if e.get("kind") == "suite" and e.get("state") == "Failed"]
    if original:
        return {"source": "original-wrapper-suite-journal", "event": original[0]}
    raw = (directory / "stderr.bin").read_bytes() if (directory / "stderr.bin").exists() else b""
    return {"source": "original-native-stderr" if raw else "native-and-journal-evidence",
            "stderrPath": str(directory / "stderr.bin"), "stderrSha256": hashlib.sha256(raw).hexdigest(),
            "nativeExitCode": native.get("rootExitCode"), "nativeErrors": native.get("terminalErrors", []),
            "journalErrors": journal.get("errors", []),
            "note": "Raw original bytes retained; no synthetic PowerShell error or expected-failure pass"}


def registration(plan):
    output = output_directory(plan)
    steps = [] if first_suite_mode(plan) else [output / "producer-contract-tests", output / "environment-probe"]
    wrapper_steps = [invocation_directory(item) for item in plan["invocations"]]
    steps += wrapper_steps
    return {"schema_version": 1, "recipeVersion": VERSION, "task_id": RUN, "owner": plan["ownerThreadId"],
            "status": "active", "source_root": str(ROOT), "output_roots": {"reports": str(output),
                **{item["invocationId"]: str(path) for item, path in zip(plan["invocations"], wrapper_steps)}},
            "priorArtifactRegistration": plan["artifactRegistration"], "dependencies": plan["pinnedInputs"],
            "steps": [{"report": str(path), "TEMP": str(path / "temp"), "TMP": str(path / "temp")} for path in steps],
            "fixtureNamespaceRegistration": {"physicalRoot": "E:/cargo-targets",
                "parentNamePattern": "mvp-test-fixtures-<original-native-PS-PID>[-<test-owner-token>]",
                "binding": "Actual original wrapper journal PID+FILETIME/owner/instance/fileIdentity",
                "keepInvocationId": None if first_suite_mode(plan) else plan["invocations"][-1]["invocationId"],
                "retention": "Preserve actual KEEP Retained roots after process closure; no producer fixture deletion",
                "nonterminal": "Preserve timeout/unknown/nonterminal/CleanupFailed roots for original owner review",
                "metadataGrantsDeletionAuthority": False},
            "processes": [], "retained": [{"path": str(path), "kind": "original-validation-evidence",
                "reason": "Native failure/suite/KEEP evidence and immutable input pins", "depends_on": [RUN], "review_after": None}
                for path in [output, *wrapper_steps]],
            "unknown": [], "cleanup": {"deleted_files": 0, "deleted_bytes": 0, "receipt": None}}


def run(plan_path, plan_sha, self_sha):
    started = time.monotonic()  # Includes self/plan/input/runtime preflight, not only the suite.
    require(os.name == "nt", "Jenkins Windows execution only")
    require(SHA256.fullmatch(self_sha) and SHA256.fullmatch(plan_sha), "Entry and plan SHA256 required")
    checked({"path": __file__, "sha256": self_sha})
    plan = validate_plan(json.loads(checked({"path": str(plan_path), "sha256": plan_sha})))
    first_only = first_suite_mode(plan)
    output = output_directory(plan)
    require(os.environ.get("JOB_NAME") == "zircon-management-tests" and os.environ.get("NODE_NAME") == "zircon-windows-agent"
            and os.environ.get("RUN_ID") == RUN, "Exact Jenkins job/node/original RUN_ID required")
    require(str(os.environ.get("BUILD_NUMBER", "")).isdigit(), "Actual Jenkins build number required")
    expected, component_files = expand_inputs(plan)
    original_membership = {"mvp": source_members(), "pester": pester_members(plan)}
    require(checked(next(p for p in expected if same_path(p["path"], RECIPE))) == checked({"path": __file__, "sha256": self_sha}),
            "Running recipe differs from the canonical pinned source")
    before = observe_inputs(expected)
    require(all(row["matchesPlan"] for row in before), "Before-run input pins differ")
    driver = physical(ROOT / ".jenkins/runtime/drivers" / plan["runtime"]["driverDigest"])
    sys.path.insert(0, str(driver))
    cargo = load("s78_fixture_native_cargo", COMPONENT / "local_cargo.py", component_files["local_cargo.py"]["sha256"])
    checks = load("s78_fixture_native_checks", COMPONENT / "run_checks.py", component_files["run_checks.py"]["sha256"])
    delivery = load("s78_fixture_native_environment", COMPONENT / "delivery.py", component_files["delivery.py"]["sha256"])
    _, runtime, _ = delivery.runtime_files(ROOT)
    require(all(runtime.get(k) == v for k, v in plan["runtime"].items()), "Selected runtime changed")
    require_fresh_outputs(plan)
    physical(output.parent)
    output.mkdir()
    manifest = registration(plan)
    write(output / "artifacts.json", manifest)  # First file, before any generated input or compiler.
    write(output / "plan.json", plan)
    write(output / "input-pins-before.json", {"inputs": before})
    summary = {"schemaVersion": 1, "recipeVersion": VERSION, "taskId": RUN, "requestId": RUN, "attemptId": RUN,
               "output": str(output), "artifactRegistration": plan["artifactRegistration"],
               "job": os.environ["JOB_NAME"], "build": int(os.environ["BUILD_NUMBER"]),
               "queueId": os.environ.get("QUEUE_ID"), "buildUrl": os.environ.get("BUILD_URL"),
               "runtime": plan["runtime"], "component": plan["component"], "message": plan["message"],
               "cwd": str(ROOT), "producerContractTests": {"status": "not-run"}, "environmentProbe": {"status": "not-run"},
               "invocations": [{**item, "status": "not-run", "argv": wrapper_command(plan, item), "suiteArgv": item["argv"],
                                "cwd": str(ROOT), "reportDirectory": str(invocation_directory(item)),
                                "nativeComplete": False, "nativeExitCode": None, "compileStatus": "not-started"}
                               for item in plan["invocations"]], "firstActionableFailure": None,
               "nineInvocationsComplete": False, "allRequirementsComplete": False, "formalAcceptance": False,
               "sourceCopies": 0, "sourceEdits": 0, "filesDeleted": 0, "releasedBytes": 0}
    temp_steps = []
    before_devices = devices()
    stopped = False
    active_step = "producer-contract-tests"
    try:
        if first_only:
            proof = json.loads(checked(plan["producerRegressionEvidence"]))
            require(proof.get("status") == "passed" and proof.get("nativeComplete") is True,
                    "Current producer regressions require actual native acceptance")
            require(proof.get("sourcePins") == {
                str(RECIPE): next(p["sha256"] for p in expected if same_path(p["path"], RECIPE)),
                str(TESTS): next(p["sha256"] for p in expected if same_path(p["path"], TESTS))},
                    "Reusable producer regression source differs")
            checked(plan["capabilityEvidence"])
            summary["producerContractTests"] = {"status": "reused", "proof": plan["producerRegressionEvidence"]}
            summary["environmentProbe"] = {"status": "reused", "proof": plan["capabilityEvidence"],
                "scope": "Reviewed build-244 capability evidence; no extra capability process"}
        else:
            unit = output / "producer-contract-tests"
            environment = prepare_step(unit, delivery, runtime)
            unit_command = [str(PYTHON), "-B", "-X", "utf8", "-m", "unittest", "discover", "-s",
                            "tools/jenkins-management/tests", "-p", "test_fixture_owner_recipe.py", "-v"]
            summary["producerContractTests"].update(status="pending", launchAttempted=True, argv=unit_command)
            temp_steps.append((unit, {}, False))
            native, complete, encoding = run_native(cargo, checks, unit_command, unit, environment, plan["contractTestsBudgetSeconds"])
            temp_steps[-1] = (unit, native, complete)
            passed = complete and native["rootExitCode"] == 0
            summary["producerContractTests"] = {"status": "passed" if passed else "failed" if complete else "pending",
                "argv": unit_command, "cwd": str(ROOT), "budgetSeconds": plan["contractTestsBudgetSeconds"],
                "nativeComplete": complete, "native": native, "encoding": encoding}
            if not passed:
                stopped = True
                summary["firstActionableFailure"] = {"step": "producer-contract-tests", **first_failure(unit, {}, native)}
            if not stopped:
                active_step = "environment-probe"
                probe = output / "environment-probe"
                environment = prepare_step(probe, delivery, runtime)
                command = [plan["powershell"], "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", PROBE_SCRIPT]
                summary["environmentProbe"].update(status="pending", launchAttempted=True, argv=command)
                temp_steps.append((probe, {}, False))
                native, complete, encoding = run_native(cargo, checks, command, probe, environment, 30)
                temp_steps[-1] = (probe, native, complete)
                metadata = None
                try:
                    raw = (probe / "stdout.bin").read_bytes()
                    rows = [json.loads(line) for line in raw.decode("utf-8-sig").splitlines() if line.startswith('{"kind":"fixture-owner-environment"')]
                    require(len(rows) == 1, "One original PowerShell metadata record required")
                    metadata = rows[0]
                    require(metadata["edition"] == "Desktop" and metadata["powershell"].startswith("5.1.")
                            and metadata["pesterVersion"] == plan["pester"]["version"]
                            and same_path(metadata["pesterPath"], plan["pester"]["path"])
                            and metadata["processId"] == native["pid"] and metadata["birthFileTimeUtc"] == native["creationTime"]
                            and same_path(metadata["temp"], probe / "temp") and same_path(metadata["tmp"], probe / "temp"),
                            "Actual PS/Pester/temporary identity differs from plan")
                    metadata_error = None
                except Exception as error:
                    metadata_error = str(error)
                passed = complete and native["rootExitCode"] == 0 and metadata_error is None
                summary["environmentProbe"] = {"status": "passed" if passed else "failed" if complete else "pending",
                    "argv": command, "cwd": str(ROOT), "budgetSeconds": 30, "nativeComplete": complete,
                    "native": native, "metadata": metadata, "error": metadata_error, "encoding": encoding,
                    "scope": "Separate fresh metadata process; no original suite or TestDrive result inferred"}
                if not passed:
                    stopped = True
                    summary["firstActionableFailure"] = {"step": "environment-probe", **first_failure(probe, {}, native), "metadataError": metadata_error}
        if first_only:
            for unselected in summary["invocations"][1:]:
                unselected["notRunReason"] = "First-suite-only mode: unconditional one-start cap, regardless of first result"
        for item in selected_invocations(plan):
            row = next(row for row in summary["invocations"] if row["invocationId"] == item["invocationId"])
            if stopped:
                row["notRunReason"] = "Stopped at earliest actionable failure; no unchanged-input replay"
                continue
            directory = invocation_directory(item)
            active_step = item["invocationId"]
            environment = prepare_step(directory, delivery, runtime, item["keep"])
            write(directory / "argv.json", item["argv"])
            command = wrapper_command(plan, item)
            for key in ("-JournalPath", "-PesterResultPath"):
                if key in command:
                    require(not physical(command[command.index(key) + 1]).exists(), "Original output must be absent")
            row.update(status="pending", launchAttempted=True, compileStatus="unknown", cleanupPending=True)
            temp_steps.append((directory, {}, False))
            budget = remaining_budget(started, item["budgetSeconds"]) if first_only else item["budgetSeconds"]
            native, complete, encoding = run_native(cargo, checks, command, directory, environment, budget)
            temp_steps[-1] = (directory, native, complete)
            row.update(status="failed" if complete and native.get("rootExitCode") != 0 else "pending",
                       nativeComplete=complete, nativeExitCode=native.get("rootExitCode"), native=native, encoding=encoding)
            journal_path = directory / ("fixture-owner-" + item["invocationId"] + ".jsonl")
            xml_path = directory / ("fixture-owner-" + item["invocationId"] + ".pester.xml")
            xml = xml_result(xml_path.read_bytes() if xml_path.exists() else None)
            original = read_journal(journal_path.read_bytes() if journal_path.exists() else None)
            suite_sha = next(p["sha256"] for p in expected if same_path(p["path"], ROOT / "tools/tests" / SUITES[item["suite"]]))
            journal = interpret_journal(original, {**item, "driverId": plan["runtime"]["driverDigest"]}, native, suite_sha, xml)
            fixtures = observe_fixtures(journal, item["keep"]) if complete else {"satisfied": False,
                "instances": [], "parents": [], "status": "pending-native-termination; no fixture cleanup attempted"}
            status = invocation_status(native, complete, journal, fixtures["satisfied"])
            after_each = observe_inputs(expected, before)
            unchanged = all(r["matchesPlan"] and r["unchanged"] for r in after_each)
            member_check = membership_observation(plan, original_membership)
            unchanged = unchanged and member_check["unchanged"]
            write(directory / "input-pins-after.json", {"inputs": after_each, "unchanged": unchanged})
            if not unchanged and status == "passed":
                status = "failed"
            row.update(status=status, nativeComplete=complete, nativeExitCode=native.get("rootExitCode"), native=native,
                       journal=journal, originalJournalPath=str(journal_path), xml=xml, originalXmlPath=str(xml_path),
                       fixtures=fixtures, compileStatus=journal["compileStatus"], pester=journal["pesterCounts"],
                       standaloneResult=journal["suiteStates"] if item["suite"] not in PESTER_SUITES else None,
                       encoding=encoding, inputPinsUnchanged=unchanged, keepEnvironment="1" if item["keep"] else "0",
                       sourceAndPesterMembership=member_check,
                       cleanupPending=not journal["lifecycleTerminal"] or not fixtures["satisfied"],
                       testDrive={"afterProcess": "PowerShell PSDrives do not survive their original process",
                                  "inProcessTeardownObservedByProducer": False})
            write(directory / "journal-observation.json", {"original": original, "interpretation": journal, "physical": fixtures})
            if status != "passed":
                stopped = True
                row["failure"] = first_failure(directory, journal, native)
                summary["firstActionableFailure"] = {"step": item["invocationId"], **row["failure"]}
    except Exception as error:
        summary["producerError"] = {"type": type(error).__name__, "message": str(error)}
        if summary["firstActionableFailure"] is None:
            summary["firstActionableFailure"] = {"step": active_step, **summary["producerError"]}
        for row in summary["invocations"]:
            if row["status"] == "not-run":
                row["notRunReason"] = "Producer stopped; original evidence retained; reconcile before any retry"
    finally:
        after = observe_inputs(expected, before)
        unchanged = all(r["matchesPlan"] and r["unchanged"] for r in after)
        member_check = membership_observation(plan, original_membership)
        unchanged = unchanged and member_check["unchanged"]
        write(output / "input-pins-after.json", {"inputs": after, "unchanged": unchanged})
        summary["inputPins"] = {"before": str(output / "input-pins-before.json"),
                                "after": str(output / "input-pins-after.json"), "unchanged": unchanged}
        summary["sourceAndPesterMembership"] = member_check
        cleanup = {"releasedBytes": 0, "filesDeleted": 0, "emptyTempDirectoriesRemoved": [],
                   "retained": [], "unknown": [], "fixtureDeletionAttempted": False}
        for directory, native, complete in temp_steps:
            path = directory / "temp"
            try:
                settled = complete or bool(native.get("cleanup", {}).get("complete") is True and native_closed(native))
                actual_temp = native_identity(physical(path))
                original_temp = json.loads((directory / "environment-before.json").read_bytes())["temporaryPaths"]["TEMP"]
                require(actual_temp["fileIdentity"] == original_temp["fileIdentity"], "Owned TEMP was replaced; preserve")
                empty = not any(path.iterdir())
                if settled and empty:
                    path.rmdir()  # Only this recipe's exclusively created, registered, empty TEMP.
                    cleanup["emptyTempDirectoriesRemoved"].append(str(path))
                else:
                    cleanup["retained"].append({"path": str(path), "reason": "Native unsettled or owned TEMP is nonempty"})
                    if not settled:
                        cleanup["unknown"].append({"path": str(path), "reason": "Original native process termination is unproven"})
            except Exception as error:
                cleanup["unknown"].append({"path": str(path), "reason": str(error)})
        for row in summary["invocations"]:
            for observation in row.get("fixtures", {}).get("instances", []):
                if observation.get("existsAfterNativeTermination") or not observation.get("satisfied"):
                    cleanup["retained"].append(observation)
            if row.get("cleanupPending"):
                cleanup["unknown"].append({"invocationId": row["invocationId"], "reason": "Original wrapper fixture lifecycle needs owner review"})
        after_devices = devices()
        summary["dosDevices"] = {"before": before_devices, "after": after_devices, "unchanged": before_devices == after_devices,
                                 "foreignMappingRemovedByProducer": False}
        summary["cleanup"] = cleanup
        summary["nineInvocationsComplete"] = all(row["status"] == "passed" for row in summary["invocations"])
        summary["firstSuiteOnly"] = first_only
        summary["suiteStarts"] = sum(bool(row.get("launchAttempted")) for row in summary["invocations"])
        summary["overallExecutionBudgetSeconds"] = 180 if first_only else None
        summary["elapsedObservedSecondsIncludingPreflight"] = time.monotonic() - started
        producer_ok = summary.get("producerError") is None
        summary["firstSuiteMeasurementPassed"] = bool(producer_ok and first_only and unchanged and summary["suiteStarts"] == 1
            and summary["invocations"][0]["status"] == "passed"
            and summary["invocations"][0].get("pester", {}).get("total") == 21
            and all(row["status"] == "not-run" for row in summary["invocations"][1:]))
        summary["status"] = ("first-suite-passed-eight-not-run-acceptance-open" if summary["firstSuiteMeasurementPassed"]
            else "nine-invocations-passed-acceptance-open" if producer_ok and summary["nineInvocationsComplete"] and unchanged
            else "stopped-original-evidence-retained")
        summary["completedAtUtc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        write(output / "cleanup.json", cleanup)
        manifest.update(status="needs-review" if cleanup["unknown"] else "active",
                        processes=[{"pid": n.get("pid"), "creationTime": n.get("creationTime"),
                                    "report": str(d), "nativeComplete": c, "rootExitCode": n.get("rootExitCode")}
                                   for d, n, c in temp_steps],
                        unknown=cleanup["unknown"], cleanup={"receipt": str(output / "cleanup.json"),
                        "deleted_files": 0, "deleted_bytes": 0, "empty_directories_removed": cleanup["emptyTempDirectoriesRemoved"]})
        manifest["retained"].extend({"path": r.get("path"), "kind": "retained-original-artifact",
            "reason": r.get("reason", "Original KEEP or nonterminal fixture; preserve for original owner"),
            "depends_on": [RUN], "review_after": None} for r in cleanup["retained"])
        write(output / "artifacts.json", manifest, replace=True)
        write(output / "summary.json", summary)
        print(json.dumps(summary, ensure_ascii=False))
    return 0 if summary.get("producerError") is None and (summary["nineInvocationsComplete"] or summary["firstSuiteMeasurementPassed"]) and unchanged and not cleanup["unknown"] else 1


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-sha", required=True)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--plan-sha", required=True)
    args = parser.parse_args(argv)
    try:
        return run(args.plan, args.plan_sha, args.self_sha)
    except Exception as error:
        print(json.dumps({"schemaVersion": 1, "taskId": RUN, "requestId": RUN, "attemptId": RUN,
                          "status": "entry-rejected-or-report-finalization-failed", "errorType": type(error).__name__,
                          "error": str(error), "allRequirementsComplete": False, "formalAcceptance": False,
                          "replayAuthorized": False, "releasedBytes": 0}, ensure_ascii=False))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
