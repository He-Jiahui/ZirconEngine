$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$sourceScript = Join-Path $repoRoot 'tools\build-editor.ps1'
$sourcePathResolver = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
Import-Module (Join-Path $repoRoot 'tools\mvp\MvpTestFixturePaths.psm1') -Force -ErrorAction Stop

function New-EditorBuildFixtureRoot {
    return New-MvpTestFixtureRoot -Prefix 'editor-build'
}

function Remove-EditorBuildFixtureRoot {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FixtureRoot
    )

    Remove-MvpTestFixtureRoot -Path $FixtureRoot
}

function Invoke-EditorBuildFixture {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ScriptPath,

        [string]$OutputDirectory,

        [string]$TargetDir,

        [ValidateSet('reuse', 'compact', 'diagnostic')]
        [string]$StorageMode = 'reuse',

        [switch]$Ephemeral
    )

    $arguments = @(
        '-NoProfile'
        '-ExecutionPolicy'
        'Bypass'
        '-File'
        $ScriptPath
    )
    if (-not [string]::IsNullOrWhiteSpace($OutputDirectory)) {
        $arguments += @('-OutputDirectory', $OutputDirectory)
    }
    if (-not [string]::IsNullOrWhiteSpace($TargetDir)) {
        $arguments += @('-TargetDir', $TargetDir)
    }
    $arguments += @('-StorageMode', $StorageMode)
    if ($Ephemeral) {
        $arguments += '-Ephemeral'
    }
    $arguments += '-SkipSmokeTest'
    $output = @(& powershell.exe @arguments 2>&1)

    return @{
        ExitCode = $LASTEXITCODE
        Output = $output
    }
}

