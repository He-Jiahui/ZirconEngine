[CmdletBinding()]
param(
    [ValidateSet('Install', 'Update', 'Query', 'Remove')]
    [string]$Action = 'Query',
    [Parameter(Mandatory = $true)]
    [string]$ConfigFile,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$utf8 = New-Object System.Text.UTF8Encoding($false)
$OutputEncoding = $utf8
[Console]::OutputEncoding = $utf8
$runPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$namePrefix = 'ZirconJenkinsTray-'

function Fail([string]$Message) {
    [Console]::Error.WriteLine($Message)
    exit 2
}

function Require-AbsolutePlainPath([object]$Value, [string]$Field, [bool]$Directory = $false) {
    if ($Value -isnot [string] -or [string]::IsNullOrWhiteSpace($Value) -or $Value.IndexOfAny([char[]]@([char]0, [char]10, [char]13, [char]34)) -ge 0) { Fail "$Field must be a plain absolute path" }
    if (-not [System.IO.Path]::IsPathRooted($Value) -or $Value -match '(^|[\\/])\.([\\/]|$)' -or $Value -match '(^|[\\/])\.\.([\\/]|$)') { Fail "$Field must be an absolute path without aliases" }
    $full = [System.IO.Path]::GetFullPath($Value)
    if ($full -ine $Value) { Fail "$Field must use its canonical absolute spelling" }
    if ($Directory -and -not [System.IO.Directory]::Exists($Value)) { Fail "$Field directory is missing" }
    if (-not $Directory -and -not [System.IO.File]::Exists($Value)) { Fail "$Field file is missing" }
    return $Value
}

function Sha256([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Check-PhysicalPath([string]$Path, [string]$Field) {
    $current = [IO.DirectoryInfo]([IO.Path]::GetPathRoot($Path))
    foreach ($part in (([IO.Path]::GetFullPath($Path).Substring(([IO.Path]::GetPathRoot($Path)).Length)) -split '[\\/]')) {
        if ([string]::IsNullOrEmpty($part)) { continue }
        $current = [IO.DirectoryInfo](Join-Path $current.FullName $part)
        if (-not $current.Exists -and -not [IO.File]::Exists($current.FullName)) { continue }
        $attrs = [IO.File]::GetAttributes($current.FullName)
        if (($attrs -band [IO.FileAttributes]::ReparsePoint) -ne 0) { Fail "$Field contains a reparse point" }
    }
}

try {
    $configPath = Require-AbsolutePlainPath $ConfigFile 'ConfigFile'
    $cfg = Get-Content -LiteralPath $configPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($cfg.schemaVersion -ne 1) { Fail 'unsupported tray configuration schema' }
    $repo = Require-AbsolutePlainPath $cfg.repoRoot 'repoRoot' $true
    $pilot = Require-AbsolutePlainPath $cfg.pilotRoot 'pilotRoot' $true
    if ($pilot -notmatch '^[DEFdef]:\\cargo-targets\\.+\\jenkins-pilot-[^\\/]+$') { Fail 'pilotRoot must be a managed drive-root cargo-targets Jenkins root' }
    $pythonw = Require-AbsolutePlainPath $cfg.pythonwExecutable 'pythonwExecutable'
    $python = Require-AbsolutePlainPath $cfg.pythonExecutable 'pythonExecutable'
    $launch = Join-Path $repo 'tools\jenkins_tray\launch.py'
    Require-AbsolutePlainPath $launch 'launch.py' $false | Out-Null
    Check-PhysicalPath $repo 'repoRoot'; Check-PhysicalPath $pilot 'pilotRoot'; Check-PhysicalPath $pythonw 'pythonwExecutable'; Check-PhysicalPath $python 'pythonExecutable'
    if ([IO.Path]::GetFileName($pythonw) -cne 'pythonw.exe' -or [IO.Path]::GetFileName($python) -cne 'python.exe') { Fail 'Python executable names are invalid' }
    if ((Sha256 $pythonw) -ne ([string]$cfg.pythonwSha256).ToLowerInvariant()) { Fail 'pythonw hash mismatch' }
    if ((Sha256 $python) -ne ([string]$cfg.pythonSha256).ToLowerInvariant()) { Fail 'python hash mismatch' }
    $verify = & $python -B $launch --config $configPath validate-config 2>$null
    if ($LASTEXITCODE -ne 0) { Fail 'tray configuration validation failed' }
    $verified = $verify | ConvertFrom-Json
    if ($verified.ok -ne $true -or $verified.result.valid -ne $true -or $verified.result.pilotRoot -cne $pilot) { Fail 'tray configuration identity validation failed' }
    $digestInput = [Text.Encoding]::UTF8.GetBytes($repo.ToLowerInvariant() + "`0" + $pilot.ToLowerInvariant())
    $digest = ([Security.Cryptography.SHA256]::Create().ComputeHash($digestInput) | ForEach-Object { $_.ToString('x2') }) -join ''
    $valueName = $namePrefix + $digest.Substring(0, 16)
    $command = '"' + $pythonw + '" -B ' + ('"' + $launch + '"') + ' --config ' + ('"' + $configPath + '"') + ' tray'
    if ($command.Length -gt 260) { Fail 'startup command exceeds 260 characters' }
    $current = $null
    try { $current = (Get-ItemProperty -LiteralPath $runPath -Name $valueName -ErrorAction Stop).$valueName } catch { }
    $matches = ($null -ne $current -and [string]$current -ceq $command)
    if ($Action -eq 'Query') {
        [Console]::Out.WriteLine((ConvertTo-Json -Compress @{ action=$Action; key=$valueName; installed=($null -ne $current); matches=$matches; command=$current }))
    } elseif ($Action -eq 'Remove') {
        if ($null -ne $current -and -not $matches) { Fail 'existing startup value is not owned by this tray profile' }
        if ($null -ne $current -and -not $DryRun) { Remove-ItemProperty -LiteralPath $runPath -Name $valueName -Force }
        [Console]::Out.WriteLine((ConvertTo-Json -Compress @{ action=$Action; key=$valueName; installed=$false; changed=($null -ne $current -and -not $DryRun); dryRun=[bool]$DryRun }))
    } else {
        if ($null -ne $current -and -not $matches) { Fail 'existing startup value differs from this profile; preserve and reconcile it' }
        if (-not (Test-Path -LiteralPath $runPath) -and -not $DryRun) { New-Item -Path $runPath -Force | Out-Null }
        if (-not $DryRun -and -not $matches) { New-ItemProperty -LiteralPath $runPath -Name $valueName -Value $command -PropertyType String -Force | Out-Null }
        if (-not $DryRun -and (Get-ItemProperty -LiteralPath $runPath -Name $valueName).$valueName -cne $command) { Fail 'startup registration readback mismatch' }
        [Console]::Out.WriteLine((ConvertTo-Json -Compress @{ action=$Action; key=$valueName; installed=($null -ne $current -or -not $DryRun); matches=($matches -or -not $DryRun); changed=(!$matches -and -not $DryRun); dryRun=[bool]$DryRun; command=$command; predictedCommand=$command }))
    }
} catch { Fail $_.Exception.Message }
