"""Bounded S80 observation of original Pester ErrorRecord references.

The collector reads only its explicit public-property allowlists. It runs after
the one original Pester invocation, without a breakpoint or a substitute error.
Diagnostic completion never accepts the resolver regression.
"""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import stat
import sys

ROOT = Path("E:/Git/ZirconEngine")
RUN = "engine-s80-resolver-failure-record-20261018-r1"
OUTPUT = Path("E:/cargo-targets/zircon-local/reports") / RUN / "native-continuation-after-213"
TEST = ROOT / "tools/tests/windows-path-resolver.Tests.ps1"
SUPPORT = ROOT / "tools/jenkins-management/resolver_entry_observation.py"
SUPPORT_SHA = "8d6f55ff380cbf457b98d277b60435d7fb315aa0c1afeebe79aefc829259a706"
PESTER = Path("C:/Program Files/WindowsPowerShell/Modules/Pester/3.4.0/Pester.psd1")
MESSAGE_SHA = "d7532bcfdabb1a678b06a0fcc66a7152b4bba3a3e3214797f1bae6f8c8bded90"
TEST_SHA = "ac96c78d6ccc3860dde237803312d8d03df90817e142b7ad865bd817a796ad33"
RESOLVER_SHA = "27d315830c00ef91e8c5241a454f7036a96a2c85fa5a04077b0815f8a64e5a08"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate_plan(plan):
    require(plan.get("schemaVersion") == 1
            and plan.get("requestId") == plan.get("attemptId") == RUN,
            "The original S80 request and attempt are required")
    require(Path(plan["output"]) == OUTPUT and plan.get("budgetSeconds") == 120,
            "Exact S80 output and 120 second original-test budget required")
    require(plan.get("continuation", {}).get("build") == 213
            and plan["continuation"].get("originalNativeStarted") is False,
            "Retain the pre-native #213 failure and original request identity")
    require(plan["message"]["sha256"] == MESSAGE_SHA, "Exact S80 message required")
    require(Path(plan["support"]["path"]) == SUPPORT
            and plan["support"]["sha256"] == SUPPORT_SHA, "Canonical pinned support required")
    require(plan["test"]["sha256"] == TEST_SHA and Path(plan["test"]["path"]) == TEST,
            "Original unchanged test required")
    require(plan["resolver"]["sha256"] == RESOLVER_SHA
            and Path(plan["resolver"]["path"]) == ROOT / "tools/maintenance/WindowsPathResolver.psm1",
            "Original unchanged resolver required")
    require(Path(plan["pesterManifest"]["path"]) == PESTER,
            "Use the installed pinned Pester, without an upgrade")
    require(plan["limits"] == {"maxObjects": 12, "maxDepth": 6, "maxUtf8Bytes": 65536},
            "Collector limits must not change")


