Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$modulePath = Join-Path $PSScriptRoot '..\mvp\MvpStagingRelease.psm1'
$fixturePathsModule = Join-Path $PSScriptRoot '..\mvp\MvpTestFixturePaths.psm1'
if (-not (Test-Path -LiteralPath $modulePath -PathType Leaf)) {
    throw "MVP staging release module is missing: $modulePath"
}
Import-Module $modulePath -Force -ErrorAction Stop
Import-Module $fixturePathsModule -Force -ErrorAction Stop
$resolverModule = Join-Path $PSScriptRoot '..\..\tools\WindowsPathResolver.psm1'
Import-Module $resolverModule -Force -ErrorAction Stop

function Assert-True {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

$stagerPath = Join-Path $PSScriptRoot '..\mvp\Stage-MvpProducts.ps1'
$stagerSource = Get-Content -LiteralPath $stagerPath -Raw
Assert-True ($stagerSource -match "Import-Module .*MvpStagingRelease\.psm1") 'MVP stager does not import the project release probe module.'
Assert-True (
    ([regex]::Matches($stagerSource, 'Test-MvpStagedProjectDirectoryReleased')).Count -eq 3
) 'MVP stager must probe project release after product, automation, and project-creation processes.'
Assert-True (
    $stagerSource -match '\$startInfo\.WorkingDirectory = if \(\$null -eq \$projectRootResolution\)' -and
    $stagerSource -match '\$workingDirectoryResolution\.DisplayPath' -and
    $stagerSource -match '\$projectRootResolution\.DisplayPath' -and
    $stagerSource -match '@\(''--project'', ''\.''\) \+ @\(\$Arguments\)' -and
    $stagerSource -notmatch '\$projectRootArgument'
) 'MVP stager must use a display-path cwd and pass --project . without reviving an absolute project CLI argument.'

$releaseSource = Get-Content -LiteralPath $modulePath -Raw
Assert-True (
    $releaseSource -match 'Resolve-ZirconWindowsPath -Path \$ProjectDirectory\)\.OperationalPath'
) 'Project release probe must retain the resolved physical project path for filesystem operations.'
Assert-True (
    ([regex]::Matches(
        $releaseSource,
        '\[ZirconEngine\.WindowsPathResolver\.NativeMethodsV4\]::MovePath\(')).Count -eq 3 -and
    $releaseSource -notmatch 'Move-ZirconWindowsPath -Source'
) 'Project release probe must reuse the initialized compiled move operation for probe and restore paths.'
Assert-True (
    $releaseSource -match '\[IO\.Directory\]::Exists\(\$StageDirectory\)' -and
    $releaseSource -match '\[IO\.Directory\]::Exists\(\$ProjectDirectory\)' -and
    $releaseSource -match '\[IO\.Directory\]::Exists\(\$probe\)' -and
    $releaseSource -match '\[IO\.File\]::Exists\(\$probe\)' -and
    $releaseSource -notmatch 'Test-Path -LiteralPath'
) 'Project release probe must use direct CLR existence checks instead of PowerShell provider calls.'

$nativeFileSystemPath = Join-Path $PSScriptRoot '..\mvp\MvpAcceptanceNativeFileSystem.psm1'
$nativeFileSystemSource = Get-Content -LiteralPath $nativeFileSystemPath -Raw
Import-Module $nativeFileSystemPath -Force -ErrorAction Stop
Assert-True (
    $nativeFileSystemSource -match '\$sourceLength = \[ZirconMvpAcceptanceNativeFileSystem\]::GetLength\(\$sourceHandle\)' -and
    $nativeFileSystemSource -match '\[byte\[\]\]\$writtenBytes = \[byte\[\]\]::new\(\$ContentBytes\.Length\)' -and
    $nativeFileSystemSource -match '\$inputStream\.ReadByte\(\) -ne -1'
) 'Native acceptance publication must verify reopened content through one exact-size byte buffer and reject concurrent growth.'
Assert-True (
    $nativeFileSystemSource -notmatch '\[IO\.MemoryStream\]::new\(\)' -and
    $nativeFileSystemSource -notmatch '\$memoryStream\.ToArray\(\)'
) 'Native acceptance publication must not restore the growable-buffer plus ToArray full-content copy.'
Assert-True (
    $nativeFileSystemSource -match 'public static bool ByteSequencesEqual\(byte\[\] expected, byte\[\] actual\)' -and
    $nativeFileSystemSource -match 'return \[ZirconMvpAcceptanceNativeFileSystem\]::ByteSequencesEqual\(\$Expected, \$Actual\)'
) 'Native acceptance publication must compare artifact bytes inside the compiled helper instead of a PowerShell per-byte loop.'
Assert-True (
    $nativeFileSystemSource -match 'for \(var index = 0; index < expected\.Length; index\+\+\)' -and
    $nativeFileSystemSource -match 'if \(expected\[index\] != actual\[index\]\)' -and
    $nativeFileSystemSource -notmatch '\.AsSpan\(\)\.SequenceEqual\('
) 'Native acceptance publication must use a compatible indexed equal-length byte comparison.'
Assert-True (
    $nativeFileSystemSource -match '\[bool\]\(\$Attributes -band \[uint32\]\[System\.IO\.FileAttributes\]::ReparsePoint\)' -and
    $nativeFileSystemSource -notmatch 'Test-MvpAcceptanceNativeFileAttribute -Attributes \$Attributes -Expected'
) 'Native source-attribute admission must test the reparse bit directly without an internal generic-wrapper call.'
Assert-True (
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\[bool\]\(\$(?:attributes|rootAttributes|nextAttributes) -band \[uint32\]\[System\.IO\.FileAttributes\]::Directory\)')).Count -eq 0 -and
    $nativeFileSystemSource -notmatch 'Test-MvpAcceptanceNativeFileAttribute\s+`\r?\n\s+-Attributes'
) 'Native file and directory admission must test the Directory bit directly without internal generic-wrapper calls.'
Assert-True (
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\[bool\]\(\$(?:attributes|rootAttributes|nextAttributes) -band \[uint32\]\[System\.IO\.FileAttributes\]::ReparsePoint\)')).Count -eq 0 -and
    ([regex]::Matches(
        $nativeFileSystemSource,
        'Assert-MvpAcceptanceNativeSourceAttributes')).Count -eq 2
) 'Native call sites must test the ReparsePoint bit directly while retaining the exported assertion ABI.'
Assert-True (
    $nativeFileSystemSource -match 'public static void VerifyDirectory\(\s*SafeFileHandle handle,\s*string path,\s*string kind\)' -and
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\[ZirconMvpAcceptanceNativeFileSystem\]::VerifyDirectory\(')).Count -eq 2 -and
    $nativeFileSystemSource -match 'VerifyDirectory\(handle, path, "directory lease path"\)'
) 'Native directory-only admission must perform its attribute checks inside the compiled helper.'
Assert-True (
    $nativeFileSystemSource -match 'public static string GetVerifiedNonDirectoryIdentity\(\s*SafeFileHandle handle,\s*string path,\s*string kind\)' -and
    $nativeFileSystemSource -match 'public static string GetVerifiedDirectoryIdentity\(\s*SafeFileHandle handle,\s*string path,\s*string kind\)' -and
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\[ZirconMvpAcceptanceNativeFileSystem\]::GetVerifiedNonDirectoryIdentity\(')).Count -eq 1 -and
    $nativeFileSystemSource -match 'GetVerifiedNonDirectoryIdentity\(handle, path, "file"\)' -and
    $nativeFileSystemSource -match 'GetVerifiedNonDirectoryIdentity\(\s*sourceHandle,\s*absoluteSourcePath,\s*"source"\)' -and
    $nativeFileSystemSource -match 'GetVerifiedNonDirectoryIdentity\(\s*destinationHandle,\s*absoluteDestinationPath,\s*"destination"\)' -and
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\[ZirconMvpAcceptanceNativeFileSystem\]::GetVerifiedDirectoryIdentity\(')).Count -eq 1 -and
    $nativeFileSystemSource -match 'captureTargetIdentity && index == 0' -and
    $nativeFileSystemSource -match 'GetVerifiedDirectoryIdentity\(handle, path, "directory lease path"\)'
) 'Native identity consumers must combine attribute admission and identity capture into one handle-information read.'
Assert-True (
    $nativeFileSystemSource -match 'public static string GetCleanupDirectoryIdentity\(SafeFileHandle handle, string path\)' -and
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\[ZirconMvpAcceptanceNativeFileSystem\]::GetCleanupDirectoryIdentity\(')).Count -eq 1
) 'Native failure restoration must combine directory admission and identity capture while preserving its cleanup error contract.'
Assert-True (
    $nativeFileSystemSource -match 'public static void DisposeHandles\(SafeFileHandle\[\] handles\)' -and
    $nativeFileSystemSource -match 'for \(var index = handles\.Length - 1; index >= 0; index--\)' -and
    $nativeFileSystemSource -match '\[ZirconMvpAcceptanceNativeFileSystem\]::DisposeHandles\(\$Handles\)' -and
    $nativeFileSystemSource -cnotmatch 'for \(\$index = \$Handles\.Count - 1'
) 'Native directory lease closure must dispose its handle array in compiled code and retain the public close API.'
Assert-True (
    $nativeFileSystemSource -match 'public static SafeFileHandle\[\] OpenNoFollowDirectoryLease\(' -and
    $nativeFileSystemSource -match 'OpenNoFollowDirectoryLeaseCore\(\s*string directoryPath,\s*string compatibleWriteLeaseRoot,\s*bool captureTargetIdentity,\s*out string targetIdentity\)' -and
    $nativeFileSystemSource -match '\[ZirconMvpAcceptanceNativeFileSystem\]::OpenNoFollowDirectoryLease\(\s*\$DirectoryPath,\s*\$CompatibleWriteLeaseRoot\)' -and
    $nativeFileSystemSource -notmatch '\$paths = \[System\.Collections\.Generic\.List\[string\]\]::new\(\)'
) 'Native directory lease acquisition must build, verify, and unwind its ancestor handle chain in compiled code while retaining the public wrapper.'
Assert-True (
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\[ZirconMvpAcceptanceNativeFileSystem\]::OpenNoFollowDirectoryLease\(')).Count -eq 5 -and
    ([regex]::Matches(
        $nativeFileSystemSource,
        'Open-MvpAcceptanceNoFollowDirectoryLease\s+`')).Count -eq 0
) 'Native internal directory-lease consumers must enter the compiled transaction directly while retaining the exported wrapper ABI.'
Assert-True (
    $nativeFileSystemSource -match 'public static string GetVerifiedDirectoryIdentityWithLease\(' -and
    $nativeFileSystemSource -match 'directoryLease = OpenNoFollowDirectoryLeaseCore\(\s*path,\s*compatibleWriteLeaseRoot,' -and
    $nativeFileSystemSource -notmatch 'var absolutePath = System\.IO\.Path\.GetFullPath\(path\);' -and
    $nativeFileSystemSource -match '\[ZirconMvpAcceptanceNativeFileSystem\]::GetVerifiedDirectoryIdentityWithLease\(\s*\$Path,\s*\$CompatibleWriteLeaseRoot\)'
) 'Native directory identity lookup must normalize once while acquiring, borrowing, verifying, and releasing its ancestor lease behind the retained wrapper.'
Assert-True (
    $nativeFileSystemSource -match 'public static void RemoveFileNoFollow\(' -and
    $nativeFileSystemSource -match '\[ZirconMvpAcceptanceNativeFileSystem\]::RemoveFileNoFollow\(\$Path, \$ExpectedIdentity\)'
) 'Native identity-bound file removal must open, verify, mark, and dispose in compiled code while retaining the public wrapper.'
Assert-True (
    $nativeFileSystemSource -match 'public static void MoveFileNoFollow\(' -and
    $nativeFileSystemSource -match '\[ZirconMvpAcceptanceNativeFileSystem\]::MoveFileNoFollow\(\s*\$SourcePath,\s*\$DestinationPath,\s*\$ExpectedSourceIdentity,\s*\$CompatibleWriteLeaseRoot\)'
) 'Native identity-bound file publication must open, verify, rename, reopen, and dispose in compiled code while retaining the public wrapper.'
Assert-True (
    ([regex]::Matches(
        $nativeFileSystemSource,
        'Close-MvpAcceptanceNoFollowDirectoryLease')).Count -eq 2 -and
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\[ZirconMvpAcceptanceNativeFileSystem\]::DisposeHandles\(\$(?:parentLease|directoryLease|rootLease)\)')).Count -eq 4
) 'Native internal lease consumers must dispose arrays directly while retaining the exported close ABI.'
Assert-True (
    $nativeFileSystemSource -match 'public static void DisposeHandles\(List<SafeFileHandle> handles\)' -and
    $nativeFileSystemSource -match '\[ZirconMvpAcceptanceNativeFileSystem\]::DisposeHandles\(\$nestedHandles\)' -and
    $nativeFileSystemSource -notmatch 'for \(\$index = \$nestedHandles\.Count - 1'
) 'Native nested-directory cleanup must dispose its retained handle list in compiled code without an array copy.'
Assert-True (
    $nativeFileSystemSource -match '\$nestedHandles = \[System\.Collections\.Generic\.List\[Microsoft\.Win32\.SafeHandles\.SafeFileHandle\]\]::new\(\)' -and
    $nativeFileSystemSource -match '\$null = \$nestedHandles\.Add\(\$nextHandle\)' -and
    $nativeFileSystemSource -notmatch '\$currentLease = Open-MvpAcceptanceNoFollowDirectoryLease'
) 'Native acceptance directory creation must retain one verified handle per nested segment instead of reopening the full ancestor chain.'
Assert-True (
    $nativeFileSystemSource -match '\$nextPath = \[IO\.Path\]::Combine\(\$currentPath, \$segment\)' -and
    $nativeFileSystemSource -match '\[IO\.Directory\]::Exists\(\$nextPath\)' -and
    $nativeFileSystemSource -notmatch 'Join-Path \$currentPath \$segment' -and
    $nativeFileSystemSource -notmatch 'Test-Path -LiteralPath \$nextPath'
) 'Native acceptance directory creation must resolve and probe each segment through direct CLR filesystem APIs.'
Assert-True (
    $nativeFileSystemSource -match '\$segments = \$RelativePath\.Split\(\s*\[char\[\]\]@\(''\\'', ''/''\),\s*\[StringSplitOptions\]::RemoveEmptyEntries\)' -and
    $nativeFileSystemSource -match '\$segments -contains ''\.\.''' -and
    $nativeFileSystemSource -notmatch '\$RelativePath -match ''\(\^\|\[\\/\]\)\\\.\\\.\(\[\\/\]\|\$\)''' -and
    $nativeFileSystemSource -notmatch '\$RelativePath -split'
) 'Native acceptance directory creation must parse relative segments once before rejecting traversal and walking the path.'
Assert-True (
    $nativeFileSystemSource -match '\$rootHandle = \$rootLease\[\$rootLease\.Count - 1\]'
) 'Native acceptance directory creation must borrow the root handle already retained by its ancestor lease.'
Assert-True (
    $nativeFileSystemSource -match 'captureTargetIdentity && index == 0' -and
    $nativeFileSystemSource -match 'targetIdentity = GetVerifiedDirectoryIdentity\(handle, path, "directory lease path"\);'
) 'Native acceptance directory identity must borrow the target handle already retained by its ancestor lease.'
Assert-True (
    ([regex]::Matches(
        $nativeFileSystemSource,
        '\$directoryHandle = \$directoryLease\[\$directoryLease\.Count - 1\]')).Count -eq 2 -and
    $nativeFileSystemSource -match 'targetIdentity = GetVerifiedDirectoryIdentity\(handle, path, "directory lease path"\);'
) 'Native acceptance directory identity, publication protection, and restoration must each borrow their existing lease target handle.'
$closeProbeHandles = [Microsoft.Win32.SafeHandles.SafeFileHandle[]]@(
    [Microsoft.Win32.SafeHandles.SafeFileHandle]::new([IntPtr]::Zero, $false),
    [Microsoft.Win32.SafeHandles.SafeFileHandle]::new([IntPtr]::Zero, $false))
try {
    Close-MvpAcceptanceNoFollowDirectoryLease -Handles $closeProbeHandles
    Assert-True (
        $closeProbeHandles[0].IsClosed -and $closeProbeHandles[1].IsClosed
    ) 'Native directory lease closure left a handle open.'
}
finally {
    foreach ($handle in $closeProbeHandles) {
        $handle.Dispose()
    }
}
$nestedCloseProbeHandles = [Collections.Generic.List[Microsoft.Win32.SafeHandles.SafeFileHandle]]::new()
$nestedCloseProbeHandles.Add(
    [Microsoft.Win32.SafeHandles.SafeFileHandle]::new([IntPtr]::Zero, $false))
try {
    [ZirconMvpAcceptanceNativeFileSystem]::DisposeHandles($nestedCloseProbeHandles)
    Assert-True $nestedCloseProbeHandles[0].IsClosed 'Native nested-directory cleanup left a handle open.'
}
finally {
    $nestedCloseProbeHandles[0].Dispose()
}
$preflightPath = Join-Path $PSScriptRoot '..\mvp\MvpStagingPreflight.psm1'
$preflightSource = Get-Content -LiteralPath $preflightPath -Raw
Import-Module $preflightPath -Force -ErrorAction Stop
Assert-True (
    $preflightSource -match 'if \(\$sizeValue -is \[long\]\) \{\s*\$sizeBytes = \$sizeValue\s*\}\s*elseif \(\$sizeValue -is \[int\]\) \{\s*\$sizeBytes = \[Int64\]\$sizeValue' -and
    $preflightSource -match '\$entryBytes -gt \(\[Int64\]::MaxValue - \$sizeBytes\)'
) 'Staging entry budgeting must retain production Int64 values, widen Int32 values, and check overflow without a string round trip.'
Assert-True (
    $preflightSource -match '\$sizeValue = \$entry\[''size_bytes''\]\s*if \(\$null -eq \$sizeValue -and -not \$entry\.Contains\(''size_bytes''\)\)' -and
    $preflightSource -notmatch 'if \(-not \$entry\.Contains\(''size_bytes''\)\)'
) 'Staging entry budgeting must read a non-null dictionary size once while retaining missing-versus-null validation.'
Assert-True (
    $preflightSource -notmatch 'Add-MvpStagingByteCount -Total \$entryBytes -FileBytes \$sizeBytes'
) 'Staging entry budgeting must not restore the per-entry PowerShell byte-count helper call.'
Assert-True (
    $preflightSource -match '\$file = \[IO\.FileInfo\]::new\(\$path\)' -and
    $preflightSource -match 'if \(-not \$file\.Exists\)' -and
    $preflightSource -match '\$fileLength = \[Int64\]\$file\.Length' -and
    $preflightSource -notmatch '\[IO\.File\]::Exists\(\$path\)'
) 'Staging input budgeting must reuse one FileInfo metadata snapshot per source file.'
Assert-True (
    $preflightSource -match 'if \(\$inputCopy -is \[Collections\.IDictionary\]\)' -and
    $preflightSource -match '\$path = \[string\]\$inputCopy\[''path''\]' -and
    $preflightSource -match '\$copyCount = \[int\]\$inputCopy\[''copy_count''\]' -and
    $preflightSource -match '\$path = \[string\]\$inputCopy\.path'
) 'Staging input budgeting must read production ordered dictionaries directly while retaining PSObject compatibility.'
Assert-True (
    $preflightSource -match '\$fileLength = \[Int64\]\$file\.Length' -and
    $preflightSource -match '\$copyCount -eq 2' -and
    $preflightSource -match '\$fileLength -gt \(\[Int64\]::MaxValue - \$fileLength\)' -and
    $preflightSource -match '\$inputCopyBytes -gt \(\[Int64\]::MaxValue - \$weightedBytes\)' -and
    $preflightSource -notmatch '\[decimal\]'
) 'Staging input budgeting must accumulate weighted input bytes through checked Int64 arithmetic without decimal conversions.'
Assert-True (
    $preflightSource -notmatch 'function Add-MvpStagingByteCount' -and
    $preflightSource -match '\$inputCopyBytes -gt \(\[Int64\]::MaxValue - \$MvpStagingEvidenceReserveBytes\)' -and
    $preflightSource -match '\$requiredFreeSpaceBytes = \$inputCopyBytes \+ \$MvpStagingEvidenceReserveBytes'
) 'Staging input budgeting must add its one fixed reserve inline with an explicit Int64 overflow guard.'
Assert-True (
    $preflightSource -match 'if \(\$availableFreeSpaceBytes -lt \$RequiredFreeSpaceBytes\) \{\s*Assert-MvpStagingCapacityValues `'
) 'Staging capacity admission must keep its directly testable policy boundary off the successful disk-probe path.'
$preflightInputPath = (Resolve-Path $preflightPath).Path
$preflightInputBytes = [IO.FileInfo]::new($preflightInputPath).Length
$orderedPreflight = Get-MvpStagingPreflight `
    -StagingRootPath (Split-Path -Path $preflightInputPath -Parent) `
    -InputCopies @([ordered]@{ path = $preflightInputPath; copy_count = 1 }) `
    -InteractiveDesktopRequired $false
$objectPreflight = Get-MvpStagingPreflight `
    -StagingRootPath (Split-Path -Path $preflightInputPath -Parent) `
    -InputCopies @([pscustomobject]@{ path = $preflightInputPath; copy_count = 1 }) `
    -InteractiveDesktopRequired $false
Assert-True (
    $orderedPreflight.input_copy_bytes -eq $preflightInputBytes -and
    $objectPreflight.input_copy_bytes -eq $preflightInputBytes
) 'Staging input budgeting changed ordered-dictionary or PSObject input-copy results.'
$missingPreflightPathRejected = $false
try {
    Get-MvpStagingPreflight `
        -StagingRootPath (Split-Path -Path $preflightInputPath -Parent) `
        -InputCopies @([ordered]@{ path = ''; copy_count = 'not-an-integer' }) `
        -InteractiveDesktopRequired $false | Out-Null
}
catch {
    $missingPreflightPathRejected = $_.Exception.Message -eq 'MVP staging disk budget contains an input without a path.'
}
Assert-True $missingPreflightPathRejected 'Staging input budgeting changed path rejection priority for ordered dictionaries.'
Assert-True (
    (Assert-MvpStagingEntryBudget `
        -Entries @([pscustomobject]@{ size_bytes = [Int64]7 }) `
        -ExpectedInputCopyBytes 7) -eq 7
) 'Staging entry budgeting changed its JSON integer result.'
Assert-True (
    (Assert-MvpStagingEntryBudget `
        -Entries @([pscustomobject]@{ size_bytes = [int]9 }) `
        -ExpectedInputCopyBytes 9) -eq 9
) 'Staging entry budgeting changed its compatible Int32 result.'
Assert-True (
    (Assert-MvpStagingEntryBudget `
        -Entries @([pscustomobject]@{ size_bytes = '11' }) `
        -ExpectedInputCopyBytes 11) -eq 11
) 'Staging entry budgeting no longer accepts its compatible integral string input.'
$missingDictionarySizeRejected = $false
try {
    Assert-MvpStagingEntryBudget `
        -Entries @([ordered]@{ path = 'missing-size' }) `
        -ExpectedInputCopyBytes 0 | Out-Null
}
catch {
    $missingDictionarySizeRejected = $_.Exception.Message -eq 'MVP staging final entry is missing size_bytes.'
}
Assert-True $missingDictionarySizeRejected 'Staging entry budgeting accepted a dictionary without size_bytes.'
$nullDictionarySizeRejected = $false
try {
    Assert-MvpStagingEntryBudget `
        -Entries @([ordered]@{ size_bytes = $null }) `
        -ExpectedInputCopyBytes 0 | Out-Null
}
catch {
    $nullDictionarySizeRejected = $_.Exception.Message -match "invalid size_bytes ''"
}
Assert-True $nullDictionarySizeRejected 'Staging entry budgeting changed present-null size_bytes into a missing-field result.'
$negativeSizeRejected = $false
try {
    Assert-MvpStagingEntryBudget `
        -Entries @([pscustomobject]@{ size_bytes = -1 }) `
        -ExpectedInputCopyBytes 0 | Out-Null
}
catch {
    $negativeSizeRejected = $_.Exception.Message -match 'invalid size_bytes'
}
Assert-True $negativeSizeRejected 'Staging entry budgeting accepted a negative size.'
$entryOverflowRejected = $false
try {
    Assert-MvpStagingEntryBudget `
        -Entries @(
            [pscustomobject]@{ size_bytes = [Int64]::MaxValue },
            [pscustomobject]@{ size_bytes = 1 }) `
        -ExpectedInputCopyBytes 0 | Out-Null
}
catch {
    $entryOverflowRejected = $_.Exception.Message -match '64-bit byte budget'
}
Assert-True $entryOverflowRejected 'Staging entry budgeting accepted a 64-bit overflow.'

$preflightEvidencePath = Join-Path $PSScriptRoot '..\mvp\MvpStagingPreflightEvidence.psm1'
$preflightEvidenceSource = Get-Content -LiteralPath $preflightEvidencePath -Raw
Import-Module $preflightEvidencePath -Force -ErrorAction Stop
Assert-True (
    $preflightEvidenceSource -match 'function Get-MvpPreflightRequiredInt64Property' -and
    ([regex]::Matches(
        $preflightEvidenceSource,
        'Get-MvpPreflightRequiredInt64Property `')).Count -eq 4 -and
    $preflightEvidenceSource -notmatch '-Value \(Get-MvpPreflightRequiredProperty -Value \$preflight'
) 'Staging preflight numeric fields must combine required-property and Int64 validation into one private dispatch.'
Assert-True (
    $preflightEvidenceSource -match '\$Value -is \[int\] -or \$Value -is \[long\]'
) 'Staging preflight evidence must retain JSON integer values without a string conversion round trip.'
Assert-True (
    $preflightEvidenceSource -match '\$sessionIdProperty = \$desktop\.PSObject\.Properties\[''session_id''\]' -and
    $preflightEvidenceSource -match '\$monitorCountProperty = \$desktop\.PSObject\.Properties\[''monitor_count''\]' -and
    $preflightEvidenceSource -notmatch 'foreach \(\$name in @\(''session_id'', ''monitor_count''\)\)'
) 'Staging preflight evidence must resolve its two desktop numeric properties once instead of traversing the same name array twice.'
$preflightManifest = [pscustomobject]@{
    preflight = [pscustomobject]@{
        input_copy_bytes = [Int64]7
        evidence_reserve_bytes = [Int64](512MB)
        required_free_space_bytes = [Int64](512MB + 7)
        available_free_space_bytes = [Int64](512MB + 1024)
        staging_drive_root = 'D:\'
        interactive_desktop = [pscustomobject]@{
            required = $true
            user_interactive = $true
            session_id = 1
            monitor_count = 1
        }
    }
}
Assert-MvpStagingPreflightEvidence `
    -Manifest $preflightManifest `
    -EntryBytes 7 `
    -StagingRoot 'D:\ZirconBuilds\mvp-staging-runs\fixture'
$preflightManifest.preflight.PSObject.Properties.Remove('input_copy_bytes')
$missingInputBudgetRejected = $false
try {
    Assert-MvpStagingPreflightEvidence `
        -Manifest $preflightManifest `
        -EntryBytes 7 `
        -StagingRoot 'D:\ZirconBuilds\mvp-staging-runs\fixture'
}
catch {
    $missingInputBudgetRejected = $_.Exception.Message -match "Staging manifest preflight is missing 'input_copy_bytes'"
}
Assert-True $missingInputBudgetRejected 'Staging preflight evidence accepted a missing input_copy_bytes field.'
$preflightManifest.preflight | Add-Member -NotePropertyName input_copy_bytes -NotePropertyValue ([Int64]7)
$preflightManifest.preflight.input_copy_bytes = '7'
Assert-MvpStagingPreflightEvidence `
    -Manifest $preflightManifest `
    -EntryBytes 7 `
    -StagingRoot 'D:\ZirconBuilds\mvp-staging-runs\fixture'
$preflightManifest.preflight.input_copy_bytes = -1
$negativePreflightRejected = $false
try {
    Assert-MvpStagingPreflightEvidence `
        -Manifest $preflightManifest `
        -EntryBytes 7 `
        -StagingRoot 'D:\ZirconBuilds\mvp-staging-runs\fixture'
}
catch {
    $negativePreflightRejected = $_.Exception.Message -match 'invalid non-negative'
}
Assert-True $negativePreflightRejected 'Staging preflight evidence accepted a negative JSON integer.'

$fixtureRoot = New-MvpTestFixtureRoot -Prefix 'zircon_mvp_staging_release'

try {
    $stageRoot = Join-Path $fixtureRoot 'stage'
    $projectRoot = Join-Path $stageRoot 'project\Fixture'
    New-Item -ItemType Directory -Force -Path $projectRoot | Out-Null
    [IO.File]::WriteAllText(
        (Join-Path $projectRoot 'zircon-project.toml'),
        "name = 'Fixture'`n",
        [Text.UTF8Encoding]::new($false)
    )

    [byte[]]$nativeExpectedBytes = [byte[]]@(0, 1, 2, 127, 128, 254, 255)
    $nativeArtifactPath = Join-Path $stageRoot 'native-artifact.bin'
    $nativeWrite = Write-MvpAcceptanceNewFileNoFollow `
        -Path $nativeArtifactPath `
        -ContentBytes $nativeExpectedBytes `
        -PassThruDetails
    Assert-True (
        [ZirconMvpAcceptanceNativeFileSystem]::ByteSequencesEqual(
            $nativeExpectedBytes,
            [byte[]]$nativeWrite.content_bytes)
    ) 'Native acceptance publication did not return the exact verified artifact bytes.'
    [byte[]]$nativeDifferentBytes = [byte[]]$nativeExpectedBytes.Clone()
    $nativeDifferentBytes[$nativeDifferentBytes.Length - 1] = 0
    Assert-True (
        -not [ZirconMvpAcceptanceNativeFileSystem]::ByteSequencesEqual(
            $nativeExpectedBytes,
            $nativeDifferentBytes) -and
        [ZirconMvpAcceptanceNativeFileSystem]::ByteSequencesEqual($null, $null) -and
        -not [ZirconMvpAcceptanceNativeFileSystem]::ByteSequencesEqual(
            $null,
            [byte[]]::new(0))
    ) 'Native acceptance publication changed unequal, null, or empty byte comparison behavior.'
    Assert-True (
        -not [string]::IsNullOrWhiteSpace([string]$nativeWrite.identity)
    ) 'Native acceptance publication did not retain the verified file identity.'

    $nativeNestedPath = Ensure-MvpAcceptanceDirectoryPathNoFollow `
        -RootPath $stageRoot `
        -RelativePath 'evidence\nested\leaf' `
        -CompatibleWriteLeaseRoot $stageRoot
    Assert-True (
        [IO.Directory]::Exists($nativeNestedPath)
    ) 'Native acceptance directory creation did not materialize the verified nested path.'
    $nativeTraversalRejected = $false
    try {
        Ensure-MvpAcceptanceDirectoryPathNoFollow `
            -RootPath $stageRoot `
            -RelativePath 'evidence\..\escape' `
            -CompatibleWriteLeaseRoot $stageRoot | Out-Null
    }
    catch {
        $nativeTraversalRejected = $_.Exception.Message -match 'is unsafe'
    }
    Assert-True $nativeTraversalRejected 'Native acceptance directory creation accepted parent traversal.'
    Assert-True (
        -not [IO.Directory]::Exists((Join-Path $stageRoot 'escape'))
    ) 'Native acceptance traversal rejection created an escaped directory.'
    $nativeFileSegmentRoot = Join-Path $stageRoot 'file-segment-root'
    [IO.Directory]::CreateDirectory($nativeFileSegmentRoot) | Out-Null
    [IO.File]::WriteAllText(
        (Join-Path $nativeFileSegmentRoot 'not-a-directory'),
        'fixture',
        [Text.UTF8Encoding]::new($false))
    $nativeFileSegmentRejected = $false
    try {
        Ensure-MvpAcceptanceDirectoryPathNoFollow `
            -RootPath $nativeFileSegmentRoot `
            -RelativePath 'not-a-directory\leaf' `
            -CompatibleWriteLeaseRoot $nativeFileSegmentRoot | Out-Null
    }
    catch {
        $nativeFileSegmentRejected = $true
    }
    Assert-True $nativeFileSegmentRejected 'Native acceptance directory creation accepted an ordinary file as an intermediate segment.'

    Test-MvpStagedProjectDirectoryReleased `
        -StageDirectory $stageRoot `
        -ProjectDirectory $projectRoot

    Assert-True (Test-Path -LiteralPath $projectRoot -PathType Container) 'Project release probe did not restore the project directory.'
    Assert-True (Test-Path -LiteralPath (Join-Path $projectRoot 'zircon-project.toml') -PathType Leaf) 'Project release probe lost project content.'
    Assert-True (-not (Test-Path -LiteralPath "$projectRoot.release-probe")) 'Project release probe left its temporary rename target behind.'

    $outsideProject = Join-Path $fixtureRoot 'outside-project'
    New-Item -ItemType Directory -Force -Path $outsideProject | Out-Null
    $outsideRejected = $false
    try {
        Test-MvpStagedProjectDirectoryReleased `
            -StageDirectory $stageRoot `
            -ProjectDirectory $outsideProject
    }
    catch {
        $outsideRejected = $_.Exception.Message -match 'outside staging root'
    }
    Assert-True $outsideRejected 'Project release probe accepted a directory outside the staging root.'

    $probePath = "$projectRoot.release-probe"
    New-Item -ItemType Directory -Force -Path $probePath | Out-Null
    $conflictRejected = $false
    try {
        Test-MvpStagedProjectDirectoryReleased `
            -StageDirectory $stageRoot `
            -ProjectDirectory $projectRoot
    }
    catch {
        $conflictRejected = $_.Exception.Message -match 'already exists'
    }
    Assert-True $conflictRejected 'Project release probe overwrote an existing probe path.'
    Assert-True (Test-Path -LiteralPath $projectRoot -PathType Container) 'Probe-path conflict changed the project directory.'

    $physicalStageRoot = (Resolve-ZirconWindowsPath -Path $stageRoot).OperationalPath
    $longProjectRoot = $physicalStageRoot
    foreach ($segment in 1..12) {
        $longProjectRoot = Join-ZirconWindowsPath -Path $longProjectRoot -ChildPath ('long-path-segment-' + ('x' * 16))
        [IO.Directory]::CreateDirectory($longProjectRoot) | Out-Null
    }
    $longProjectRoot = Join-ZirconWindowsPath -Path $longProjectRoot -ChildPath 'Project'
    [IO.Directory]::CreateDirectory($longProjectRoot) | Out-Null
    Assert-True ($longProjectRoot.Length -gt 260) 'Long-path release fixture did not exceed MAX_PATH.'

    Test-MvpStagedProjectDirectoryReleased `
        -StageDirectory $physicalStageRoot `
        -ProjectDirectory $longProjectRoot

    Assert-True ([IO.Directory]::Exists($longProjectRoot)) 'Long-path release probe did not restore the project directory.'
    Assert-True (-not [IO.Directory]::Exists("$longProjectRoot.release-probe")) 'Long-path release probe left its temporary rename target behind.'

    Write-Output 'MVP staged project release contract passed'
}
finally {
    if (Test-Path -LiteralPath $fixtureRoot) {
        $resolvedFixtureRoot = [IO.Path]::GetFullPath($fixtureRoot)
        $fixtureDisplayPattern = '^[D-F]:\\ZirconBuilds\\mvp-test-fixtures-' + [regex]::Escape([string]$PID) + '\\zircon_mvp_staging_release-[0-9a-f]{32}$'
        if ($resolvedFixtureRoot -notmatch $fixtureDisplayPattern) {
            throw "Refusing to remove staging release fixture outside the approved fixture root: $resolvedFixtureRoot"
        }
        Remove-MvpTestFixtureRoot -Path (Resolve-ZirconWindowsPath -Path $resolvedFixtureRoot).OperationalPath
    }
}
