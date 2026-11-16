Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# No advanced parameter block: Cargo -p must not bind to PipelineVariable.
# Only wrapper arguments before the Cargo verb are consumed; -- is preserved.
try {
    $localArguments = @($args)
    foreach ($argument in $localArguments) {
        if ([string]$argument -match '^(?i)(-JenkinsIdentityFile|--jenkins-identity-file|--identity-file)([=:]|$)') {
            throw 'Jenkins identity options are retired; Agent-local execution never reads an identity file.'
        }
    }
    $codeRepo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
    $RepoRoot = $codeRepo
    $pythonSelection = @{}
    $forwarded = @()
    $seen = @{}
    $argumentIndex = 0
    $switches = @{
        '-QueryOnly' = '--query-only'; '-NoLocked' = '--unlocked'; '-DryRun' = '--dry-run';
        '-IndependentPreview' = '--independent-preview'
    }
    $values = @{
        '-TargetDir' = '--target-dir'; '-BuildRoot' = '--build-root';
        '-EvidenceDirectory' = '--evidence-directory'; '-EvidenceInputManifest' = '--evidence-input-manifest';
        '-EvidenceTimeoutSeconds' = '--evidence-timeout-seconds'; '-ReceiptDirectory' = '--receipt-directory'
    }
    while ($argumentIndex -lt $localArguments.Count) {
        $option = [string]$localArguments[$argumentIndex]
        if ($switches.ContainsKey($option)) {
            if ($seen.ContainsKey($option)) { throw "Repeated wrapper option: $option" }
            $seen[$option] = $true
            $forwarded += $switches[$option]
            $argumentIndex++
            continue
        }
        if ($values.ContainsKey($option) -or $option -in @('-RepoRoot', '-PythonInterpreter')) {
            if ($seen.ContainsKey($option)) { throw "Repeated wrapper option: $option" }
            $seen[$option] = $true
            if ($argumentIndex + 1 -ge $localArguments.Count) { throw "$option requires a value" }
            $argumentIndex++
            $value = [string]$localArguments[$argumentIndex]
            if ([string]::IsNullOrWhiteSpace($value)) { throw "$option requires a nonempty value" }
            if ($option -eq '-RepoRoot') { $RepoRoot = $value }
            elseif ($option -eq '-PythonInterpreter') { $pythonSelection.PythonInterpreter = $value }
            else { $forwarded += @($values[$option], $value) }
            $argumentIndex++
            continue
        }
        break
    }
    $cargoArguments = if ($argumentIndex -lt $localArguments.Count) {
        @($localArguments[$argumentIndex..($localArguments.Count - 1)])
    } else { @() }
    if ($cargoArguments.Count -gt 0 -and $cargoArguments[0] -eq '--') {
        $cargoArguments = @($cargoArguments | Select-Object -Skip 1)
    }
    . (Join-Path $PSScriptRoot 'agent-cargo-common.ps1')
    $python = Resolve-AgentPython @pythonSelection
    $resolvedRepo = (Resolve-Path -LiteralPath $RepoRoot).ProviderPath
    $pythonArguments = @('-B', '-X', 'utf8', '-m', 'tools.dev.local_cargo', '--repo-root', $resolvedRepo) +
        $forwarded + @('--') + $cargoArguments
    # Import code from this authorized worktree, not a caller-supplied source root.
    Push-Location -LiteralPath $codeRepo
    try {
        & $python @pythonArguments
        $cargoExitCode = $LASTEXITCODE
    } finally { Pop-Location }
    exit $cargoExitCode
} catch {
    [ordered]@{status='pending';reasonCode='agent_local_cargo_input';executionOwner='Agent';
        message=$_.Exception.Message;commandExecuted=$false;formalAcceptance=$false} | ConvertTo-Json -Compress
    exit 2
}
