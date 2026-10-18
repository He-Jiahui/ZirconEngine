"""Exercise official installer extension functions in Windows PowerShell, without installing."""
from __future__ import annotations

import base64
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[3]
INSTALLER = ROOT / "tools/dev/install-jenkins-management.ps1"
ADDON = ROOT / "tools/jenkins-management/checkout-commit.groovy"
DELIVERY_ADDON = ROOT / "tools/jenkins-management/delivery.groovy"
BRANCH_ADDON = ROOT / "tools/jenkins-management/branch-integration.groovy"
MARKER = "ZR_INSTALLER_EXTENSION_RESULTS:"
OLD_CORE = "// pinned old core\n"
NEW_CORE = "// pinned new core\n"


def _command(cases, raw, delivery_raw, branch_raw=b""):
    path = base64.b64encode(str(INSTALLER).encode()).decode()
    payload = base64.b64encode(json.dumps(cases, separators=(",", ":")).encode()).decode()
    addon = base64.b64encode(raw).decode()
    delivery_addon = base64.b64encode(delivery_raw).decode()
    branch_addon = base64.b64encode(branch_raw).decode()
    return r"""
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSEdition -ne 'Desktop') { throw 'Windows PowerShell Desktop is required' }
$path = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('__PATH__'))
$tokens = $null
$errors = $null
$tree = [Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$errors)
if ($null -eq $tree -or $errors.Count) { throw 'Canonical installer did not parse' }
$names = @('Hash-Bytes','Get-ManagementJobExtensionPrefix','Join-ManagementJobExtension','Assert-ManagementJob')
$definitions = ''
foreach ($name in $names) {
    $nodes = @($tree.FindAll({
        param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq $name
    }, $true))
    if ($nodes.Count -ne 1) { throw ('Expected one official function: ' + $name) }
    $definitions += $nodes[0].Extent.Text + [Environment]::NewLine
}
. ([ScriptBlock]::Create($definitions))
function New-ExtensionCaseJob([string]$Script) {
    $parameters = ''
    foreach ($name in @('SESSION_ID','REQUEST_ID','OPERATION_ID','ACTION','INPUT_JSON','EXPECTED_DRIVER_DIGEST','RUNTIME_OPERATION_ID')) {
        $default = if ($name -ceq 'EXPECTED_DRIVER_DIGEST') { 'driver-fixture' } elseif ($name -ceq 'RUNTIME_OPERATION_ID') { 'runtime-fixture' } else { '' }
        $parameters += "<hudson.model.StringParameterDefinition><name>$name</name><defaultValue>$default</defaultValue><trim>true</trim></hudson.model.StringParameterDefinition>"
    }
    $parameters += '<hudson.model.BooleanParameterDefinition><name>QUERY_ONLY</name><defaultValue>false</defaultValue></hudson.model.BooleanParameterDefinition>'
    $escaped = [Security.SecurityElement]::Escape($Script)
    return [xml]"<flow-definition><actions/><description>Bounded Jenkins management MVP; no M8 acceptance</description><keepDependencies>false</keepDependencies><properties><hudson.model.ParametersDefinitionProperty><parameterDefinitions>$parameters</parameterDefinitions></hudson.model.ParametersDefinitionProperty></properties><definition class='org.jenkinsci.plugins.workflow.cps.CpsFlowDefinition' plugin='workflow-cps'><script>$escaped</script><sandbox>true</sandbox></definition><triggers/><disabled>false</disabled></flow-definition>"
}
$branchBaseBytes = [Convert]::FromBase64String('__BRANCH__')
$branchBaseText = [Text.Encoding]::UTF8.GetString($branchBaseBytes).TrimStart([char]0xfeff).Replace(([string][char]13 + [char]10), [string][char]10)
$deliveryBaseBytes = [Convert]::FromBase64String('__DELIVERY__')
$deliveryBaseText = [Text.Encoding]::UTF8.GetString($deliveryBaseBytes).TrimStart([char]0xfeff).Replace(([string][char]13 + [char]10), [string][char]10)
$baseBytes = [Convert]::FromBase64String('__ADDON__')
$baseText = [Text.Encoding]::UTF8.GetString($baseBytes)
if ($baseText.StartsWith([string][char]0xfeff, [StringComparison]::Ordinal)) { $baseText = $baseText.Substring(1) }
$baseText = $baseText.Replace(([string][char]13 + [char]10), [string][char]10)
$cases = ConvertFrom-Json ([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('__CASES__')))
$rows = New-Object 'System.Collections.Generic.List[object]'
foreach ($case in $cases) {
    try {
        [byte[]]$bytes = $baseBytes
        switch ([string]$case.bytes) {
            'changed' { $bytes = [Text.Encoding]::UTF8.GetBytes($baseText + '// drift') }
            'duplicate' { $bytes = [Text.Encoding]::UTF8.GetBytes($baseText + $baseText) }
            'leading' { $bytes = [Text.Encoding]::UTF8.GetBytes('// foreign' + [char]10 + $baseText) }
            'missing-end' { $bytes = [Text.Encoding]::UTF8.GetBytes($baseText.Replace('// END ZIRCON SCOPED CHECKOUT COMMIT','')) }
            'crlf' { $bytes = [Text.Encoding]::UTF8.GetBytes($baseText.Replace([string][char]10, ([string][char]13 + [char]10))) }
            'bom-crlf' { $bytes = [byte[]](@(239,187,191) + [Text.Encoding]::UTF8.GetBytes($baseText.Replace([string][char]10, ([string][char]13 + [char]10)))) }
            'trailing-newline' { $bytes = [byte[]](@($baseBytes) + @(10)) }
            'invalid-utf8' { $bytes = [byte[]]@(255) }
        }
        $pin = [string]$case.pin
        if ($case.branch -eq $true -and $null -eq $case.pin) { $pin = Hash-Bytes $baseBytes }
        if ($case.pinMode -ceq 'actual') { $pin = Hash-Bytes $bytes }
        $prefix = ''
        if ($case.addon -eq $true) {
            $prefix = Get-ManagementJobExtensionPrefix -AddonBytes $bytes -ExpectedSha256 $pin
        }
        $checkoutPrefix = $prefix
        $deliveryPrefix = ''
        if ($case.delivery -eq $true) {
            [byte[]]$deliveryBytes = $deliveryBaseBytes
            switch ([string]$case.deliveryBytes) {
                'changed' { $deliveryBytes = [Text.Encoding]::UTF8.GetBytes($deliveryBaseText + '// drift') }
                'duplicate' { $deliveryBytes = [Text.Encoding]::UTF8.GetBytes($deliveryBaseText + $deliveryBaseText) }
                'missing-end' { $deliveryBytes = [Text.Encoding]::UTF8.GetBytes($deliveryBaseText.Replace('// END ZIRCON VALIDATED DELIVERY','')) }
                'crlf' { $deliveryBytes = [Text.Encoding]::UTF8.GetBytes($deliveryBaseText.Replace([string][char]10, ([string][char]13 + [char]10))) }
            }
            $deliveryPin = [string]$case.deliveryPin
            if ($case.branch -eq $true -and $null -eq $case.deliveryPin) { $deliveryPin = Hash-Bytes $deliveryBaseBytes }
            if ($case.deliveryPinMode -ceq 'actual') { $deliveryPin = Hash-Bytes $deliveryBytes }
            $deliveryPrefix = Get-ManagementJobExtensionPrefix -AddonBytes $deliveryBytes -ExpectedSha256 $deliveryPin -Kind 'delivery'
            $prefix = $deliveryPrefix + $checkoutPrefix
        }
        $branchPrefix = ''
        if ($case.branch -eq $true) {
            [byte[]]$branchBytes = $branchBaseBytes
            switch ([string]$case.branchBytes) {
                'changed' { $branchBytes = [Text.Encoding]::UTF8.GetBytes($branchBaseText + '// drift') }
                'duplicate' { $branchBytes = [Text.Encoding]::UTF8.GetBytes($branchBaseText + $branchBaseText) }
                'missing-end' { $branchBytes = [Text.Encoding]::UTF8.GetBytes($branchBaseText.Replace('// END ZIRCON BRANCH QUEUE','')) }
            }
            $branchPin = [string]$case.branchPin
            if ($null -eq $case.branchPin) { $branchPin = Hash-Bytes $branchBaseBytes }
            if ($case.branchPinMode -ceq 'actual') { $branchPin = Hash-Bytes $branchBytes }
            $branchPrefix = Get-ManagementJobExtensionPrefix -AddonBytes $branchBytes -ExpectedSha256 $branchPin -Kind 'branch-integration'
            $prefix = $branchPrefix + $deliveryPrefix + $checkoutPrefix
        }
        $expected = Join-ManagementJobExtension -Core ([string]$case.core) -Prefix $prefix
        $actual = $expected
        switch ([string]$case.actual) {
            'missing-branch' { $actual = $deliveryPrefix + $checkoutPrefix + [string]$case.core }
            'duplicate-branch' { $actual = $branchPrefix + $expected }
            'moved-branch' { $actual = $deliveryPrefix + $branchPrefix + $checkoutPrefix + [string]$case.core }
            'missing-delivery' { $actual = $checkoutPrefix + [string]$case.core }
            'duplicate-delivery' { $actual = $deliveryPrefix + $expected }
            'moved-delivery' { $actual = $checkoutPrefix + $deliveryPrefix + [string]$case.core }
            'bare' { $actual = [string]$case.core }
            'duplicate' { $actual = $prefix + $expected }
            'suffix' { $actual = [string]$case.core + $prefix }
            'leading' { $actual = '// foreign' + [char]10 + $expected }
            'changed-core' { $actual = $expected + '// foreign core' + [char]10 }
            'base-addon' {
                $actual = (Get-ManagementJobExtensionPrefix -AddonBytes $baseBytes -ExpectedSha256 (Hash-Bytes $baseBytes)) + [string]$case.core
            }
        }
        $job = New-ExtensionCaseJob $actual
        if ($case.tamper -ceq 'disabled') { $job.'flow-definition'.disabled = 'true' }
        if ($case.tamper -ceq 'sandbox') { $job.'flow-definition'.definition.sandbox = 'false' }
        Assert-ManagementJob $job $expected 'driver-fixture' 'runtime-fixture'
        if ($case.resume -eq $true) {
            $old = Join-ManagementJobExtension -Core ('// pinned old core' + [char]10) -Prefix $prefix
            $archive = New-ExtensionCaseJob $old
            if ($case.archive -ceq 'bare') { $archive = New-ExtensionCaseJob ('// pinned old core' + [char]10) }
            if ($case.archive -ceq 'missing-delivery') { $archive = New-ExtensionCaseJob ($checkoutPrefix + '// pinned old core' + [char]10) }
            if ($case.archive -ceq 'missing-branch') { $archive = New-ExtensionCaseJob ($deliveryPrefix + $checkoutPrefix + '// pinned old core' + [char]10) }
            Assert-ManagementJob $archive $old 'driver-fixture' 'runtime-fixture'
        }
        $rows.Add([ordered]@{id=[string]$case.id;status='accepted';value=$expected;error=$null}) | Out-Null
    } catch {
        $rows.Add([ordered]@{id=[string]$case.id;status='rejected';value=$null;error=$_.Exception.Message}) | Out-Null
    }
}
[Console]::Out.WriteLine('ZR_INSTALLER_EXTENSION_RESULTS:' + (ConvertTo-Json -InputObject @($rows.ToArray()) -Depth 5 -Compress))
""".replace("__PATH__", path).replace("__ADDON__", addon).replace("__DELIVERY__", delivery_addon).replace("__BRANCH__", branch_addon).replace("__CASES__", payload)


class InstallerExtensionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        raw = ADDON.read_bytes()
        pin = hashlib.sha256(raw).hexdigest()
        delivery_raw = DELIVERY_ADDON.read_bytes() if getattr(cls, "delivery_only", False) or getattr(cls, "branch_only", False) else b""
        branch_raw = BRANCH_ADDON.read_bytes() if getattr(cls, "branch_only", False) else b""
        branch_pin = hashlib.sha256(branch_raw).hexdigest()
        cls.branch_prefix = branch_raw.decode("utf-8-sig").replace("\r\n", "\n").rstrip("\r\n") + "\n\n"
        delivery_pin = hashlib.sha256(delivery_raw).hexdigest()
        cls.delivery_prefix = delivery_raw.decode("utf-8-sig").replace("\r\n", "\n").rstrip("\r\n") + "\n\n"
        cls.prefix = raw.decode("utf-8-sig").replace("\r\n", "\n").rstrip("\r\n") + "\n\n"

        def case(identifier, **overrides):
            value = dict(id=identifier, addon=True, pin=pin, core=OLD_CORE)
            value.update(overrides)
            return value

        def delivery(identifier, **overrides):
            value = dict(delivery=True, deliveryPin=delivery_pin)
            value.update(overrides)
            return case(identifier, **value)

        cases = [
            dict(id="plain", addon=False, core=OLD_CORE),
            dict(id="plain-rejects-addon", addon=False, core=OLD_CORE, actual="base-addon"),
            case("old"),
            case("new", core=NEW_CORE),
            case("resume", core=NEW_CORE, resume=True),
            case("resume-wrong-archive", core=NEW_CORE, resume=True, archive="bare"),
            case("missing", actual="bare"),
            case("duplicate", actual="duplicate"),
            case("suffix", actual="suffix"),
            case("leading", actual="leading"),
            case("changed-core", actual="changed-core"),
            case("disabled", tamper="disabled"),
            case("sandbox", tamper="sandbox"),
            case("source-drift", bytes="changed"),
            case("wrong-pin", pin="0" * 64),
            case("empty-pin", pin=""),
            case("invalid-template", bytes="missing-end", pinMode="actual"),
            case("duplicate-template", bytes="duplicate", pinMode="actual"),
            case("leading-template", bytes="leading", pinMode="actual"),
            case("invalid-utf8", bytes="invalid-utf8", pinMode="actual"),
            case("crlf", bytes="crlf", pinMode="actual"),
            case("bom-crlf", bytes="bom-crlf", pinMode="actual"),
            case("raw-newline-drift", bytes="trailing-newline"),
            case("nested-core", core=cls.prefix + OLD_CORE),
            delivery("delivery-old"),
            delivery("delivery-new", core=NEW_CORE),
            delivery("delivery-resume", core=NEW_CORE, resume=True),
            delivery("delivery-crlf", deliveryBytes="crlf", deliveryPinMode="actual"),
            delivery("delivery-missing", actual="missing-delivery"),
            delivery("delivery-duplicate", actual="duplicate-delivery"),
            delivery("delivery-moved", actual="moved-delivery"),
            delivery("delivery-source-drift", deliveryBytes="changed"),
            delivery("delivery-wrong-pin", deliveryPin="0" * 64),
            delivery("delivery-incomplete", deliveryBytes="missing-end", deliveryPinMode="actual"),
            delivery("delivery-duplicate-template", deliveryBytes="duplicate", deliveryPinMode="actual"),
            delivery("delivery-nested-core", core=cls.delivery_prefix + OLD_CORE),
            delivery("delivery-resume-wrong-archive", core=NEW_CORE, resume=True, archive="missing-delivery"),
        ]
        if getattr(cls, "branch_only", False):
            def branch(identifier, **overrides):
                value = dict(branch=True, branchPin=branch_pin)
                value.update(overrides)
                row = delivery(identifier, **value)
                for field in ("pin", "deliveryPin", "branchPin"):
                    row.pop(field, None)
                return row
            cases = [
                branch("branch-old"),
                branch("branch-new", core=NEW_CORE),
                branch("branch-resume", core=NEW_CORE, resume=True),
                branch("branch-missing", actual="missing-branch"),
                branch("branch-duplicate", actual="duplicate-branch"),
                branch("branch-moved", actual="moved-branch"),
                branch("branch-source-drift", branchBytes="changed"),
                branch("branch-incomplete", branchBytes="missing-end", branchPinMode="actual"),
                branch("branch-duplicate-template", branchBytes="duplicate", branchPinMode="actual"),
                branch("branch-nested-core", core=cls.branch_prefix + OLD_CORE),
                branch("branch-api-drift", actual="changed-core"),
                branch("branch-resume-wrong-archive", core=NEW_CORE, resume=True, archive="missing-branch"),
            ]
        elif getattr(cls, "delivery_only", False):
            cases = [row for row in cases if row.get("delivery") is True]
        else:
            cases = [row for row in cases if row.get("delivery") is not True]
        powershell = shutil.which("powershell.exe") or shutil.which("powershell")
        if powershell is None:
            raise RuntimeError("Windows PowerShell Desktop is required; this is not a passing skip")
        command = _command(cases, raw, delivery_raw, branch_raw)
        if len(command) > 24000:
            raise RuntimeError("Installer extension fixture exceeded the bounded native command")
        completed = subprocess.run(
            [powershell, "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", command],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
            timeout=120, check=False,
        )
        if completed.returncode:
            raise RuntimeError(f"Official installer functions failed: {completed.stderr.strip()}")
        lines = completed.stdout.splitlines()
        if len(lines) != 1 or not lines[0].startswith(MARKER):
            raise RuntimeError("Installer extension fixture produced no unique completion marker")
        rows = json.loads(lines[0][len(MARKER):])
        cls.rows = {row["id"]: row for row in rows}
        if len(rows) != len(cases) or set(cls.rows) != {case["id"] for case in cases}:
            raise RuntimeError("Installer extension fixture returned an incomplete case set")

    def accepted(self, identifier, expected):
        row = self.rows[identifier]
        self.assertEqual(row["status"], "accepted", row)
        self.assertEqual(row["value"], expected, row)
        self.assertIsNone(row["error"], row)

    def rejected(self, *identifiers):
        for identifier in identifiers:
            with self.subTest(identifier=identifier):
                row = self.rows[identifier]
                self.assertEqual(row["status"], "rejected", row)
                self.assertIsNone(row["value"], row)
                self.assertTrue(row["error"], row)

    def test_plain_installation_stays_exact_and_does_not_drop_an_unpinned_addon(self):
        self.accepted("plain", OLD_CORE)
        self.rejected("plain-rejects-addon")

    def test_extension_is_preserved_across_old_new_and_prepared_resume_jobs(self):
        self.accepted("old", self.prefix + OLD_CORE)
        self.accepted("new", self.prefix + NEW_CORE)
        self.accepted("resume", self.prefix + NEW_CORE)
        self.rejected("resume-wrong-archive")

    def test_missing_duplicate_moved_or_changed_routes_and_job_settings_are_rejected(self):
        self.rejected("missing", "duplicate", "suffix", "leading", "changed-core", "disabled", "sandbox", "nested-core")

    def test_raw_source_pin_and_complete_unique_template_are_required(self):
        self.rejected("source-drift", "wrong-pin", "empty-pin", "invalid-template", "duplicate-template", "leading-template", "invalid-utf8")

    def test_bom_and_crlf_normalization_preserves_script_but_not_raw_source_pins(self):
        self.accepted("crlf", self.prefix + OLD_CORE)
        self.accepted("bom-crlf", self.prefix + OLD_CORE)
        self.rejected("raw-newline-drift")

