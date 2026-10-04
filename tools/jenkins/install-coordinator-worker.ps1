[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [ValidateSet("Install", "Update", "Remove", "Query", "Package")]
    [string]$Action = "Query",
    [string]$RepoRoot,
    [string]$InstallRoot,
    [string]$StorageRoot,
    [string]$Endpoint = "https://127.0.0.1:6518",
    [string]$NodeId,
    [string]$Token,
    [string]$TaskName,
    [string]$Python = "python.exe",
    [string]$Archive,
    [string]$CaFile,
    [string]$ClientCertificate,
    [string]$ClientKey,
    [string]$ChildEndpoint,
    [string]$VsDevCmdPath = $env:ZIRCON_VSDEVCMD_PATH,
    [string[]]$Label = @(),
    [switch]$PrepareMsvc,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($Action -notin @('Query', 'Remove')) {
    throw 'The old coordinator worker is retired. Installation, update and packaging are disabled; only Query and Remove are supported.'
}

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
}
$resolvedRepoRoot = [IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($InstallRoot)) {
    $InstallRoot = $env:ZIRCON_WORKER_INSTALL_ROOT
}
if ([string]::IsNullOrWhiteSpace($StorageRoot)) {
    $StorageRoot = $env:ZIRCON_WORKER_STORAGE_ROOT
}
if ([string]::IsNullOrWhiteSpace($NodeId)) { $NodeId = $env:ZIRCON_WORKER_NODE_ID }
if ([string]::IsNullOrWhiteSpace($Token)) { $Token = $env:ZIRCON_WORKER_TOKEN }

