[CmdletBinding()]
param(
    [string]$PlanPath,
    [string]$ReceiptPath,
    [Parameter(Mandatory=$true)][string]$ExpectedScriptSha256,
    [switch]$Probe
)
$ErrorActionPreference='Stop'
if ($env:JOB_NAME -ne 'zircon-records' -or -not $env:BUILD_URL) { throw 'Jenkins records worker required' }
# The Jenkins service environment does not carry the interactive PSModulePath.
# Load only the two built-in modules needed by this pinned entry.
Import-Module ($PSHOME+'/Modules/Microsoft.PowerShell.Utility/Microsoft.PowerShell.Utility.psd1') -ErrorAction Stop
Import-Module ($PSHOME+'/Modules/Microsoft.PowerShell.Management/Microsoft.PowerShell.Management.psd1') -ErrorAction Stop
if ((Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ExpectedScriptSha256) { throw 'Cleanup entry changed' }
$repo='E:\Git\ZirconEngine'
$controls=Join-Path $repo '.jenkins\state\records-queue\cleanup'
$inbox=Join-Path $repo '.jenkins\records'
function Plain([string]$Path) {
    if ($Path -notmatch '^[DEF]:[\\/]' -or $Path.Substring(2).Contains(':') -or $Path -match '(^|[\\/])\.\.([\\/]|$)') { throw 'Physical absolute path required' }
    $absolute=[IO.Path]::GetFullPath($Path)
    $cursor=$absolute
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            if ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Path alias: $cursor" }
        }
        $cursor=[IO.Path]::GetDirectoryName($cursor)
    }
    return $absolute
}
function Deleted-Bytes($Rows) {
    $total=0L
    foreach ($row in $Rows) { $total+=[long]$row.bytes }
    return $total
}
$config=Join-Path $repo '.codex\config.toml'
$native='D:\CodexData\.codex\skills\artifact-lifecycle\scripts\GuardedFileDelete.cs'
if ($Probe) {
    $plan=@{permissionConfigSha256=(Get-FileHash -LiteralPath $config -Algorithm SHA256).Hash.ToLowerInvariant();helperSha256=(Get-FileHash -LiteralPath $native -Algorithm SHA256).Hash.ToLowerInvariant()}
} else {
    $planPath=Plain $PlanPath
    $receiptPath=Plain $ReceiptPath
    if ([IO.Path]::GetDirectoryName($planPath) -ne $controls -or [IO.Path]::GetDirectoryName($receiptPath) -ne $controls) { throw 'Fixed cleanup controls required' }
    $plan=Get-Content -LiteralPath $planPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($plan.repositoryRoot -ne $repo -or $plan.authorization -ne 'user: enqueue all tasks, archive evidence, then empty inbox') { throw 'Original cleanup authorization missing' }
    if ((Get-FileHash -LiteralPath $config -Algorithm SHA256).Hash.ToLowerInvariant() -ne $plan.permissionConfigSha256) { throw 'Permissions changed' }
}
$python=Join-Path $repo '.jenkins\runtime\python\python.exe'
$permission=& $python -B -X utf8 'D:/CodexData/.codex/skills/artifact-lifecycle/scripts/artifact_inventory.py' permissions --config $config --profile zircon-strict | ConvertFrom-Json
if ($LASTEXITCODE -ne 0 -or $permission.filesystem.'E:/Git/ZirconEngine/.jenkins' -ne 'write') { throw 'Actual config does not permit records lifecycle writes' }
if ((Get-FileHash -LiteralPath (Plain $native) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $plan.helperSha256) { throw 'Guarded deletion helper changed' }
$compilerTmp='E:\cargo-targets\zircon-local\tmp\jenkins-records-monitor-20261012'
$null=Plain $compilerTmp
$null=New-Item -ItemType Directory -Path $compilerTmp -Force
$env:TEMP=$compilerTmp
$env:TMP=$compilerTmp
$nativeSource=Get-Content -LiteralPath $native -Raw -Encoding UTF8
if (($nativeSource.Split(@('return info;'),[StringSplitOptions]::None)).Count -ne 2) { throw 'Guarded helper contract changed' }
# Apply the inbox's stricter single-link rule inside the existing pinned handle inspection.
$nativeSource=$nativeSource.Replace('return info;','if (!directory && info.Links != 1) throw new IOException("Linked inbox files are not accepted: " + path); return info;')
Add-Type -TypeDefinition $nativeSource
if ($Probe) {
    $probePath=Join-Path $compilerTmp ('guard-probe-'+$env:BUILD_NUMBER+'.json')
    $stream=[IO.FileStream]::new($probePath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
    try { $raw=[Text.Encoding]::UTF8.GetBytes('before'); $stream.Write($raw,0,$raw.Length); $stream.Flush($true) } finally { $stream.Dispose() }
    $original=Get-Item -LiteralPath $probePath
    $originalHash=(Get-FileHash -LiteralPath $probePath -Algorithm SHA256).Hash
    [IO.File]::WriteAllText($probePath,'change',[Text.UTF8Encoding]::new($false))
    [IO.File]::SetLastWriteTimeUtc($probePath,$original.LastWriteTimeUtc)
    $rejected=$false
    try { [ArtifactGuardedDelete]::RemoveVerified($probePath,[long]$original.Length,[long]$original.LastWriteTimeUtc.Ticks,$originalHash) }
    catch { if ($_.Exception.ToString() -notmatch 'File hash changed') { throw }; $rejected=$true }
    if (-not $rejected -or -not (Test-Path -LiteralPath $probePath)) { throw 'Changed file was not preserved by the native guard' }
    $current=Get-Item -LiteralPath $probePath
    $currentHash=(Get-FileHash -LiteralPath $probePath -Algorithm SHA256).Hash
    [ArtifactGuardedDelete]::RemoveVerified($probePath,[long]$current.Length,[long]$current.LastWriteTimeUtc.Ticks,$currentHash)
    if (Test-Path -LiteralPath $probePath) { throw 'Exact verified fixture was not removed' }
    $probeBytes=Deleted-Bytes @(@{path=$probePath;bytes=[long]$current.Length;sha256=$currentHash})
    if ($probeBytes -ne 6L) { throw 'Native cleanup receipt byte accounting failed' }
    Write-Output (@{status='passed';nativeGuardVerified=$true;changedBytesPreserved=$true;probeFixtureDeleted=$true;deletedBytes=$probeBytes} | ConvertTo-Json -Compress)
    return
}
$deleted=[Collections.Generic.List[object]]::new()
try {
    foreach ($file in $plan.files) {
        $path=Plain $file.path
        $archive=Plain $file.archivePath
        if ([IO.Path]::GetDirectoryName($path) -ne $inbox -or [IO.Path]::GetExtension($path) -notin @('.md','.json')) { throw 'Exact inbox message leaf required' }
        if (-not $archive.StartsWith($repo+'\.jenkins\state\records-queue\archive\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Retained contract must be in the fixed archive' }
        if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256) { throw 'Retained evidence changed' }
        if (-not (Test-Path -LiteralPath $path)) { continue }
        [ArtifactGuardedDelete]::VerifyFingerprint($path,[long]$file.bytes,[long]$file.mtimeUtcTicks,[string]$file.sha256)
    }
    foreach ($file in $plan.files) {
        $path=Plain $file.path
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $archive=Plain $file.archivePath
        $archiveParents=[ArtifactGuardedDelete]::LockDirectoryTree([IO.Path]::GetDirectoryName($archive))
        $archiveStream=$null
        try {
            $archiveStream=[IO.FileStream]::new($archive,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
            $hash=[Security.Cryptography.SHA256]::Create()
            try { $archiveHash=([BitConverter]::ToString($hash.ComputeHash($archiveStream))).Replace('-','').ToLowerInvariant() } finally { $hash.Dispose() }
            if ($archiveHash -ne $file.sha256) { throw 'Retained evidence changed before deletion' }
            [ArtifactGuardedDelete]::RemoveVerified($path,[long]$file.bytes,[long]$file.mtimeUtcTicks,[string]$file.sha256)
        } finally {
            if ($archiveStream) { $archiveStream.Dispose() }
            $archiveParents.Dispose()
        }
        $deleted.Add(@{path=$path;bytes=[long]$file.bytes;sha256=$file.sha256})
    }
    $result=@{success=$true;taskId=$plan.taskId;inventoryDigest=$plan.inventoryDigest;deletedFiles=$deleted.ToArray();deletedBytes=(Deleted-Bytes $deleted);finishedUtc=[DateTime]::UtcNow.ToString('o');buildUrl=$env:BUILD_URL}
} catch {
    $result=@{success=$false;taskId=$plan.taskId;inventoryDigest=$plan.inventoryDigest;deletedFiles=$deleted.ToArray();error=$_.Exception.Message;buildUrl=$env:BUILD_URL}
}
$raw=[Text.UTF8Encoding]::new($false).GetBytes(($result | ConvertTo-Json -Depth 8))
$stream=[IO.FileStream]::new($receiptPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
try { $stream.Write($raw,0,$raw.Length); $stream.Flush($true) } finally { $stream.Dispose() }
if (-not $result.success) { throw $result.error }
