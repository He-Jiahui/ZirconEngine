[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$PlanPath,
      [Parameter(Mandatory=$true)][string]$ReceiptPath,
      [Parameter(Mandatory=$true)][string]$ExpectedScriptSha256)
$ErrorActionPreference='Stop'
if ($env:JOB_NAME -ne 'zircon-records-followup' -or -not $env:BUILD_URL) { throw 'Fixed supplementary Jenkins worker required' }
Import-Module ($PSHOME+'/Modules/Microsoft.PowerShell.Utility/Microsoft.PowerShell.Utility.psd1') -ErrorAction Stop
Import-Module ($PSHOME+'/Modules/Microsoft.PowerShell.Management/Microsoft.PowerShell.Management.psd1') -ErrorAction Stop
if ((Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ExpectedScriptSha256) { throw 'Supplementary cleanup source changed' }
$repo='E:\Git\ZirconEngine'
$controls=$repo+'\.jenkins\state\records-queue\cleanup'
$inbox=$repo+'\.jenkins\records'
function Plain([string]$Path) {
    if ($Path -notmatch '^[DEF]:[\\/]' -or $Path.Substring(2).Contains(':') -or $Path -match '(^|[\\/])\.\.([\\/]|$)') { throw 'Physical absolute path required' }
    $absolute=[IO.Path]::GetFullPath($Path)
    $cursor=$absolute
    while ($cursor) {
        if ((Test-Path -LiteralPath $cursor) -and ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Path aliases are not permitted' }
        $cursor=[IO.Path]::GetDirectoryName($cursor)
    }
    return $absolute
}
$planPath=Plain $PlanPath
$receiptPath=Plain $ReceiptPath
if ([IO.Path]::GetDirectoryName($planPath) -ne $controls -or [IO.Path]::GetDirectoryName($receiptPath) -ne $controls) { throw 'Fixed cleanup controls required' }
$plan=Get-Content -LiteralPath $planPath -Raw -Encoding UTF8 | ConvertFrom-Json
if ($plan.repositoryRoot -ne $repo -or $plan.taskId -ne ('followup-'+$plan.inventoryDigest) -or $plan.inventoryDigest -notmatch '^[0-9a-f]{64}$' -or $plan.authorization -ne 'user: enqueue all tasks, archive evidence, then empty inbox') { throw 'Supplement cleanup identity or authorization differs' }
$config=$repo+'\.codex\config.toml'
if ((Get-FileHash -LiteralPath $config -Algorithm SHA256).Hash.ToLowerInvariant() -ne $plan.permissionConfigSha256) { throw 'Permissions changed' }
$permission=& ($repo+'\.jenkins\runtime\python\python.exe') -B -X utf8 'D:/CodexData/.codex/skills/artifact-lifecycle/scripts/artifact_inventory.py' permissions --config $config --profile zircon-strict | ConvertFrom-Json
if ($LASTEXITCODE -ne 0 -or $permission.filesystem.'E:/Git/ZirconEngine/.jenkins' -ne 'write') { throw 'Actual config does not permit supplemental lifecycle writes' }
$native=Plain 'D:/CodexData/.codex/skills/artifact-lifecycle/scripts/GuardedFileDelete.cs'
if ((Get-FileHash -LiteralPath $native -Algorithm SHA256).Hash.ToLowerInvariant() -ne $plan.helperSha256) { throw 'Native deletion helper changed' }
$compilerTmp=Plain 'E:/cargo-targets/zircon-local/tmp/jenkins-records-monitor-20261012'
$null=New-Item -ItemType Directory -Path $compilerTmp -Force
$env:TEMP=$compilerTmp
$env:TMP=$compilerTmp
$nativeSource=Get-Content -LiteralPath $native -Raw -Encoding UTF8
if (($nativeSource.Split(@('return info;'),[StringSplitOptions]::None)).Count -ne 2) { throw 'Native helper contract changed' }
$nativeSource=$nativeSource.Replace('return info;','if (!directory && info.Links != 1) throw new IOException("Linked inbox files are not accepted: " + path); return info;')
Add-Type -TypeDefinition $nativeSource
$deleted=[Collections.Generic.List[object]]::new()
try {
    foreach ($file in $plan.files) {
        $path=Plain $file.path
        $archive=Plain $file.archivePath
        if ([IO.Path]::GetDirectoryName($path) -ne $inbox -or [IO.Path]::GetExtension($path) -notin @('.md','.json')) { throw 'Exact inbox message required' }
        if ([IO.Path]::GetDirectoryName($archive) -ne ($repo+'\.jenkins\state\records-queue\archive\'+$plan.inventoryDigest)) { throw 'Exact supplement archive required' }
        [ArtifactGuardedDelete]::VerifyFingerprint($path,[long]$file.bytes,[long]$file.mtimeUtcTicks,[string]$file.sha256)
        if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256) { throw 'Retained supplementary evidence changed' }
    }
    foreach ($file in $plan.files) {
        $parents=[ArtifactGuardedDelete]::LockDirectoryTree([IO.Path]::GetDirectoryName($file.archivePath))
        $stream=$null
        try {
            $stream=[IO.FileStream]::new($file.archivePath,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
            $hash=[Security.Cryptography.SHA256]::Create()
            try { $savedHash=([BitConverter]::ToString($hash.ComputeHash($stream))).Replace('-','').ToLowerInvariant() } finally { $hash.Dispose() }
            if ($savedHash -ne $file.sha256) { throw 'Retained evidence changed before deletion' }
            [ArtifactGuardedDelete]::RemoveVerified($file.path,[long]$file.bytes,[long]$file.mtimeUtcTicks,[string]$file.sha256)
            $deleted.Add(@{path=$file.path;bytes=[long]$file.bytes;sha256=$file.sha256})
        } finally {
            if ($stream) { $stream.Dispose() }
            $parents.Dispose()
        }
    }
    $total=0L
    foreach ($row in $deleted) { $total+=[long]$row.bytes }
    $result=@{success=$true;taskId=$plan.taskId;parentTaskId=$plan.parentTaskId;inventoryDigest=$plan.inventoryDigest;deletedFiles=$deleted.ToArray();deletedBytes=$total;buildUrl=$env:BUILD_URL;finishedUtc=[DateTime]::UtcNow.ToString('o')}
} catch {
    $result=@{success=$false;taskId=$plan.taskId;parentTaskId=$plan.parentTaskId;inventoryDigest=$plan.inventoryDigest;deletedFiles=$deleted.ToArray();error=$_.Exception.Message;buildUrl=$env:BUILD_URL}
}
$raw=[Text.UTF8Encoding]::new($false).GetBytes(($result | ConvertTo-Json -Depth 12))
$receiptStream=[IO.FileStream]::new($receiptPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
try { $receiptStream.Write($raw,0,$raw.Length);$receiptStream.Flush($true) } finally { $receiptStream.Dispose() }
if (-not $result.success) { throw $result.error }
