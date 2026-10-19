"""Run the original six resolver cases normally under one admitted Jenkins deadline."""
from pathlib import Path
import argparse, ctypes, hashlib, importlib.util, json, os, sys, time
import xml.etree.ElementTree as ET

ROOT = Path("E:/Git/ZirconEngine")
REQUEST = "engine-s83-resolver-regression-20261018-r1"
RUN = "engine-s96-resolver-entry-20261018-r1"
OUTPUT = Path("E:/cargo-targets/zircon-local/reports") / REQUEST / "s96-loader-entry"
HELPER = ROOT / "tools/jenkins-management/fixture_owner_recipe.py"
HELPER_SHA = "296f6161131d8c9ebc8e5ab086a095854013debb18eff7f9994c708f8ca9e37a"
TEST = ROOT / "tools/tests/windows-path-resolver.Tests.ps1"
TEST_SHA = "edc34164cbd3cb06279e279960816792af8b1718da69c737c52dee937c58c01b"
RECIPE = ROOT / "tools/jenkins-management/resolver_regression_recipe.py"
REGRESSIONS = ROOT / "tools/jenkins-management/tests/test_resolver_regression_recipe.py"
DESCRIBE = "Windows path resolver"
ORIGINAL_CASES = (
    "rejects drive-relative paths before normalizing their per-drive working directory",
    "hides a verbatim prefix when rejecting a drive-relative path",
    "resolves an uncreated child through its existing junction ancestor",
    "normalizes an uncreated dotdot tail only after resolving its junction ancestor",
    "resolves an uncreated child through a temporary SUBST drive",
    "uses one physical identity for hard-linked file aliases",
)

