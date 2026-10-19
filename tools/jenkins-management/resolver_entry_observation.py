"""Versioned, Jenkins-owned observation of the original resolver entry frame.

This recipe does not fix or accept the resolver. It records one target-frame
breakpoint while running the unchanged original Pester file with its normal exit
semantics. Source files, installed Pester and historical receipts stay read-only.
"""
from __future__ import annotations

import argparse
import ctypes
import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import stat
import sys
import xml.etree.ElementTree as ET

VERSION = 1
ROOT = Path("E:/Git/ZirconEngine")
RUN = "engine-s79-resolver-entry-observation-20261018-r1"
OUTPUT = Path("E:/cargo-targets/zircon-local/reports") / RUN
TEST = ROOT / "tools/tests/windows-path-resolver.Tests.ps1"
TEST_SHA = "ac96c78d6ccc3860dde237803312d8d03df90817e142b7ad865bd817a796ad33"
RESOLVER_SHA = "27d315830c00ef91e8c5241a454f7036a96a2c85fa5a04077b0815f8a64e5a08"
PESTER = Path("C:/Program Files/WindowsPowerShell/Modules/Pester/3.4.0/Pester.psd1")
ORIGINAL_CASES = (
    "rejects drive-relative paths before normalizing their per-drive working directory",
    "hides a verbatim prefix when rejecting a drive-relative path",
    "resolves an uncreated child through its existing junction ancestor",
    "normalizes an uncreated dotdot tail only after resolving its junction ancestor",
    "resolves an uncreated child through a temporary SUBST drive",
    "uses one physical identity for hard-linked file aliases",
)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def physical(path):
    path = Path(path).absolute()
    require(".." not in path.parts, "Path traversal is not supported")
    for entry in (path, *path.parents):
        if entry.exists() or entry.is_symlink():
            info = entry.lstat()
            require(not (getattr(info, "st_file_attributes", 0) & 0x400)
                    and not stat.S_ISLNK(info.st_mode), "Path alias: " + str(entry))
    if path.exists():
        require(os.path.normcase(str(path.resolve())) == os.path.normcase(str(path)),
                "Noncanonical physical path")
    return path


def pin(path):
    path = physical(path)
    with path.open("rb") as stream:
        before = os.fstat(stream.fileno())
        raw = stream.read()
        after = os.fstat(stream.fileno())
    require(stat.S_ISREG(after.st_mode), "An ordinary file is required")
    keys = ("st_dev", "st_ino", "st_size", "st_mtime_ns")
    require(all(getattr(before, k) == getattr(after, k) for k in keys)
            and (path.stat().st_dev, path.stat().st_ino) == (after.st_dev, after.st_ino),
            "Input changed while it was read")
    system = str(path).casefold().startswith("c:")
    require(system or after.st_nlink == 1, "Non-system hard-linked input")
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(),
            "bytes": len(raw), "lines": None if path.suffix.casefold() == ".exe"
            else len(raw.decode("utf-8-sig").splitlines()), "mtime_ns": after.st_mtime_ns,
            "fileId": after.st_ino, "device": after.st_dev,
            "hardLinkCount": after.st_nlink, "readOnlyInstalledSystemInput": system}, raw


def checked(item):
    actual, raw = pin(item["path"])
    for key in ("sha256", "bytes", "lines", "mtime_ns", "fileId", "device", "hardLinkCount"):
        if key in item:
            require(actual[key] == item[key], "Input drift: " + item["path"] + ":" + key)
    if "size" in item:
        require(actual["bytes"] == item["size"], "Input size changed")
    return raw


def write(path, value, *, raw=False):
    path = physical(path)
    require(path.is_relative_to(OUTPUT), "Output must stay inside this registered run")
    data = value if raw else (json.dumps(value, ensure_ascii=False, sort_keys=True,
                                        indent=2) + "\n").encode("utf-8")
    with path.open("xb") as stream:
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


def validate_plan(plan):
    require(plan.get("schemaVersion") == VERSION and plan.get("requestId") == RUN
            and plan.get("attemptId") == RUN, "Original S79 identity is required")
    require(Path(plan["output"]) == OUTPUT and plan.get("budgetSeconds") == 120,
            "Exact registered output and original 120 second budget are required")
    require(plan["test"]["sha256"] == TEST_SHA and Path(plan["test"]["path"]) == TEST
            and plan["resolver"]["sha256"] == RESOLVER_SHA,
            "The original resolver inputs must remain unchanged")
    require(Path(plan["resolver"]["path"]) == ROOT / "tools/maintenance/WindowsPathResolver.psm1",
            "Resolver path differs")
    require(plan["amendment"]["sha256"] == "bfb1036013cab88b3a509982830bba7ecb65fa1a2fe9c423feef4d6494b41d51",
            "The exact source-owner route amendment is required")