COLLECTOR = r'''
function Add-S80Reason {
    param($State, [string]$Reason)
    $State.partial = $true
    if ($State.reasons.Count -lt 64) { [void]$State.reasons.Add($Reason) }
    else { $State.additionalReasons = $State.additionalReasons + 1 }
}
function Convert-S80Scalar {
    param($Value, $State, [string]$Label)
    if ($null -eq $Value) { return [ordered]@{ state = 'null'; exists = $true; isNull = $true; type = $null; value = $null } }
    $type = $Value.GetType().FullName
    if ($Value -is [Type]) { $text = $Value.FullName }
    elseif ($Value -is [string]) { $text = $Value }
    elseif ($Value -is [bool] -or $Value -is [byte] -or $Value -is [sbyte] -or $Value -is [int16] -or $Value -is [uint16] -or $Value -is [int32] -or $Value -is [uint32] -or $Value -is [int64] -or $Value -is [uint64] -or $Value -is [decimal] -or $Value -is [double] -or $Value -is [single]) {
        return [ordered]@{ state = 'value'; exists = $true; isNull = $false; type = $type; value = $Value }
    }
    elseif ($Value -is [Enum]) { $text = [Enum]::GetName($Value.GetType(), $Value) }
    else {
        Add-S80Reason $State ($Label + ':nonprimitive-not-serialized')
        return [ordered]@{ state = 'unsupported-type'; exists = $true; isNull = $false; type = $type; value = $null }
    }
    if ($null -ne $text -and $text.Length -gt 2048) {
        Add-S80Reason $State ($Label + ':scalar-character-limit')
        return [ordered]@{ state = 'truncated'; exists = $true; isNull = $false; type = $type; value = $text.Substring(0, 2048); originalCharacters = $text.Length }
    }
    return [ordered]@{ state = 'value'; exists = $true; isNull = $false; type = $type; value = $text }
}
function Read-S80Property {
    param($Object, [string]$Name, $State, [string]$Label, [switch]$ResultNoteProperty)
    if ($null -eq $Object) {
        return @{ raw = $null; field = [ordered]@{ state = 'missing'; exists = $false; isNull = $null; type = $null; value = $null } }
    }
    try {
        if ($ResultNoteProperty.IsPresent) {
            # Only the explicitly named Pester result member may be a note property.
            $property = $Object.PSObject.Properties[$Name]
            if ($null -eq $property) { return @{ raw = $null; field = [ordered]@{ state = 'missing'; exists = $false; isNull = $null; type = $null; value = $null } } }
            $value = $property.Value
        }
        else {
            $property = $Object.GetType().GetProperty($Name, [Reflection.BindingFlags]'Instance,Public')
            if ($null -eq $property -or -not $property.CanRead -or $property.GetIndexParameters().Length -ne 0) {
                return @{ raw = $null; field = [ordered]@{ state = 'missing'; exists = $false; isNull = $null; type = $null; value = $null } }
            }
            $value = $property.GetValue($Object, $null)
        }
        $field = [ordered]@{ state = $(if ($null -eq $value) { 'null' } else { 'value' }); exists = $true; isNull = ($null -eq $value); type = $(if ($null -ne $value) { $value.GetType().FullName } else { $null }) }
        return @{ raw = $value; field = $field }
    }
    catch {
        Add-S80Reason $State ($Label + ':public-getter-failed')
        $message = $_.Exception.Message
        if ($message.Length -gt 256) { $message = $message.Substring(0, 256) }
        return @{ raw = $null; field = [ordered]@{ state = 'getter-failed'; exists = $true; isNull = $null; type = $null; value = $null; error = $message } }
    }
}
function Get-S80ScalarProperty {
    param($Object, [string]$Name, $State, [string]$Label)
    $read = Read-S80Property $Object $Name $State $Label
    if ($read.field.state -in @('missing', 'getter-failed')) { return $read.field }
    return Convert-S80Scalar $read.raw $State $Label
}
function Get-S80Invocation {
    param($Read, $State, [string]$Label)
    if ($Read.field.state -ne 'value') { return $Read.field }
    if ($Read.raw -isnot [Management.Automation.InvocationInfo]) {
        Add-S80Reason $State ($Label + ':not-InvocationInfo')
        return [ordered]@{ state = 'wrong-type'; exists = $true; isNull = $false; type = $Read.raw.GetType().FullName; value = $null }
    }
    $invocation = $Read.raw
    $fields = [ordered]@{}
    foreach ($name in @('ScriptName', 'ScriptLineNumber', 'OffsetInLine', 'Line', 'Statement', 'PositionMessage', 'InvocationName')) {
        $fields[$name] = Get-S80ScalarProperty $invocation $name $State ($Label + '.' + $name)
    }
    $command = Read-S80Property $invocation 'MyCommand' $State ($Label + '.MyCommand')
    if ($command.field.state -eq 'value') {
        $commandFields = [ordered]@{}
        foreach ($name in @('Name', 'CommandType', 'Path', 'ModuleName')) {
            $commandFields[$name] = Get-S80ScalarProperty $command.raw $name $State ($Label + '.MyCommand.' + $name)
        }
        $fields['MyCommand'] = [ordered]@{ state = 'value'; exists = $true; isNull = $false; type = $command.raw.GetType().FullName; value = $commandFields }
    }
    else { $fields['MyCommand'] = $command.field }
    return [ordered]@{ state = 'value'; exists = $true; isNull = $false; type = $invocation.GetType().FullName; value = $fields }
}
function Visit-S80Object {
    param($Object, [int]$Depth, $State)
    for ($index = 0; $index -lt $State.objects.Count; $index++) {
        if ([Object]::ReferenceEquals($State.objects[$index], $Object)) {
            return [ordered]@{ nodeId = $index; referenceKind = $(if ($State.active.Contains($index)) { 'back-reference' } else { 'shared-reference' }) }
        }
    }
    if ($Depth -gt 6) {
        $State.depthLimitReached = $true
        Add-S80Reason $State 'maximum-depth'
        return [ordered]@{ nodeId = $null; referenceKind = 'depth-limit' }
    }
    if ($State.objects.Count -ge 12) {
        $State.nodeLimitReached = $true
        Add-S80Reason $State 'maximum-distinct-objects'
        return [ordered]@{ nodeId = $null; referenceKind = 'node-limit' }
    }
    $id = $State.objects.Count
    [void]$State.objects.Add($Object)
    [void]$State.active.Add($id)
    $node = [ordered]@{ nodeId = $id; depth = $Depth; type = $Object.GetType().FullName; kind = $(if ($Object -is [Management.Automation.ErrorRecord]) { 'ErrorRecord' } else { 'Exception' }); fields = [ordered]@{}; edges = [Collections.ArrayList]::new() }
    [void]$State.nodes.Add($node)
    try {
        if ($Object -is [Management.Automation.ErrorRecord]) {
            foreach ($name in @('FullyQualifiedErrorId', 'ScriptStackTrace')) {
                $node.fields[$name] = Get-S80ScalarProperty $Object $name $State ('node' + $id + '.' + $name)
            }
            $category = Read-S80Property $Object 'CategoryInfo' $State ('node' + $id + '.CategoryInfo')
            if ($category.field.state -eq 'value') {
                $node.fields['CategoryInfo'] = [ordered]@{ state = 'value'; exists = $true; isNull = $false; type = $category.raw.GetType().FullName; value = [ordered]@{
                    Activity = (Get-S80ScalarProperty $category.raw 'Activity' $State ('node' + $id + '.CategoryInfo.Activity'))
                    Reason = (Get-S80ScalarProperty $category.raw 'Reason' $State ('node' + $id + '.CategoryInfo.Reason'))
                } }
            }
            else { $node.fields['CategoryInfo'] = $category.field }
            $details = Read-S80Property $Object 'ErrorDetails' $State ('node' + $id + '.ErrorDetails')
            if ($details.field.state -eq 'value') {
                $node.fields['ErrorDetails'] = [ordered]@{ state = 'value'; exists = $true; isNull = $false; type = $details.raw.GetType().FullName; value = [ordered]@{ Message = (Get-S80ScalarProperty $details.raw 'Message' $State ('node' + $id + '.ErrorDetails.Message')) } }
            }
            else { $node.fields['ErrorDetails'] = $details.field }
            $node.fields['InvocationInfo'] = Get-S80Invocation (Read-S80Property $Object 'InvocationInfo' $State ('node' + $id + '.InvocationInfo')) $State ('node' + $id + '.InvocationInfo')
            $edgeNames = @('Exception')
        }
        else {
            foreach ($name in @('Message', 'HResult', 'ParameterName', 'ParameterType', 'TypeSpecified', 'Line', 'Offset')) {
                $node.fields[$name] = Get-S80ScalarProperty $Object $name $State ('node' + $id + '.' + $name)
            }
            $node.fields['CommandInvocation'] = Get-S80Invocation (Read-S80Property $Object 'CommandInvocation' $State ('node' + $id + '.CommandInvocation')) $State ('node' + $id + '.CommandInvocation')
            $edgeNames = @('ErrorRecord', 'InnerException')
        }
        foreach ($name in $edgeNames) {
            $read = Read-S80Property $Object $name $State ('node' + $id + '.' + $name)
            $edge = [ordered]@{ name = $name; field = $read.field; nodeId = $null; referenceKind = 'not-followed' }
            if ($read.field.state -eq 'value') {
                $valid = if ($name -eq 'ErrorRecord') { $read.raw -is [Management.Automation.ErrorRecord] } else { $read.raw -is [Exception] }
                if ($valid) {
                    $reference = Visit-S80Object $read.raw ($Depth + 1) $State
                    $edge.nodeId = $reference.nodeId
                    $edge.referenceKind = $reference.referenceKind
                }
                else {
                    $edge.referenceKind = 'wrong-public-property-type'
                    Add-S80Reason $State ('node' + $id + '.' + $name + ':wrong-edge-type')
                }
            }
            [void]$node.edges.Add($edge)
        }
    }
    finally { [void]$State.active.Remove($id) }
    return [ordered]@{ nodeId = $id; referenceKind = 'new-reference' }
}
function Get-S80ErrorGraph {
    param([AllowNull()][object[]]$TestResults)
    $state = @{ objects = [Collections.Generic.List[object]]::new(); nodes = [Collections.ArrayList]::new(); active = [Collections.Generic.List[int]]::new(); partial = $false; reasons = [Collections.Generic.List[string]]::new(); additionalReasons = 0; nodeLimitReached = $false; depthLimitReached = $false }
    $roots = [Collections.ArrayList]::new()
    $index = 0
    foreach ($result in @($TestResults)) {
        if ($roots.Count -ge 12) { Add-S80Reason $state 'root-list-output-limit'; break }
        $read = Read-S80Property $result 'ErrorRecord' $state ('TestResult' + $index + '.ErrorRecord') -ResultNoteProperty
        $root = [ordered]@{ testResultIndex = $index; testResultType = $(if ($null -ne $result) { $result.GetType().FullName } else { $null }); field = $read.field; nodeId = $null; referenceKind = 'not-followed' }
        if ($read.field.state -eq 'value') {
            if ($read.raw -is [Management.Automation.ErrorRecord]) {
                $reference = Visit-S80Object $read.raw 0 $state
                $root.nodeId = $reference.nodeId
                $root.referenceKind = $reference.referenceKind
            }
            else {
                $root.referenceKind = 'not-ErrorRecord'
                Add-S80Reason $state ('TestResult' + $index + ':root-not-ErrorRecord')
            }
        }
        [void]$roots.Add($root)
        $index++
    }
    $graph = [ordered]@{
        schemaVersion = 1; status = $(if ($state.partial) { 'partial' } else { 'complete' })
        rootCount = @($roots | Where-Object { $null -ne $_.nodeId }).Count
        testResultsObserved = $roots.Count; distinctObjectsObserved = $state.objects.Count; nodeCount = $state.nodes.Count
        roots = $roots; nodes = $state.nodes; nodeLimitReached = $state.nodeLimitReached; depthLimitReached = $state.depthLimitReached
        maxObjects = 12; maxDepth = 6; maxUtf8Bytes = 65536; scalarCharacterLimit = 2048
        outputBudgetReservedBytes = 4096; outputBudgetReached = $false; nodesOmittedForOutput = 0
        truncationReasons = $state.reasons; additionalReasons = $state.additionalReasons
        serializationFailed = $false; outputUtf8Bytes = 0
    }
    # Remove only DTO nodes if worst-case scalar data exceeds the output budget.
    # Original objects, results and exception state are never modified.
    while ($true) {
        try { $json = $graph | ConvertTo-Json -Depth 24 -Compress -ErrorAction Stop }
        catch {
            return [ordered]@{ schemaVersion = 1; status = 'failed'; serializationFailed = $true; reason = 'bounded-DTO-serialization-failed'; rootCount = 0; nodeCount = 0; nodes = @(); roots = @(); maxObjects = 12; maxDepth = 6; maxUtf8Bytes = 65536; outputUtf8Bytes = 0 }
        }
        $bytes = [Text.Encoding]::UTF8.GetByteCount($json)
        if ($bytes -le 61440) {
            if ($graph.outputUtf8Bytes -eq $bytes) { break }
            $graph.outputUtf8Bytes = $bytes
            continue
        }
        $graph.status = 'partial'; $graph.outputBudgetReached = $true
        if ($graph.nodes.Count -eq 0) { throw 'Bounded graph metadata exceeds its output budget.' }
        $removed = $graph.nodes[$graph.nodes.Count - 1].nodeId
        $graph.nodes.RemoveAt($graph.nodes.Count - 1)
        $graph.nodesOmittedForOutput++
        $graph.nodeCount = $graph.nodes.Count
        foreach ($root in $graph.roots) { if ($root.nodeId -eq $removed) { $root.nodeId = $null; $root.referenceKind = 'output-budget' } }
        foreach ($node in $graph.nodes) { foreach ($edge in $node.edges) { if ($edge.nodeId -eq $removed) { $edge.nodeId = $null; $edge.referenceKind = 'output-budget' } } }
        $graph.rootCount = @($graph.roots | Where-Object { $null -ne $_.nodeId }).Count
    }
    return $graph
}
'''

