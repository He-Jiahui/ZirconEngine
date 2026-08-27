$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Set-Location 'E:\Git\ZirconEngine'

$fixturePathsModule = Join-Path $PSScriptRoot '..\..\tools\mvp\MvpTestFixturePaths.psm1'
Import-Module $fixturePathsModule -Force -ErrorAction Stop

$runtimeRoot = $null
$runnerExitCode = $null
$primaryError = $null
$cleanupError = $null
try {
    if ($env:MVP_VALIDATION_WAVE -notmatch '^\d+$') {
        throw 'Tooling15 validation wave must be numeric.'
    }
    $wave = $env:MVP_VALIDATION_WAVE
    $env:MVP_POWERSHELL_VERSION = '7.4.19'
    $env:MVP_POWERSHELL_SHA256 = 'CD62AD6D8174CC6FB85B335A0058444BC934FE27C39FA97FE342134286D28AF9'
    $env:MVP_PESTER_VERSION = '4.10.1'
    $runtimeRoot = New-MvpTestFixtureRoot -Prefix "tooling15-wave${wave}-runtime"
    $powerShellRoot = Join-Path $runtimeRoot "pwsh-$env:MVP_POWERSHELL_VERSION"
    $env:MVP_PESTER_MODULE_ROOT = Join-Path $runtimeRoot 'psmodules'
    $env:MVP_EVIDENCE_ROOT = Join-Path $runtimeRoot 'evidence'
    $archivePath = Join-Path $runtimeRoot "PowerShell-$env:MVP_POWERSHELL_VERSION-win-x64.zip"
    $archiveUri = "https://github.com/PowerShell/PowerShell/releases/download/v$env:MVP_POWERSHELL_VERSION/PowerShell-$env:MVP_POWERSHELL_VERSION-win-x64.zip"
    New-Item -ItemType Directory -Force -Path $runtimeRoot, $powerShellRoot, $env:MVP_PESTER_MODULE_ROOT, $env:MVP_EVIDENCE_ROOT | Out-Null
    Write-Host "TOOLING15_WAVE${wave}_RUNTIME_ROOT $runtimeRoot"
    Invoke-WebRequest -Uri $archiveUri -OutFile $archivePath
    $archiveSha256 = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash
    if (-not [string]::Equals($archiveSha256, $env:MVP_POWERSHELL_SHA256, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Wave$wave PowerShell archive digest mismatch: expected=$env:MVP_POWERSHELL_SHA256 actual=$archiveSha256."
    }
    Expand-Archive -LiteralPath $archivePath -DestinationPath $powerShellRoot
    Save-Module -Name Pester -RequiredVersion $env:MVP_PESTER_VERSION -Path $env:MVP_PESTER_MODULE_ROOT -Repository PSGallery -Force -ErrorAction Stop
    $pinnedPwsh = Join-Path $powerShellRoot 'pwsh.exe'
    & $pinnedPwsh `
        -NoProfile `
        -NonInteractive `
        -ExecutionPolicy Bypass `
        -File '.\.codex\sessions\tooling15-integrated-pinned-runner.ps1'
    $runnerExitCode = $LASTEXITCODE
}
catch {
    $primaryError = $_
}
finally {
    if ($null -ne $runtimeRoot) {
        try {
            Remove-MvpTestFixtureRoot -Path $runtimeRoot
        }
        catch {
            $cleanupError = $_
        }
    }
}

if ($null -ne $primaryError) {
    if ($null -ne $cleanupError) {
        Write-Warning "Tooling15 fixture cleanup also failed: $($cleanupError.Exception.Message)"
    }
    Write-Error $primaryError
    exit 1
}
if ($null -ne $cleanupError) {
    Write-Error $cleanupError
    exit 1
}
exit $runnerExitCode
