[CmdletBinding()]
param(
    [string]$RepoRoot = (Get-Location).Path,
    [double]$LookbackHours = 4,
    [int]$MaxPlans = 10,
    [int]$MaxSessions = 10
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$formalPlanDir = Join-Path $resolvedRepoRoot 'docs\plans'
$legacyPlanDir = Join-Path $resolvedRepoRoot '.codex\plans'
$sessionDir = Join-Path $resolvedRepoRoot '.codex\sessions'
$cutoff = (Get-Date).AddHours(-$LookbackHours)
$timestampFormat = 'yyyy-MM-dd HH:mm:ss zzz'

function Get-RepoRelativePath {
    param(
        [string]$BasePath,
        [string]$TargetPath
    )

    return [System.IO.Path]::GetRelativePath($BasePath, $TargetPath).Replace('\', '/')
}

function Get-MarkdownInfo {
    param(
        [string]$Path
    )

    $lines = @(Get-Content -LiteralPath $Path -Encoding UTF8)
    $status = $null
    $heading = $null
    $inFrontmatter = $false

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]

        if ($i -eq 0 -and $line -eq '---') {
            $inFrontmatter = $true
            continue
        }

        if ($inFrontmatter) {
            if ($line -eq '---') {
                $inFrontmatter = $false
                continue
            }

            if (-not $status -and $line -match '^\s*status:\s*(.+?)\s*$') {
                $status = $matches[1].Trim().Trim("'`"")
            }

            continue
        }

        if (-not $heading -and $line -match '^\s*#\s+(.+?)\s*$') {
            $heading = $matches[1].Trim()
            break
        }
    }

    if (-not $heading) {
        $heading = [System.IO.Path]::GetFileNameWithoutExtension($Path)
    }

    if (-not $status) {
        $status = 'unknown'
    }

    return [pscustomobject]@{
        Heading = $heading
        Status = $status
    }
}

function Get-RecentPlanFiles {
    param(
        [string]$DirectoryPath,
        [datetime]$CutoffTime,
        [int]$Limit
    )

    if (-not (Test-Path -LiteralPath $DirectoryPath)) {
        return @()
    }

    return @(
        Get-ChildItem -LiteralPath $DirectoryPath -Recurse -File -Filter '*.md' |
            Where-Object { $_.LastWriteTime -ge $CutoffTime } |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First $Limit
    )
}

function Invoke-CoordinatorJson {
    param([string[]]$Arguments)

    $python = Get-Command python -ErrorAction SilentlyContinue
    if ($null -eq $python) {
        return $null
    }
    Push-Location $resolvedRepoRoot
    try {
        $raw = & $python.Source -m tools.session_coordinator --repo-root $resolvedRepoRoot --json @Arguments 2>$null
        if ($LASTEXITCODE -ne 0 -or -not $raw) {
            return $null
        }
        return ($raw -join [Environment]::NewLine) | ConvertFrom-Json
    }
    catch {
        return $null
    }
    finally {
        Pop-Location
    }
}

function Get-RecentSessionFiles {
    param(
        [string]$DirectoryPath,
        [datetime]$CutoffTime,
        [int]$Limit
    )

    if (-not (Test-Path -LiteralPath $DirectoryPath)) {
        return @()
    }

    return @(
        Get-ChildItem -LiteralPath $DirectoryPath -File -Filter '*.md' |
            Where-Object { $_.LastWriteTime -ge $CutoffTime } |
            Sort-Object LastWriteTime -Descending |
            ForEach-Object {
                $info = Get-MarkdownInfo -Path $_.FullName
                if ($info.Status -notmatch '^(completed|done|archived)$') {
                    [pscustomobject]@{
                        File = $_
                        Info = $info
                    }
                }
            } |
            Select-Object -First $Limit
    )
}

$output = New-Object System.Collections.Generic.List[string]
$output.Add('# Recent Coordination Context')
$output.Add('')
$output.Add("- Generated: $(Get-Date -Format $timestampFormat)")
$output.Add("- Lookback window: last $LookbackHours hour(s)")
$output.Add("- Cutoff: $($cutoff.ToString($timestampFormat))")
$output.Add('')
$output.Add('## Coordinator Service')

$coordinatorHealth = Invoke-CoordinatorJson -Arguments @('status')
if ($null -eq $coordinatorHealth) {
    $output.Add('- Offline; using recursive Markdown compatibility scan.')
}
else {
    $output.Add("- status=$($coordinatorHealth.status) | branch=$($coordinatorHealth.branch) | mode=$($coordinatorHealth.mode)")
    $coordinatorSessions = Invoke-CoordinatorJson -Arguments @('session', 'list')
    if ($null -ne $coordinatorSessions) {
        $output.Add("- indexed Sessions: $(@($coordinatorSessions.sessions).Count)")
    }
    $failureAudit = Invoke-CoordinatorJson -Arguments @('failure', 'audit')
    if ($null -ne $failureAudit) {
        $output.Add("- Failure graph: $($failureAudit.audit.node_count) node(s), $(@($failureAudit.audit.diagnostics).Count) diagnostic(s)")
    }
}

$output.Add('')
$output.Add('## Recent Plans')

$recentPlans = @(
    @(
        Get-RecentPlanFiles -DirectoryPath $formalPlanDir -CutoffTime $cutoff -Limit ($MaxPlans * 2)
        Get-RecentPlanFiles -DirectoryPath $legacyPlanDir -CutoffTime $cutoff -Limit $MaxPlans
    ) |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First $MaxPlans
)
if ($recentPlans.Count -eq 0) {
    $output.Add('- No plan files updated within the lookback window.')
}
else {
    foreach ($plan in $recentPlans) {
        $info = Get-MarkdownInfo -Path $plan.FullName
        $relativePath = Get-RepoRelativePath -BasePath $resolvedRepoRoot -TargetPath $plan.FullName
        $output.Add(("- {0} | `{1}` | {2}" -f $plan.LastWriteTime.ToString('yyyy-MM-dd HH:mm'), $relativePath, $info.Heading))
    }
}

$output.Add('')
$output.Add('## Active Session Notes')

if (-not (Test-Path -LiteralPath $sessionDir)) {
    $output.Add('- `.codex/sessions/` is missing. Create it before publishing the current task state.')
}
else {
    $recentSessions = @(Get-RecentSessionFiles -DirectoryPath $sessionDir -CutoffTime $cutoff -Limit $MaxSessions)
    if ($recentSessions.Count -eq 0) {
        $output.Add('- No active session notes updated within the lookback window.')
    }
    else {
        foreach ($session in $recentSessions) {
            $relativePath = Get-RepoRelativePath -BasePath $resolvedRepoRoot -TargetPath $session.File.FullName
            $status = $session.Info.Status
            $output.Add(("- {0} | status={1} | `{2}` | {3}" -f $session.File.LastWriteTime.ToString('yyyy-MM-dd HH:mm'), $status, $relativePath, $session.Info.Heading))
        }
    }
}

$output -join [Environment]::NewLine