$hasher = [Security.Cryptography.SHA256]::Create()
try {
    $digest = $hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($resolvedRepoRoot.ToLowerInvariant()))
} finally {
    $hasher.Dispose()
}
if ([string]::IsNullOrWhiteSpace($TaskName)) {
    $TaskName = "ZirconCoordinatorWorker-" + ((($digest | ForEach-Object { $_.ToString("x2") }) -join "").Substring(0, 12))
}
if ([string]::IsNullOrWhiteSpace($InstallRoot) -and $Action -in @("Install", "Update", "Remove")) {
    throw "Use an explicit -InstallRoot (or ZIRCON_WORKER_INSTALL_ROOT) for worker software and credentials."
}
if (-not [string]::IsNullOrWhiteSpace($InstallRoot)) {
    if ($InstallRoot -notmatch "^[A-Za-z]:[\\/]" -or $InstallRoot.StartsWith("\\")) {
        throw "-InstallRoot must be an absolute local path, not a UNC or device path."
    }
    $InstallRoot = [IO.Path]::GetFullPath($InstallRoot)
}
if ($Action -in @("Install", "Update")) {
    if ([string]::IsNullOrWhiteSpace($StorageRoot)) {
        throw "Install/Update requires an explicit dedicated -StorageRoot (or ZIRCON_WORKER_STORAGE_ROOT)."
    }
    if ($StorageRoot -notmatch "^[A-Za-z]:[\\/]" -or $StorageRoot.StartsWith("\\")) {
        throw "-StorageRoot must be an absolute local path, not a UNC or device path."
    }
    $StorageRoot = [IO.Path]::GetFullPath($StorageRoot)
    $installPrefix = $InstallRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    $storagePrefix = $StorageRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if ($InstallRoot.Equals($StorageRoot, [StringComparison]::OrdinalIgnoreCase) -or
        $InstallRoot.StartsWith($storagePrefix, [StringComparison]::OrdinalIgnoreCase) -or
        $StorageRoot.StartsWith($installPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "-InstallRoot and -StorageRoot must be separate directory trees."
    }
    if ([string]::IsNullOrWhiteSpace($NodeId) -or [string]::IsNullOrWhiteSpace($Token)) {
        throw "Install/Update requires -NodeId and -Token (or their ZIRCON_WORKER_* environment variables)."
    }
    if (($ClientCertificate -and -not $ClientKey) -or ($ClientKey -and -not $ClientCertificate)) {
        throw "-ClientCertificate and -ClientKey must be supplied together."
    }
}

$packageRoot = if ($InstallRoot) { Join-Path $InstallRoot "package" } else { Join-Path $resolvedRepoRoot "package" }
$pythonRoot = if ($InstallRoot) { Join-Path $InstallRoot "python" } else { Join-Path $resolvedRepoRoot ".codex\state\coordinator-worker\python" }
$pythonExe = Join-Path $pythonRoot "Scripts\python.exe"
$configPath = if ($InstallRoot) { Join-Path $InstallRoot "worker.json" } else { $null }
$launcherPath = if ($InstallRoot) { Join-Path $InstallRoot "run-coordinator-worker.ps1" } else { Join-Path $resolvedRepoRoot "run-coordinator-worker.ps1" }
$archivePath = if ($Archive) { [IO.Path]::GetFullPath($Archive) } else { Join-Path $resolvedRepoRoot "coordinator-worker.zip" }

function Invoke-External {
    param([string]$File, [string[]]$Arguments)
    if ($DryRun) {
        Write-Output ("[dry-run] {0} {1}" -f $File, ($Arguments -join " "))
        return
    }
    & $File @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Command failed: $File" }
}

function Copy-WorkerPackage {
    param([string]$SourceRoot, [string]$DestinationRoot)
    $sessionSource = Join-Path $SourceRoot "tools\session_coordinator"
    $requiredFiles = @(
        "__init__.py", "models.py", "portable_paths.py", "cargo_command_policy.py",
        "processes.py", "windows_job_process.py", "requirements-worker.txt"
    )
    foreach ($relative in $requiredFiles) {
        $source = if ($relative -eq "requirements-worker.txt") {
            Join-Path $SourceRoot $relative
        } else {
            Join-Path $sessionSource $relative
        }
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "Worker package source is missing: $source"
        }
    }
    if (-not (Test-Path -LiteralPath (Join-Path $sessionSource "remote_worker") -PathType Container)) {
        throw "Worker package source folder is missing: remote_worker"
    }
    $destinationSession = Join-Path $DestinationRoot "tools\session_coordinator"
    New-Item -ItemType Directory -Force -Path $destinationSession | Out-Null
    foreach ($relative in $requiredFiles) {
        if ($relative -eq "requirements-worker.txt") { continue }
        Copy-Item -LiteralPath (Join-Path $sessionSource $relative) -Destination $destinationSession -Force
    }
    Copy-Item -LiteralPath (Join-Path $SourceRoot "requirements-worker.txt") -Destination $DestinationRoot -Force
    Copy-WorkerTree `
        -SourceRoot (Join-Path $sessionSource "remote_worker") `
        -DestinationRoot (Join-Path $destinationSession "remote_worker")
    $protocolDestination = Join-Path $destinationSession "worker_protocol"
    New-Item -ItemType Directory -Force -Path $protocolDestination | Out-Null
    Set-Content -LiteralPath (Join-Path $protocolDestination "__init__.py") -Value '"""Worker protocol models and provider contracts."""' -Encoding UTF8
    foreach ($relative in @("models.py", "providers.py", "cache.py")) {
        $source = Join-Path (Join-Path $sessionSource "worker_protocol") $relative
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "Worker protocol source is missing: $source"
        }
        Copy-Item -LiteralPath $source -Destination $protocolDestination -Force
    }
    $remoteTransportSource = Join-Path $sessionSource "remote_transport"
    $remoteTransportDestination = Join-Path $destinationSession "remote_transport"
    New-Item -ItemType Directory -Force -Path $remoteTransportDestination | Out-Null
    Set-Content -LiteralPath (Join-Path $remoteTransportDestination "__init__.py") -Value '"""Worker transport wire helpers."""' -Encoding UTF8
    foreach ($relative in @("protocol.py")) {
        $source = Join-Path $remoteTransportSource $relative
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "Worker package source is missing: $source"
        }
        Copy-Item -LiteralPath $source -Destination $remoteTransportDestination -Force
    }
    $sourceInstaller = Join-Path $SourceRoot "tools\jenkins\install-coordinator-worker.ps1"
    if (-not (Test-Path -LiteralPath $sourceInstaller -PathType Leaf)) {
        throw "Worker installer is missing: $sourceInstaller"
    }
    $toolsDestination = Join-Path $DestinationRoot "tools"
    New-Item -ItemType Directory -Force -Path $toolsDestination | Out-Null
    Set-Content -LiteralPath (Join-Path $toolsDestination "__init__.py") -Value '"""Standalone Zircon worker package root."""' -Encoding UTF8
    Copy-Item -LiteralPath $sourceInstaller -Destination $toolsDestination -Force
}

