[CmdletBinding()]
param(
    [ValidateSet('Query', 'Install', 'Update', 'Remove')]
    [string]$Action = 'Query',
    [string]$RepoRoot,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}
$resolvedRepo = (Resolve-Path -LiteralPath $RepoRoot).Path
$codexRoot = Join-Path $resolvedRepo '.codex'
$hooksPath = Join-Path $codexRoot 'hooks.json'
$configPath = Join-Path $codexRoot 'config.toml'

function Get-RepositoryKey {
    $bytes = [Text.Encoding]::UTF8.GetBytes($resolvedRepo.ToLowerInvariant())
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { $hash = $hasher.ComputeHash($bytes) } finally { $hasher.Dispose() }
    return (($hash | ForEach-Object { $_.ToString('x2') }) -join '')
}

function New-HookHandler {
    param([string]$Event, [switch]$WithStatus)
    $handler = [ordered]@{
        type = 'command'
        command = 'python3 "$(git rev-parse --show-toplevel)/.codex/hooks/zircon_session_sync.py" --event ' + $Event
        commandWindows = 'powershell.exe -NoProfile -NonInteractive -Command "& { $root = (& git rev-parse --show-toplevel); & py -3 (Join-Path $root ''.codex/hooks/zircon_session_sync.py'') --event ' + $Event + ' }"'
        timeout = 5
    }
    if ($WithStatus) { $handler.statusMessage = 'Synchronizing Zircon Session' }
    return [pscustomobject]$handler
}

function Get-ManagedHookObject {
    return [pscustomobject][ordered]@{
        hooks = [pscustomobject][ordered]@{
            SessionStart = @([pscustomobject][ordered]@{
                matcher = 'startup|resume|clear|compact'
                hooks = @(New-HookHandler -Event 'SessionStart' -WithStatus)
            })
            UserPromptSubmit = @([pscustomobject][ordered]@{
                hooks = @(New-HookHandler -Event 'UserPromptSubmit')
            })
            Stop = @([pscustomobject][ordered]@{
                hooks = @(New-HookHandler -Event 'Stop')
            })
            SubagentStart = @([pscustomobject][ordered]@{
                hooks = @(New-HookHandler -Event 'SubagentStart')
            })
            SubagentStop = @([pscustomobject][ordered]@{
                hooks = @(New-HookHandler -Event 'SubagentStop')
            })
        }
    }
}

function ConvertTo-CanonicalJson {
    param([object]$Value)
    return ($Value | ConvertTo-Json -Depth 10 -Compress)
}

function Test-ManagedHooks {
    if (-not (Test-Path -LiteralPath $hooksPath -PathType Leaf)) { return $false }
    try {
        $current = Get-Content -LiteralPath $hooksPath -Raw -Encoding UTF8 | ConvertFrom-Json
        return [string]::Equals(
            (ConvertTo-CanonicalJson $current),
            (ConvertTo-CanonicalJson (Get-ManagedHookObject)),
            [StringComparison]::Ordinal
        )
    } catch { return $false }
}

function Test-HooksFeatureEnabled {
    if (-not (Test-Path -LiteralPath $configPath -PathType Leaf)) { return $false }
    $text = Get-Content -LiteralPath $configPath -Raw -Encoding UTF8
    $match = [regex]::Match($text, '(?ms)^\s*\[features\]\s*$\s*(?<body>.*?)(?=^\s*\[|\z)')
    return $match.Success -and [regex]::IsMatch($match.Groups['body'].Value, '(?m)^\s*hooks\s*=\s*true\s*(?:#.*)?$')
}

function Enable-HooksFeature {
    param([string]$Text)
    $lines = [Collections.Generic.List[string]]::new()
    foreach ($line in [regex]::Split($Text, '\r?\n')) { $lines.Add($line) }
    $feature = -1
    for ($index = 0; $index -lt $lines.Count; $index++) {
        if ($lines[$index].Trim() -eq '[features]') { $feature = $index; break }
    }
    if ($feature -lt 0) {
        while ($lines.Count -gt 0 -and $lines[$lines.Count - 1] -eq '') { $lines.RemoveAt($lines.Count - 1) }
        if ($lines.Count -gt 0) { $lines.Add('') }
        $lines.Add('[features]')
        $lines.Add('hooks = true')
    } else {
        $end = $lines.Count
        for ($index = $feature + 1; $index -lt $lines.Count; $index++) {
            if ($lines[$index] -match '^\s*\[') { $end = $index; break }
        }
        $hookLine = -1
        for ($index = $feature + 1; $index -lt $end; $index++) {
            if ($lines[$index] -match '^\s*hooks\s*=') { $hookLine = $index; break }
        }
        if ($hookLine -ge 0) {
            $comment = if ($lines[$hookLine] -match '(#.*)$') { ' ' + $Matches[1] } else { '' }
            $lines[$hookLine] = 'hooks = true' + $comment
        } else {
            $lines.Insert($feature + 1, 'hooks = true')
        }
    }
    return (($lines -join "`n").TrimEnd("`n") + "`n")
}

