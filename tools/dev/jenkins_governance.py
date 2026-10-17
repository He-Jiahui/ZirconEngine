"""Operator transport for the bounded Jenkins host-governance job.

This client never opens State for writing, runs Git mutations, generates source
copies, or deletes artifacts. Jenkins owns every submitted lifecycle operation.
"""
from __future__ import annotations

import argparse
import ast
import base64
import hashlib
import json
from pathlib import Path
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
import zlib


JOB = "zircon-governance"
TASK = "disk-governance-20261015"
BASE = "http://127.0.0.1:18080/"


class Transport:
    def __init__(self, root: Path):
        self.root = root
        value = json.loads((root / ".jenkins/state/credentials.json").read_text(encoding="utf-8"))
        token = value.get("apiToken") or value.get("password")
        if value.get("authenticationMode") != "api-token" or not token or not value.get("username"):
            raise ValueError("configured_jenkins_api_token_required")
        self.authorization = "Basic " + base64.b64encode((value["username"] + ":" + token).encode()).decode()

    def request(self, path, *, method="GET", data=None, content_type=None):
        if path.startswith(("http:", "https:", "//")):
            raise ValueError("local_jenkins_path_required")
        headers = {"Authorization": self.authorization, "Accept": "application/json"}
        if content_type:
            headers["Content-Type"] = content_type
        request = urllib.request.Request(BASE + path.lstrip("/"), data=data, method=method, headers=headers)
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, *args, **kwargs):
                return None
        try:
            with urllib.request.build_opener(NoRedirect()).open(request, timeout=30) as result:
                return result.status, result.read(), dict(result.headers.items())
        except urllib.error.HTTPError as error:
            return error.code, error.read(), dict(error.headers.items())


def manifest(root: Path):
    selected = json.loads((root / ".jenkins/state/deployment/driver.json").read_text(encoding="utf-8"))
    from tools.jenkins.governance.paths import physical, file_fact
    from tools.jenkins.deployment.driver import verify_driver
    from tools.jenkins.processes.identity import current_identity
    verify_driver(physical(selected["root"]), expected_digest=selected["driverDigest"])
    deployment = json.loads((root / ".jenkins/state/deployment-operation.json").read_text(encoding="utf-8"))
    if deployment.get("state") != "running" or not deployment.get("health"):
        raise ValueError("configured_jenkins_runtime_not_ready")
    owner = current_identity(deployment["pid"]).to_dict()
    if str(owner.get("creationTime")) != str(deployment["creationTime"]):
        raise ValueError("jenkins_runtime_process_identity_changed")
    # Pin the actual static import closure, rather than unrelated concurrent
    # Jenkins tasks or their tests. Deferred imports are included as well.
    paths = _implementation_closure(root)
    source = {path.relative_to(root).as_posix(): file_fact(path)["sha256"] for path in paths}
    python = root / ".jenkins/runtime/python/python.exe"
    return {"schemaVersion": 1, "jobName": JOB, "taskId": TASK, "controlRoot": str(root),
            "reportRoot": "E:/cargo-targets/zircon-local/reports/" + TASK,
            "driverDigest": selected["driverDigest"], "sourcePins": source,
            "python": str(python), "pythonSha256": file_fact(python)["sha256"],
            "authorization": {"source": "user", "task": "C/D/E ownership governance, local integration and bounded cleanup",
                              "claimClosure": "explicit central chat response", "push": False}}


