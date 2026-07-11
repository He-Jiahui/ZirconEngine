[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [ValidateSet("Install", "Update", "Remove", "Query")]
    [string]$Action = "Query",
    [Parameter(Mandatory = $true)]
    [string]$TrayExecutable,
    [string]$RepoRoot,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

$resolvedRepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
$resolvedExecutable = [IO.Path]::GetFullPath($TrayExecutable)
if ($Action -in @("Install", "Update") -and -not (Test-Path -LiteralPath $resolvedExecutable -PathType Leaf)) {
    throw "Tray executable is missing: $resolvedExecutable"
}

$encoded = [Text.Encoding]::UTF8.GetBytes($resolvedRepoRoot.ToLowerInvariant())
$hasher = [Security.Cryptography.SHA256]::Create()
try {
    $hashBytes = $hasher.ComputeHash($encoded)
} finally {
    $hasher.Dispose()
}
$digest = (($hashBytes | ForEach-Object { $_.ToString("X2") }) -join "").Substring(0, 10)
$valueName = "ZirconSessionTray-$digest"
$providerPath = "Registry::HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run"
$registryPath = "HKCU\Software\Microsoft\Windows\CurrentVersion\Run"
$commandLine = '"' + $resolvedExecutable.Replace('"', '""') + '" --repo-root "' + $resolvedRepoRoot.Replace('"', '""') + '"'

function Get-StartupValueOrNull {
    try {
        $item = Get-ItemProperty -LiteralPath $providerPath -Name $valueName -ErrorAction Stop
        return [string]$item.PSObject.Properties[$valueName].Value
    }
    catch {
        if ($_.CategoryInfo.Category -eq [Management.Automation.ErrorCategory]::ObjectNotFound) {
            return $null
        }
        throw
    }
}

if ($Action -eq "Query") {
    $current = Get-StartupValueOrNull
    if ($null -eq $current) {
        Write-Output "$valueName is not installed"
    }
    else {
        Write-Output "$valueName=$current"
    }
    exit 0
}

if ($Action -eq "Remove") {
    if ($DryRun) {
        Write-Output "[$valueName] remove HKCU Run value"
        exit 0
    }
    if ($null -ne (Get-StartupValueOrNull) -and $PSCmdlet.ShouldProcess($valueName, "Remove current-user tray startup")) {
        Remove-ItemProperty -LiteralPath $providerPath -Name $valueName -ErrorAction Stop
    }
    if ($null -ne (Get-StartupValueOrNull)) {
        throw "Tray startup removal verification failed: $valueName"
    }
    exit 0
}

if ($DryRun) {
    Write-Output "[$valueName] set HKCU Run value to $commandLine"
    exit 0
}
if ($PSCmdlet.ShouldProcess($valueName, "Set current-user tray startup")) {
    & reg.exe ADD $registryPath /V $valueName /T REG_SZ /D $commandLine /F | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Tray startup registration failed: $valueName"
    }
}
if (-not [string]::Equals((Get-StartupValueOrNull), $commandLine, [StringComparison]::Ordinal)) {
    throw "Tray startup registration verification failed: $valueName"
}
Write-Output "Tray startup configuration ready for $resolvedRepoRoot"