RUNNER = r'''$ErrorActionPreference = 'Stop'
. '__OUTPUT__\collector.ps1'
$testPath = '__TEST__'
$metadata = [ordered]@{ powershell = [string]$PSVersionTable.PSVersion; edition = [string]$PSVersionTable.PSEdition; processId = $PID; birthFileTimeUtc = [string]([Diagnostics.Process]::GetCurrentProcess().StartTime.ToUniversalTime().ToFileTimeUtc()); pester = $null; path = $null; physicalTemp = [IO.Path]::GetTempPath(); breakpointsCreated = 0 }
$result = $null
$originalError = $null
$diagnosticError = $null
try {
    Import-Module '__PESTER__' -Force
    $module = (Get-Command Invoke-Pester).Module
    $metadata.pester = [string]$module.Version
    $metadata.path = $module.Path
    $result = Invoke-Pester -Script $testPath -PassThru -Quiet -OutputFormat NUnitXml -OutputFile '__OUTPUT__\pester-results.xml'
}
catch { $originalError = $_ }
finally {
    $counts = if ($null -ne $result) { [ordered]@{ total = $result.TotalCount; passed = $result.PassedCount; failed = $result.FailedCount; skipped = $result.SkippedCount; pending = $result.PendingCount } } else { $null }
    try {
        $graph = Get-S80ErrorGraph -TestResults $(if ($null -ne $result) { @($result.TestResult) } else { @() })
        $diagnosticState = $graph.status
        if ($null -eq $result) { $diagnosticState = 'failed-no-original-Pester-result' }
        elseif ($result.FailedCount -eq 0) { $diagnosticState = 'failure-not-reproduced' }
        elseif ($graph.rootCount -eq 0) { $diagnosticState = 'failed-missing-original-error-roots' }
        $graphJson = $graph | ConvertTo-Json -Depth 24 -Compress
        if ([Text.Encoding]::UTF8.GetByteCount($graphJson) -gt 65536) { throw 'Graph output exceeds its registered UTF8 byte budget.' }
        [IO.File]::WriteAllText('__OUTPUT__\error-graph.json', $graphJson, (New-Object Text.UTF8Encoding($false)))
        $bindingCandidates = @()
        foreach ($node in $graph.nodes) {
            foreach ($name in @('InvocationInfo', 'CommandInvocation')) {
                if ($node.fields.Contains($name)) {
                    $invocation = $node.fields[$name]
                    if ($invocation.state -eq 'value' -and $invocation.value.ScriptName.value -eq $testPath -and $invocation.value.ScriptLineNumber.value -eq 2 -and $invocation.value.MyCommand.state -eq 'value' -and -not [string]::IsNullOrEmpty($invocation.value.MyCommand.value.Name.value)) {
                        $bindingCandidates += [ordered]@{ nodeId = $node.nodeId; field = $name }
                    }
                }
            }
        }
        $bindingState = if ($bindingCandidates.Count -gt 0) { 'whitelisted-original-line2-invocation-candidate' } elseif ($diagnosticState -eq 'complete') { 'original-binding-extent-unavailable' } else { 'unknown-incomplete-diagnostic' }
    }
    catch {
        $diagnosticError = $_.Exception.GetType().FullName
        $diagnosticState = 'failed'
        $bindingState = 'unknown-incomplete-diagnostic'
        $bindingCandidates = @()
    }
    $proof = [ordered]@{
        metadata = $metadata; diagnostic = [ordered]@{ status = $diagnosticState; errorType = $diagnosticError; graphPath = '__OUTPUT__\error-graph.json'; originalBindingExtent = $bindingState; candidateFields = $bindingCandidates }
        originalCounts = $counts; originalErrorOutsidePester = [ordered]@{ exists = ($null -ne $originalError); type = $(if ($null -ne $originalError) { $originalError.GetType().FullName } else { $null }); notUsedAsGraphRoot = $true }
        testDriveRemaining = [bool](Get-PSDrive -Name TestDrive -ErrorAction SilentlyContinue); ordinaryRegressionAccepted = $false
    }
    [IO.File]::WriteAllText('__OUTPUT__\observation.json', ($proof | ConvertTo-Json -Depth 12), (New-Object Text.UTF8Encoding($false)))
    $metadata | ConvertTo-Json -Compress
    if ($null -ne $counts) { $counts | ConvertTo-Json -Compress }
}
if ($null -ne $originalError) { throw $originalError }
if ($null -eq $result -or $result.TotalCount -ne 6 -or $result.PassedCount -ne 6 -or $result.FailedCount -ne 0 -or $result.SkippedCount -ne 0) { exit 1 }
exit 0
'''