def _implementation_closure(root):
    pending = [*sorted((root / "tools/jenkins/governance").glob("*.py")), root / "tools/dev/local_cargo.py"]
    selected = set()
    def include(module):
        if module != "tools" and not module.startswith("tools."):
            return
        components = module.split(".")
        for count in range(1, len(components) + 1):
            base = root.joinpath(*components[:count])
            for candidate in (base.with_suffix(".py"), base / "__init__.py"):
                if candidate.is_file() and candidate not in selected:
                    pending.append(candidate)
    while pending:
        path = pending.pop()
        if path in selected:
            continue
        selected.add(path)
        parts = path.relative_to(root).with_suffix("").parts
        package = ".".join(parts[:-1])
        if parts[-1] == "__init__":
            package = ".".join(parts[:-1])
        for node in ast.walk(ast.parse(path.read_text(encoding="utf-8-sig"), filename=str(path))):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    include(alias.name)
            elif isinstance(node, ast.ImportFrom):
                if node.level:
                    parent = package.split(".")[:len(package.split(".")) - node.level + 1]
                    module = ".".join(parent + ([node.module] if node.module else []))
                else:
                    module = node.module or ""
                include(module)
                for alias in node.names:
                    include(module + "." + alias.name)
    return sorted(selected)


def definition(pins):
    packed = base64.b64encode(zlib.compress(json.dumps(pins, sort_keys=True, separators=(",", ":")).encode())).decode()
    bootstrap = (Path(pins["controlRoot"]) / "tools/jenkins/governance/bootstrap.py").as_posix()
    expected = pins["sourcePins"]["tools/jenkins/governance/bootstrap.py"]
    # Fixed script and base64-only environment transport; request bytes never become shell source.
    ps = ("$ErrorActionPreference = 'Stop'\n"
          f"$bootstrapStream = [IO.File]::OpenRead('{bootstrap}')\n"
          "try { $bootstrapHash = [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($bootstrapStream)).Replace('-', '').ToLowerInvariant() } finally { $bootstrapStream.Dispose() }\n"
          f"if ($bootstrapHash -ne '{expected}') {{ throw 'governance_bootstrap_changed' }}\n"
          f"& '{Path(pins['python']).as_posix()}' -B -X utf8 '{bootstrap}' --from-environment\n"
          "if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }\n")
    script = ("properties([parameters([string(name: 'REQUEST_B64', defaultValue: '', description: 'Fixed governance request in base64')])])\n"
        "node('zircon-windows-agent') {\n"
        "  ws('E:/cargo-targets/zircon-local/jenkins-governance-workspace') {\n"
        "  def request = params.REQUEST_B64 ?: ''\n"
        "  if (!(request ==~ /[A-Za-z0-9+\\/=]{1,500000}/)) { error('invalid_governance_request_transport') }\n"
        f"  withEnv(['ZGOV_PINS={packed}', 'ZGOV_REQUEST=' + request, 'PYTHONDONTWRITEBYTECODE=1']) {{\n"
        "    try { powershell(script: '''" + ps + "''') } finally {\n"
        "      archiveArtifacts(artifacts: 'governance/**/*.json,governance/**/*.md,governance/**/*.bin', allowEmptyArchive: true, fingerprint: true)\n"
        "    }\n"
        "  }\n"
        "  }\n"
        "}\n")
    project = ET.Element("flow-definition", {"plugin": "workflow-job"})
    ET.SubElement(project, "description").text = "Bounded C/D/E ownership governance. One existing State; explicit claim reply before integration or cleanup."
    ET.SubElement(project, "keepDependencies").text = "false"
    properties = ET.SubElement(project, "properties")
    parameters = ET.SubElement(ET.SubElement(properties, "hudson.model.ParametersDefinitionProperty"), "parameterDefinitions")
    parameter = ET.SubElement(parameters, "hudson.model.StringParameterDefinition")
    ET.SubElement(parameter, "name").text = "REQUEST_B64"
    ET.SubElement(parameter, "description").text = "Fixed governance request in base64"
    ET.SubElement(parameter, "defaultValue").text = ""
    ET.SubElement(parameter, "trim").text = "true"
    configured = ET.SubElement(project, "definition", {"class": "org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition", "plugin": "workflow-cps"})
    ET.SubElement(configured, "script").text = script
    ET.SubElement(configured, "sandbox").text = "true"
    ET.SubElement(project, "triggers")
    ET.SubElement(project, "disabled").text = "false"
    return ET.tostring(project, encoding="utf-8", xml_declaration=True)