SCRIPT = r"""
param([string]$TestPath, [string]$PesterManifest, [string]$OutputRoot)
$ErrorActionPreference = 'Stop'
$record = [ordered]@{
    metadata = $null; originalCounts = $null; cases = @(); errors = @()
    bootstrapError = $null; collectorErrors = @(); normalEntry = $true
    sourceRewritten = $false; breakpointUsed = $false; traceUsed = $false; repoRootInjected = $false
    testDrivePresentAfterInvoke = $null
}
$code = 1
$bootstrapFailure = $null
function Convert-S96Error($item) {
    [ordered]@{
        message = [string]$item.Exception.Message
        exceptionType = [string]$item.Exception.GetType().FullName
        fullyQualifiedErrorId = [string]$item.FullyQualifiedErrorId
        category = [string]$item.CategoryInfo
        position = [string]$item.InvocationInfo.PositionMessage
        invocationLine = [string]$item.InvocationInfo.Line
        scriptName = [string]$item.InvocationInfo.ScriptName
        scriptLineNumber = $item.InvocationInfo.ScriptLineNumber
        scriptStackTrace = [string]$item.ScriptStackTrace
    }
}
function Add-S96CollectorError([string]$phase, $item) {
    $record.collectorErrors += [ordered]@{phase=$phase; message=[string]$item}
    $script:code = 1
}
try {
    Import-Module -Name $PesterManifest -Force -ErrorAction Stop
    $pesterCommand = Get-Command Invoke-Pester -CommandType Function -ErrorAction Stop
    $selfProcess = [Diagnostics.Process]::GetCurrentProcess()
    $record.metadata = [ordered]@{
        powershell = [string]$PSVersionTable.PSVersion
        edition = [string]$PSVersionTable.PSEdition; processId = $PID
        birthFileTimeUtc = [string]$selfProcess.StartTime.ToUniversalTime().ToFileTimeUtc()
        executable = [string]$selfProcess.MainModule.FileName
        cwd = [string](Get-Location).ProviderPath
        buildNumber = $env:BUILD_NUMBER; jobName = $env:JOB_NAME
        nodeName = $env:NODE_NAME; runId = $env:RUN_ID
        temp = $env:TEMP; tmp = $env:TMP; tmpdir = $env:TMPDIR
        physicalTemp = [IO.Path]::GetTempPath()
        pesterVersion = [string]$pesterCommand.Module.Version
        pesterPath = [IO.Path]::Combine($pesterCommand.Module.ModuleBase, 'Pester.psd1')
        pesterModulePath = [string]$pesterCommand.Module.Path
    }
    if ($PSVersionTable.PSEdition -ne 'Desktop' -or $PSVersionTable.PSVersion.Major -ne 5 -or
        $PSVersionTable.PSVersion.Minor -ne 1 -or $pesterCommand.Module.Version -ne [version]'3.4.0') {
        throw 'Actual Desktop PowerShell 5.1 and Pester 3.4.0 are required'
    }
    if ($env:BUILD_NUMBER -notmatch '^[0-9]+$' -or [long]$env:BUILD_NUMBER -le 0) {
        throw 'Actual positive child BUILD_NUMBER is required'
    }
    # Original, unfiltered invocation; no injected source variables or instrumentation.
    $result = Invoke-Pester -Script $TestPath -PassThru -Quiet -OutputFormat NUnitXml -OutputFile ([IO.Path]::Combine($OutputRoot, 'pester-results.xml'))
    $record.originalCounts = [ordered]@{
        total=$result.TotalCount; passed=$result.PassedCount; failed=$result.FailedCount
        skipped=$result.SkippedCount; pending=$result.PendingCount; inconclusive=$result.InconclusiveCount
    }
    $record.cases = @($result.TestResult | ForEach-Object {
        [ordered]@{name=[string]$_.Name; describe=[string]$_.Describe; context=[string]$_.Context
            passed=[bool]$_.Passed; result=[string]$_.Result
            failureMessage=[string]$_.FailureMessage; stackTrace=[string]$_.StackTrace}
    })
    if ($result.TotalCount -eq 6 -and $result.PassedCount -eq 6 -and $result.FailedCount -eq 0 -and
        $result.SkippedCount -eq 0 -and $result.PendingCount -eq 0) { $code = 0 }
} catch {
    $code = 1
    $bootstrapFailure = $_
    # This fallback remains available when ConvertTo-Json itself is failing.
    try { [Console]::Error.WriteLine(('S96 bootstrap: ' + [string]$bootstrapFailure)) }
    catch { Add-S96CollectorError 'bootstrap-stderr' $_ }
    try { $record.bootstrapError = Convert-S96Error $bootstrapFailure }
    catch { Add-S96CollectorError 'bootstrap-error-encoding' $_ }
} finally {
    try {
        $record.testDrivePresentAfterInvoke = [bool](Get-PSDrive -Name TestDrive -ErrorAction SilentlyContinue)
    } catch { Add-S96CollectorError 'test-drive-observation' $_ }
    # Raw $Error also contains expected ShouldThrow/absent TestDrive diagnostics.
    # Preserve it; only runner/collector failures and actual test results decide failure.
    $history = @($Error)
    foreach ($item in $history) {
        try { $record.errors += Convert-S96Error $item }
        catch { Add-S96CollectorError 'raw-error-encoding' $_ }
    }
    try {
        $data = [Text.UTF8Encoding]::new($false).GetBytes(($record | ConvertTo-Json -Depth 16))
        $stream = [IO.File]::Open([IO.Path]::Combine($OutputRoot, 'observation.json'),
            [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
        try { $stream.Write($data, 0, $data.Length); $stream.Flush($true) } finally { $stream.Dispose() }
    } catch {
        $persistenceFailure = $_
        try {
            if ($null -ne $bootstrapFailure) {
                [Console]::Error.WriteLine(('S96 bootstrap: ' + [string]$bootstrapFailure))
            }
            [Console]::Error.WriteLine(('S96 observation-persistence: ' + [string]$persistenceFailure))
        } catch { Add-S96CollectorError 'persistence-stderr' $_ }
        Add-S96CollectorError 'observation-persistence' $persistenceFailure
        try { [Console]::Error.WriteLine(($record | ConvertTo-Json -Depth 16)) }
        catch {
            $stderrFailure = $_
            Add-S96CollectorError 'stderr-json-encoding' $stderrFailure
            try { [Console]::Error.WriteLine(('S96 stderr-json-encoding: ' + [string]$stderrFailure)) }
            catch { $code = 1 }
        }
    }
}
exit $code
"""

def require(value, message):
    if not value:
        raise ValueError(message)

def ordinary(path):
    path = Path(path).absolute()
    require(path.drive and ".." not in path.parts, "Absolute canonical path required")
    for member in (path, *path.parents):
        if member.exists() or member.is_symlink():
            info = member.lstat()
            require(not member.is_symlink() and not getattr(info, "st_file_attributes", 0) & 0x400,
                    "Path alias rejected: " + str(member))
    if path.exists():
        require(os.path.normcase(str(path.resolve())) == os.path.normcase(str(path)),
                "Noncanonical path rejected")
    return path