# An Action automatically continues unless it explicitly calls break. Only the
# unique stopped original frame supplies variables; Action locals are not a
# fallback. This script never assigns the test's automatic variables.
SCRIPT = r'''$ErrorActionPreference = 'Stop'
$testPath = '__TEST__'
$observationPath = '__OUTPUT__\observation.json'
$pesterXmlPath = '__OUTPUT__\pester-results.xml'
$diagnostic = [ordered]@{ schemaVersion = 1; recipeVersion = 1; status = 'not-hit'; hits = 0; observations = @(); diagnosticFailures = @(); removedOwnedBreakpoint = $false }
$metadata = [ordered]@{ powershell = [string]$PSVersionTable.PSVersion; edition = [string]$PSVersionTable.PSEdition; processId = $PID; birthFileTimeUtc = [string]([Diagnostics.Process]::GetCurrentProcess().StartTime.ToUniversalTime().ToFileTimeUtc()); pester = $null; path = $null; physicalTemp = [IO.Path]::GetTempPath() }
$limitedCommand = { param($command)
    if ($null -eq $command) { return [ordered]@{ isNull = $true; type = $null } }
    $record = [ordered]@{ isNull = $false; type = $command.GetType().FullName }
    foreach ($property in @('CommandType', 'Name', 'Path')) {
        $member = $command.PSObject.Properties[$property]
        $record[$property] = [ordered]@{ exists = ($null -ne $member); isNull = ($null -eq $member -or $null -eq $member.Value); value = $(if ($null -ne $member -and $null -ne $member.Value) { [string]$member.Value } else { $null }) }
    }
    return $record
}.GetNewClosure()
$limitedInvocation = { param($invocation, [bool]$errorContext = $false)
    if ($null -eq $invocation) { return $null }
    $record = [ordered]@{ type = $invocation.GetType().FullName; MyCommand = (& $limitedCommand $invocation.MyCommand) }
    $properties = @('ScriptName', 'PSScriptRoot', 'PSCommandPath', 'InvocationName')
    if ($errorContext) { $properties += @('ScriptLineNumber', 'OffsetInLine', 'Line', 'PositionMessage') }
    foreach ($property in $properties) {
        $member = $invocation.PSObject.Properties[$property]
        $record[$property] = [ordered]@{ exists = ($null -ne $member); isNull = ($null -eq $member -or $null -eq $member.Value); value = $(if ($null -ne $member -and $null -ne $member.Value) { [string]$member.Value } else { $null }) }
    }
    return $record
}.GetNewClosure()
$limitedError = { param($record)
    if ($null -eq $record) { return $null }
    return [ordered]@{ exceptionType = $record.Exception.GetType().FullName; message = $record.Exception.Message; fullyQualifiedErrorId = $record.FullyQualifiedErrorId; InvocationInfo = (& $limitedInvocation $record.InvocationInfo $true); ScriptStackTrace = $record.ScriptStackTrace }
}.GetNewClosure()
$action = {
    try {
        $diagnostic.hits++
        if ($diagnostic.hits -gt 16) { throw 'Diagnostic hit budget exceeded.' }
        $targetFrames = @(Get-PSCallStack | Where-Object { [string]::Equals($_.ScriptName, $testPath, [StringComparison]::OrdinalIgnoreCase) -and $_.ScriptLineNumber -eq 2 })
        if ($targetFrames.Count -ne 1) { throw ('Expected exactly one original line2 frame; found ' + $targetFrames.Count) }
        $targetFrame = $targetFrames[0]
        $frameVariables = $targetFrame.GetFrameVariables()
        $observedVariables = [ordered]@{}
        foreach ($variableName in @('PSScriptRoot', 'PSCommandPath', 'MyInvocation')) {
            $exists = $frameVariables.ContainsKey($variableName)
            $variableValue = if ($exists) { $frameVariables[$variableName].Value } else { $null }
            $entry = [ordered]@{ exists = $exists; isNull = ($null -eq $variableValue); type = $(if ($null -ne $variableValue) { $variableValue.GetType().FullName } else { $null }); value = $null }
            if ($null -ne $variableValue) {
                if ($variableName -eq 'MyInvocation') { $entry.value = & $limitedInvocation $variableValue }
                elseif ($variableValue -is [string]) { $entry.value = $variableValue }
                else { throw ('Unexpected automatic variable type: ' + $variableName) }
            }
            $observedVariables[$variableName] = $entry
        }
        $diagnostic.observations += [ordered]@{ ScriptName = $targetFrame.ScriptName; ScriptLineNumber = $targetFrame.ScriptLineNumber; InvocationInfo = (& $limitedInvocation $targetFrame.InvocationInfo); variables = $observedVariables; variableSource = 'unique-original-frame.GetFrameVariables' }
        $diagnostic.status = 'captured'
    }
    catch {
        $diagnostic.status = 'diagnostic-failed'
        $diagnostic.diagnosticFailures += & $limitedError $_
    }
}.GetNewClosure()
$ownedBreakpoint = $null
$result = $null
$originalError = $null
$breakpointSetupError = $null
try {
    $pesterModule = Get-Module -ListAvailable Pester | Where-Object { $_.Version.Major -lt 5 } | Sort-Object Version -Descending | Select-Object -First 1
    if ($null -eq $pesterModule -or [string]$pesterModule.Version -ne '3.4.0' -or $pesterModule.Path -ine '__PESTER__' -or $PSVersionTable.PSVersion.Major -ne 5 -or $PSVersionTable.PSEdition -ne 'Desktop') { throw 'Pinned Desktop PS5.1 / Pester3.4.0 is required.' }
    Import-Module $pesterModule.Path -Force
    $metadata.pester = [string]$pesterModule.Version
    $metadata.path = $pesterModule.Path
    try { $ownedBreakpoint = Set-PSBreakpoint -Script $testPath -Line 2 -Action $action }
    catch { $breakpointSetupError = & $limitedError $_; $diagnostic.status = 'diagnostic-failed'; $diagnostic.diagnosticFailures += $breakpointSetupError }
    $result = Invoke-Pester -Script $testPath -PassThru -Quiet -OutputFormat NUnitXml -OutputFile $pesterXmlPath
}
catch { $originalError = $_ }
finally {
    if ($null -ne $ownedBreakpoint) {
        try { Remove-PSBreakpoint -Id $ownedBreakpoint.Id; $diagnostic.removedOwnedBreakpoint = $true }
        catch { $diagnostic.status = 'diagnostic-failed'; $diagnostic.diagnosticFailures += & $limitedError $_ }
    }
    if ($diagnostic.hits -eq 0 -and $diagnostic.status -eq 'not-hit') { $diagnostic.status = 'diagnostic-failed-not-hit' }
    $counts = if ($null -ne $result) { [ordered]@{ total = $result.TotalCount; passed = $result.PassedCount; failed = $result.FailedCount; skipped = $result.SkippedCount; pending = $result.PendingCount } } else { $null }
    $testErrors = @()
    if ($null -ne $result) {
        foreach ($testResult in @($result.TestResult)) {
            if ($null -ne $testResult.ErrorRecord) { $testErrors += & $limitedError $testResult.ErrorRecord }
        }
    }
    $proof = [ordered]@{ metadata = $metadata; diagnostic = $diagnostic; originalCounts = $counts; originalError = (& $limitedError $originalError); testErrors = $testErrors; testDriveRemaining = [bool](Get-PSDrive -Name TestDrive -ErrorAction SilentlyContinue); ordinaryRegressionAccepted = $false }
    [IO.File]::WriteAllText($observationPath, ($proof | ConvertTo-Json -Depth 16), (New-Object Text.UTF8Encoding($false)))
    $metadata | ConvertTo-Json -Compress
    if ($null -ne $counts) { $counts | ConvertTo-Json -Compress }
}
if ($null -ne $originalError) { throw $originalError }
if ($null -eq $result -or $result.TotalCount -ne 6 -or $result.PassedCount -ne 6 -or $result.FailedCount -ne 0 -or $result.SkippedCount -ne 0) { exit 1 }
exit 0
'''