class DeliveryInstallerExtensionTests(unittest.TestCase):
    delivery_only = True
    setUpClass = classmethod(InstallerExtensionTests.setUpClass.__func__)
    accepted = InstallerExtensionTests.accepted
    rejected = InstallerExtensionTests.rejected

    def test_delivery_and_checkout_extensions_preserve_old_new_and_resumed_jobs(self):
        prefix = self.delivery_prefix + self.prefix
        self.accepted("delivery-old", prefix + OLD_CORE)
        self.accepted("delivery-new", prefix + NEW_CORE)
        self.accepted("delivery-resume", prefix + NEW_CORE)
        self.accepted("delivery-crlf", prefix + OLD_CORE)

    def test_delivery_route_loss_duplication_source_drift_and_wrong_archive_are_rejected(self):
        self.rejected(
            "delivery-missing", "delivery-duplicate", "delivery-moved",
            "delivery-source-drift", "delivery-wrong-pin", "delivery-incomplete",
            "delivery-duplicate-template", "delivery-nested-core",
            "delivery-resume-wrong-archive",
        )


class BranchIntegrationInstallerExtensionTests(unittest.TestCase):
    branch_only = True
    setUpClass = classmethod(InstallerExtensionTests.setUpClass.__func__)
    accepted = InstallerExtensionTests.accepted
    rejected = InstallerExtensionTests.rejected

    def test_branch_delivery_checkout_preserve_old_new_resume_and_reject_drift(self):
        prefix = self.branch_prefix + self.delivery_prefix + self.prefix
        self.accepted("branch-old", prefix + OLD_CORE)
        self.accepted("branch-new", prefix + NEW_CORE)
        self.accepted("branch-resume", prefix + NEW_CORE)
        self.rejected(
            "branch-missing", "branch-duplicate", "branch-moved", "branch-source-drift",
            "branch-incomplete", "branch-duplicate-template", "branch-nested-core",
            "branch-api-drift", "branch-resume-wrong-archive",
        )

        retained = {
            "prefix": ("// BEGIN USER SYNC CLEAN CHECKOUT 20261018\n"
                       "// Preserve <opaque> & quoted \"value\" and 'value'\n"
                       "// END USER SYNC CLEAN CHECKOUT 20261018\n\n"
                       "// BEGIN ZIRCON REMOTE MAIN SYNC\n// existing remote route\n// END ZIRCON REMOTE MAIN SYNC\n\n"
                       "// BEGIN ZIRCON OLD BRANCH CLEANUP\n// existing cleanup route\n// END ZIRCON OLD BRANCH CLEANUP\n\n"
                       "// BEGIN ZIRCON BRANCH QUEUE\n"
                       "def existingBranch = 'E:/Git/ZirconEngine/.jenkins/runtime/branch-integration/"
                       "5abace38210c7aadd34624f6ad8f6f6f20e8fdaaea1a87c6e0aab192b09b38d7/branch_integration.py'\n"
                       "// END ZIRCON BRANCH QUEUE\n\n"
                       "// BEGIN ZIRCON VALIDATED DELIVERY\n// retained delivery\n// END ZIRCON VALIDATED DELIVERY\n\n"
                       "// BEGIN ZIRCON SCOPED CHECKOUT COMMIT\n// retained checkout\n// END ZIRCON SCOPED CHECKOUT COMMIT\n\n"),
            "oldCore": OLD_CORE, "newCore": NEW_CORE,
        }
        retained["prefixSha256"] = hashlib.sha256(retained["prefix"].encode()).hexdigest()
        payload = base64.b64encode(json.dumps(retained, separators=(",", ":")).encode()).decode()
        command = _command([], b"", b"")
        old_names = "$names = @('Hash-Bytes','Get-ManagementJobExtensionPrefix','Join-ManagementJobExtension','Assert-ManagementJob')"
        new_names = old_names[:-1] + ",'Get-ManagementJobRetainedPrefix','Set-ManagementJobPipeline')"
        self.assertEqual(1, command.count(old_names))
        command = command.replace(old_names, new_names, 1)
        marker = "[Console]::Out.WriteLine('ZR_INSTALLER_EXTENSION_RESULTS:' + (ConvertTo-Json -InputObject @($rows.ToArray()) -Depth 5 -Compress))"
        self.assertEqual(1, command.count(marker))
        command = command.replace(marker, "", 1) + r"""
$inputCase = ConvertFrom-Json ([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('__RETAINED__')))
$oldPipeline = [string]$inputCase.prefix + [string]$inputCase.oldCore
$newPipeline = [string]$inputCase.prefix + [string]$inputCase.newCore
$api = New-ExtensionCaseJob $oldPipeline
$retainedHome = New-ExtensionCaseJob $oldPipeline
$observed = Get-ManagementJobRetainedPrefix -ApiScript ([string]$api.'flow-definition'.definition.script) -HomeScript ([string]$retainedHome.'flow-definition'.definition.script) -BareCore ([string]$inputCase.oldCore) -ExpectedSha256 ([string]$inputCase.prefixSha256)
if ($observed -cne [string]$inputCase.prefix) { throw 'The complete prefix was not retained' }
$beforeXml = '<!-- preserve non-script bytes -->' + $api.OuterXml
$afterXml = Set-ManagementJobPipeline -XmlText $beforeXml -Pipeline $newPipeline
[xml]$after = $afterXml
Assert-ManagementJob $api $oldPipeline 'driver-fixture' 'runtime-fixture'
Assert-ManagementJob $retainedHome $oldPipeline 'driver-fixture' 'runtime-fixture'
Assert-ManagementJob $after $newPipeline 'driver-fixture' 'runtime-fixture'
$beforeShell = [regex]::Replace($beforeXml, '(?s)(<script>).*?(</script>)', '$1__SCRIPT__$2')
$afterShell = [regex]::Replace($afterXml, '(?s)(<script>).*?(</script>)', '$1__SCRIPT__$2')
if ($beforeShell -cne $afterShell) { throw 'The non-script configuration changed' }
$again = Get-ManagementJobRetainedPrefix -ApiScript ([string]$after.'flow-definition'.definition.script) -HomeScript ([string]$after.'flow-definition'.definition.script) -BareCore ([string]$inputCase.newCore) -ExpectedSha256 ([string]$inputCase.prefixSha256)
if ($again -cne $observed) { throw 'The new core changed the original prefix' }
$rejected = New-Object 'System.Collections.Generic.List[string]'
function Assert-RetainedCaseRejected([string]$Id, [scriptblock]$Case) {
    $failed = $false
    try { $null = & $Case } catch { $failed = $true }
    if (-not $failed) { throw ('Accepted invalid retained-prefix case: ' + $Id) }
    $rejected.Add($Id)
}
Assert-RetainedCaseRejected 'prefix-drift' {
    Get-ManagementJobRetainedPrefix -ApiScript ('// drift' + $oldPipeline) -HomeScript ('// drift' + $oldPipeline) -BareCore ([string]$inputCase.oldCore) -ExpectedSha256 ([string]$inputCase.prefixSha256)
}
Assert-RetainedCaseRejected 'wrong-prefix-pin' {
    Get-ManagementJobRetainedPrefix -ApiScript $oldPipeline -HomeScript $oldPipeline -BareCore ([string]$inputCase.oldCore) -ExpectedSha256 ('0' * 64)
}
Assert-RetainedCaseRejected 'api-home-drift' {
    Get-ManagementJobRetainedPrefix -ApiScript $oldPipeline -HomeScript ($oldPipeline + '// drift') -BareCore ([string]$inputCase.oldCore) -ExpectedSha256 ([string]$inputCase.prefixSha256)
}
Assert-RetainedCaseRejected 'missing-core' {
    Get-ManagementJobRetainedPrefix -ApiScript ([string]$inputCase.prefix) -HomeScript ([string]$inputCase.prefix) -BareCore ([string]$inputCase.oldCore) -ExpectedSha256 ([string]$inputCase.prefixSha256)
}
Assert-RetainedCaseRejected 'duplicate-core' {
    Get-ManagementJobRetainedPrefix -ApiScript ($oldPipeline + [string]$inputCase.oldCore) -HomeScript ($oldPipeline + [string]$inputCase.oldCore) -BareCore ([string]$inputCase.oldCore) -ExpectedSha256 ([string]$inputCase.prefixSha256)
}
Assert-RetainedCaseRejected 'core-is-not-suffix' {
    Get-ManagementJobRetainedPrefix -ApiScript ($oldPipeline + '// suffix') -HomeScript ($oldPipeline + '// suffix') -BareCore ([string]$inputCase.oldCore) -ExpectedSha256 ([string]$inputCase.prefixSha256)
}
Assert-RetainedCaseRejected 'duplicate-script-element' {
    Set-ManagementJobPipeline -XmlText ($beforeXml + '<script>foreign</script>') -Pipeline $newPipeline
}
[Console]::Out.WriteLine('ZR_RETAINED_PREFIX_RESULTS:' + (ConvertTo-Json -Depth 5 -Compress ([ordered]@{status='passed';prefix=$observed;nonScriptBytesEqual=$true;rejected=@($rejected.ToArray())})))
""".replace("__RETAINED__", payload)
        powershell = shutil.which("powershell.exe") or shutil.which("powershell")
        self.assertIsNotNone(powershell, "Windows PowerShell Desktop is required")
        self.assertLessEqual(len(command), 24000)
        completed = subprocess.run(
            [powershell, "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", command],
            capture_output=True, text=True, encoding="utf-8", errors="replace",
            timeout=120, check=False,
        )
        self.assertEqual(0, completed.returncode, completed.stderr)
        lines = completed.stdout.splitlines()
        self.assertEqual(1, len(lines), completed.stdout)
        self.assertTrue(lines[0].startswith("ZR_RETAINED_PREFIX_RESULTS:"), completed.stdout)
        actual = json.loads(lines[0][len("ZR_RETAINED_PREFIX_RESULTS:"):])
        self.assertEqual("passed", actual["status"])
        self.assertEqual(retained["prefix"], actual["prefix"])
        self.assertTrue(actual["nonScriptBytesEqual"])
        self.assertEqual(
            ["prefix-drift", "wrong-prefix-pin", "api-home-drift", "missing-core",
             "duplicate-core", "core-is-not-suffix", "duplicate-script-element"],
            actual["rejected"],
        )

        import importlib.util
        from unittest.mock import Mock, patch
        spec = importlib.util.spec_from_file_location("branch_entry_reuse_test", ROOT / "tools/jenkins-management/branch_integration.py")
        branch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(branch)
        endpoint = "/job/zircon-management/config.xml"
        core = "node('zircon-windows-agent') { /* --capsule retained */ }\n"
        installed_script = prefix + core
        doc = branch.ET.fromstring("<flow-definition><description>foreign</description><properties/><definition><script/></definition></flow-definition>")
        doc.find("definition/script").text = installed_script
        raw = branch.ET.tostring(doc, encoding="utf-8")
        client = Mock()
        client.request.return_value = (200, raw, {})
        with patch.object(Path, "read_text", side_effect=AssertionError("Reuse must not read the mutable template")):
            result = branch.install_entry(None, ROOT, client, reuse_existing=True)
        self.assertEqual("installed", result["status"])
        self.assertTrue(result["reused"])
        client.request.assert_called_once_with(endpoint)

        for damaged in (branch.BEGIN + core, branch.END + branch.BEGIN + core,
                        branch.BEGIN + branch.BEGIN + branch.END + core):
            doc.find("definition/script").text = damaged
            client = Mock()
            client.request.return_value = (200, branch.ET.tostring(doc, encoding="utf-8"), {})
            with self.assertRaisesRegex(ValueError, "markers"):
                branch.install_entry(None, ROOT, client, reuse_existing=True)
            client.request.assert_called_once_with(endpoint)

        template = branch.BEGIN + "\n// explicit maintenance template\n" + branch.END + "\n"
        for old_script, reuse in ((installed_script, False), (core, True)):
            doc.find("definition/script").text = old_script
            before = branch.ET.tostring(doc, encoding="utf-8")
            suffix = old_script
            if branch.BEGIN in suffix:
                start, end = suffix.index(branch.BEGIN), suffix.index(branch.END) + len(branch.END)
                suffix = suffix[:start] + suffix[end:].lstrip("\r\n")
            expected = template.rstrip() + "\n\n" + suffix
            doc.find("definition/script").text = expected
            after = branch.ET.tostring(doc, encoding="utf-8")
            client = Mock()
            client.request.side_effect = [(200, before, {}), (200, b"", {}), (200, after, {})]
            with patch.object(Path, "read_text", return_value=template):
                result = branch.install_entry(None, ROOT, client, reuse_existing=reuse)
            self.assertEqual("installed", result["status"])
            self.assertEqual(3, client.request.call_count)
            posted = client.request.call_args_list[1]
            self.assertEqual("POST", posted.kwargs["method"])
            self.assertEqual(expected, branch.ET.fromstring(posted.kwargs["data"]).findtext("definition/script"))


if __name__ == "__main__":
    unittest.main()
