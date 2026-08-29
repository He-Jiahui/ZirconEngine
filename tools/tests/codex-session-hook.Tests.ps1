[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$installer = Join-Path $repoRoot 'tools\install-codex-session-hook.ps1'
$fixture = Join-Path ([IO.Path]::GetTempPath()) ('zircon-codex-hook-' + [guid]::NewGuid().ToString('N'))
$oldLocalAppData = $env:LOCALAPPDATA

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

function Get-RepositoryKey {
    param([string]$Path)
    $bytes = [Text.Encoding]::UTF8.GetBytes(([IO.Path]::GetFullPath($Path)).ToLowerInvariant())
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { return (($hasher.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') }) -join '') } finally { $hasher.Dispose() }
}

function Write-RuntimeDescriptor {
    param(
        [int]$DescriptorVersion,
        [int]$SchemaVersion,
        [int[]]$ControlApiVersions,
        [string]$RepositoryKey
    )
    $runtimePath = Join-Path $fixture '.codex\state\session-coordinator\runtime.json'
    New-Item -ItemType Directory -Path (Split-Path -Parent $runtimePath) -Force | Out-Null
    $runtime = [pscustomobject][ordered]@{
        descriptor_version = $DescriptorVersion
        host = '127.0.0.1'
        repository_key = $RepositoryKey
        schema_version = $SchemaVersion
        control_api_versions = @($ControlApiVersions)
    }
    [IO.File]::WriteAllText(
        $runtimePath,
        (($runtime | ConvertTo-Json -Depth 5) + "`n"),
        [Text.UTF8Encoding]::new($false)
    )
}

try {
    New-Item -ItemType Directory -Path (Join-Path $fixture '.codex') -Force | Out-Null
    $env:LOCALAPPDATA = Join-Path $fixture 'local-app-data'
    $config = @'
# preserve this comment
approval_policy = "never"

[features]
other_feature = true # preserve inline comment

[unrelated]
value = "keep"
'@
    [IO.File]::WriteAllText(
        (Join-Path $fixture '.codex\config.toml'),
        $config,
        [Text.UTF8Encoding]::new($false)
    )

    $dryBefore = [IO.File]::ReadAllBytes((Join-Path $fixture '.codex\config.toml'))
    $dry = & $installer -Action Install -RepoRoot $fixture -DryRun | ConvertFrom-Json
    Assert-True $dry.dryRun 'Install dry-run did not report dryRun=true.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture '.codex\hooks.json'))) 'Dry-run created hooks.json.'
    Assert-True ([Convert]::ToBase64String($dryBefore) -eq [Convert]::ToBase64String([IO.File]::ReadAllBytes((Join-Path $fixture '.codex\config.toml')))) 'Dry-run changed config.toml.'

    $installed = & $installer -Action Install -RepoRoot $fixture | ConvertFrom-Json
    Assert-True $installed.configured 'Install did not configure the exact Hook definition.'
    Assert-True $installed.featureEnabled 'Install did not enable features.hooks.'
    Assert-True $installed.reviewRequired 'A changed Hook definition did not require /hooks review.'
    $configAfter = Get-Content -LiteralPath (Join-Path $fixture '.codex\config.toml') -Raw
    Assert-True ($configAfter.Contains('# preserve this comment')) 'Install removed an unrelated comment.'
    Assert-True ($configAfter.Contains('other_feature = true # preserve inline comment')) 'Install changed an unrelated feature.'
    Assert-True ($configAfter.Contains('[unrelated]')) 'Install removed an unrelated TOML table.'

    $hookHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $fixture '.codex\hooks.json')).Hash
    $configHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $fixture '.codex\config.toml')).Hash
    $updated = & $installer -Action Update -RepoRoot $fixture | ConvertFrom-Json
    Assert-True (-not $updated.changed) 'Byte-stable update reported a change.'
    Assert-True ($hookHash -eq (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $fixture '.codex\hooks.json')).Hash) 'Byte-stable update changed hooks.json.'
    Assert-True ($configHash -eq (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $fixture '.codex\config.toml')).Hash) 'Byte-stable update changed config.toml.'

    $query = & $installer -Action Query -RepoRoot $fixture | ConvertFrom-Json
    Assert-True $query.configured 'Query did not report configured=true.'
    Assert-True $query.featureEnabled 'Query did not report featureEnabled=true.'

    $fixtureKey = Get-RepositoryKey -Path $fixture
    foreach ($schemaVersion in @(68, 69, 70)) {
        Write-RuntimeDescriptor -DescriptorVersion 2 -SchemaVersion $schemaVersion -ControlApiVersions @(1) -RepositoryKey $fixtureKey
        $compatibility = & $installer -Action Query -RepoRoot $fixture | ConvertFrom-Json
        Assert-True $compatibility.daemonCompatible "Compatible descriptor was rejected for internal schema $schemaVersion."
    }
    Write-RuntimeDescriptor -DescriptorVersion 1 -SchemaVersion 69 -ControlApiVersions @(1) -RepositoryKey $fixtureKey
    Assert-True (-not (& $installer -Action Query -RepoRoot $fixture | ConvertFrom-Json).daemonCompatible) 'Unsupported old descriptor was accepted.'
    Write-RuntimeDescriptor -DescriptorVersion 3 -SchemaVersion 69 -ControlApiVersions @(1) -RepositoryKey $fixtureKey
    Assert-True (-not (& $installer -Action Query -RepoRoot $fixture | ConvertFrom-Json).daemonCompatible) 'Unsupported future descriptor was accepted.'
    Write-RuntimeDescriptor -DescriptorVersion 2 -SchemaVersion 69 -ControlApiVersions @(2) -RepositoryKey $fixtureKey
    Assert-True (-not (& $installer -Action Query -RepoRoot $fixture | ConvertFrom-Json).daemonCompatible) 'Descriptor without control API v1 was accepted.'
    Write-RuntimeDescriptor -DescriptorVersion 2 -SchemaVersion 69 -ControlApiVersions @(1) -RepositoryKey ('f' * 64)
    Assert-True (-not (& $installer -Action Query -RepoRoot $fixture | ConvertFrom-Json).daemonCompatible) 'Foreign repository descriptor was accepted.'

    $hookEntry = Join-Path $repoRoot '.codex\hooks\zircon_session_sync.py'
    $stopPayload = [pscustomobject]@{
        session_id = 'acceptance-thread'
        transcript_path = (Join-Path $fixture 'private-transcript.jsonl')
        cwd = $repoRoot
        hook_event_name = 'Stop'
        model = 'gpt-5-codex'
        permission_mode = 'default'
        turn_id = 'acceptance-turn'
        last_assistant_message = 'acceptance-secret-must-not-persist'
    } | ConvertTo-Json -Compress
    $stopOutput = $stopPayload | py -3 $hookEntry --event Stop | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Real Stop Hook entry point failed.' }
    Assert-True $stopOutput.continue 'Real Stop Hook did not emit continuation JSON.'

    $repoKeyBytes = [Text.Encoding]::UTF8.GetBytes($repoRoot.ToLowerInvariant())
    $repoHasher = [Security.Cryptography.SHA256]::Create()
    try { $repoKey = (($repoHasher.ComputeHash($repoKeyBytes) | ForEach-Object { $_.ToString('x2') }) -join '') } finally { $repoHasher.Dispose() }
    $realSpool = Join-Path $env:LOCALAPPDATA (Join-Path 'Zircon Session Coordinator\codex-hook' $repoKey)
    $realPending = Join-Path $realSpool 'pending'
    $realTriggerText = if (Test-Path -LiteralPath $realPending) {
        (@(Get-ChildItem -LiteralPath $realPending -Filter '*.json' -File | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw }) -join "`n")
    } else { '' }
    Assert-True (-not $realTriggerText.Contains('acceptance-secret-must-not-persist')) 'Real Hook persisted assistant content.'
    if (Test-Path -LiteralPath $realSpool) { Remove-Item -LiteralPath $realSpool -Recurse -Force }

    $spool = Join-Path $env:LOCALAPPDATA (Join-Path 'Zircon Session Coordinator\codex-hook' $fixtureKey)
    New-Item -ItemType Directory -Path (Join-Path $spool 'pending') -Force | Out-Null
    [IO.File]::WriteAllText((Join-Path $spool 'pending\trigger.json'), '{}')

    $removed = & $installer -Action Remove -RepoRoot $fixture | ConvertFrom-Json
    Assert-True (-not $removed.configured) 'Remove left the managed hooks.json configured.'
    Assert-True (-not $removed.featureEnabled) 'Remove left the managed features.hooks flag enabled.'
    Assert-True (-not (Test-Path -LiteralPath $spool)) 'Remove left the repository-scoped spool.'
    Assert-True (Test-Path -LiteralPath (Join-Path $fixture '.codex\config.toml')) 'Remove deleted project config.toml.'
    $remainingConfig = Get-Content -LiteralPath (Join-Path $fixture '.codex\config.toml') -Raw
    Assert-True ($remainingConfig.Contains('[unrelated]')) 'Remove changed unrelated project configuration.'

    Write-Host 'Codex Session Hook installer acceptance passed'
} finally {
    $env:LOCALAPPDATA = $oldLocalAppData
    if (Test-Path -LiteralPath $fixture) { Remove-Item -LiteralPath $fixture -Recurse -Force }
}