def _previous_maintenance_hook(prefix, marker):
    """Wait for the original audit and expose its retained review status."""
    template = r'''
__MARKER__
def zirconGovernanceAgent(Closure body) {
  timeout(time: 5, unit: 'MINUTES') {
    node('zircon-windows-agent') { body() }
  }
}
def zirconGovernanceRequest = '__PREFIX__'
def zirconGovernanceB64
zirconGovernanceAgent {
  withEnv(['ZGOV_HOOK_REQUEST=' + zirconGovernanceRequest]) {
    zirconGovernanceB64 = powershell(returnStdout: true, script: """$ErrorActionPreference = 'Stop'
[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($env:ZGOV_HOOK_REQUEST))""").trim()
  }
}
def zirconGovernanceOperation = 'scheduled-' + env.BUILD_NUMBER
def zirconGovernanceReport = [schemaVersion: 1, operationId: zirconGovernanceOperation,
    requestId: zirconGovernanceOperation, attemptId: 'attempt-1', status: 'failed',
    scope: 'registered CAS cleanup followed by retained governance audit', reclaimedBytes: report.reclaimedBytes]
try {
  def zirconGovernanceRun
  timeout(time: 20, unit: 'MINUTES') {
    zirconGovernanceRun = build(job: 'zircon-governance', wait: true, propagate: false,
        parameters: [string(name: 'REQUEST_B64', value: zirconGovernanceB64)])
  }
  zirconGovernanceReport.put('auditBuild', zirconGovernanceRun.number)
  zirconGovernanceReport.put('auditResult', zirconGovernanceRun.result)
  if (zirconGovernanceRun.result != 'SUCCESS') error('Governance audit failed; original request ' + zirconGovernanceOperation)
  zirconGovernanceAgent {
    withEnv(['ZGOV_EXPECTED_OPERATION=' + zirconGovernanceOperation,
             'ZGOV_EXPECTED_BUILD=' + zirconGovernanceRun.number]) {
      def observation = readJson(powershell(returnStdout: true, label: 'Verify original governance audit receipt', script: """$ErrorActionPreference = 'Stop'
$operation = $env:ZGOV_EXPECTED_OPERATION
if ($operation -notmatch '^scheduled-[0-9]+$' -or $env:ZGOV_EXPECTED_BUILD -notmatch '^[0-9]+$') { throw 'Invalid original scheduled audit identity' }
$directory = 'E:/cargo-targets/zircon-local/reports/__TASK__/' + $operation
$receiptPath = Join-Path $directory 'receipt.json'
$reviewPath = Join-Path $directory 'inventory-review.json'
foreach ($path in @($receiptPath, $reviewPath)) {
    $cursor = $path
    while ($cursor) {
        if ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Audit receipt path contains an alias' }
        $cursor = [IO.Path]::GetDirectoryName($cursor)
    }
}
$receipt = Get-Content -LiteralPath $receiptPath -Raw -Encoding UTF8 | ConvertFrom-Json
$identity = $receipt.identity
if ($receipt.action -ne 'audit' -or $identity.taskId -ne '__TASK__' -or
    $identity.sessionId -ne 'jenkins-maintenance' -or $identity.operationId -ne $operation -or
    $identity.requestId -ne $operation -or $identity.attemptId -ne 'attempt-1' -or
    $identity.driverDigest -ne $env:ZIRCON_DRIVER_DIGEST -or
    -not $receipt.finishedAt -or $identity.jenkinsBuildUrl -notmatch ('/job/zircon-governance/' + $env:ZGOV_EXPECTED_BUILD + '/$')) {
    throw 'Audit receipt differs from the original scheduled request/build'
}
$reviewFact = $receipt.result.review
if ([IO.Path]::GetFullPath($reviewFact.path) -ne [IO.Path]::GetFullPath($reviewPath) -or
    (Get-Item -LiteralPath $reviewPath).Length -ne $reviewFact.bytes -or
    (Get-FileHash -LiteralPath $reviewPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $reviewFact.sha256) {
    throw 'Original inventory review bytes changed'
}
$review = Get-Content -LiteralPath $reviewPath -Raw -Encoding UTF8 | ConvertFrom-Json
$reasons = @()
if ($review.missingInventory -or -not $receipt.result.inventoryDigest) { $reasons += 'missing-inventory' }
if ($review.coverage -and $review.coverage.status -ne 'complete') { $reasons += 'incomplete-inventory-coverage' }
if ($review.summary.omissions -gt 0) { $reasons += 'inventory-omissions' }
if ($review.summary.legacyOwners -gt 0) { $reasons += 'unreconciled-legacy-owners' }
if ($receipt.result.status -notin @('complete', 'passed', 'accepted')) { $reasons += [string]$receipt.result.status }
@{status = $(if ($reasons.Count) { 'needs-review' } else { 'complete' }); reasons = $reasons;
  receipt = $receiptPath; receiptSha256 = (Get-FileHash -LiteralPath $receiptPath -Algorithm SHA256).Hash.ToLowerInvariant();
  inventoryDigest = $receipt.result.inventoryDigest; summary = $receipt.result.summary} | ConvertTo-Json -Depth 8 -Compress
"""))
      zirconGovernanceReport.put('audit', observation)
      zirconGovernanceReport.status = observation.status
    }
  }
} catch (err) {
  zirconGovernanceReport.put('error', err.toString())
  throw err
} finally {
  zirconGovernanceAgent {
    ws('E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup') {
      def name = 'scheduled-governance-' + env.BUILD_NUMBER + '.json'
      def ledger = 'scheduled-governance-' + env.BUILD_NUMBER + '.artifacts.json'
      writeFile(file: ledger, text: groovy.json.JsonOutput.toJson([schemaVersion: 1,
          taskId: 'jenkins-scheduled-cleanup', owner: zirconGovernanceOperation,
          artifacts: [[path: pwd() + '/' + name, retainUntil: 'original maintenance and ownership review reconciled']]]))
      writeFile(file: name, text: groovy.json.JsonOutput.prettyPrint(groovy.json.JsonOutput.toJson(zirconGovernanceReport)))
      archiveArtifacts(artifacts: name + ',' + ledger, fingerprint: true)
    }
    if (zirconGovernanceReport.status == 'needs-review') currentBuild.result = 'UNSTABLE'
    currentBuild.description = report.status + '; reclaimed ' + report.reclaimedBytes + ' logical bytes; audit ' + zirconGovernanceReport.status
  }
}
'''
    # Single-quoted Groovy source prevents PowerShell variables from becoming GStrings.
    return template.replace('"""', chr(39) * 3).replace('__MARKER__', marker).replace('__PREFIX__', prefix).replace('__TASK__', TASK)