Describe 'Editor build bundle script' {
    BeforeEach {
        $fixtureRoot = New-EditorBuildFixtureRoot
        $fixtureTools = Join-Path $fixtureRoot 'tools'
        $fixtureValidatorDirectory = Join-Path $fixtureRoot '.codex\skills\zircon-dev\scripts'
        $fixtureAssets = Join-Path $fixtureRoot 'zircon_runtime\assets\fonts'
        $fixtureEditorAssets = Join-Path $fixtureRoot 'zircon_editor\assets\icons'
        $fixtureScript = Join-Path $fixtureTools 'build-editor.ps1'
        $fixturePathResolver = Join-Path $fixtureTools 'WindowsPathResolver.psm1'
        $fixtureCoordinator = Join-Path $fixtureTools 'zircon-session.ps1'
        $fixtureValidator = Join-Path $fixtureValidatorDirectory 'validate-matrix.ps1'
        $callLog = Join-Path $fixtureRoot 'validator-calls.log'
        $artifactLog = Join-Path $fixtureRoot 'validator-artifacts.log'
        $coordinatorLog = Join-Path $fixtureRoot 'coordinator-calls.log'

        [System.IO.Directory]::CreateDirectory($fixtureTools) | Out-Null
        [System.IO.Directory]::CreateDirectory($fixtureValidatorDirectory) | Out-Null
        [System.IO.Directory]::CreateDirectory($fixtureAssets) | Out-Null
        [System.IO.Directory]::CreateDirectory($fixtureEditorAssets) | Out-Null
        Copy-Item -LiteralPath $sourceScript -Destination $fixtureScript
        Copy-Item -LiteralPath $sourcePathResolver -Destination $fixturePathResolver
        Set-Content -LiteralPath (Join-Path $fixtureAssets 'fixture.txt') -Value 'asset fixture'
        Set-Content -LiteralPath (Join-Path $fixtureEditorAssets 'fixture.svg') -Value '<svg />'

        @'
[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Command = 'status',
    [switch]$Json,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

if (-not $Json) {
    Write-Output 'Coordinator ready.'
}

$allArguments = @($Command) + @($Arguments)
[System.IO.File]::AppendAllText(
    $env:BUILD_EDITOR_TEST_COORDINATOR_LOG,
    ($allArguments -join '|') + [Environment]::NewLine)
$command = $allArguments | Where-Object { $_ -like 'staging-*' } | Select-Object -First 1
$leaseId = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
$ownerIndex = [Array]::IndexOf($allArguments, '--owner-pid')
$ownerPid = [int]$allArguments[$ownerIndex + 1]
$finalIndex = [Array]::IndexOf($allArguments, '--final-path')
$finalPath = if ($finalIndex -ge 0) {
    $allArguments[$finalIndex + 1]
}
else {
    $acquireArguments = @(
        (Get-Content -LiteralPath $env:BUILD_EDITOR_TEST_COORDINATOR_LOG | Select-Object -First 1) -split '\|'
    )
    $acquireFinalIndex = [Array]::IndexOf($acquireArguments, '--final-path')
    $acquireArguments[$acquireFinalIndex + 1]
}
$path = [System.IO.Path]::GetFullPath($finalPath)
$root = [System.IO.Directory]::GetParent($path)
while ($null -ne $root -and $root.Name -ne 'ZirconBuilds') {
    $root = $root.Parent
}
if ($null -eq $root) {
    throw "Fixture final path is not below ZirconBuilds: $finalPath"
}
$status = switch ($command) {
    'staging-acquire' { 'active' }
    'staging-begin-publish' { 'publishing' }
    'staging-complete-publish' { 'published' }
    'staging-release' { 'released' }
    default { throw "Unexpected artifact command: $($allArguments -join ' ')" }
}
@{
    requestId = 'fixture-request'
    lease = @{
        leaseId = $leaseId
        purpose = 'build-editor'
        stagingPath = [System.IO.Path]::Combine($root.FullName, "mvp-product-inputs-build-editor-$leaseId")
        finalPath = $finalPath
        ownerPid = $ownerPid
        status = $status
    }
} | ConvertTo-Json -Compress
exit 0
'@ | Set-Content -LiteralPath $fixtureCoordinator -Encoding UTF8

        @'
[CmdletBinding()]
param(
    [string]$RepoRoot,
    [string]$Package,
    [string]$Bin,
    [switch]$NoDefaultFeatures,
    [string]$Features,
    [switch]$SkipTest,
    [switch]$MvpProductInputArtifactOutput,
    [string]$ArtifactOutputDirectory,
    [string[]]$PublishArtifact,
    [string]$TargetDir,
    [ValidateSet('reuse', 'compact', 'diagnostic')]
    [string]$StorageMode = 'reuse',
    [switch]$Ephemeral
)

$record = '{0}|{1}|{2}|{3}|{4}|{5}|{6}|{7}|{8}|{9}' -f `
    $Package, $Bin, $NoDefaultFeatures.IsPresent, $Features, $SkipTest.IsPresent, ($PublishArtifact -join ','), $MvpProductInputArtifactOutput.IsPresent, $TargetDir, $Ephemeral.IsPresent, $StorageMode
[System.IO.File]::AppendAllText($env:BUILD_EDITOR_TEST_LOG, $record + [Environment]::NewLine)
[System.IO.File]::AppendAllText($env:BUILD_EDITOR_TEST_ARTIFACT_LOG, $ArtifactOutputDirectory + [Environment]::NewLine)

if ($env:BUILD_EDITOR_TEST_REQUIRE_COORDINATOR -eq '1') {
    $coordinatorCalls = @(Get-Content -LiteralPath $env:BUILD_EDITOR_TEST_COORDINATOR_LOG)
    if ($coordinatorCalls.Count -eq 0 -or $coordinatorCalls[0] -notmatch 'staging-acquire') {
        throw 'Product staging was not acquired before the managed validator ran.'
    }
}

if ($env:BUILD_EDITOR_TEST_FAIL_PACKAGE -eq $Package) {
    exit 17
}

[System.IO.Directory]::CreateDirectory($ArtifactOutputDirectory) | Out-Null
foreach ($artifact in $PublishArtifact) {
    [System.IO.File]::WriteAllBytes(
        [System.IO.Path]::Combine($ArtifactOutputDirectory, $artifact),
        [System.Text.Encoding]::UTF8.GetBytes("fixture:$artifact"))
}

exit 0
'@ | Set-Content -LiteralPath $fixtureValidator -Encoding UTF8

        $env:BUILD_EDITOR_TEST_LOG = $callLog
        $env:BUILD_EDITOR_TEST_ARTIFACT_LOG = $artifactLog
        $env:BUILD_EDITOR_TEST_COORDINATOR_LOG = $coordinatorLog
        $env:BUILD_EDITOR_TEST_FINAL_PATH = $null
        $env:BUILD_EDITOR_TEST_REQUIRE_COORDINATOR = $null
        $env:BUILD_EDITOR_TEST_FAIL_PACKAGE = $null
    }

    AfterEach {
        $env:BUILD_EDITOR_TEST_LOG = $null
        $env:BUILD_EDITOR_TEST_ARTIFACT_LOG = $null
        $env:BUILD_EDITOR_TEST_COORDINATOR_LOG = $null
        $env:BUILD_EDITOR_TEST_FINAL_PATH = $null
        $env:BUILD_EDITOR_TEST_REQUIRE_COORDINATOR = $null
        $env:BUILD_EDITOR_TEST_FAIL_PACKAGE = $null
        Remove-EditorBuildFixtureRoot -FixtureRoot $fixtureRoot
    }

    It 'publishes the editor, runtime DLL, runtime assets, and editor assets only after both builds succeed' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $env:BUILD_EDITOR_TEST_FINAL_PATH = $bundle
        $env:BUILD_EDITOR_TEST_REQUIRE_COORDINATOR = '1'

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $bundle

        $result.ExitCode | Should Be 0
        Test-Path -LiteralPath (Join-Path $bundle 'zircon_editor.exe') | Should Be $true
        Test-Path -LiteralPath (Join-Path $bundle 'zircon_runtime.dll') | Should Be $true
        Test-Path -LiteralPath (Join-Path $bundle 'assets\fonts\fixture.txt') | Should Be $true
        Test-Path -LiteralPath (Join-Path $bundle 'assets\icons\fixture.svg') | Should Be $true

        $calls = @(Get-Content -LiteralPath $callLog)
        $calls.Count | Should Be 2
        $calls[0] | Should Be 'zircon_app|zircon_editor|True|target-editor-host|True|zircon_editor.exe|True||False|reuse'
        $calls[1] | Should Be 'zircon_runtime||True|target-editor-host|True|zircon_runtime.dll|True||False|reuse'
        $artifactDirectories = @(Get-Content -LiteralPath $artifactLog)
        $artifactDirectories.Count | Should Be 2
        foreach ($artifactDirectory in $artifactDirectories) {
            $artifactDirectory | Should Match '^\\\\\?\\[D-F]:\\ZirconBuilds\\mvp-product-inputs-build-editor-[0-9a-f]{32}$'
        }
        $coordinatorCalls = @(Get-Content -LiteralPath $coordinatorLog)
        $coordinatorCalls.Count | Should Be 3
        $coordinatorCalls[0] | Should Match 'staging-acquire'
        $coordinatorCalls[1] | Should Match 'staging-begin-publish'
        $coordinatorCalls[2] | Should Match 'staging-complete-publish'
    }

    It 'forwards one explicit managed target directory to both package builds' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $targetDir = Join-Path $fixtureRoot 'cargo-target'
        $env:BUILD_EDITOR_TEST_FINAL_PATH = $bundle

        $result = Invoke-EditorBuildFixture `
            -ScriptPath $fixtureScript `
            -OutputDirectory $bundle `
            -TargetDir $targetDir

        $result.ExitCode | Should Be 0
        $calls = @(Get-Content -LiteralPath $callLog)
        $calls.Count | Should Be 2
        $calls[0] | Should Be "zircon_app|zircon_editor|True|target-editor-host|True|zircon_editor.exe|True|$targetDir|False|reuse"
        $calls[1] | Should Be "zircon_runtime||True|target-editor-host|True|zircon_runtime.dll|True|$targetDir|False|reuse"
    }

    It 'forwards ephemeral lane selection to both package builds' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $env:BUILD_EDITOR_TEST_FINAL_PATH = $bundle

        $result = Invoke-EditorBuildFixture `
            -ScriptPath $fixtureScript `
            -OutputDirectory $bundle `
            -Ephemeral

        $result.ExitCode | Should Be 0
        $calls = @(Get-Content -LiteralPath $callLog)
        $calls.Count | Should Be 2
        $calls[0] | Should Be 'zircon_app|zircon_editor|True|target-editor-host|True|zircon_editor.exe|True||True|reuse'
        $calls[1] | Should Be 'zircon_runtime||True|target-editor-host|True|zircon_runtime.dll|True||True|reuse'
    }

    It 'forwards diagnostic storage mode to both package builds' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $env:BUILD_EDITOR_TEST_FINAL_PATH = $bundle

        $result = Invoke-EditorBuildFixture `
            -ScriptPath $fixtureScript `
            -OutputDirectory $bundle `
            -StorageMode 'diagnostic'

        $result.ExitCode | Should Be 0
        $calls = @(Get-Content -LiteralPath $callLog)
        $calls.Count | Should Be 2
        $calls[0] | Should Be 'zircon_app|zircon_editor|True|target-editor-host|True|zircon_editor.exe|True||False|diagnostic'
        $calls[1] | Should Be 'zircon_runtime||True|target-editor-host|True|zircon_runtime.dll|True||False|diagnostic'
    }

    It 'removes its staged artifact directory when the runtime build fails' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $env:BUILD_EDITOR_TEST_FINAL_PATH = $bundle
        $env:BUILD_EDITOR_TEST_REQUIRE_COORDINATOR = '1'
        $env:BUILD_EDITOR_TEST_FAIL_PACKAGE = 'zircon_runtime'

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $bundle

        $result.ExitCode | Should Be 1
        Test-Path -LiteralPath $bundle | Should Be $false
        $artifactDirectories = @(Get-Content -LiteralPath $artifactLog)
        $artifactDirectories.Count | Should Be 2
        foreach ($artifactDirectory in $artifactDirectories) {
            [System.IO.Directory]::Exists($artifactDirectory) | Should Be $false
        }
        $coordinatorCalls = @(Get-Content -LiteralPath $coordinatorLog)
        $coordinatorCalls.Count | Should Be 2
        $coordinatorCalls[0] | Should Match 'staging-acquire'
        $coordinatorCalls[1] | Should Match 'staging-release'
    }

    It 'rejects reparse-point asset content without leaving staged artifacts' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $fixtureAssetFile = Join-Path $fixtureAssets 'fixture.txt'
        [System.IO.File]::Delete($fixtureAssetFile)
        [System.IO.Directory]::Delete($fixtureAssets)
        New-Item -ItemType Junction -Path $fixtureAssets -Target $repoRoot | Out-Null

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $bundle

        $result.ExitCode | Should Be 1
        ($result.Output -join [Environment]::NewLine) | Should Match 'Refusing to copy a reparse-point bundle asset directory'
        Test-Path -LiteralPath $bundle | Should Be $false
        $artifactDirectories = @(Get-Content -LiteralPath $artifactLog)
        $artifactDirectories.Count | Should Be 2
        foreach ($artifactDirectory in $artifactDirectories) {
            [System.IO.Directory]::Exists($artifactDirectory) | Should Be $false
        }
    }

    It 'does not overwrite an existing output directory' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        [System.IO.Directory]::CreateDirectory($bundle) | Out-Null
        Set-Content -LiteralPath (Join-Path $bundle 'sentinel.txt') -Value 'keep me'

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $bundle

        $result.ExitCode | Should Be 1
        (Get-Content -LiteralPath (Join-Path $bundle 'sentinel.txt')) | Should Be 'keep me'
        Test-Path -LiteralPath $callLog | Should Be $false
    }

    It 'requires a requested bundle parent to exist before invoking the managed validator' {
        $bundle = Join-Path $fixtureRoot 'missing-parent\editor-bundle'

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $bundle

        $result.ExitCode | Should Be 1
        ($result.Output -join [Environment]::NewLine) | Should Match 'parent must already exist'
        Test-Path -LiteralPath (Join-Path $fixtureRoot 'missing-parent') | Should Be $false
        Test-Path -LiteralPath $callLog | Should Be $false
    }

    It 'resolves a relative output below the approved artifact root' {
        $approvedRoot = Join-Path ([System.IO.Path]::GetPathRoot($fixtureRoot)) 'ZirconBuilds'
        $relativeFixtureRoot = $fixtureRoot.Substring($approvedRoot.Length).TrimStart('\')
        $relativeBundle = Join-Path $relativeFixtureRoot 'relative-editor-bundle'
        $bundle = Join-Path $fixtureRoot 'relative-editor-bundle'

        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory $relativeBundle

        if ($result.ExitCode -ne 0) {
            throw "Relative bundle fixture failed: $($result.Output -join [Environment]::NewLine)"
        }
        $result.ExitCode | Should Be 0
        Test-Path -LiteralPath (Join-Path $bundle 'zircon_editor.exe') | Should Be $true
        Test-Path -LiteralPath (Join-Path $bundle 'zircon_runtime.dll') | Should Be $true
    }

    It 'rejects a C drive output before invoking the managed validator' {
        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory 'C:\ZirconBuilds\editor-bundle'

        $result.ExitCode | Should Be 1
        ($result.Output -join [Environment]::NewLine) | Should Match 'approved D:\\ZirconBuilds, E:\\ZirconBuilds, or F:\\ZirconBuilds'
        Test-Path -LiteralPath $callLog | Should Be $false
    }

    It 'rejects a drive-relative output before invoking the managed validator' {
        $result = Invoke-EditorBuildFixture -ScriptPath $fixtureScript -OutputDirectory 'C:editor-bundle'

        $result.ExitCode | Should Be 1
        ($result.Output -join [Environment]::NewLine) | Should Match 'must be drive-rooted'
        Test-Path -LiteralPath $callLog | Should Be $false
    }

    It 'publishes through a root-bound native rename rather than a re-resolved destination path' {
        $builderSource = Get-Content -LiteralPath $sourceScript -Raw
        $resolverSource = Get-Content -LiteralPath $sourcePathResolver -Raw

        $builderSource | Should Match 'Open-ZirconWindowsDirectoryLease `\s*-Path \$bundleOutput\.ApprovedRootOperationalPath'
        $builderSource | Should Match 'Open-ZirconWindowsDirectoryLease `\s*-Path \$stagingDirectory `\s*-ExpectedOperationalPath \$stagingDirectory `\s*-ForMove'
        $builderSource | Should Match '-DenyWrite'
        $builderSource | Should Match '-NoFollow'
        $builderSource | Should Match 'Move-ZirconWindowsLeasedPathWithinRoot `\s*-SourceLease \$stagingLease `\s*-Destination \$finalDirectory `\s*-ApprovedRoot \$bundleOutput\.ApprovedRootOperationalPath'
        $builderSource | Should Match 'Remove-ZirconWindowsLeasedDirectoryTree -Lease \$stagingLease'
        $resolverSource | Should Match 'SetFileInformationByHandle'
        $resolverSource | Should Match 'MovePathWithinRoot'
        $resolverSource | Should Match 'MoveLeasedPathWithinRoot'
        $resolverSource | Should Match 'DeleteLeasedEmptyDirectory'
        $resolverSource | Should Match 'DeleteLeasedDirectoryContents'
        $resolverSource | Should Match 'NativeMethodsV4'
        $resolverSource | Should Match 'RootDirectory = IntPtr\.Zero'
        $resolverSource | Should Match 'OpenPinnedDirectoryChain'
        $resolverSource | Should Match 'FileFlagOpenReparsePoint'
        $resolverSource | Should Match 'NtQueryDirectoryFile'
        $resolverSource | Should Match 'NtCreateFile'
        $resolverSource | Should Match 'OpenDirectoryEntryRelative'
        $resolverSource | Should Not Match 'Directory\.EnumerateFileSystemEntries'
        $resolverSource | Should Match 'denyWrite \? FileShareRead : FileShareRead \| FileShareWrite'
        $resolverSource | Should Match 'FileTraverse \| FileReadAttributes'
        $resolverSource | Should Match 'FileShareRead \| FileShareWrite'
    }

    It 'loads the versioned resolver interop when a legacy type is already loaded' {
        $legacyProbe = Join-Path $fixtureRoot 'legacy-resolver-probe.ps1'
        @'
param(
    [Parameter(Mandatory = $true)]
    [string]$ResolverPath
)

Add-Type -TypeDefinition @"
namespace ZirconEngine.WindowsPathResolver
{
    public static class NativeMethods
    {
        public static string LegacyMarker() { return "legacy"; }
    }

    public static class NativeMethodsV2
    {
        public static string PreviousMarker() { return "previous"; }
    }

    public static class NativeMethodsV3
    {
        public static string PreviousVersionMarker() { return "previous-version"; }
    }
}
"@ -ErrorAction Stop

Import-Module $ResolverPath -Force -DisableNameChecking -ErrorAction Stop
[void](Resolve-ZirconWindowsPath -Path $PWD)
if ($null -eq ('ZirconEngine.WindowsPathResolver.NativeMethodsV4' -as [type])) {
    throw 'NativeMethodsV4 did not load after the legacy types.'
}
Write-Output 'NativeMethodsV4 loaded'
'@ | Set-Content -LiteralPath $legacyProbe -Encoding UTF8

        $probeOutput = @(& powershell.exe `
            -NoProfile `
            -ExecutionPolicy Bypass `
            -File $legacyProbe `
            -ResolverPath $fixturePathResolver 2>&1)

        $LASTEXITCODE | Should Be 0
        ($probeOutput -join [Environment]::NewLine) | Should Match 'NativeMethodsV4'
    }

    It 'prevents replacement while an approved directory lease is held' {
        Import-Module $fixturePathResolver -Force -DisableNameChecking -ErrorAction Stop
        $fixtureOperationalPath = (Resolve-ZirconWindowsPath -Path $fixtureRoot).OperationalPath
        $movedFixtureRoot = "$fixtureRoot-lease-replacement"
        $lease = Open-ZirconWindowsDirectoryLease `
            -Path $fixtureOperationalPath `
            -ExpectedOperationalPath $fixtureOperationalPath `
            -DenyWrite
        try {
            $moveFailure = $null
            try {
                Move-Item -LiteralPath $fixtureRoot -Destination $movedFixtureRoot -ErrorAction Stop
            }
            catch {
                $moveFailure = $_
            }

            $moveFailure | Should Not BeNullOrEmpty
            [System.IO.Directory]::Exists($fixtureOperationalPath) | Should Be $true
        }
        finally {
            $lease.Dispose()
            if ([System.IO.Directory]::Exists($movedFixtureRoot)) {
                [System.IO.Directory]::Move($movedFixtureRoot, $fixtureRoot)
            }
        }
    }

    It 'marks an empty held staging directory for deletion through its lease' {
        Import-Module $fixturePathResolver -Force -DisableNameChecking -ErrorAction Stop
        $stagingDirectory = Join-Path $fixtureRoot 'leased-empty-staging'
        [System.IO.Directory]::CreateDirectory($stagingDirectory) | Out-Null
        $stagingOperationalPath = (Resolve-ZirconWindowsPath -Path $stagingDirectory).OperationalPath
        $lease = Open-ZirconWindowsDirectoryLease `
            -Path $stagingOperationalPath `
            -ExpectedOperationalPath $stagingOperationalPath `
            -ForMove `
            -DenyWrite `
            -NoFollow
        try {
            Remove-ZirconWindowsLeasedDirectory -Lease $lease
        }
        finally {
            $lease.Dispose()
        }

        [System.IO.Directory]::Exists($stagingOperationalPath) | Should Be $false
    }

    It 'removes a held staging tree without following an external junction' {
        Import-Module $fixturePathResolver -Force -DisableNameChecking -ErrorAction Stop
        $fixtureDriveRoot = [System.IO.Path]::GetPathRoot($fixtureRoot)
        $stagingDirectory = Join-Path $fixtureRoot 'leased-staging-tree'
        $nestedDirectory = Join-Path $stagingDirectory 'nested'
        $outsideTarget = Join-Path $fixtureDriveRoot ('zircon-build-editor-cleanup-outside-' + [guid]::NewGuid().ToString('N'))
        $junctionDirectory = Join-Path $stagingDirectory 'outside-link'
        $sentinel = Join-Path $outsideTarget 'sentinel.txt'
        [System.IO.Directory]::CreateDirectory($nestedDirectory) | Out-Null
        [System.IO.Directory]::CreateDirectory($outsideTarget) | Out-Null
        Set-Content -LiteralPath (Join-Path $nestedDirectory 'payload.txt') -Value 'staging'
        Set-Content -LiteralPath $sentinel -Value 'outside'
        New-Item -ItemType Junction -Path $junctionDirectory -Target $outsideTarget | Out-Null
        $lease = $null
        try {
            $stagingOperationalPath = (Resolve-ZirconWindowsPath -Path $stagingDirectory).OperationalPath
            $lease = Open-ZirconWindowsDirectoryLease `
                -Path $stagingOperationalPath `
                -ExpectedOperationalPath $stagingOperationalPath `
                -ForMove `
                -DenyWrite `
                -NoFollow
            Remove-ZirconWindowsLeasedDirectoryTree -Lease $lease
            $lease.Dispose()
            $lease = $null

            [System.IO.Directory]::Exists($stagingOperationalPath) | Should Be $false
            [System.IO.File]::Exists($sentinel) | Should Be $true
        }
        finally {
            if ($null -ne $lease) {
                $lease.Dispose()
            }
            if ([System.IO.Directory]::Exists($junctionDirectory)) {
                [System.IO.Directory]::Delete($junctionDirectory, $false)
            }
            if ([System.IO.Directory]::Exists($stagingDirectory)) {
                [System.IO.Directory]::Delete($stagingDirectory, $true)
            }
            if ([System.IO.Directory]::Exists($outsideTarget)) {
                [System.IO.Directory]::Delete($outsideTarget, $true)
            }
        }
    }

    It 'rejects a resolved output parent outside the approved root without moving the source' {
        Import-Module $sourcePathResolver -Force -DisableNameChecking -ErrorAction Stop
        $fixtureDriveRoot = [System.IO.Path]::GetPathRoot($fixtureRoot)
        $approvedRoot = Join-Path $fixtureDriveRoot 'ZirconBuilds'
        $sourceDirectory = Join-Path $fixtureRoot 'rename-source'
        $outsideTarget = Join-Path $fixtureDriveRoot ('zircon-build-editor-rename-outside-' + [guid]::NewGuid().ToString('N'))
        $junctionDirectory = Join-Path $fixtureRoot 'rename-outside-root'
        $fixtureRelativePath = $fixtureRoot.Substring($approvedRoot.Length).TrimStart('\')
        [System.IO.Directory]::CreateDirectory($sourceDirectory) | Out-Null
        [System.IO.Directory]::CreateDirectory($outsideTarget) | Out-Null
        try {
            New-Item -ItemType Junction -Path $junctionDirectory -Target $outsideTarget | Out-Null
            $moveFailure = $null
            try {
                Move-ZirconWindowsPath `
                    -Source (Resolve-ZirconWindowsPath -Path $sourceDirectory).OperationalPath `
                    -Destination (Join-ZirconWindowsPath `
                        -Path (Resolve-ZirconWindowsPath -Path $approvedRoot).OperationalPath `
                        -ChildPath (Join-Path $fixtureRelativePath 'rename-outside-root\published')) `
                    -ApprovedRoot (Resolve-ZirconWindowsPath -Path $approvedRoot).OperationalPath
            }
            catch {
                $moveFailure = $_
            }

            $moveFailure | Should Not BeNullOrEmpty
            $moveFailure.Exception.Message | Should Match 'outside the approved root'
            [System.IO.Directory]::Exists($sourceDirectory) | Should Be $true
            [System.IO.Directory]::Exists((Join-Path $outsideTarget 'published')) | Should Be $false
        }
        finally {
            if ([System.IO.Directory]::Exists($junctionDirectory)) {
                [System.IO.Directory]::Delete($junctionDirectory, $false)
            }
            if ([System.IO.Directory]::Exists($outsideTarget)) {
                [System.IO.Directory]::Delete($outsideTarget, $false)
            }
        }
    }

    It 'rejects an approved-looking output that resolves through a junction outside build roots' {
        $fixtureDriveRoot = [System.IO.Path]::GetPathRoot($fixtureRoot)
        $outsideTarget = Join-Path $fixtureDriveRoot ('zircon-build-editor-outside-' + [guid]::NewGuid().ToString('N'))
        $junctionDirectory = Join-Path $fixtureRoot 'outside-build-root'
        [System.IO.Directory]::CreateDirectory($outsideTarget) | Out-Null
        try {
            New-Item -ItemType Junction -Path $junctionDirectory -Target $outsideTarget | Out-Null
            $result = Invoke-EditorBuildFixture `
                -ScriptPath $fixtureScript `
                -OutputDirectory (Join-Path $junctionDirectory 'editor-bundle')

            $result.ExitCode | Should Be 1
            ($result.Output -join [Environment]::NewLine) | Should Match 'approved D:\\ZirconBuilds, E:\\ZirconBuilds, or F:\\ZirconBuilds'
            Test-Path -LiteralPath $callLog | Should Be $false
        }
        finally {
            if ([System.IO.Directory]::Exists($junctionDirectory)) {
                [System.IO.Directory]::Delete($junctionDirectory, $false)
            }
            if ([System.IO.Directory]::Exists($outsideTarget)) {
                [System.IO.Directory]::Delete($outsideTarget, $false)
            }
        }
    }
}
