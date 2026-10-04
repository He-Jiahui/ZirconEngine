function Get-ManagedBuildPolicy {
    param(
        [string]$RepoRoot,
        [string]$SourceRoot,
        [string[]]$Command,
        [string]$StorageMode = 'reuse',
        [string]$LinkMode = 'auto',
        [string]$Linker = 'auto',
        [switch]$DryRunMode
    )
    $policyArguments = @('-m', 'tools.session_coordinator.build_policy', '--repo-root', $RepoRoot,
        '--storage-mode', $StorageMode, '--link-mode', $LinkMode, '--linker', $Linker)
    if ($DryRunMode) { $policyArguments += '--dry-run' }
    if ($SourceRoot) { $policyArguments += @('--source-root', $SourceRoot) }
    $policyArguments += @('--', 'cargo') + $Command
    Push-Location $windowsPathResolverRepoRoot
    try {
        $policyJson = & python @policyArguments
        if ($LASTEXITCODE -ne 0) { throw 'Could not resolve the coordinator build policy.' }
        return ($policyJson | ConvertFrom-Json)
    } finally { Pop-Location }
}

function Assert-ManagedCompilePolicy {
    param([string]$RepoRoot, [string]$SourceRoot, [object]$Policy,
          [string]$StorageMode, [string]$LinkMode, [string]$Linker)
    $sealed = Get-ManagedBuildPolicy -RepoRoot $RepoRoot -SourceRoot $SourceRoot `
        -Command @($Policy.command | Select-Object -Skip 1) `
        -StorageMode $StorageMode -LinkMode $LinkMode -Linker $Linker
    foreach ($field in @('platform', 'toolchain', 'target_architecture', 'workspace', 'build_config')) {
        if ($Policy.compatibility.$field -cne $sealed.compatibility.$field) {
            throw "Compiler policy changed while sealing sources ($field); retry with the current configuration."
        }
    }
}

function Invoke-ManagedCompileWorkspaceCoordinator {
    param([string[]]$CommandArguments)
    & python @CommandArguments
}

function Initialize-ManagedCompileWorkspace {
    param([string]$RepoRoot, [string]$TargetDirectory, [string]$CommandsJson,
          [string]$SourceRoot, [string]$ExpectedDigest)
    $inputRoot = if ($SourceRoot) { $SourceRoot } else { $RepoRoot }
    $syncArguments = @('-m', 'tools.session_coordinator.compile_workspaces', 'prepare',
        '--source-root', $inputRoot, '--target-dir', $TargetDirectory)
    if ($ExpectedDigest) { $syncArguments += @('--expected-digest', $ExpectedDigest) }
    if (-not $SourceRoot -and (Test-Path -LiteralPath (Join-Path $RepoRoot '.git'))) { $syncArguments += '--live-repository' }
    $commandsFilePath = $null
    $locationPushed = $false
    $primaryFailure = $null
    try {
        if ($CommandsJson) {
            $commandsFilePath = Join-Path ([IO.Path]::GetTempPath()) (
                'zircon-managed-compile-commands-{0}.json' -f [guid]::NewGuid().ToString('N')
            )
            $payload = [Text.UTF8Encoding]::new($false).GetBytes($CommandsJson)
            $stream = [IO.File]::Open(
                $commandsFilePath,
                [IO.FileMode]::CreateNew,
                [IO.FileAccess]::Write,
                [IO.FileShare]::None
            )
            try {
                $stream.Write($payload, 0, $payload.Length)
                $stream.Flush($true)
            }
            finally {
                $stream.Dispose()
            }
            $syncArguments += @('--commands-json-file', $commandsFilePath)
        }
        Push-Location $windowsPathResolverRepoRoot
        $locationPushed = $true
        $syncJson = Invoke-ManagedCompileWorkspaceCoordinator -CommandArguments $syncArguments
        if ($LASTEXITCODE -ne 0) { throw 'Could not synchronize the leased compiler workspace.' }
        return ($syncJson | ConvertFrom-Json)
    }
    catch {
        $primaryFailure = $_
        throw
    }
    finally {
        if ($locationPushed) { Pop-Location }
        if ($commandsFilePath) {
            try { [IO.File]::Delete($commandsFilePath) }
            catch {
                if ($null -eq $primaryFailure) { throw }
                Write-Warning ("Could not remove managed compile command manifest '{0}': {1}" -f $commandsFilePath, $_.Exception.Message)
            }
        }
    }
}

function Test-ManagedCompileWorkspace {
    param([string]$TargetDirectory)
    Push-Location $windowsPathResolverRepoRoot
    try {
        & python -m tools.session_coordinator.compile_workspaces verify --target-dir $TargetDirectory
        if ($LASTEXITCODE -ne 0) { throw 'Compiler workspace changed during validation.' }
    } finally { Pop-Location }
}

function ConvertTo-ManagedEnvironment {
    param([object]$Policy)
    $values = @{}
    foreach ($property in $Policy.environment.PSObject.Properties) { $values[$property.Name] = $property.Value }
    return $values
}