def render_runner(plan):
    validate_plan(plan)
    return SCRIPT.replace("__TEST__", str(TEST)).replace("__OUTPUT__", str(OUTPUT)).replace("__PESTER__", str(PESTER)).encode("ascii")


def devices():
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    query = kernel.QueryDosDeviceW
    query.argtypes = (ctypes.c_wchar_p, ctypes.c_wchar_p, ctypes.c_uint32)
    query.restype = ctypes.c_uint32
    result = {}
    for letter in "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
        buffer = ctypes.create_unicode_buffer(32768)
        length = query(letter + ":", buffer, len(buffer))
        if length:
            result[letter + ":"] = buffer[:length].split(chr(0))
    return result


def xml_cases(raw):
    if raw is None:
        return {"present": False, "attributes": None, "cases": [], "originalNamedCases": 0}
    root = ET.fromstring(raw)
    cases = [{"name": n.get("name"), "description": n.get("description"),
              "executed": n.get("executed"), "result": n.get("result"),
              "success": n.get("success"), "message": n.findtext("failure/message"),
              "stackTrace": n.findtext("failure/stack-trace")} for n in root.iter("test-case")]
    return {"present": True, "attributes": dict(root.attrib), "cases": cases,
            "originalNamedCases": sum(any((x["name"] or "").endswith(name)
                                           for name in ORIGINAL_CASES) for x in cases)}