def maintenance_hook(prefix, marker):
    """The existing audit now consumes only this run's bounded scan."""
    hook = _previous_maintenance_hook(prefix, marker)
    scan = r'''try {
  def scanRequest = readJson(zirconGovernanceRequest)
  def scanOperation = zirconGovernanceOperation + '-inventory'
  scanRequest.put('operationId', scanOperation)
  scanRequest.put('requestId', scanOperation)
  scanRequest.put('action', 'scan')
  scanRequest.put('payload', [inventoryPolicy: 'managed-roots-v1'])
  def scanB64
  zirconGovernanceAgent {
    withEnv(['ZGOV_SCAN_REQUEST=' + groovy.json.JsonOutput.toJson(scanRequest)]) {
      scanB64 = powershell(returnStdout: true, script: """$ErrorActionPreference = 'Stop'
[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($env:ZGOV_SCAN_REQUEST))""").trim()
    }
  }
  def scanRun
  timeout(time: 12, unit: 'MINUTES') {
    scanRun = build(job: 'zircon-governance', wait: true, propagate: false,
        parameters: [string(name: 'REQUEST_B64', value: scanB64)])
  }
  zirconGovernanceReport.put('inventoryBuild', scanRun.number)
  if (scanRun.result != 'SUCCESS') error('Exact bounded governance inventory failed')
  def exactInventory
  zirconGovernanceAgent {
    withEnv(['ZGOV_SCAN_OPERATION=' + scanOperation, 'ZGOV_SCAN_BUILD=' + scanRun.number]) {
      exactInventory = readJson(powershell(returnStdout: true, label: 'Verify exact bounded governance inventory', script: """$ErrorActionPreference = 'Stop'
$operation = $env:ZGOV_SCAN_OPERATION
if ($operation -notmatch '^scheduled-[0-9]+-inventory$' -or $env:ZGOV_SCAN_BUILD -notmatch '^[0-9]+$') { throw 'Invalid bounded inventory identity' }
$directory = 'E:/cargo-targets/zircon-local/reports/__TASK__/' + $operation
$paths = @((Join-Path $directory 'receipt.json'), (Join-Path $directory 'inventory.json'), (Join-Path $directory 'managed-index.json'), (Join-Path $directory 'native-scan/native-receipt.json'))
foreach ($path in $paths) {
    $cursor = $path
    while ($cursor) {
        if ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Bounded inventory path contains an alias' }
        $cursor = [IO.Path]::GetDirectoryName($cursor)
    }
}
$receipt = Get-Content -LiteralPath $paths[0] -Raw -Encoding UTF8 | ConvertFrom-Json
$id = $receipt.identity
$result = $receipt.result
if ($receipt.action -ne 'scan' -or -not $receipt.finishedAt -or $id.taskId -ne '__TASK__' -or
    $id.sessionId -ne 'jenkins-maintenance' -or $id.operationId -ne $operation -or $id.requestId -ne $operation -or
    $id.attemptId -ne 'attempt-1' -or $id.driverDigest -ne $env:ZIRCON_DRIVER_DIGEST -or
    $id.jenkinsBuildUrl -notmatch ('/job/zircon-governance/' + $env:ZGOV_SCAN_BUILD + '/$') -or
    $result.inventoryPolicy -ne 'managed-roots-v1' -or $result.inventoryId -ne $operation -or
    $result.inventoryDigest -notmatch '^[0-9a-f]{64}$' -or $result.binding.driverDigest -ne $id.driverDigest -or
    $result.binding.capsuleDigest -ne $id.capsuleDigest -or
    $result.binding.runtimeOperationId -ne $env:ZIRCON_RUNTIME_OPERATION_ID) { throw 'Bounded inventory request/build/runtime changed' }
$facts = @($result.inventoryRef, $result.managedIndexRef, $result.nativeReceipt)
for ($i = 0; $i -lt $facts.Count; $i++) {
    $path = $paths[$i + 1]
    if ([IO.Path]::GetFullPath($facts[$i].path) -cne [IO.Path]::GetFullPath($path) -or
        (Get-Item -LiteralPath $path).Length -ne $facts[$i].bytes -or
        (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $facts[$i].sha256) { throw 'Bounded inventory retained bytes changed' }
}
$inventory = Get-Content -LiteralPath $paths[1] -Raw -Encoding UTF8 | ConvertFrom-Json
$native = Get-Content -LiteralPath $paths[3] -Raw -Encoding UTF8 | ConvertFrom-Json
if ($inventory.inventoryDigest -cne $result.inventoryDigest -or $inventory.globalInventoryComplete -ne $false -or
    $native.rootExitCode -ne 0 -or $native.processTree.terminal -ne $true -or
    $native.processTree.nativeWaiterJoined -ne $true -or $native.processTree.pipeEOF -ne $true -or
    @($native.terminalErrors).Count -or @($native.nativeJobEvidence.errors).Count -or
    $native.nativeJobEvidence.records.afterClose.activeProcesses -ne 0) { throw 'Bounded inventory native proof invalid' }
@{inventoryId = $result.inventoryId; inventoryDigest = $result.inventoryDigest; inventoryPolicy = $result.inventoryPolicy;
  status = $result.status; receiptSha256 = (Get-FileHash -LiteralPath $paths[0] -Algorithm SHA256).Hash.ToLowerInvariant()} | ConvertTo-Json -Compress
"""))
    }
  }
  zirconGovernanceReport.put('inventory', exactInventory)
  def auditRequest = readJson(zirconGovernanceRequest)
  auditRequest.put('payload', [inventoryPolicy: 'managed-roots-v1',
      inventoryId: exactInventory.inventoryId, inventoryDigest: exactInventory.inventoryDigest])
  zirconGovernanceAgent {
    withEnv(['ZGOV_HOOK_REQUEST=' + groovy.json.JsonOutput.toJson(auditRequest)]) {
      zirconGovernanceB64 = powershell(returnStdout: true, script: """$ErrorActionPreference = 'Stop'
[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($env:ZGOV_HOOK_REQUEST))""").trim()
    }
  }
  def zirconGovernanceRun'''
    scan = scan.replace('"""', chr(39) * 3).replace('__TASK__', TASK)
    hook = hook.replace("try {\n  def zirconGovernanceRun", scan)
    hook = hook.replace("inventoryDigest = $receipt.result.inventoryDigest; summary = $receipt.result.summary",
        "inventoryDigest = $receipt.result.inventoryDigest; inventoryId = $receipt.result.inventoryId; inventoryPolicy = $receipt.result.inventoryPolicy; summary = $receipt.result.summary")
    return hook.replace("      zirconGovernanceReport.put('audit', observation)",
        "      if (observation.inventoryId != exactInventory.inventoryId || observation.inventoryDigest != exactInventory.inventoryDigest || observation.inventoryPolicy != 'managed-roots-v1') error('Audit differs from this exact bounded inventory')\n      zirconGovernanceReport.put('audit', observation)")