function Copy-WorkerTree {
    param([string]$SourceRoot, [string]$DestinationRoot)
    if (-not (Test-Path -LiteralPath $SourceRoot -PathType Container)) {
        throw "Worker package source folder is missing: $SourceRoot"
    }
    [IO.Directory]::CreateDirectory($DestinationRoot) | Out-Null
    foreach ($sourceFile in [IO.Directory]::EnumerateFiles($SourceRoot)) {
        $extension = [IO.Path]::GetExtension($sourceFile)
        if ($extension.Equals(".pyc", [StringComparison]::OrdinalIgnoreCase) -or
            $extension.Equals(".pyo", [StringComparison]::OrdinalIgnoreCase)) {
            continue
        }
        $destinationFile = Join-Path $DestinationRoot ([IO.Path]::GetFileName($sourceFile))
        [IO.File]::Copy($sourceFile, $destinationFile, $true)
    }
    foreach ($sourceDirectory in [IO.Directory]::EnumerateDirectories($SourceRoot)) {
        if ([IO.Path]::GetFileName($sourceDirectory).Equals("__pycache__", [StringComparison]::OrdinalIgnoreCase)) {
            continue
        }
        $destinationDirectory = Join-Path $DestinationRoot ([IO.Path]::GetFileName($sourceDirectory))
        Copy-WorkerTree -SourceRoot $sourceDirectory -DestinationRoot $destinationDirectory
    }
}

function Write-WorkerLauncher {
    $content = @'
[CmdletBinding()]
param()
$ErrorActionPreference = "Stop"
$install = $PSScriptRoot
$package = Join-Path $install "package"
$state = Join-Path $install "worker.json"
$settings = Get-Content -Raw -LiteralPath $state | ConvertFrom-Json
$secure = ConvertTo-SecureString ([string]$settings.credentialProtected)
$token = [System.Net.NetworkCredential]::new("worker", $secure).Password
$python = Join-Path $install "python\Scripts\python.exe"
if (-not (Test-Path -LiteralPath $python)) { $python = "python.exe" }
$workerStorage = [string]$settings.storageRoot
$workerTemp = Join-Path $workerStorage "temp"
New-Item -ItemType Directory -Force -Path $workerTemp | Out-Null
$env:TEMP = $workerTemp
$env:TMP = $workerTemp
$env:PYTHONDONTWRITEBYTECODE = "1"
$env:PYTHONPATH = $package
$env:ZIRCON_WORKER_STORAGE_ROOT = $workerStorage
$env:ZIRCON_WORKER_NODE_ID = [string]$settings.nodeId
$env:ZIRCON_WORKER_TOKEN = $token
$env:ZIRCON_COORDINATOR_ENDPOINT = [string]$settings.endpoint
$arguments = @("-m", "tools.session_coordinator.remote_worker", "serve")
if ($settings.caFile) { $arguments += @("--ca-file", [string]$settings.caFile) }
if ($settings.clientCertificate) { $arguments += @("--client-certificate", [string]$settings.clientCertificate) }
if ($settings.clientKey) { $arguments += @("--client-key", [string]$settings.clientKey) }
if ($settings.childEndpoint) { $arguments += @("--child-endpoint", [string]$settings.childEndpoint) }
if ($settings.prepareMsvc) { $arguments += "--prepare-msvc" }
if ($settings.vsDevCmdPath) { $arguments += @("--vsdevcmd-path", [string]$settings.vsDevCmdPath) }
foreach ($label in @($settings.labels)) { if ($label) { $arguments += @("--label", [string]$label) } }
& $python @arguments
exit $LASTEXITCODE
'@
    Set-Content -LiteralPath $launcherPath -Value $content -Encoding UTF8
}

function Register-WorkerTask {
    $powershell = (Get-Command powershell.exe -ErrorAction Stop).Source
    $arguments = @("-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass", "-File", $launcherPath)
    $command = ('"{0}" {1}' -f $powershell, (($arguments | ForEach-Object { '"' + $_.Replace('"', '""') + '"' }) -join " "))
    Invoke-External "schtasks.exe" @("/Create", "/TN", $TaskName, "/SC", "ONLOGON", "/RL", "LIMITED", "/F", "/TR", $command)
}