CONTRACT = r'''$ErrorActionPreference = 'Stop'
. '__OUTPUT__\collector.ps1'
Add-Type -ReferencedAssemblies ([Management.Automation.ErrorRecord].Assembly.Location) -TypeDefinition @'
using System;
using System.Management.Automation;
public sealed class S80ContractException : Exception {
    public ErrorRecord ErrorRecord { get; set; }
    public string ParameterName { get { throw new InvalidOperationException("synthetic getter failure"); } }
    public S80ContractException(string message) : base(message) { }
}
'@
function Assert-S80Contract { param([bool]$Condition, [string]$Message) if (-not $Condition) { throw $Message } }
$cases = [Collections.Generic.List[string]]::new()
$exception = [S80ContractException]::new('synthetic cycle')
$errorRecord = [Management.Automation.ErrorRecord]::new($exception, 's80.synthetic', [Management.Automation.ErrorCategory]::InvalidArgument, $null)
$exception.ErrorRecord = $errorRecord
$result = [pscustomobject]@{ ErrorRecord = $errorRecord }
$cycle = Get-S80ErrorGraph -TestResults @($result, $result)
Assert-S80Contract ($cycle.distinctObjectsObserved -eq 2 -and $cycle.roots[0].nodeId -eq $cycle.roots[1].nodeId) 'Reference-equal roots were duplicated.'
Assert-S80Contract (@($cycle.nodes[1].edges | Where-Object { $_.name -eq 'ErrorRecord' -and $_.nodeId -eq 0 -and $_.referenceKind -eq 'back-reference' }).Count -eq 1) 'Actual public cycle was not bounded by reference identity.'
$cases.Add('cycle-and-shared-reference')
Assert-S80Contract ($cycle.status -eq 'partial' -and $cycle.nodes[1].fields.ParameterName.state -eq 'getter-failed') 'A failing public getter must remain a partial diagnostic.'
$cases.Add('public-getter-failure')
Assert-S80Contract ([Object]::ReferenceEquals($result.ErrorRecord, $errorRecord) -and [Object]::ReferenceEquals($exception.ErrorRecord, $errorRecord) -and $exception.Message -eq 'synthetic cycle') 'Collector changed an original synthetic object or result.'
$cases.Add('result-and-object-reference-unchanged')
$inner = [Exception]::new('leaf')
for ($i = 0; $i -lt 8; $i++) { $inner = [Exception]::new('depth', $inner) }
$deep = Get-S80ErrorGraph -TestResults @([pscustomobject]@{ ErrorRecord = [Management.Automation.ErrorRecord]::new($inner, 's80.depth', [Management.Automation.ErrorCategory]::InvalidArgument, $null) })
Assert-S80Contract ($deep.depthLimitReached -and $deep.status -eq 'partial' -and @($deep.nodes | Where-Object { $_.depth -gt 6 }).Count -eq 0) 'Depth limit did not preserve partial status.'
$cases.Add('depth-limit')
$many = @()
for ($i = 0; $i -lt 12; $i++) { $many += [pscustomobject]@{ ErrorRecord = [Management.Automation.ErrorRecord]::new([Exception]::new('node'), 's80.nodes', [Management.Automation.ErrorCategory]::InvalidArgument, $null) } }
$limited = Get-S80ErrorGraph -TestResults $many
Assert-S80Contract ($limited.nodeLimitReached -and $limited.distinctObjectsObserved -eq 12 -and $limited.status -eq 'partial') 'Distinct-object limit did not preserve partial status.'
$cases.Add('node-limit')
$missing = Get-S80ErrorGraph -TestResults @([pscustomobject]@{ Other = 1 }, [pscustomobject]@{ ErrorRecord = $null }, [pscustomobject]@{ ErrorRecord = 'wrong-type' })
Assert-S80Contract ($missing.rootCount -eq 0 -and $missing.roots[0].field.state -eq 'missing' -and $missing.roots[1].field.state -eq 'null' -and $missing.roots[2].referenceKind -eq 'not-ErrorRecord') 'Missing/null/wrong roots were substituted.'
$cases.Add('missing-null-wrong-root')
$long = Get-S80ErrorGraph -TestResults @([pscustomobject]@{ ErrorRecord = [Management.Automation.ErrorRecord]::new([Exception]::new(('x' * 100000)), 's80.scalar', [Management.Automation.ErrorCategory]::InvalidArgument, $null) })
$longJson = $long | ConvertTo-Json -Depth 24 -Compress
Assert-S80Contract ($long.status -eq 'partial' -and [Text.Encoding]::UTF8.GetByteCount($longJson) -le 65536 -and $long.nodes[1].fields.Message.state -eq 'truncated') 'Scalar/output budget did not retain a parseable partial graph.'
$cases.Add('scalar-and-valid-UTF8-output-budget')
[IO.File]::WriteAllText('__OUTPUT__\collector-contract.json', ([ordered]@{ status = 'passed'; tests = $cases.Count; cases = $cases.ToArray(); nativePid = $PID; birthFileTimeUtc = [string]([Diagnostics.Process]::GetCurrentProcess().StartTime.ToUniversalTime().ToFileTimeUtc()); syntheticObjectsOnly = $true } | ConvertTo-Json -Depth 8), (New-Object Text.UTF8Encoding($false)))
exit 0
'''