def _updated_maintenance_script(current, prefix, marker):
    hook = maintenance_hook(prefix, marker)
    if marker not in current:
        return current.rstrip() + hook
    if current.count(marker) != 1:
        raise ValueError("ambiguous_scheduled_governance_hook")
    if current.endswith(hook):
        return None
    previous = _previous_maintenance_hook(prefix, marker)
    if not current.endswith(previous):
        raise ValueError("unreviewed_scheduled_governance_hook")
    return current[:-len(previous)] + hook


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default="E:/Git/ZirconEngine")
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("install")
    commands.add_parser("status")
    commands.add_parser("attach-maintenance")
    submit = commands.add_parser("submit")
    submit.add_argument("--action", required=True)
    submit.add_argument("--operation-id", required=True)
    submit.add_argument("--session-id", default="01a13b17-3395-72b1-8100-6cc3f5400796")
    submit.add_argument("--payload-base64", default="e30=")
    submit.add_argument("--attempt-id", default="attempt-1")
    submit.add_argument("--request-id", help="Original request identity when a proven terminal attempt is followed by a new attempt")
    build = commands.add_parser("build")
    build.add_argument("--number", type=int, required=True)
    args = parser.parse_args()
    root = Path(args.repo_root)
    transport = Transport(root)
    if args.command == "install":
        pins = manifest(root)
        xml = definition(pins)
        status, _, _ = transport.request("job/" + JOB + "/config.xml")
        path = "job/" + JOB + "/config.xml" if status == 200 else "createItem?name=" + JOB
        status, body, _ = transport.request(path, method="POST", data=xml, content_type="application/xml")
        if status not in {200, 201, 302}:
            raise ValueError("governance_job_install_failed:" + str(status))
        status, live, _ = transport.request("job/" + JOB + "/config.xml")
        installed = ET.fromstring(live).findtext("definition/script")
        if status != 200 or installed != ET.fromstring(xml).findtext("definition/script"):
            raise ValueError("governance_job_readback_mismatch")
        print(json.dumps({"status": "installed", "job": BASE + "job/" + JOB + "/", "taskId": TASK,
                          "pinnedFiles": len(pins["sourcePins"]), "driverDigest": pins["driverDigest"]}))
    elif args.command == "attach-maintenance":
        path = "job/zircon-scheduled-cleanup/config.xml"
        status, previous, _ = transport.request(path)
        if status != 200:
            raise ValueError("existing_scheduled_maintenance_unavailable")
        document = ET.fromstring(previous)
        script = document.find("definition/script")
        if script is None:
            raise ValueError("existing_scheduled_maintenance_script_unavailable")
        marker = "// ZIRCON HOST GOVERNANCE AUDIT"
        prefix = ("{\"taskId\":\"" + TASK + "\",\"sessionId\":\"jenkins-maintenance\","
                  "\"requestId\":\"scheduled-' + env.BUILD_NUMBER + '\","
                  "\"operationId\":\"scheduled-' + env.BUILD_NUMBER + '\","
                  "\"attemptId\":\"attempt-1\",\"action\":\"audit\",\"payload\":{}}")
        updated_script = _updated_maintenance_script(script.text or "", prefix, marker)
        if updated_script is None:
            print(json.dumps({"status": "already-attached", "job": "zircon-scheduled-cleanup"}))
            return
        script.text = updated_script
        status, latest, _ = transport.request(path)
        if status != 200 or latest != previous:
            raise ValueError("scheduled_maintenance_changed_during_review")
        updated = ET.tostring(document, encoding="utf-8", xml_declaration=True)
        status, _, _ = transport.request(path, method="POST", data=updated, content_type="application/xml")
        if status not in {200, 201, 302}:
            raise ValueError("scheduled_governance_hook_install_failed")
        status, live, _ = transport.request(path)
        if status != 200 or ET.fromstring(live).findtext("definition/script") != script.text:
            raise ValueError("scheduled_governance_hook_readback_mismatch")
        print(json.dumps({"status": "attached", "job": "zircon-scheduled-cleanup", "action": "audit",
                          "previousDefinitionSha256": hashlib.sha256(previous).hexdigest(),
                          "currentDefinitionSha256": hashlib.sha256(live).hexdigest()}))
    elif args.command == "submit":
        from tools.jenkins.governance.worker import ACTIONS, ID
        if args.action not in ACTIONS or not ID.fullmatch(args.operation_id) or not ID.fullmatch(args.session_id) or not ID.fullmatch(args.attempt_id):
            raise ValueError("invalid_governance_request")
        if args.request_id and not ID.fullmatch(args.request_id):
            raise ValueError("invalid_original_request_id")
        request = {"taskId": TASK, "sessionId": args.session_id, "requestId": args.request_id or args.operation_id,
                   "operationId": args.operation_id, "attemptId": args.attempt_id, "action": args.action,
                   "payload": json.loads(base64.b64decode(args.payload_base64, validate=True))}
        data = urllib.parse.urlencode({"REQUEST_B64": base64.b64encode(json.dumps(request).encode()).decode()}).encode()
        status, _, headers = transport.request("job/" + JOB + "/buildWithParameters", method="POST", data=data,
                                             content_type="application/x-www-form-urlencoded")
        if status not in {200, 201, 302}:
            raise ValueError("governance_submit_failed:" + str(status))
        print(json.dumps({"status": "queued", "operationId": args.operation_id, "queueUrl": headers.get("Location")}))
    else:
        path = "job/" + JOB + (f"/{args.number}/api/json" if args.command == "build" else "/api/json")
        status, body, _ = transport.request(path)
        if status != 200:
            raise ValueError("governance_query_failed:" + str(status))
        value = json.loads(body)
        keys = ("building", "result", "number", "url", "duration", "actions") if args.command == "build" else (
            "name", "color", "lastBuild", "lastCompletedBuild", "queueItem", "inQueue")
        print(json.dumps({key: value.get(key) for key in keys}))


if __name__ == "__main__":
    main()