def output_encoding(raw, acp):
    matches = []
    for encoding in dict.fromkeys(("cp" + str(acp), "utf-8")):
        try:
            if raw.decode(encoding).encode(encoding) == raw:
                matches.append(encoding)
        except (UnicodeError, LookupError):
            pass
    return {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(),
            "strictRoundtripEncodings": matches, "allAscii": all(v < 128 for v in raw),
            "selected": matches[0] if matches else None, "systemAnsiCodePage": acp}


def run(plan_path, plan_sha, self_sha):
    checked({"path": __file__, "sha256": self_sha})
    plan = json.loads(checked({"path": str(plan_path), "sha256": plan_sha}))
    validate_plan(plan)
    require(os.environ.get("JOB_NAME") == "zircon-management-tests"
            and os.environ.get("NODE_NAME") == "zircon-windows-agent"
            and os.environ.get("RUN_ID") == RUN, "Exact Jenkins job, node and RUN_ID required")
    for item in plan["controlPins"]:
        checked(item)
    checked(plan["artifactRegistration"])
    before = [pin(item["path"])[0] for item in plan["pinnedInputs"]]
    for expected in plan["pinnedInputs"]:
        checked(expected)
    require(not physical(OUTPUT).exists(), "Original run output must be absent; reconcile before reuse")
    OUTPUT.mkdir()
    write(OUTPUT / "artifacts.json", {"schemaVersion": 1, "recipeVersion": VERSION,
          "taskId": RUN, "ownerThreadId": plan["ownerThreadId"], "paths": [str(OUTPUT)],
          "dependencies": plan["pinnedInputs"], "message": plan["message"],
          "amendment": plan["amendment"], "retention": "Original diagnostic and failure/native evidence; no source copy",
          "filesDeleted": 0, "releasedBytes": 0})
    write(OUTPUT / "plan.json", plan)
    write(OUTPUT / "input-pins-before.json", {"inputs": before})
    runner = render_runner(plan)
    write(OUTPUT / "runner.ps1", runner, raw=True)
    write(OUTPUT / "generated-runner-pin.json", pin(OUTPUT / "runner.ps1")[0])
    component = Path(plan["component"]["path"]).parent
    facts = json.loads(checked(plan["component"]))["files"]
    # Resolve native supervision from the selected immutable driver, as the
    # existing bounded Jenkins recipes do, rather than an ambient PYTHONPATH.
    driver = physical(ROOT / ".jenkins/runtime/drivers" / plan["runtime"]["driverDigest"])
    sys.path.insert(0, str(driver))
    cargo = load("s79_native_cargo", component / "local_cargo.py", facts["local_cargo.py"]["sha256"])
    checks = load("s79_native_checks", component / "run_checks.py", facts["run_checks.py"]["sha256"])
    delivery = load("s79_native_environment", component / "delivery.py", facts["delivery.py"]["sha256"])
    _, runtime, _ = delivery.runtime_files(ROOT)
    require(all(runtime[k] == v for k, v in plan["runtime"].items()), "Selected runtime changed")
    (OUTPUT / "temp").mkdir()
    environment = delivery.environment(OUTPUT, runtime)
    environment.update(RUN_ID=RUN, JOB_NAME=os.environ["JOB_NAME"], NODE_NAME=os.environ["NODE_NAME"])
    require(Path(environment["TEMP"]) == OUTPUT / "temp"
            and Path(environment["TMP"]) == OUTPUT / "temp", "Owned fixture TEMP/TMP required")
    before_devices = devices()
    write(OUTPUT / "fixture-before.json", {"temp": environment["TEMP"], "children": [],
          "dosDevices": before_devices, "owner": RUN, "fixturesCreatedOnlyByOriginalTests": True})
    unit = OUTPUT / "producer-contract-tests"
    unit.mkdir()
    unit_command = [str(ROOT / ".jenkins/runtime/python/python.exe"), "-B", "-X", "utf8", "-m",
                    "unittest", "discover", "-s", "tools/jenkins-management/tests", "-p",
                    "test_resolver_entry_observation.py", "-v"]
    unit_native = cargo._run_owned_command(unit_command, ROOT, environment, unit, 90)
    write(unit / "native-terminal.json", unit_native)
    unit_ok = checks.native_complete(unit_native, unit_command, ROOT) and unit_native["rootExitCode"] == 0
    command = [plan["powershell"], "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
               "-File", str(OUTPUT / "runner.ps1")]
    native = None
    if unit_ok:
        leaf = OUTPUT / "resolver-tests"
        leaf.mkdir()
        native = cargo._run_owned_command(command, ROOT, environment, leaf, 120)
        write(leaf / "native-terminal.json", native)
    after = [pin(item["path"])[0] for item in plan["pinnedInputs"]]
    unchanged = before == after
    write(OUTPUT / "input-pins-after.json", {"inputs": after, "unchanged": unchanged})
    after_devices = devices()
    remaining = sorted(x.name for x in (OUTPUT / "temp").iterdir())
    write(OUTPUT / "fixture-after.json", {"temp": environment["TEMP"], "remaining": remaining,
          "dosDevices": after_devices, "dosDevicesUnchanged": before_devices == after_devices,
          "ownedTempEmpty": not remaining, "noForeignMappingRemoved": True})
    observation = json.loads((OUTPUT / "observation.json").read_bytes()) if (OUTPUT / "observation.json").exists() else None
    xml_path = OUTPUT / "pester-results.xml"
    xml = xml_cases(xml_path.read_bytes() if xml_path.exists() else None)
    write(OUTPUT / "xml-observation.json", xml)
    complete = native is not None and checks.native_complete(native, command, ROOT)
    diagnostic_ok = bool(observation and observation["diagnostic"]["status"] == "captured"
                         and observation["diagnostic"]["removedOwnedBreakpoint"]
                         and not observation["diagnostic"]["diagnosticFailures"])
    encoding = {}
    if native is not None:
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        for name in ("stdout", "stderr"):
            encoding[name] = output_encoding((OUTPUT / "resolver-tests" / (name + ".bin")).read_bytes(), int(kernel.GetACP()))
    # Diagnostic success does not replace the original test's exit code, counts,
    # NUnit error or ordinary-regression acceptance.
    write(OUTPUT / "raw-encoding.json", encoding)
    temp_removed = bool(unit_ok and complete and not remaining)
    if temp_removed:
        (OUTPUT / "temp").rmdir()
    write(OUTPUT / "cleanup.json", {"removedOwnEmptyTempOnly": temp_removed,
          "filesDeleted": 0, "releasedBytes": 0, "allFailureEvidenceRetained": True,
          "remainingOwnedFixtureNames": remaining, "unknownMappingsPreserved": True})
    summary = {"schemaVersion": 1, "recipeVersion": VERSION, "requestId": RUN, "attemptId": RUN,
               "message": plan["message"], "amendment": plan["amendment"], "runtime": plan["runtime"],
               "job": os.environ["JOB_NAME"], "build": int(os.environ["BUILD_NUMBER"]),
               "producerContractTestsPassed": unit_ok, "powershellStarted": native is not None,
               "argv": command, "cwd": str(ROOT), "budgetSeconds": 120,
               "nativeComplete": bool(complete), "nativeExitCode": native["rootExitCode"] if native else None,
               "diagnosticCaptured": diagnostic_ok, "originalCounts": observation.get("originalCounts") if observation else None,
               "originalNamedCases": xml["originalNamedCases"], "originalXmlPresent": xml["present"],
               "originalTestPassed": bool(complete and native["rootExitCode"] == 0),
               "inputPinsUnchanged": unchanged, "dosDevicesUnchanged": before_devices == after_devices,
               "ownedFixtureTempEmpty": not remaining, "ordinaryRegressionAccepted": False,
               "formalAcceptance": False, "allRequirementsComplete": False,
               "status": "diagnostic-captured-original-result-retained" if diagnostic_ok else "diagnostic-failed",
               "sourceCopies": 0, "sourceEdits": 0, "filesDeleted": 0, "releasedBytes": 0,
               "completedAtUtc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
    write(OUTPUT / "summary.json", summary)
    print(json.dumps(summary, ensure_ascii=False))
    return 0 if unit_ok and complete and diagnostic_ok and unchanged and not remaining and before_devices == after_devices else 1


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True, type=Path)
    parser.add_argument("--plan-sha", required=True)
    parser.add_argument("--self-sha", required=True)
    args = parser.parse_args(argv)
    return run(args.plan, args.plan_sha, args.self_sha)


if __name__ == "__main__":
    raise SystemExit(main())
