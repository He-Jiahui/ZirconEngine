[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Command = "status",
    [string]$RepoRoot,
    [switch]$Json,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

$resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$python = (Get-Command python -ErrorAction Stop).Source

function Get-BaseArguments {
    $base = @("-m", "tools.session_coordinator", "--repo-root", $resolvedRepoRoot)
    if ($Json) {
        $base += "--json"
    }
    return $base
}

function Start-Coordinator {
    $runtimePath = Join-Path $resolvedRepoRoot ".codex\state\session-coordinator\runtime.json"
    if (Test-Path -LiteralPath $runtimePath) {
        & $python @((Get-BaseArguments) + @("status")) *> $null
        if ($LASTEXITCODE -eq 0) {
            return
        }
    }

    $serveArguments = @("-m", "tools.session_coordinator", "--repo-root", $resolvedRepoRoot, "serve")
    Start-Process -FilePath $python -ArgumentList $serveArguments -WorkingDirectory $resolvedRepoRoot -WindowStyle Hidden | Out-Null

    for ($attempt = 0; $attempt -lt 300; $attempt++) {
        Start-Sleep -Milliseconds 100
        & $python @((Get-BaseArguments) + @("status")) *> $null
        if ($LASTEXITCODE -eq 0) {
            return
        }
    }
    throw "Zircon Session coordinator did not become healthy within 30 seconds."
}

if ($Command -eq "start") {
    Start-Coordinator
    & $python @((Get-BaseArguments) + @("status"))
    exit $LASTEXITCODE
}

if ($Command -notin @("status", "stop")) {
    Start-Coordinator
}

& $python @((Get-BaseArguments) + @($Command) + $Arguments)
exit $LASTEXITCODE
