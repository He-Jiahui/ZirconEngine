"""Historical Codex binding and raw Jenkins config fences; no Jenkins transport."""
from __future__ import annotations

import hashlib
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).absolute().parents[2]
INSTALLER = ROOT / "tools/dev/install-jenkins-management.ps1"
_PRELUDE = r"""
$ErrorActionPreference = 'Stop'
$taskSource = [IO.File]::ReadAllText('@@INSTALLER@@')
$taskTokens = $null
$taskErrors = $null
$taskTree = [Management.Automation.Language.Parser]::ParseInput($taskSource, [ref]$taskTokens, [ref]$taskErrors)
if ($taskErrors.Count) { throw ($taskErrors | Out-String) }
foreach ($taskFunctionName in @('Assert-Physical','Hash-Bytes','Resolve-PreviousCodexBinding','Assert-PreviousManagementConfig')) {
    $taskFunctions = @($taskTree.FindAll({
        param($candidate)
        $candidate -is [Management.Automation.Language.FunctionDefinitionAst] -and $candidate.Name -ceq $taskFunctionName
    }, $true))
    if ($taskFunctions.Count -ne 1) { throw 'Missing or ambiguous installer helper' }
    . ([ScriptBlock]::Create($taskFunctions[0].Extent.Text))
}
function Assert-Rejected([scriptblock]$Action, [string]$Reason) {
    $taskRejected = $false
    try { & $Action | Out-Null }
    catch { if ($_.Exception.Message -notlike ('*' + $Reason + '*')) { throw }; $taskRejected = $true }
    if (-not $taskRejected) { throw ('Expected rejection: ' + $Reason) }
}
function Get-Assignment([string]$Name) {
    $taskAssignments = @($taskTree.FindAll({
        param($candidate)
        $candidate -is [Management.Automation.Language.AssignmentStatementAst] -and
            $candidate.Left.Extent.Text -ceq ('$' + $Name)
    }, $true))
    if ($taskAssignments.Count -ne 1) { throw ('Missing or ambiguous assignment: ' + $Name) }
    return $taskAssignments[0]
}
"""