def read_hash(path, expected):
    path = ordinary(path)
    with path.open("rb") as stream:
        before = os.fstat(stream.fileno())
        raw = stream.read()
        after = os.fstat(stream.fileno())
    require(before.st_nlink == after.st_nlink == 1 and
            (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) ==
            (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns) and
            (path.stat().st_dev, path.stat().st_ino) == (after.st_dev, after.st_ino),
            "Input changed or is an alias")
    require(hashlib.sha256(raw).hexdigest() == expected, "SHA256 changed: " + str(path))
    return raw

def load_helper():
    raw = read_hash(HELPER, HELPER_SHA)
    spec = importlib.util.spec_from_file_location("s96_shared_evidence_helpers", HELPER)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    exec(compile(raw, str(HELPER), "exec"), module.__dict__)
    return module

def business_result(f, raw, observation):
    """Require exact NUnit Describe/It identities and the ordinary PassThru objects."""
    result = f.xml_result(raw)
    if result.get("parseError"):
        raise ET.ParseError(result["parseError"])
    if result.get("present") is not True:
        return None
    counts = observation.get("originalCounts")
    objects = observation.get("cases")
    required_observation = ("normalEntry", "sourceRewritten", "breakpointUsed", "traceUsed", "repoRootInjected")
    if (not isinstance(counts, dict) or any(counts.get(k) is None for k in
            ("total", "passed", "failed", "skipped", "pending")) or objects is None or
            observation.get("collectorErrors") is None or "bootstrapError" not in observation or
            any(observation.get(k) is None for k in required_observation)):
        return None
    for key, expected in {"total": 6, "passed": 6, "failed": 0, "skipped": 0, "pending": 0}.items():
        require(type(counts.get(key)) is int and counts[key] == expected, "PassThru count unknown or failed: " + key)
    # Pester 3.4.0 omits InconclusiveCount from PassThru. Preserve that null;
    # its XML and each real object must independently rule out inconclusive cases.
    inconclusive = counts.get("inconclusive")
    require(inconclusive is None or (type(inconclusive) is int and inconclusive == 0),
            "PassThru inconclusive result disagrees")
    require(isinstance(objects, list) and len(objects) == 6, "Exactly six PassThru objects required")
    if any(not isinstance(c, dict) or any(k not in c or c[k] is None for k in
            ("name", "describe", "context", "passed", "result", "failureMessage", "stackTrace")) for c in objects):
        return None
    expected_names = set(ORIGINAL_CASES)
    require({c.get("name") for c in objects} == expected_names, "PassThru names missing, extra or duplicated")
    for case in objects:
        require(case.get("describe") == DESCRIBE and case.get("context") == "" and
                case.get("passed") is True and case.get("result") == "Passed" and
                not case.get("failureMessage") and not case.get("stackTrace"),
                "PassThru Describe/It identity or result differs")
    attributes = result.get("attributes") or {}
    required = {"name": "Pester", "total": "6", "errors": "0", "failures": "0", "not-run": "0",
                "inconclusive": "0", "ignored": "0", "skipped": "0", "invalid": "0"}
    if any(attributes.get(k) is None for k in required):
        return None
    require(all(attributes.get(k) == v for k, v in required.items()), "NUnit counts unknown or failed")
    root = ET.fromstring(raw)
    require(root.tag == "test-results", "Original NUnit root required")
    suites = list(root.iter("test-suite"))
    if any(any(s.get(k) is None for k in ("type", "name", "executed", "result", "success")) for s in suites):
        return None
    require(len(suites) == 2 and suites[0].get("type") == "Powershell" and
            suites[0].get("name") == "Pester" and suites[1].get("type") == "TestFixture" and
            suites[1].get("name") == DESCRIBE, "Extra loader/Describe/parameterized suite rejected")
    require(all(s.get("executed") == "True" and s.get("result") == "Success" and
                s.get("success") == "True" for s in suites), "NUnit suite not successful")
    cases = result["cases"]
    if any(any(c.get(k) is None for k in ("name", "description", "executed", "success", "result")) for c in cases):
        return None
    require(len(cases) == 6 and len(list(suites[1].iter("test-case"))) == 6 and
            {c.get("name") for c in cases} == {DESCRIBE + "." + n for n in expected_names},
            "Exact full XML identities required; suffix matching is forbidden")
    for case in cases:
        require(case.get("description") in expected_names and
                case.get("name") == DESCRIBE + "." + case["description"] and
                case.get("executed") == "True" and case.get("success") == "True" and
                case.get("result") == "Success" and not case.get("message") and not case.get("stackTrace"),
                "XML case identity, execution or result differs")
    require(isinstance(observation.get("collectorErrors"), list) and not observation["collectorErrors"],
            "Runner collector failed or its status is unknown")
    require(observation.get("normalEntry") is True and observation.get("bootstrapError") is None and
            all(observation.get(k) is False for k in
                ("sourceRewritten", "breakpointUsed", "traceUsed", "repoRootInjected")),
            "Ordinary runner failed or execution was instrumented")
    return result

