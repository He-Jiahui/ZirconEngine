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
    $realTriggerText = (@(Get-ChildItem -LiteralPath (Join-Path $realSpool 'pending') -Filter '*.json' -File | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw }) -join "`n")
    Assert-True (-not $realTriggerText.Contains('acceptance-secret-must-not-persist')) 'Real Hook persisted assistant content.'
    Remove-Item -LiteralPath $realSpool -Recurse -Force

    $keyBytes = [Text.Encoding]::UTF8.GetBytes(([IO.Path]::GetFullPath($fixture)).ToLowerInvariant())
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { $key = (($hasher.ComputeHash($keyBytes) | ForEach-Object { $_.ToString('x2') }) -join '') } finally { $hasher.Dispose() }
    $spool = Join-Path $env:LOCALAPPDATA (Join-Path 'Zircon Session Coordinator\codex-hook' $key)
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