function Test-SafeArchive {
    param([string]$Path, [string]$Destination)
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $destinationFull = [IO.Path]::GetFullPath($Destination).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    $zip = [IO.Compression.ZipFile]::OpenRead($Path)
    try {
        foreach ($entry in $zip.Entries) {
            $candidate = [IO.Path]::GetFullPath((Join-Path $Destination $entry.FullName))
            if (-not $candidate.StartsWith($destinationFull, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Worker archive contains a path outside its install directory."
            }
        }
    } finally {
        $zip.Dispose()
    }
}

switch ($Action) {
    "Query" {
        $configured = $configPath -and (Test-Path -LiteralPath $configPath -PathType Leaf)
        [pscustomobject]@{ taskName = $TaskName; configured = $configured; installRoot = $InstallRoot; state = $configPath }
        return
    }
    "Remove" {
        if (Get-Command schtasks.exe -ErrorAction SilentlyContinue) {
            Invoke-External "schtasks.exe" @("/Delete", "/TN", $TaskName, "/F")
        }
        Write-Output "Removed scheduled task $TaskName; worker code, credentials, and storage remain for explicit operator cleanup."
        return
    }
    "Package" {
        if ($DryRun) {
            Write-Output "[dry-run] package standalone worker at $archivePath"
            return
        }
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $parent = Split-Path -Parent $archivePath
        if (-not [string]::IsNullOrWhiteSpace($parent)) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
        $stageRoot = Join-Path ([IO.Path]::GetTempPath()) ("zircon-worker-package-" + [Guid]::NewGuid().ToString("N"))
        New-Item -ItemType Directory -Path $stageRoot | Out-Null
        try {
            Copy-WorkerPackage -SourceRoot $resolvedRepoRoot -DestinationRoot $stageRoot
            if (Test-Path -LiteralPath $archivePath) { Remove-Item -LiteralPath $archivePath -Force }
            [IO.Compression.ZipFile]::CreateFromDirectory($stageRoot, $archivePath, [IO.Compression.CompressionLevel]::Optimal, $false)
            Write-Output $archivePath
        } finally {
            $tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
            $stageFull = [IO.Path]::GetFullPath($stageRoot)
            if ($stageFull.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase) -and
                $stageFull.StartsWith("$(Join-Path ([IO.Path]::GetTempPath()) 'zircon-worker-package-')", [StringComparison]::OrdinalIgnoreCase)) {
                Remove-Item -LiteralPath $stageFull -Recurse -Force
            } else {
                throw "Refusing to remove an unexpected package staging path."
            }
        }
        return
    }
    "Install" { }
    "Update" { }
}

if ($DryRun) {
    Write-Output "[dry-run] install worker package at $InstallRoot using storage $StorageRoot and register $TaskName"
    return
}

New-Item -ItemType Directory -Force -Path $InstallRoot, $StorageRoot | Out-Null
$workerTemp = Join-Path $StorageRoot "temp"
$pipCacheRoot = Join-Path $StorageRoot "pip-cache"
New-Item -ItemType Directory -Force -Path $workerTemp, $pipCacheRoot | Out-Null
$env:TEMP = $workerTemp
$env:TMP = $workerTemp
$env:PIP_CACHE_DIR = $pipCacheRoot
$env:PYTHONDONTWRITEBYTECODE = "1"
$expectedPackageRoot = [IO.Path]::GetFullPath((Join-Path $InstallRoot "package"))
$resolvedPackageRoot = [IO.Path]::GetFullPath($packageRoot)
if (-not $resolvedPackageRoot.Equals($expectedPackageRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to replace a worker package outside the explicit install root."
}
if (Test-Path -LiteralPath $resolvedPackageRoot) {
    $existingPackage = Get-Item -LiteralPath $resolvedPackageRoot -Force
    if (($existingPackage.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Refusing to replace a linked worker package directory."
    }
    Remove-Item -LiteralPath $resolvedPackageRoot -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $resolvedPackageRoot | Out-Null
if ($Archive) {
    $archiveFull = [IO.Path]::GetFullPath($Archive)
    if (-not (Test-Path -LiteralPath $archiveFull -PathType Leaf)) { throw "Worker archive does not exist: $archiveFull" }
    Test-SafeArchive -Path $archiveFull -Destination $packageRoot
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    Expand-Archive -LiteralPath $archiveFull -DestinationPath $packageRoot -Force
} else {
    Copy-WorkerPackage -SourceRoot $resolvedRepoRoot -DestinationRoot $packageRoot
}

$secureToken = ConvertTo-SecureString -String $Token -AsPlainText -Force
$settings = [ordered]@{
    endpoint = $Endpoint
    nodeId = $NodeId
    credentialProtected = ConvertFrom-SecureString $secureToken
    storageRoot = $StorageRoot
    caFile = $CaFile
    clientCertificate = $ClientCertificate
    clientKey = $ClientKey
    childEndpoint = $ChildEndpoint
    prepareMsvc = [bool]$PrepareMsvc
    vsDevCmdPath = $VsDevCmdPath
    labels = @($Label)
}
$settings | ConvertTo-Json -Depth 4 -Compress | Set-Content -LiteralPath $configPath -Encoding UTF8
Write-WorkerLauncher
if (-not (Test-Path -LiteralPath $pythonExe -PathType Leaf)) {
    Invoke-External $Python @("-m", "venv", $pythonRoot)
}
$requirementsPath = Join-Path $packageRoot "requirements-worker.txt"
if (-not (Test-Path -LiteralPath $requirementsPath -PathType Leaf)) { throw "Worker requirements are missing: $requirementsPath" }
Invoke-External $pythonExe @("-m", "pip", "install", "--no-compile", "--cache-dir", $pipCacheRoot, "--requirement", $requirementsPath)
Register-WorkerTask
Write-Output "Installed worker $NodeId at $InstallRoot; persistent worker data stays under $StorageRoot."