def clock():
    return time.monotonic()

def system_code_page():
    return int(ctypes.WinDLL("kernel32").GetACP())

def observed_all(*facts):
    """An incomplete observation is unknown, even if another known fact differs."""
    return all(facts) if all(type(value) is bool for value in facts) else None

def run(plan_path, plan_sha, self_sha, entered=None):
    entered = clock() if entered is None else entered
    deadline = entered + 180.0  # Plan loading, environment and fsync consume this same allowance.
    def remaining():
        value = deadline - clock()
        require(value > 0, "Inclusive 180s deadline exhausted; no new child")
        return value
    read_hash(__file__, self_sha)
    require(ordinary(__file__) == ordinary(RECIPE), "Only the reviewed canonical recipe may execute")
    plan = json.loads(read_hash(plan_path, plan_sha))
    require(plan.get("executionAdmitted") is True and plan.get("dispatchAdmitted") is True,
            "Preparation only: exact patch, current canonical pins and formal route remain subject to review")
    require(plan["requestId"] == plan["attemptId"] == REQUEST and plan["runId"] == RUN,
            "Original request/attempt and separate normal-entry RUN required")
    require(Path(plan["output"]) == OUTPUT and plan["overallBudgetSeconds"] == 180 and
            plan["maximumNativeStarts"] == 1 and plan["terminalDrainBudgetSeconds"] == 30,
            "Exact output and inclusive execution/separate bounded terminal budgets required")
    require(os.environ.get("JOB_NAME") == "zircon-management-tests" and
            os.environ.get("NODE_NAME") == "zircon-windows-agent" and os.environ.get("RUN_ID") == RUN,
            "Actual Jenkins job/node/RUN required")
    build = os.environ.get("BUILD_NUMBER", "")
    require(build.isascii() and build.isdigit() and int(build) > 0, "Actual parent BUILD_NUMBER required")
    require(Path(plan["test"]["path"]) == TEST and plan["test"]["sha256"] == TEST_SHA and
            tuple(plan["expectedOriginalCases"]) == ORIGINAL_CASES, "Exact current original six-case source required")
    f = load_helper()
    expected = plan["pinnedInputs"]
    before = f.observe_inputs(expected)
    require(bool(before) and all(item["matchesPlan"] for item in before), "Current complete input closure changed")
    f.checked(plan["artifactRegistration"])
    members = f.pester_members(plan)
    pinned_paths = {str(Path(item["path"])) for item in expected}
    required_paths = {str(p) for p in (RECIPE, REGRESSIONS, HELPER, TEST, Path(plan["resolver"]["path"]),
                                      Path(plan["powershell"]), Path(sys.executable), Path(plan["component"]["path"]))}
    require(len(pinned_paths) == len(expected) and required_paths.issubset(pinned_paths) and
            members == plan["pesterMembers"] and set(members).issubset(pinned_paths),
            "Direct inputs or actual imported Pester membership are not completely pinned")
    driver = f.physical(ROOT / ".jenkins/runtime/drivers" / plan["runtime"]["driverDigest"])
    sys.path.insert(0, str(driver))
    component = Path(plan["component"]["path"]).parent
    facts = json.loads(f.checked(plan["component"]))["files"]
    require({str(component / name) for name in facts}.issubset(pinned_paths),
            "Sealed native component members must be in the same closure")
    cargo = f.load("s96_native_cargo", component / "local_cargo.py", facts["local_cargo.py"]["sha256"])
    checks = f.load("s96_native_checks", component / "run_checks.py", facts["run_checks.py"]["sha256"])
    delivery = f.load("s96_native_environment", component / "delivery.py", facts["delivery.py"]["sha256"])
    _, actual_runtime, _ = delivery.runtime_files(ROOT)
    require(all(actual_runtime.get(k) == v for k, v in plan["runtime"].items()), "Runtime changed")
    remaining()
    require(not f.physical(OUTPUT).exists(), "Existing output: reconcile original identity; never replay")
    output_owned = False; output_identity = None
    native = None; streams = {}; error = None; xml = {}; observation = {}; after = None
    identity_ok = business_ok = unchanged = same_members = same_devices = fixtures_settled = None
    dispatched = returned = None; devices_before = devices_after = None
    original_temp = None; temp_children = None; raw_xml = None
    secondary_errors = []; persistence_errors = []
    def record_error(exception, phase):
        nonlocal error
        item = {"phase": phase, "type": type(exception).__name__, "message": str(exception)}
        if error is None:
            error = item
        else:
            secondary_errors.append(item)
        return None
    def attempt(phase, action):
        try:
            return action()
        except Exception as exception:
            return record_error(exception, phase)
    def write(name, value, raw=False):
        require(output_owned, "No write without exclusive output ownership")
        require(output_identity is not None and
                f.native_identity(f.physical(OUTPUT))["fileIdentity"] == output_identity["fileIdentity"],
                "Owned output directory identity changed or is unknown")
        path = f.physical(OUTPUT / name)
        require(path.is_relative_to(OUTPUT), "Unregistered output")
        data = value if raw else (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
        with path.open("xb") as stream:
            stream.write(data); stream.flush(); os.fsync(stream.fileno())
    try:
        OUTPUT.mkdir()  # No exist_ok: a raced/unknown directory remains entirely foreign.
        output_owned = True
        output_identity = f.native_identity(f.physical(OUTPUT))
        write("artifacts.json", {"ownerRequestId": REQUEST, "ownerAttemptId": REQUEST, "runId": RUN,
              "root": str(OUTPUT), "dependencies": expected, "registration": plan["artifactRegistration"],
              "retention": "Original native/loader evidence; no deletion or replay authority"})
        write("input-pins-before.json", before)
        runner = SCRIPT.lstrip().replace("\n", "\r\n").encode("utf-8-sig")
        require(hashlib.sha256(runner).hexdigest() == plan["runnerSha256"], "Reviewed ordinary runner differs")
        write("runner.ps1", runner, raw=True)
        (OUTPUT / "temp").mkdir()
        leaf = OUTPUT / "resolver-tests"; leaf.mkdir()
        environment = delivery.environment(OUTPUT, actual_runtime)
        environment.update({k: os.environ[k] for k in ("RUN_ID", "JOB_NAME", "NODE_NAME", "BUILD_NUMBER")})
        for key in ("TEMP", "TMP", "TMPDIR"):
            require(f.same_path(environment[key], OUTPUT / "temp"), "Actual owned E-root TEMP required")
        devices_before = f.devices()
        original_temp = f.native_identity(f.physical(OUTPUT / "temp"))
        write("environment-before.json", {"runtime": actual_runtime, "dosDevices": devices_before,
              "temporaryPaths": {k: f.native_identity(f.physical(environment[k])) for k in ("TEMP", "TMP", "TMPDIR")},
              "build": build, "runner": f.pin(OUTPUT / "runner.ps1")[0]})
        command = [plan["powershell"], "-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy",
                   "Bypass", "-File", str(OUTPUT / "runner.ps1"), "-TestPath", plan["test"]["path"],
                   "-PesterManifest", plan["pester"]["path"], "-OutputRoot", str(OUTPUT)]
        for item in plan["controlPins"]:
            f.checked(item)
        write("launch-intent.json", {"argv": command, "cwd": str(ROOT), "requestId": REQUEST,
              "attemptId": REQUEST, "runId": RUN, "maximumNativeStarts": 1})
        timeout = remaining()  # AFTER fsync; this is the last operation before the one child call.
        dispatched = clock()
        try:
            native = cargo._run_owned_command(command, ROOT, environment, leaf, timeout)
        finally:
            returned = clock()
    except Exception as exception:
        if not output_owned:
            raise  # Never create fallback evidence in a directory this invocation did not acquire.
        record_error(exception, "owned-preflight-or-native-command")
    if not output_owned:
        raise ValueError("Exclusive output ownership was not obtained")
    # All owned-output observations are independent: one failure cannot hide another
    # or bypass the final best-effort unknown/failure summary.
    if native is not None and not isinstance(native, dict):
        record_error(ValueError("Native helper returned unknown evidence"), "native-return-type")
        native = None
    if native is not None:
        attempt("native-terminal-persistence", lambda: write("resolver-tests/native-terminal.json", native))
        streams = attempt("native-stream-observation", lambda: f.native_stream_observation(
            OUTPUT / "resolver-tests", command, ROOT, checks, system_code_page(), native)) or {}
    def read_observation():
        value = json.loads(f.pin(OUTPUT / "observation.json")[1])
        require(isinstance(value, dict), "Child observation must be a JSON object")
        return value
    observation = attempt("child-observation-read", read_observation) or {}
    metadata = observation.get("metadata")
    metadata = metadata if isinstance(metadata, dict) else {}
    def child_identity():
        required = ("processId", "birthFileTimeUtc", "buildNumber", "jobName", "nodeName", "runId", "edition",
                    "powershell", "pesterVersion", "pesterPath", "pesterModulePath", "executable", "cwd",
                    "temp", "tmp", "tmpdir", "physicalTemp")
        if (native is None or native.get("pid") is None or native.get("creationTime") is None or
                any(metadata.get(k) is None for k in required)):
            return None
        return bool(metadata["processId"] == native["pid"] and
            str(metadata.get("birthFileTimeUtc")) == native.get("creationTime") and
            metadata.get("buildNumber") == build and metadata.get("jobName") == os.environ["JOB_NAME"] and
            metadata.get("nodeName") == os.environ["NODE_NAME"] and metadata.get("runId") == RUN and
            metadata.get("edition") == "Desktop" and str(metadata.get("powershell", "")).startswith("5.1.") and
            metadata.get("pesterVersion") == "3.4.0" and
            f.same_path(metadata.get("pesterPath", ""), plan["pester"]["path"]) and
            f.same_path(metadata.get("pesterModulePath", ""), Path(plan["pester"]["path"]).with_suffix(".psm1")) and
            f.same_path(metadata.get("executable", ""), plan["powershell"]) and
            f.same_path(metadata.get("cwd", ""), ROOT) and
            all(f.same_path(metadata.get(k, ""), OUTPUT / "temp") for k in ("temp", "tmp", "tmpdir", "physicalTemp")))
    identity_ok = attempt("child-process-and-environment-identity", child_identity)
    raw_xml = attempt("xml-read", lambda: f.pin(OUTPUT / "pester-results.xml")[1])
    if raw_xml is not None:
        xml = attempt("xml-observation-parse", lambda: f.xml_result(raw_xml)) or {}
        def assess_business():
            try:
                value = business_result(f, raw_xml, observation)
                return None if value is None else True
            except ValueError as exception:
                record_error(exception, "six-case-object-xml-assessment")
                return False  # Complete observed evidence actually disagrees.
        business_ok = attempt("six-case-object-xml-assessment", assess_business)
    after = attempt("post-input-pins", lambda: f.observe_inputs(expected, before))
    unchanged = attempt("post-input-assessment", lambda: None if after is None else
                        observed_all(*[observed_all(item.get("matchesPlan"), item.get("unchanged")) for item in after])
                        if after else False)
    current_members = attempt("post-pester-membership", lambda: f.pester_members(plan))
    same_members = None if current_members is None else current_members == members
    devices_after = attempt("post-dos-devices", f.devices)
    same_devices = None if devices_before is None or devices_after is None else devices_after == devices_before
    current_temp = attempt("post-temp-identity", lambda: f.native_identity(f.physical(OUTPUT / "temp")))
    temp_children = attempt("post-temp-children", lambda: sorted(p.name for p in f.physical(OUTPUT / "temp").iterdir()))
    def assess_fixtures():
        if (original_temp is None or current_temp is None or temp_children is None or
                original_temp.get("fileIdentity") is None or current_temp.get("fileIdentity") is None or
                type(observation.get("testDrivePresentAfterInvoke")) is not bool):
            return None
        return (current_temp["fileIdentity"] == original_temp["fileIdentity"] and temp_children == [] and
                observation["testDrivePresentAfterInvoke"] is False)
    fixtures_settled = attempt("post-fixture-assessment", assess_fixtures)
    complete = observed_all(streams.get("nativeComplete"), streams.get("bothStreamsComplete"),
                            not streams["errors"] if isinstance(streams.get("errors"), list) else None)
    outputs = {
        "input-pins-after.json": {"inputs": after, "unchanged": unchanged,
            "pesterMembersAfter": current_members, "pesterMembershipUnchanged": same_members},
        "stream-observation.json": streams, "xml-observation.json": xml,
        "fixture-after.json": {"dosDevicesBefore": devices_before, "dosDevices": devices_after,
            "dosDevicesUnchanged": same_devices, "ownedTempBefore": original_temp, "ownedTempAfter": current_temp,
            "testDrivePresentAfterInvoke": observation.get("testDrivePresentAfterInvoke"),
            "ownedTempChildren": temp_children, "ownedTempAndTestDriveSettled": fixtures_settled,
            "foreignMappingDeletionAttempted": False, "fixtureDeletionAttemptedByRecipe": False}}
    for name, value in outputs.items():
        try:
            write(name, value)
        except Exception as exception:
            persistence_errors.append({"path": name, "type": type(exception).__name__, "message": str(exception)})
            record_error(exception, "evidence-persistence:" + name)
    passed = bool(all(fact is True for fact in
                  (complete, identity_ok, business_ok, unchanged, same_members, same_devices, fixtures_settled)) and
                  (native or {}).get("rootExitCode") == 0 and error is None)
    summary = {"requestId": REQUEST, "attemptId": REQUEST, "runId": RUN,
        "job": os.environ["JOB_NAME"], "build": int(build), "outputOwned": output_owned,
        "status": "passed" if passed else "failed" if complete else
            "failed-before-native-start" if dispatched is None else "pending-native-proof",
        "normalSixCaseExecutionPassed": passed, "nativeComplete": complete,
        "nativeExitCode": (native or {}).get("rootExitCode"), "nativeStarts": int(dispatched is not None),
        "originalCounts": observation.get("originalCounts"), "exactObjectAndXmlAgreement": business_ok,
        "childEnvironment": {k: metadata.get(k)
                             for k in ("buildNumber", "jobName", "nodeName", "runId")},
        "runnerCollectorErrors": observation.get("collectorErrors"),
        "sameProcessMetadataMatches": identity_ok, "inputPinsUnchanged": unchanged,
        "pesterMembershipUnchanged": same_members, "dosDevicesUnchanged": same_devices,
        "ownedTempAndTestDriveSettled": fixtures_settled,
        "overallBudgetSecondsIncludingPreflight": 180, "terminalDrainBudgetSeconds": 30,
        "preflightSeconds": dispatched - entered if dispatched is not None else None,
        "nativeCallSecondsIncludingTerminalDrain": returned - dispatched if returned is not None else None,
        "terminalDrainActualSeconds": None, "terminalDrainSeparateClockAvailable": False,
        "producerElapsedSeconds": clock() - entered, "error": error,
        "secondaryErrors": secondary_errors, "persistenceErrors": persistence_errors,
        "jobHandleRetainedUntilCliExit": (native or {}).get("processTree", {}).get("jobHandleRetainedUntilCliExit"),
        "liveHandleDisposalObserved": False, "directCompilerReceipt": False, "formalAcceptance": False,
        "sourceCopies": 0, "canonicalSourceEdits": 0, "filesDeleted": 0, "releasedBytes": 0, "cleanupClosed": False}
    try:
        write("summary.json", summary)
    except Exception as exception:
        record_error(exception, "summary-persistence")
        summary.update(status="failed-summary-persistence", normalSixCaseExecutionPassed=False,
                       error=error, secondaryErrors=secondary_errors)
        summary["summaryPersistenceError"] = {"type": type(exception).__name__, "message": str(exception)}
        print(json.dumps(summary, ensure_ascii=False), file=sys.stderr)
        return 1
    print(json.dumps(summary, ensure_ascii=False))
    return 0 if passed else 1

def main():
    entered = clock()  # Argument parsing is inside the single deadline too.
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--plan-sha", required=True)
    parser.add_argument("--self-sha", required=True)
    args = parser.parse_args()
    try:
        return run(args.plan, args.plan_sha, args.self_sha, entered)
    except Exception as error:
        print(json.dumps({"status": "preflight-failed", "reason": str(error),
                          "normalSixCaseExecutionPassed": False, "formalAcceptance": False}), file=sys.stderr)
        return 1

if __name__ == "__main__":
    raise SystemExit(main())
