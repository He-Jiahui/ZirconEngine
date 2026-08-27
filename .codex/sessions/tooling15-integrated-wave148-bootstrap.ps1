$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Set-Location 'E:\Git\ZirconEngine'

try {
    $env:MVP_POWERSHELL_VERSION = '7.4.19'
    $env:MVP_POWERSHELL_SHA256 = 'CD62AD6D8174CC6FB85B335A0058444BC934FE27C39FA97FE342134286D28AF9'
    $env:MVP_PESTER_VERSION = '4.10.1'
    $runtimeRoot = Join-Path 'D:\ZirconBuilds' ("tooling15-wave148-runtime-{0}" -f [DateTimeOffset]::Now.ToString('yyyyMMdd-HHmmss'))
    $powerShellRoot = Join-Path $runtimeRoot "pwsh-$env:MVP_POWERSHELL_VERSION"
    $env:MVP_PESTER_MODULE_ROOT = Join-Path $runtimeRoot 'psmodules'
    $env:MVP_EVIDENCE_ROOT = Join-Path $runtimeRoot 'evidence'
    $archivePath = Join-Path $runtimeRoot "PowerShell-$env:MVP_POWERSHELL_VERSION-win-x64.zip"
    $archiveUri = "https://github.com/PowerShell/PowerShell/releases/download/v$env:MVP_POWERSHELL_VERSION/PowerShell-$env:MVP_POWERSHELL_VERSION-win-x64.zip"
    New-Item -ItemType Directory -Force -Path $runtimeRoot, $powerShellRoot, $env:MVP_PESTER_MODULE_ROOT, $env:MVP_EVIDENCE_ROOT | Out-Null
    Write-Host "TOOLING15_WAVE148_RUNTIME_ROOT $runtimeRoot"
    Invoke-WebRequest -Uri $archiveUri -OutFile $archivePath
    $archiveSha256 = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash
    if (-not [string]::Equals($archiveSha256, $env:MVP_POWERSHELL_SHA256, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Wave148 PowerShell archive digest mismatch: expected=$env:MVP_POWERSHELL_SHA256 actual=$archiveSha256."
    }
    Expand-Archive -LiteralPath $archivePath -DestinationPath $powerShellRoot
    Save-Module -Name Pester -RequiredVersion $env:MVP_PESTER_VERSION -Path $env:MVP_PESTER_MODULE_ROOT -Repository PSGallery -Force -ErrorAction Stop
    $pinnedPwsh = Join-Path $powerShellRoot 'pwsh.exe'
    & $pinnedPwsh `
        -NoProfile `
        -NonInteractive `
        -ExecutionPolicy Bypass `
        -File '.\.codex\sessions\tooling15-integrated-wave148-pinned-runner.ps1'
    exit $LASTEXITCODE
}
catch {
    Write-Error $_
    exit 1
}