def render(script):
    return script.replace("__TEST__", str(TEST)).replace("__OUTPUT__", str(OUTPUT)).replace("__PESTER__", str(PESTER)).encode("ascii")


def graph_facts(raw):
    require(len(raw) <= 65536, "Graph exceeds its original byte limit")
    graph = json.loads(raw.decode("utf-8", errors="strict"))
    require(graph["status"] in ("complete", "partial", "failed"), "Unknown graph status")
    require(graph["nodeCount"] == len(graph["nodes"]) <= 12, "Graph node limit violated")
    require(all(0 <= n["depth"] <= 6 for n in graph["nodes"]), "Graph depth limit violated")
    ids = {n["nodeId"] for n in graph["nodes"]}
    require(len(ids) == len(graph["nodes"]), "Duplicate graph node identity")
    for ref in [*graph["roots"], *(edge for n in graph["nodes"] for edge in n["edges"])]:
        require(ref["nodeId"] is None or ref["nodeId"] in ids, "Dangling graph reference")
    return graph


def run(plan_path, plan_sha, self_sha):
    for entry in (SUPPORT, *SUPPORT.parents):
        info = entry.lstat()
        require(not (getattr(info, "st_file_attributes", 0) & 0x400)
                and not stat.S_ISLNK(info.st_mode), "Support input path alias")
    require(SUPPORT.stat().st_nlink == 1
            and os.path.normcase(str(SUPPORT.resolve())) == os.path.normcase(str(SUPPORT)),
            "Canonical support input identity required")
    support_raw = SUPPORT.read_bytes()
    require(hashlib.sha256(support_raw).hexdigest() == SUPPORT_SHA,
            "Canonical support changed before load")
    spec = importlib.util.spec_from_file_location("s80_pinned_support", SUPPORT)
    support = importlib.util.module_from_spec(spec)
    exec(compile(support_raw, str(SUPPORT), "exec"), support.__dict__)
    plan = json.loads(support.checked({"path": str(plan_path), "sha256": plan_sha}))
    validate_plan(plan)
    support.checked({"path": __file__, "sha256": self_sha})
    require(hashlib.sha256(support_raw).hexdigest() == plan["support"]["sha256"],
            "Canonical support changed before load")
    require(os.environ.get("JOB_NAME") == "zircon-management-tests"
            and os.environ.get("NODE_NAME") == "zircon-windows-agent"
            and os.environ.get("RUN_ID") == RUN, "Exact Jenkins job/node/run required")
    for item in plan["controlPins"]:
        support.checked(item)
    before = []
    for item in plan["pinnedInputs"]:
        support.checked(item)
        before.append(support.pin(item["path"])[0])
    support.checked(plan["artifactRegistration"])
    require(not support.physical(OUTPUT).exists(), "Existing S80 output must be reconciled")
    OUTPUT.mkdir()

    def write(path, value, raw=False):
        path = support.physical(path)
        require(path.is_relative_to(OUTPUT), "Unregistered S80 output")
        data = value if raw else (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode("utf-8")
        with path.open("xb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())

    write(OUTPUT / "artifacts.json", {"taskId": RUN, "requestId": RUN, "attemptId": RUN,
          "ownerThreadId": plan["ownerThreadId"], "artifactRegistration": plan["artifactRegistration"],
          "output": str(OUTPUT), "expectedPaths": ["collector.ps1", "runner.ps1", "collector-contract.ps1",
          "producer-contract-tests", "collector-contract-tests", "resolver-tests", "temp", "error-graph.json",
          "pester-results.xml", "observation.json", "summary.json"],
          "retention": "Keep original failures, graph, XML, raw/native evidence. No fixture tree deletion.",
          "sourceCopies": 0, "filesDeleted": 0, "releasedBytes": 0})
    write(OUTPUT / "plan.json", plan)
    write(OUTPUT / "input-pins-before.json", before)
    for name, content in [("collector.ps1", COLLECTOR), ("runner.ps1", RUNNER), ("collector-contract.ps1", CONTRACT)]:
        write(OUTPUT / name, render(content), raw=True)
    (OUTPUT / "temp").mkdir()
    component = Path(plan["component"]["path"]).parent
    facts = json.loads(support.checked(plan["component"]))["files"]
    sys.path.insert(0, str(ROOT / ".jenkins/runtime/drivers" / plan["runtime"]["driverDigest"]))
    cargo = support.load("s80_native_cargo", component / "local_cargo.py", facts["local_cargo.py"]["sha256"])
    checks = support.load("s80_native_checks", component / "run_checks.py", facts["run_checks.py"]["sha256"])
    delivery = support.load("s80_native_environment", component / "delivery.py", facts["delivery.py"]["sha256"])
    environment = delivery.environment(OUTPUT, plan["runtime"])
    environment.update(RUN_ID=RUN, JOB_NAME=os.environ["JOB_NAME"], NODE_NAME=os.environ["NODE_NAME"])
    for variable in ("TEMP", "TMP"):
        require(support.physical(environment[variable]) == OUTPUT / "temp"
                and Path(environment[variable]).is_dir(), "Registered physical compiler TEMP/TMP required")
    before_devices = support.devices()
    write(OUTPUT / "fixture-before.json", {"temp": environment["TEMP"], "children": [str(p) for p in (OUTPUT / "temp").iterdir()],
          "dosDevices": before_devices, "fixturesOwnedOnlyByOriginalTest": True})
    unit_command = [sys.executable, "-B", "-X", "utf8", "-m", "unittest",
                    "discover", "-s", str(ROOT / "tools/jenkins-management/tests"),
                    "-p", "test_resolver_failure_record.py", "-v"]
    unit = OUTPUT / "producer-contract-tests"
    unit.mkdir()
    unit_native = cargo._run_owned_command(unit_command, ROOT, environment, unit, 90)
    write(unit / "native-terminal.json", unit_native)
    unit_ok = checks.native_complete(unit_native, unit_command, ROOT) and unit_native["rootExitCode"] == 0
    contract_native = None
    contract_ok = False
    powershell = plan["powershell"]
    contract_command = [powershell, "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
                        "-File", str(OUTPUT / "collector-contract.ps1")]
    if unit_ok:
        leaf = OUTPUT / "collector-contract-tests"
        leaf.mkdir()
        contract_native = cargo._run_owned_command(contract_command, ROOT, environment, leaf, 90)
        write(leaf / "native-terminal.json", contract_native)
        contract_ok = checks.native_complete(contract_native, contract_command, ROOT) and contract_native["rootExitCode"] == 0
    command = [powershell, "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
               "-File", str(OUTPUT / "runner.ps1")]
    native = None
    if unit_ok and contract_ok:
        leaf = OUTPUT / "resolver-tests"
        leaf.mkdir()
        native = cargo._run_owned_command(command, ROOT, environment, leaf, 120)
        write(leaf / "native-terminal.json", native)
    after = [support.pin(item["path"])[0] for item in plan["pinnedInputs"]]
    write(OUTPUT / "input-pins-after.json", after)
    pin_match = before == after
    for item in plan["controlPins"]:
        support.checked(item)
    observation = json.loads((OUTPUT / "observation.json").read_bytes()) if (OUTPUT / "observation.json").exists() else None
    graph = graph_facts((OUTPUT / "error-graph.json").read_bytes()) if (OUTPUT / "error-graph.json").exists() else None
    complete = native is not None and checks.native_complete(native, command, ROOT)
    after_devices = support.devices()
    remaining = [str(p) for p in (OUTPUT / "temp").iterdir()]
    temp_removed = False
    if complete and not remaining:
        (OUTPUT / "temp").rmdir()
        temp_removed = True
    write(OUTPUT / "fixture-after.json", {"remainingTempChildren": remaining,
          "dosDevices": after_devices, "testDriveRemaining": observation["testDriveRemaining"] if observation else None})
    encoding = {}
    acp = ctypes.windll.kernel32.GetACP()
    if native is not None:
        for name in ("stdout.bin", "stderr.bin"):
            raw = (OUTPUT / "resolver-tests" / name).read_bytes()
            encoding[name] = support.output_encoding(raw, acp)
    xml = support.xml_cases((OUTPUT / "pester-results.xml").read_bytes()) if (OUTPUT / "pester-results.xml").exists() else None
    diagnostic_complete = bool(complete and observation and graph
                               and observation["diagnostic"]["status"] in ("complete", "partial", "failure-not-reproduced"))
    write(OUTPUT / "cleanup.json", {"removedOwnEmptyTempOnly": temp_removed,
          "remainingTempChildren": remaining, "unknownFixturesNeverDeleted": True, "dosMappingsUnchanged": before_devices == after_devices,
          "filesDeleted": 0, "releasedBytes": 0, "wholeMonitorCleanupClosed": False})
    summary = {"schemaVersion": 1, "taskId": RUN, "requestId": RUN, "attemptId": RUN,
               "job": os.environ["JOB_NAME"], "build": os.environ.get("BUILD_NUMBER"),
               "producerContractTestsPassed": unit_ok, "actualCollectorContractTestsPassed": contract_ok,
               "nativePowerShellStarted": native is not None, "nativeComplete": bool(complete),
               "originalExitCode": native["rootExitCode"] if native else None,
               "originalTestPassed": bool(complete and native["rootExitCode"] == 0),
               "diagnosticComplete": diagnostic_complete, "diagnostic": observation["diagnostic"] if observation else None,
               "graph": {"status": graph["status"], "roots": graph["rootCount"], "nodes": graph["nodeCount"],
                         "utf8Bytes": (OUTPUT / "error-graph.json").stat().st_size} if graph else None,
               "originalCounts": observation["originalCounts"] if observation else None,
               "xml": xml, "encoding": encoding, "inputsUnchanged": pin_match,
               "dosMappingsUnchanged": before_devices == after_devices, "ordinaryRegressionAccepted": False,
               "formalAcceptance": False, "allRequirementsComplete": False,
               "diagnosticFailureNeverChangesOriginalExit": True, "filesDeleted": 0, "releasedBytes": 0}
    write(OUTPUT / "summary.json", summary)
    print(json.dumps(summary, ensure_ascii=False))
    return 0 if unit_ok and contract_ok and diagnostic_complete and pin_match and before_devices == after_devices else 1


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-sha", required=True)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--plan-sha", required=True)
    args = parser.parse_args(argv)
    return run(Path(args.plan), args.plan_sha, args.self_sha)


if __name__ == "__main__":
    raise SystemExit(main())