function Disable-HooksFeature {
    param([string]$Text)
    $lines = [Collections.Generic.List[string]]::new()
    foreach ($line in [regex]::Split($Text, '\r?\n')) { $lines.Add($line) }
    $feature = -1
    for ($index = 0; $index -lt $lines.Count; $index++) {
        if ($lines[$index].Trim() -eq '[features]') { $feature = $index; break }
    }
    if ($feature -ge 0) {
        $end = $lines.Count
        for ($index = $feature + 1; $index -lt $lines.Count; $index++) {
            if ($lines[$index] -match '^\s*\[') { $end = $index; break }
        }
        for ($index = $feature + 1; $index -lt $end; $index++) {
            if ($lines[$index] -match '^\s*hooks\s*=\s*true\s*(?:#.*)?$') {
                $lines.RemoveAt($index)
                break
            }
        }
    }
    return (($lines -join "`n").TrimEnd("`n") + "`n")
}

function Write-AtomicUtf8 {
    param([string]$Path, [string]$Text)
    $parent = Split-Path -Parent $Path
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    $temporary = Join-Path $parent ('.tmp-' + [guid]::NewGuid().ToString('N'))
    try {
        [IO.File]::WriteAllText($temporary, $Text, [Text.UTF8Encoding]::new($false))
        Move-Item -LiteralPath $temporary -Destination $Path -Force
    } finally {
        Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
    }
}

function Get-SpoolPath {
    $local = [Environment]::GetEnvironmentVariable('LOCALAPPDATA')
    if ([string]::IsNullOrWhiteSpace($local)) { return $null }
    $base = [IO.Path]::GetFullPath((Join-Path $local 'Zircon Session Coordinator\codex-hook'))
    $spool = [IO.Path]::GetFullPath((Join-Path $base (Get-RepositoryKey)))
    $prefix = $base.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if (-not $spool.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Repository spool escaped the managed LocalAppData root.'
    }
    return $spool
}

function Test-DaemonCompatible {
    $runtimePath = Join-Path $resolvedRepo '.codex\state\session-coordinator\runtime.json'
    try {
        $runtime = Get-Content -LiteralPath $runtimePath -Raw -Encoding UTF8 | ConvertFrom-Json
        return $runtime.descriptor_version -eq 2 -and
            $runtime.host -eq '127.0.0.1' -and
            $runtime.repository_key -eq (Get-RepositoryKey) -and
            $runtime.schema_version -eq 27 -and
            @($runtime.control_api_versions) -contains 1
    } catch { return $false }
}

$configured = Test-ManagedHooks
$featureEnabled = Test-HooksFeatureEnabled
$changed = $false
$reviewRequired = $configured

if ($Action -in @('Install', 'Update')) {
    $desiredHooks = (Get-ManagedHookObject | ConvertTo-Json -Depth 10) + "`n"
    $configText = if (Test-Path -LiteralPath $configPath) {
        Get-Content -LiteralPath $configPath -Raw -Encoding UTF8
    } else { '' }
    $desiredConfig = Enable-HooksFeature $configText
    $changed = (-not $configured) -or (-not $featureEnabled)
    $reviewRequired = $changed
    if (-not $DryRun) {
        if (-not $configured) { Write-AtomicUtf8 -Path $hooksPath -Text $desiredHooks }
        if (-not $featureEnabled) { Write-AtomicUtf8 -Path $configPath -Text $desiredConfig }
        $configured = Test-ManagedHooks
        $featureEnabled = Test-HooksFeatureEnabled
        if (-not $configured -or -not $featureEnabled) { throw 'Codex Hook installation verification failed.' }
    }
} elseif ($Action -eq 'Remove') {
    $reviewRequired = $false
    if ((Test-Path -LiteralPath $hooksPath) -and -not $configured) {
        throw 'Refusing to remove a project hooks.json that is not the exact managed definition.'
    }
    $spool = Get-SpoolPath
    $changed = $configured -or ($null -ne $spool -and (Test-Path -LiteralPath $spool))
    if (-not $DryRun) {
        if ($configured) { Remove-Item -LiteralPath $hooksPath -Force }
        if ($featureEnabled) {
            $configText = Get-Content -LiteralPath $configPath -Raw -Encoding UTF8
            Write-AtomicUtf8 -Path $configPath -Text (Disable-HooksFeature $configText)
            $featureEnabled = Test-HooksFeatureEnabled
        }
        if ($null -ne $spool -and (Test-Path -LiteralPath $spool)) {
            Remove-Item -LiteralPath $spool -Recurse -Force
        }
        $configured = Test-ManagedHooks
    }
}

$spoolPath = Get-SpoolPath
$pending = if ($null -ne $spoolPath -and (Test-Path -LiteralPath (Join-Path $spoolPath 'pending'))) {
    @(Get-ChildItem -LiteralPath (Join-Path $spoolPath 'pending') -Filter '*.json' -File -ErrorAction SilentlyContinue).Count
} else { 0 }
$quarantined = if ($null -ne $spoolPath -and (Test-Path -LiteralPath (Join-Path $spoolPath 'quarantine'))) {
    @(Get-ChildItem -LiteralPath (Join-Path $spoolPath 'quarantine') -Filter '*.json' -File -ErrorAction SilentlyContinue).Count
} else { 0 }

[pscustomobject][ordered]@{
    action = $Action
    configured = $configured
    featureEnabled = $featureEnabled
    reviewRequired = $reviewRequired
    daemonCompatible = (Test-DaemonCompatible)
    pendingCount = $pending
    quarantineCount = $quarantined
    changed = $changed
    dryRun = [bool]$DryRun
} | ConvertTo-Json -Compress