class ManagementCodexMigrationTests(unittest.TestCase):
    def run_powershell(self, body):
        executable = shutil.which("powershell.exe")
        self.assertIsNotNone(executable, "Installer tests require the fixed Windows agent")
        source = _PRELUDE.replace("@@INSTALLER@@", str(INSTALLER).replace("'", "''")) + body
        result = subprocess.run([executable, "-NoProfile", "-NonInteractive", "-Command", source],
                                capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=45)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        return result.stdout.strip()

    def test_absent_previous_leaf_is_evidence_and_ordinary_binding_is_unchanged(self):
        self.run_powershell(r"""
$current = 'C:/current/codex.exe'
$previous = [IO.Path]::Combine([IO.Path]::GetTempPath(), [guid]::NewGuid().ToString('N'), 'removed-codex.exe')
if (Test-Path -LiteralPath $previous) { throw 'Fixture leaf must be absent' }
$resolved = Resolve-PreviousCodexBinding $current $previous ('a' * 64) $true
if ($resolved -cne [IO.Path]::GetFullPath($previous)) { throw 'Historical path was not retained' }
if ((Resolve-PreviousCodexBinding $current '' '' $true) -cne $current) { throw 'Ordinary binding changed' }
""")

    def test_historical_pins_require_a_pair_existing_installation_and_absolute_path(self):
        self.run_powershell(r"""
$current = 'C:/current/codex.exe'
$previous = 'C:/removed/codex.exe'
Assert-Rejected { Resolve-PreviousCodexBinding $current $previous '' $true } 'must be paired'
Assert-Rejected { Resolve-PreviousCodexBinding $current '' ('a' * 64) $true } 'must be paired'
Assert-Rejected { Resolve-PreviousCodexBinding $current $previous ('a' * 64) $false } 'existing installation'
foreach ($bad in @('removed/codex.exe', 'C:removed/codex.exe', 'C:/alias/../codex.exe', 'C:/removed/./codex.exe')) {
    Assert-Rejected { Resolve-PreviousCodexBinding $current $bad ('a' * 64) $true } 'absolute physical path'
}
$bad = 'C:/removed/evil' + [char]10 + 'codex.exe'
Assert-Rejected { Resolve-PreviousCodexBinding $current $bad ('a' * 64) $true } 'Unsafe previous Codex path'
$bad = 'C:/removed/"codex.exe'
Assert-Rejected { Resolve-PreviousCodexBinding $current $bad ('a' * 64) $true } 'Unsafe previous Codex path'
""")

    def test_missing_leaf_does_not_allow_an_existing_reparse_ancestor(self):
        self.run_powershell(r"""
function Test-Path { param([string]$LiteralPath) return $LiteralPath -ceq [IO.Path]::GetFullPath('C:/reviewed-alias') }
function Get-Item { param([string]$LiteralPath, [switch]$Force) return [pscustomobject]@{Attributes=[IO.FileAttributes]::ReparsePoint} }
Assert-Rejected {
    Resolve-PreviousCodexBinding 'C:/current/codex.exe' 'C:/reviewed-alias/removed/codex.exe' ('a' * 64) $true
} 'Path alias'
""")

    def test_raw_config_pin_rejects_byte_drift_and_preserves_unpinned_ordinary_calls(self):
        self.run_powershell(r"""
$old = [Text.Encoding]::UTF8.GetBytes(('<original/>' + [char]13 + [char]10))
$pin = Hash-Bytes $old
Assert-PreviousManagementConfig $old $pin
$changed = [Text.Encoding]::UTF8.GetBytes(('<original/>' + [char]10))
Assert-Rejected { Assert-PreviousManagementConfig $changed $pin } 'raw bytes changed'
$changed = [Text.Encoding]::UTF8.GetBytes('<foreign/>')
Assert-Rejected { Assert-PreviousManagementConfig $changed $pin } 'raw bytes changed'
Assert-PreviousManagementConfig ([Text.Encoding]::UTF8.GetBytes('<ordinary/>')) ''
""")

    def test_actual_token_tables_keep_current_and_previous_hosts_separate(self):
        self.run_powershell(r"""
$repo = 'E:/fixture'
$python = 'E:/fixture/python.exe'
$codex = 'C:/current/codex.exe'
$taskPreviousCodex = 'C:/removed/codex.exe'
$capsule = 'E:/fixture/new-capsule'
$ExpectedDriverDigest = 'a' * 64
$ExpectedRuntimeOperationId = 'start-current'
$taskOldSourceRoot = 'E:/fixture/old-capsule'
$taskOldInstalled = [pscustomobject]@{driverDigest=('b' * 64);runtimeOperationId='start-previous'}
$current = & ([ScriptBlock]::Create((Get-Assignment 'tokens').Right.Extent.Text))
$previous = & ([ScriptBlock]::Create((Get-Assignment 'taskOldTokens').Right.Extent.Text))
if ($current.CODEX -cne $codex -or $previous.CODEX -cne $taskPreviousCodex) { throw 'Current and previous host bindings crossed' }
if ($previous.CAPSULE -cne $taskOldSourceRoot -or $previous.DRIVER -cne $taskOldInstalled.driverDigest -or
        $previous.RUNTIME -cne $taskOldInstalled.runtimeOperationId) { throw 'Historical capsule binding changed' }
""")

    def test_resume_uses_archive_while_fresh_update_rechecks_raw_live_config(self):
        old = b"<original/>\r\n"
        with tempfile.TemporaryDirectory(prefix="management-config-proof-") as temporary:
            archive = Path(temporary) / "synthetic-previous.config.xml"
            archive.write_bytes(old)
            body = r"""
$taskHistoryConfig = '@@ARCHIVE@@'
$taskFinalResponse = [pscustomobject]@{Content='<new-job/>'}
$pin = '@@PIN@@'
$selection = Get-Assignment 'taskFinalPreviousConfigBytes'
$taskPostedAlready = $true
$proof = & ([ScriptBlock]::Create($selection.Right.Extent.Text))
Assert-PreviousManagementConfig $proof $pin
$taskPostedAlready = $false
$proof = & ([ScriptBlock]::Create($selection.Right.Extent.Text))
Assert-Rejected { Assert-PreviousManagementConfig $proof $pin } 'raw bytes changed'
$taskFinalResponse.Content = [IO.File]::ReadAllText($taskHistoryConfig)
$proof = & ([ScriptBlock]::Create($selection.Right.Extent.Text))
Assert-PreviousManagementConfig $proof $pin
$calls = @($taskTree.FindAll({
    param($candidate)
    $candidate -is [Management.Automation.Language.CommandAst] -and
        $candidate.GetCommandName() -ceq 'Assert-PreviousManagementConfig' -and
        $candidate.CommandElements[1].Extent.Text -ceq '$taskFinalPreviousConfigBytes'
}, $true))
if ($calls.Count -ne 1 -or $calls[0].Extent.StartOffset -le $selection.Extent.EndOffset) { throw 'Final original config fence is missing' }
$posts = @($taskTree.FindAll({
    param($candidate)
    $candidate -is [Management.Automation.Language.CommandAst] -and
        $candidate.GetCommandName() -ceq 'Invoke-WebRequest' -and $candidate.Extent.Text -match '-Method Post'
}, $true))
if ($posts.Count -lt 1 -or ($posts | Where-Object {$_.Extent.StartOffset -lt $calls[0].Extent.EndOffset}).Count) {
    throw 'A POST precedes the final raw original config fence'
}
"""
            body = body.replace("@@ARCHIVE@@", str(archive).replace("'", "''"))
            body = body.replace("@@PIN@@", hashlib.sha256(old).hexdigest())
            self.run_powershell(body)


if __name__ == "__main__":
    unittest.main()
