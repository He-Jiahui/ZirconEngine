Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$buildSetModule = Join-Path $repoRoot 'tools\mvp\MvpBuildSet.psm1'
if (Test-Path -LiteralPath $buildSetModule) {
    Import-Module $buildSetModule -Force -ErrorAction Stop
}

function Invoke-BuildSetFixtureGit {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string[]]$Arguments
    )

    $previousErrorActionPreference = $ErrorActionPreference
    try {
        # Windows PowerShell promotes Git's successful CRLF warning to an ErrorRecord when
        # the enclosing fixture runs with ErrorActionPreference=Stop.
        $ErrorActionPreference = 'Continue'
        $output = @(& git -C $RepositoryRoot @Arguments 2>&1)
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previousErrorActionPreference
    }
    if ($exitCode -ne 0) {
        throw "Fixture git command failed: git -C $RepositoryRoot $($Arguments -join ' ')`n$($output -join [Environment]::NewLine)"
    }
    return $output
}

function New-BuildSetFixtureRepository {
    param([Parameter(Mandatory)][string]$Name)

    $fixtureRoot = Join-Path $TestDrive $Name
    New-Item -ItemType Directory -Path (Join-Path $fixtureRoot 'src') -Force | Out-Null
    [IO.File]::WriteAllText((Join-Path $fixtureRoot 'Cargo.toml'), "[package]`nname = 'fixture'`nversion = '0.1.0'`n", [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $fixtureRoot 'src\lib.rs'), 'pub const VALUE: u32 = 1;', [Text.UTF8Encoding]::new($false))
    Invoke-BuildSetFixtureGit -RepositoryRoot $fixtureRoot -Arguments @('init') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $fixtureRoot -Arguments @('config', 'user.email', 'fixture@example.invalid') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $fixtureRoot -Arguments @('config', 'user.name', 'fixture') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $fixtureRoot -Arguments @('add', '--all') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $fixtureRoot -Arguments @('commit', '-m', 'fixture baseline') | Out-Null
    [IO.File]::WriteAllText((Join-Path $fixtureRoot 'src\lib.rs'), 'pub const VALUE: u32 = 2;', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $fixtureRoot 'untracked-noise.txt'), 'must not enter the BuildSet', [Text.UTF8Encoding]::new($false))
    return $fixtureRoot
}

function New-BuildSetSubmoduleFixtureRepository {
    param([Parameter(Mandatory)][string]$Name)

    $submoduleRoot = Join-Path $TestDrive "$Name-submodule"
    New-Item -ItemType Directory -Path $submoduleRoot -Force | Out-Null
    [IO.File]::WriteAllText((Join-Path $submoduleRoot 'README.md'), 'fixture submodule', [Text.UTF8Encoding]::new($false))
    Invoke-BuildSetFixtureGit -RepositoryRoot $submoduleRoot -Arguments @('init') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $submoduleRoot -Arguments @('config', 'user.email', 'fixture@example.invalid') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $submoduleRoot -Arguments @('config', 'user.name', 'fixture') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $submoduleRoot -Arguments @('add', '--all') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $submoduleRoot -Arguments @('commit', '-m', 'submodule baseline') | Out-Null

    $sourceRoot = New-BuildSetFixtureRepository -Name $Name
    Invoke-BuildSetFixtureGit -RepositoryRoot $sourceRoot -Arguments @('add', '--all') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $sourceRoot -Arguments @('commit', '-m', 'source baseline') | Out-Null
    Invoke-BuildSetFixtureGit `
        -RepositoryRoot $sourceRoot `
        -Arguments @('-c', 'protocol.file.allow=always', 'submodule', 'add', $submoduleRoot, 'deps/fixture') | Out-Null
    Invoke-BuildSetFixtureGit -RepositoryRoot $sourceRoot -Arguments @('commit', '-am', 'add fixture submodule') | Out-Null
    return $sourceRoot
}

Describe 'MVP product BuildSet' {
    It 'encodes BuildSet SHA-256 values through CLR uppercase hex conversion' {
        $module = Get-Module -Name MvpBuildSet -ErrorAction Stop
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $moduleSource | Should Match '\[BitConverter\]::ToString\('
        $moduleSource | Should Not Match 'ForEach-Object \{ \$_.ToString\(''X2''\) \}'
        $identityStart = $moduleSource.IndexOf('function Get-MvpBuildSetId')
        $identitySource = $moduleSource.Substring(
            $identityStart,
            $moduleSource.IndexOf('function Write-MvpBuildSetJson') - $identityStart)
        $identitySource | Should Match '\[Security\.Cryptography\.CryptoStream\]::new'
        $identitySource | Should Match '\[IO\.BinaryWriter\]::new'
        $identitySource | Should Match '\$cryptoStream\.FlushFinalBlock\(\)'
        $identitySource | Should Not Match 'MemoryStream'
        $identitySource | Should Not Match 'ToArray'

        $gitRevision = 'a' * 40
        $overlaySha256 = 'B' * 64
        $files = @(
            [pscustomobject]@{ relative_path = 'Cargo.toml'; sha256 = ('C' * 64); byte_length = 17 },
            [pscustomobject]@{ relative_path = 'src/lib.rs'; sha256 = ('D' * 64); byte_length = 29 }
        )
        $actualIdentity = & $module {
            param($Revision, $Overlay, $Entries)
            Get-MvpBuildSetId -GitRevision $Revision -DirtyOverlaySha256 $Overlay -Files $Entries
        } $gitRevision $overlaySha256 $files
        $legacyMaterial = [IO.MemoryStream]::new()
        $encoding = [Text.UTF8Encoding]::new($false)
        try {
            foreach ($segment in @(
                    'zircon-mvp-build-set-v1', $gitRevision, $overlaySha256,
                    'Cargo.toml', ('C' * 64), '17',
                    'src/lib.rs', ('D' * 64), '29'
                )) {
                [byte[]]$segmentBytes = $encoding.GetBytes($segment)
                [byte[]]$lengthBytes = [BitConverter]::GetBytes([int64]$segmentBytes.LongLength)
                $legacyMaterial.Write($lengthBytes, 0, $lengthBytes.Length)
                $legacyMaterial.Write($segmentBytes, 0, $segmentBytes.Length)
            }
            $legacyHasher = [Security.Cryptography.SHA256]::Create()
            try {
                $expectedIdentity = [BitConverter]::ToString($legacyHasher.ComputeHash($legacyMaterial.ToArray())).Replace('-', '')
            }
            finally {
                $legacyHasher.Dispose()
            }
        }
        finally {
            $legacyMaterial.Dispose()
        }
        $actualIdentity | Should Be $expectedIdentity
    }

    It 'does not publish a completed manifest when final self-validation fails' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'failed-self-validation-source'
        $buildSetRoot = Join-Path $TestDrive 'failed-self-validation-build-set'
        Mock -CommandName Assert-MvpProductBuildSet -ModuleName MvpBuildSet -ParameterFilter {
            $ManifestPath -like '*failed-self-validation-build-set*build-set-pending.json'
        } {
            throw 'fixture self-validation failure'
        }

        {
            New-MvpProductBuildSet -RepositoryRoot $sourceRoot -BuildSetRoot $buildSetRoot
        } | Should Throw 'fixture self-validation failure'

        (Test-Path -LiteralPath (Join-Path $buildSetRoot 'build-set.json') -PathType Leaf) | Should Be $false
        (Test-Path -LiteralPath (Join-Path $buildSetRoot 'build-set-incomplete.json') -PathType Leaf) | Should Be $true
    }

    It 'materializes a newly added staged tracked file into the immutable snapshot' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'added-tracked-source'
        [IO.File]::WriteAllText((Join-Path $sourceRoot 'added.rs'), 'pub const ADDED: u32 = 4;', [Text.UTF8Encoding]::new($false))
        Invoke-BuildSetFixtureGit -RepositoryRoot $sourceRoot -Arguments @('add', 'added.rs') | Out-Null

        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'added-tracked-build-set')

        (Get-Content -LiteralPath (Join-Path $buildSet.snapshot_root 'added.rs') -Raw) | Should Be 'pub const ADDED: u32 = 4;'
        @($buildSet.files | Where-Object { $_.relative_path -eq 'added.rs' }).Count | Should Be 1
        (Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path).build_set_id | Should Be $buildSet.build_set_id
    }

    It 'does not retain a tracked file deleted by the dirty overlay' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'deleted-tracked-source'
        Remove-Item -LiteralPath (Join-Path $sourceRoot 'src\lib.rs') -Force

        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'deleted-tracked-build-set')

        (Test-Path -LiteralPath (Join-Path $buildSet.snapshot_root 'src\lib.rs')) | Should Be $false
        @($buildSet.files | Where-Object { $_.relative_path -eq 'src/lib.rs' }).Count | Should Be 0
        (Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path).build_set_id | Should Be $buildSet.build_set_id
    }

    It 'derives the same BuildSetId from identical immutable source bytes' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'stable-id-source'

        $first = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'stable-id-first')
        $second = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'stable-id-second')

        $second.build_set_id | Should Be $first.build_set_id
        $second.dirty_overlay_sha256 | Should Be $first.dirty_overlay_sha256
    }

    It 'rejects a snapshot file changed after publication' {
        Get-Command New-MvpProductBuildSet -ErrorAction SilentlyContinue | Should Not BeNullOrEmpty

        $sourceRoot = New-BuildSetFixtureRepository -Name 'tamper-source'
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'tamper-build-set')
        [IO.File]::WriteAllText((Join-Path $buildSet.snapshot_root 'src\lib.rs'), 'pub const VALUE: u32 = 3;', [Text.UTF8Encoding]::new($false))

        {
            Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path
        } | Should Throw 'content differs'
    }

    It 'rejects a Git submodule that has no materialized source closure' {
        $sourceRoot = New-BuildSetSubmoduleFixtureRepository -Name 'submodule-source'

        {
            New-MvpProductBuildSet `
                -RepositoryRoot $sourceRoot `
                -BuildSetRoot (Join-Path $TestDrive 'submodule-build-set')
        } | Should Throw 'submodule'
    }

    It 'rejects an unmaterialized Git LFS pointer instead of hashing the pointer text' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'lfs-pointer-source'
        [IO.File]::WriteAllText(
            (Join-Path $sourceRoot 'payload.bin'),
            "version https://git-lfs.github.com/spec/v1`noid sha256:$('A' * 64)`nsize 123`n",
            [Text.UTF8Encoding]::new($false)
        )
        Invoke-BuildSetFixtureGit -RepositoryRoot $sourceRoot -Arguments @('add', 'payload.bin') | Out-Null

        {
            New-MvpProductBuildSet `
                -RepositoryRoot $sourceRoot `
                -BuildSetRoot (Join-Path $TestDrive 'lfs-pointer-build-set')
        } | Should Throw 'LFS pointer'
    }

    It 'materializes dirty tracked source while excluding untracked files' {
        Get-Command New-MvpProductBuildSet -ErrorAction SilentlyContinue | Should Not BeNullOrEmpty

        $sourceRoot = New-BuildSetFixtureRepository -Name 'materialization-source'
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'materialization-build-set')

        $buildSet.schema_version | Should Be 1
        $buildSet.build_set_kind | Should Be 'zircon_mvp_product_build_set'
        $buildSet.build_set_id | Should Match '^[0-9A-F]{64}$'
        (Get-Content -LiteralPath (Join-Path $buildSet.snapshot_root 'src\lib.rs') -Raw) | Should Be 'pub const VALUE: u32 = 2;'
        (Test-Path -LiteralPath (Join-Path $buildSet.snapshot_root 'untracked-noise.txt')) | Should Be $false
        (Test-Path -LiteralPath $buildSet.manifest_path -PathType Leaf) | Should Be $true

        [IO.File]::WriteAllText((Join-Path $sourceRoot 'src\lib.rs'), 'pub const VALUE: u32 = 3;', [Text.UTF8Encoding]::new($false))
        (Get-Content -LiteralPath (Join-Path $buildSet.snapshot_root 'src\lib.rs') -Raw) | Should Be 'pub const VALUE: u32 = 2;'

        $validated = Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path
        $validated.build_set_id | Should Be $buildSet.build_set_id
    }

    It 'materializes staged tracked source changes into the immutable snapshot' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'staged-materialization-source'
        Invoke-BuildSetFixtureGit -RepositoryRoot $sourceRoot -Arguments @('add', 'src/lib.rs') | Out-Null

        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'staged-materialization-build-set')

        (Get-Content -LiteralPath (Join-Path $buildSet.snapshot_root 'src\lib.rs') -Raw) | Should Be 'pub const VALUE: u32 = 2;'
        $buildSet.dirty_overlay_sha256 | Should Not Be ('0' * 64)
        (Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path).build_set_id | Should Be $buildSet.build_set_id
    }

    It 'retains captured bytes when the active source returns from B to HEAD A' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'source-return-source'
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'source-return-build-set')

        [IO.File]::WriteAllText((Join-Path $sourceRoot 'src\lib.rs'), 'pub const VALUE: u32 = 1;', [Text.UTF8Encoding]::new($false))

        (Get-Content -LiteralPath (Join-Path $buildSet.snapshot_root 'src\lib.rs') -Raw) | Should Be 'pub const VALUE: u32 = 2;'
        (Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path).build_set_id | Should Be $buildSet.build_set_id
    }

    It 'keeps the BuildSet verifiable after its publication root is renamed' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'relocatable-source'
        $partialRoot = Join-Path $TestDrive 'relocatable.partial-fixture'
        $publishedRoot = Join-Path $TestDrive 'relocatable-published'
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot $partialRoot

        [IO.Directory]::Move($partialRoot, $publishedRoot)
        $publishedManifestPath = Join-Path $publishedRoot 'build-set.json'
        $persisted = Get-Content -LiteralPath $publishedManifestPath -Raw | ConvertFrom-Json
        $validated = Assert-MvpProductBuildSet -ManifestPath $publishedManifestPath

        $persisted.snapshot_relative_path | Should Be 'source'
        $persisted.PSObject.Properties['snapshot_root'] | Should BeNullOrEmpty
        $validated.build_set_id | Should Be $buildSet.build_set_id
        $validated.snapshot_root | Should Be (Join-Path $publishedRoot 'source')
    }

    It 'verifies a published BuildSet through a Windows device-path manifest' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'device-manifest-source'
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'device-manifest-build-set')
        $deviceManifestPath = '\\?\' + $buildSet.manifest_path

        $validated = Assert-MvpProductBuildSet -ManifestPath $deviceManifestPath

        $validated.build_set_id | Should Be $buildSet.build_set_id
        $validated.snapshot_root | Should Be ('\\?\' + $buildSet.snapshot_root)
    }

    It 'rejects unknown BuildSet manifest and file-entry properties' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'strict-schema-source'
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'strict-schema-build-set')
        $manifest = Get-Content -LiteralPath $buildSet.manifest_path -Raw | ConvertFrom-Json

        $manifest | Add-Member -NotePropertyName unreviewed -NotePropertyValue $true
        [IO.File]::WriteAllText(
            $buildSet.manifest_path,
            ($manifest | ConvertTo-Json -Depth 8),
            [Text.UTF8Encoding]::new($false))
        { Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path } |
            Should Throw 'unknown property'

        $manifest.PSObject.Properties.Remove('unreviewed')
        $manifest.files[0] | Add-Member -NotePropertyName unreviewed -NotePropertyValue $true
        [IO.File]::WriteAllText(
            $buildSet.manifest_path,
            ($manifest | ConvertTo-Json -Depth 8),
            [Text.UTF8Encoding]::new($false))
        { Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path } |
            Should Throw 'unknown property'
    }

    It 'rejects a non-canonical backslash manifest path before filesystem access' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'canonical-path-source'
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'canonical-path-build-set')
        $manifest = Get-Content -LiteralPath $buildSet.manifest_path -Raw | ConvertFrom-Json
        $sourceEntry = @($manifest.files | Where-Object { $_.relative_path -eq 'src/lib.rs' })[0]
        $sourceEntry.relative_path = 'src\lib.rs'
        [IO.File]::WriteAllText(
            $buildSet.manifest_path,
            ($manifest | ConvertTo-Json -Depth 8),
            [Text.UTF8Encoding]::new($false))

        { Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path } |
            Should Throw 'unsafe relative path'
    }

    It 'rejects a snapshot root replaced by a directory junction' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'junction-source'
        $buildSet = New-MvpProductBuildSet `
            -RepositoryRoot $sourceRoot `
            -BuildSetRoot (Join-Path $TestDrive 'junction-build-set')
        $junctionTarget = Join-Path $TestDrive 'junction-target'
        [IO.Directory]::Move($buildSet.snapshot_root, $junctionTarget)
        New-Item -ItemType Junction -Path $buildSet.snapshot_root -Target $junctionTarget | Out-Null

        { Assert-MvpProductBuildSet -ManifestPath $buildSet.manifest_path } |
            Should Throw 'reparse-point directory'
    }

    It 'rejects a symbolic-link Git index entry even when Windows materializes link text' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'symbolic-link-source'
        [IO.File]::WriteAllText((Join-Path $sourceRoot 'symlink-target.txt'), '../outside', [Text.UTF8Encoding]::new($false))
        $objectId = [string](Invoke-BuildSetFixtureGit `
                -RepositoryRoot $sourceRoot `
                -Arguments @('hash-object', '-w', 'symlink-target.txt') |
            Select-Object -Last 1).Trim()
        Invoke-BuildSetFixtureGit `
            -RepositoryRoot $sourceRoot `
            -Arguments @('update-index', '--add', '--cacheinfo', '120000', $objectId, 'linked-source') | Out-Null
        $sourceIndex = @(Invoke-BuildSetFixtureGit `
                -RepositoryRoot $sourceRoot `
                -Arguments @('ls-files', '--stage', '--', 'linked-source'))

        $sourceIndex | Should Match '^120000 [0-9a-f]{40} 0\tlinked-source$'

        {
            New-MvpProductBuildSet `
                -RepositoryRoot $sourceRoot `
                -BuildSetRoot (Join-Path $TestDrive 'symbolic-link-build-set')
        } | Should Throw 'symbolic link'
    }

    It 'releases a worktree when source enumeration fails before publication' {
        $sourceRoot = New-BuildSetFixtureRepository -Name 'failed-enumeration-source'
        $buildSetRoot = Join-Path $TestDrive 'failed-enumeration-build-set'
        Mock -CommandName Get-MvpBuildSetTrackedFiles -ModuleName MvpBuildSet -ParameterFilter {
            $SnapshotRoot -like '*failed-enumeration-build-set*\source'
        } {
            throw 'fixture source enumeration failure'
        }

        {
            New-MvpProductBuildSet -RepositoryRoot $sourceRoot -BuildSetRoot $buildSetRoot
        } | Should Throw 'fixture source enumeration failure'

        (Test-Path -LiteralPath (Join-Path $buildSetRoot 'source\.git')) | Should Be $false
        $worktreeOutput = @(& git -C $sourceRoot worktree list --porcelain)
        $worktreeOutput | Should Not Match ([Regex]::Escape((Join-Path $buildSetRoot 'source')))
        (Test-Path -LiteralPath (Join-Path $buildSetRoot 'build-set-incomplete.json') -PathType Leaf) | Should Be $true
    }

}

Describe 'MVP BuildSet index allocation contracts' {
    It 'uses direct Git process argument and output buffers' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $gitStart = $moduleSource.IndexOf('function Invoke-MvpBuildSetGit')
        $gitSource = $moduleSource.Substring(
            $gitStart,
            $moduleSource.IndexOf('function Invoke-MvpBuildSetGitBytes') - $gitStart)
        $captureStart = $moduleSource.IndexOf('function Invoke-MvpBuildSetGitBytes')
        $captureSource = $moduleSource.Substring(
            $captureStart,
            $moduleSource.IndexOf('function Assert-MvpBuildSetSourceIndexModePolicy') - $captureStart)
        $sourcePolicyStart = $moduleSource.IndexOf('function Assert-MvpBuildSetSourceIndexModePolicy')
        $sourcePolicySource = $moduleSource.Substring(
            $sourcePolicyStart,
            $moduleSource.IndexOf('function Assert-MvpBuildSetExactProperties') - $sourcePolicyStart)

        $moduleSource | Should Match '\$script:MvpBuildSetLineSeparators = \[string\[\]\]@\("`r`n", "`n"\)'
        $moduleSource | Should Match '\$script:MvpBuildSetNulSeparator = \[char\[\]\]@\(\[char\]0\)'
        $gitSource | Should Match '\$quotedArguments = \[string\[\]\]::new\(\$Arguments\.Length\)'
        $gitSource | Should Match 'for \(\$index = 0; \$index -lt \$Arguments\.Length; \$index\+\+\)'
        $gitSource | Should Match '\$argument\.Contains\(''"''\)'
        $gitSource | Should Match '\[string\]::Join\('' '', \$quotedArguments\)'
        $gitSource | Should Match '\$stdoutTask\.Result\.Split\(\s*\$script:MvpBuildSetLineSeparators,\s*\[StringSplitOptions\]::RemoveEmptyEntries\)'
        $gitSource | Should Not Match 'Where-Object|ForEach-Object|-split'
        $captureSource | Should Match '\$output\.GetBuffer\(\)'
        $captureSource | Should Match '\[Tuple\[object, int\]\]::new'
        $captureSource | Should Not Match '\$output\.ToArray\(\)'
        $sourcePolicySource | Should Match '\.Split\(\s*\$script:MvpBuildSetNulSeparator,\s*\[StringSplitOptions\]::RemoveEmptyEntries\)'
        $sourcePolicySource | Should Match '\$indexBuffer = \$null'
        $sourcePolicySource | Should Match '\$indexCapture = \$null'
        $sourcePolicySource | Should Match 'if \(\$metadata\[0\] -eq ''120000'' -or\s+\$metadata\[0\] -eq ''160000''\) \{\s+\$relativePath = \$entry\.Substring\(\$separator \+ 1\)\.Replace'
        $sourcePolicySource | Should Not Match '\.Split\(\[char\]0\)|\$entry\.Length -eq 0'
    }

    It 'consumes canonical tracked Git paths without a no-op separator scan' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)

        $trackedSource | Should Match '(?m)^\s+\$relativePath = \$entry\.Substring\(\$separator \+ 1\)$'
        $trackedSource | Should Not Match '\$entry\.Substring\(\$separator \+ 1\)\.Replace'
    }

    It 'indexes typed single-value Git results without pipeline construction' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $publisherStart = $moduleSource.IndexOf('function New-MvpProductBuildSet')
        $publisherSource = $moduleSource.Substring(
            $publisherStart,
            $moduleSource.IndexOf('function Assert-MvpProductBuildSet') - $publisherStart)

        $publisherSource | Should Match '\[string\[\]\]\$reportedRootLines = Invoke-MvpBuildSetGit'
        $publisherSource | Should Match '\$reportedRoot = \[string\]\$reportedRootLines\[0\]'
        $publisherSource | Should Match '\[string\[\]\]\$revisionLines = Invoke-MvpBuildSetGit'
        $publisherSource | Should Match '(?m)^\s+\$revision = \[string\]\$revisionLines\[0\]$'
        $publisherSource | Should Not Match 'Select-Object -First 1|\$reportedRoot\.Trim\(\)|\$revisionLines\[0\]\.Trim\(\)'
    }

    It 'resolves fixed publication children without provider cmdlets' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $publisherStart = $moduleSource.IndexOf('function New-MvpProductBuildSet')
        $publisherSource = $moduleSource.Substring(
            $publisherStart,
            $moduleSource.IndexOf('function Assert-MvpProductBuildSet') - $publisherStart)

        $publisherSource | Should Match '\$snapshotRoot = \[IO\.Path\]::Combine\(\$finalRoot, ''source''\)'
        $publisherSource | Should Match '\$overlayPath = \[IO\.Path\]::Combine\(\$finalRoot, ''tracked-dirty-overlay\.patch''\)'
        $publisherSource | Should Match '\$manifestPath = \[IO\.Path\]::Combine\(\$finalRoot, ''build-set\.json''\)'
        $publisherSource | Should Match '\$pendingManifestPath = \[IO\.Path\]::Combine\(\$finalRoot, ''build-set-pending\.json''\)'
        $publisherSource | Should Not Match 'Join-Path \$finalRoot'
    }

    It 'resolves the publication parent without a provider pipeline' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $publisherStart = $moduleSource.IndexOf('function New-MvpProductBuildSet')
        $publisherSource = $moduleSource.Substring(
            $publisherStart,
            $moduleSource.IndexOf('function Assert-MvpProductBuildSet') - $publisherStart)

        $publisherSource | Should Match '\$parent = \[IO\.Path\]::GetDirectoryName\(\$finalRoot\)'
        $publisherSource | Should Not Match 'Split-Path -Parent \$finalRoot'
    }

    It 'discards successful Git output before line projection for no-output commands' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $gitStart = $moduleSource.IndexOf('function Invoke-MvpBuildSetGit')
        $gitSource = $moduleSource.Substring(
            $gitStart,
            $moduleSource.IndexOf('function Invoke-MvpBuildSetGitBytes') - $gitStart)
        $publisherStart = $moduleSource.IndexOf('function New-MvpProductBuildSet')
        $publisherSource = $moduleSource.Substring(
            $publisherStart,
            $moduleSource.IndexOf('function Assert-MvpProductBuildSet') - $publisherStart)

        $gitSource | Should Match '\[switch\]\$DiscardOutput'
        $gitSource | Should Match 'if \(\$DiscardOutput\) \{\s*return\s*\}\s*return \$stdoutTask\.Result\.Split\('
        @([regex]::Matches($publisherSource, '-DiscardOutput')).Count | Should Be 4
    }

    It 'caches the resolved Git executable path across publication calls' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $publisherStart = $moduleSource.IndexOf('function New-MvpProductBuildSet')
        $publisherSource = $moduleSource.Substring(
            $publisherStart,
            $moduleSource.IndexOf('function Assert-MvpProductBuildSet') - $publisherStart)

        $publisherSource | Should Match '\$gitPath = \[string\]\$git\.Source'
        @([regex]::Matches($publisherSource, '\$git\.Source')).Count | Should Be 1
        @([regex]::Matches($publisherSource, '-GitPath \$gitPath')).Count | Should Be 8
    }

    It 'discards CLR directory results without Out-Null pipelines' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $publisherStart = $moduleSource.IndexOf('function New-MvpProductBuildSet')
        $publisherSource = $moduleSource.Substring(
            $publisherStart,
            $moduleSource.IndexOf('function Assert-MvpProductBuildSet') - $publisherStart)

        @([regex]::Matches($publisherSource, '\$null = \[IO\.Directory\]::CreateDirectory\(')).Count | Should Be 2
        $publisherSource | Should Not Match '\[IO\.Directory\]::CreateDirectory\([^\r\n]+\) \| Out-Null'
    }

    It 'serializes one manifest object without a pipeline' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $writerStart = $moduleSource.IndexOf('function Write-MvpBuildSetJson')
        $writerSource = $moduleSource.Substring(
            $writerStart,
            $moduleSource.IndexOf('function Write-MvpBuildSetIncompleteReceipt') - $writerStart)

        $writerSource | Should Match 'ConvertTo-Json -InputObject \$Value -Depth 12'
        $writerSource | Should Not Match '\$Value \| ConvertTo-Json'
    }

    It 'resolves the incomplete receipt child without a provider cmdlet' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $incompleteStart = $moduleSource.IndexOf('function Write-MvpBuildSetIncompleteReceipt')
        $incompleteSource = $moduleSource.Substring(
            $incompleteStart,
            $moduleSource.IndexOf('function Assert-MvpBuildSetInventory') - $incompleteStart)

        $incompleteSource | Should Match '\$path = \[IO\.Path\]::Combine\(\$BuildSetRoot, ''build-set-incomplete\.json''\)'
        $incompleteSource | Should Not Match 'Join-Path \$BuildSetRoot'
    }

    It 'reuses one UTF8 encoder across BuildSet identity and persistence paths' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw

        $moduleSource | Should Match '\$script:MvpBuildSetUtf8 = \[Text\.UTF8Encoding\]::new\(\$false\)'
        @([regex]::Matches($moduleSource, '\[Text\.UTF8Encoding\]::new\(\$false\)')).Count | Should Be 1
        $moduleSource | Should Match '\$script:MvpBuildSetUtf8\.GetString\('
        $moduleSource | Should Match '\$encoding = \$script:MvpBuildSetUtf8'
        $moduleSource | Should Match '\$script:MvpBuildSetUtf8\.GetBytes\('
        $moduleSource | Should Match 'ReadAllText\(\$resolvedManifestPath, \$script:MvpBuildSetUtf8\)'
    }

    It 'reuses one normalized snapshot root and resolves hot-path children inline' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring($validatorStart)

        $moduleSource | Should Not Match 'function Resolve-MvpBuildSetChildPath(?:Normalized)?'
        foreach ($source in @($trackedSource, $validatorSource)) {
            $source | Should Match '\$snapshotRootPrefix = \$(?:normalized)?SnapshotRoot \+ \[IO\.Path\]::DirectorySeparatorChar'
            $source | Should Match '\$path = \[IO\.Path\]::Combine\(\$(?:normalized)?SnapshotRoot, \$platformRelativePath\)'
            $source | Should Match '\.StartsWith\(\$snapshotRootPrefix, \[StringComparison\]::OrdinalIgnoreCase\)'
            $source | Should Not Match 'Resolve-MvpBuildSetChildPath(?:Normalized)?\s+`?\s*-'
            $source | Should Not Match '\$path = \[IO\.Path\]::GetFullPath\(\s*\[IO\.Path\]::Combine\('
        }
        $trackedSource | Should Match '\$script:MvpBuildSetUnsafeRelativePathPattern\.IsMatch\(\$relativePath\)'
        $validatorSource | Should Match '\$relativePath\.IndexOf\(\[char\]92\) -ge 0'
        $validatorSource | Should Match '\$script:MvpBuildSetUnsafeRelativePathPattern\.IsMatch\(\$relativePath\)'
        $validatorSource | Should Not Match '\$normalizedRelativePath'
    }

    It 'derives traversal-relative paths while reusing directory metadata' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $snapshotStart = $moduleSource.IndexOf('function Get-MvpBuildSetSnapshotFilesNoFollow')
        $snapshotSource = $moduleSource.Substring(
            $snapshotStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles') - $snapshotStart)

        $snapshotSource | Should Match '\$rootPrefix = \$root \+ \[IO\.Path\]::DirectorySeparatorChar'
        $snapshotSource | Should Match '\$entryPath\.StartsWith\(\$rootPrefix, \[StringComparison\]::OrdinalIgnoreCase\)'
        $snapshotSource | Should Match '\$entryPath\.Substring\(\$rootPrefix\.Length\)'
        $snapshotSource | Should Match '\$pending = \[Collections\.Generic\.Stack\[IO\.DirectoryInfo\]\]::new\(\)'
        $snapshotSource | Should Match '\$pending\.Push\(\[IO\.DirectoryInfo\]::new\(\$root\)\)'
        $snapshotSource | Should Match '\$entryAttributes = \$entry\.Attributes'
        @([regex]::Matches($snapshotSource, '\$entry\.Attributes')).Count | Should Be 1
        $snapshotSource | Should Match '\$pending\.Push\(\[IO\.DirectoryInfo\]\$entry\)'
        $snapshotSource | Should Not Match '\[IO\.DirectoryInfo\]::new\(\$directory\)|Stack\[string\]'
        $snapshotSource | Should Not Match 'Get-MvpBuildSetRelativePath'
        $moduleSource | Should Not Match 'function Get-MvpBuildSetRelativePath'
    }

    It 'validates exact properties without redundant reverse-name lookups' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $assertionStart = $moduleSource.IndexOf('function Assert-MvpBuildSetExactProperties')
        $assertionSource = $moduleSource.Substring(
            $assertionStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetSnapshotFilesNoFollow') - $assertionStart)

        $assertionSource | Should Match '\$actualCount = 0'
        $assertionSource | Should Match '\$actualCount\+\+'
        $assertionSource | Should Match 'foreach \(\$property in \$Value\.PSObject\.Properties\)'
        $assertionSource | Should Match 'foreach \(\$name in \$ExpectedNames\)'
        $assertionSource | Should Match '\[string\]::Equals\(\$property\.Name, \$name, \[StringComparison\]::Ordinal\)'
        $assertionSource | Should Not Match '\$actualNames|HashSet\[string\]\]::new|\$Value\.PSObject\.Properties\[\$name\]'
        $assertionSource | Should Not Match 'Where-Object|ForEach-Object'
    }

    It 'reuses frozen manifest property names and validates file entries inline' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        $moduleSource | Should Match '\$script:MvpBuildSetManifestPropertyNames = \[string\[\]\]\s*@\('
        $validatorSource | Should Match '-ExpectedNames \$script:MvpBuildSetManifestPropertyNames'
        $validatorSource | Should Match '\$filePropertyCount = 0'
        $validatorSource | Should Match 'foreach \(\$property in \$file\.PSObject\.Properties\)'
        $validatorSource | Should Match '\$propertyName -cne ''relative_path'''
        $validatorSource | Should Match '\$propertyName -cne ''sha256'''
        $validatorSource | Should Match '\$propertyName -cne ''byte_length'''
        $validatorSource | Should Match '\$filePropertyCount -ne 3'
        $moduleSource | Should Not Match '\$script:MvpBuildSetFileEntryPropertyNames'
        $validatorSource | Should Not Match 'Assert-MvpBuildSetExactProperties\s+`\s+-Value \$file'
        $validatorSource | Should Not Match '-ExpectedNames @\('
    }

    It 'admits relative paths without allocating segment split pipelines' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        $moduleSource | Should Match '\$script:MvpBuildSetUnsafeRelativePathPattern = \[Text\.RegularExpressions\.Regex\]::new'
        foreach ($source in @($trackedSource, $validatorSource)) {
            $source | Should Match '\$script:MvpBuildSetUnsafeRelativePathPattern\.IsMatch\(\$relativePath\)'
            $source | Should Match '\$platformRelativePath = \$relativePath\.Replace'
            $source | Should Not Match '\.Split\(''\/''\)|Where-Object|Resolve-MvpBuildSetChildPath'
        }
        $validatorSource | Should Match '\$relativePath\.IndexOf\(\[char\]92\) -ge 0'
    }

    It 'compares sorted inventory without full expected projections or join strings' {
        $module = Get-Module -Name MvpBuildSet -ErrorAction Stop
        $matchingPaths = [Collections.Generic.List[string]]::new(
            [string[]]@('a.rs', 'b.rs'))
        $matchingActual = [Collections.Generic.List[string]]::new(
            [string[]]@('a.rs', 'b.rs'))
        & $module {
            param($Actual, $Expected)
            Assert-MvpBuildSetInventory `
                -ActualFiles $Actual `
                -ExpectedFiles $Expected
        } $matchingActual $matchingPaths
        $failure = $null
        try {
            $mismatchedPaths = [Collections.Generic.List[string]]::new(
                [string[]]@('a.rs', 'missing.rs'))
            $mismatchedActual = [Collections.Generic.List[string]]::new(
                [string[]]@('a.rs', 'extra.rs'))
            & $module {
                param($Actual, $Expected)
                Assert-MvpBuildSetInventory `
                    -ActualFiles $Actual `
                    -ExpectedFiles $Expected
            } $mismatchedActual $mismatchedPaths
        }
        catch {
            $failure = $_
        }
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $inventoryStart = $moduleSource.IndexOf('function Assert-MvpBuildSetInventory')
        $inventorySource = $moduleSource.Substring(
            $inventoryStart,
            $moduleSource.IndexOf('function New-MvpProductBuildSet') - $inventoryStart)
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'Unexpected: extra\.rs; missing: missing\.rs'
        $inventorySource | Should Match '\$inventoryMatches'
        $inventorySource | Should Match '\[Linq\.Enumerable\]::SequenceEqual\(\s*\$ActualFiles,\s*\$ExpectedFiles,\s*\[StringComparer\]::Ordinal\)'
        $inventorySource | Should Match '\[StringComparer\]::Ordinal\.Compare'
        $inventorySource | Should Not Match '\$ExpectedPaths|\$ManifestFiles|SetEquals|HashSet|-join \[char\]0|Where-Object|ForEach-Object|-notin'
        $validatorSource | Should Match '\$manifestPaths = \[Collections\.Generic\.List\[string\]\]::new\(\$files\.Count\)'
        $validatorSource | Should Match '\$manifestPaths\.Add\(\$relativePath\)'
        $validatorSource | Should Match '-ExpectedFiles \$manifestPaths'
        $validatorSource | Should Not Match '\$seen|\$seen\.Add|\[Collections\.Generic\.HashSet\[string\]\]::new'
    }

    It 'streams identity segments without temporary length arrays or direct transform calls' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $identityStart = $moduleSource.IndexOf('function Get-MvpBuildSetId')
        $identitySource = $moduleSource.Substring(
            $identityStart,
            $moduleSource.IndexOf('function Write-MvpBuildSetJson') - $identityStart)
        $publisherStart = $moduleSource.IndexOf('function New-MvpProductBuildSet')
        $publisherSource = $moduleSource.Substring(
            $publisherStart,
            $moduleSource.IndexOf('function Assert-MvpProductBuildSet') - $publisherStart)

        $identitySource | Should Match '\[Security\.Cryptography\.CryptoStream\]::new\(\s*\[IO\.Stream\]::Null'
        $identitySource | Should Match '\[IO\.BinaryWriter\]::new\(\$cryptoStream, \$encoding, \$true\)'
        $identitySource | Should Match '\$writer\.Write\(\[int64\]\$bytes\.LongLength\)'
        $identitySource | Should Match '\$writer\.Write\(\$bytes\)'
        $identitySource | Should Not Match 'BitConverter\]::GetBytes|TransformBlock|TransformFinalBlock'
        $identitySource | Should Match '\[BitConverter\]::ToString\(\$hasher\.Hash\)\.Replace\(''-'', ''''\)'
        $publisherSource | Should Match '\$overlayStream = \[IO\.File\]::OpenRead\(\$overlayPath\)'
        $publisherSource | Should Match '\$overlayHasData = \$overlayStream\.Length -gt 0'
        $publisherSource | Should Match '\$overlayHasher\.ComputeHash\(\$overlayStream\)'
        $publisherSource | Should Match '\$overlayStream\.Dispose\(\)'
        $publisherSource | Should Not Match '\[IO\.FileInfo\]::new\(\$overlayPath\)|Get-MvpBuildSetFileSha256'
        $moduleSource | Should Not Match '\$script:MvpBuildSetUpperHexDigits|function Get-MvpBuildSetBytesSha256|function ConvertTo-MvpBuildSetUpperHex|function Get-MvpBuildSetFileSha256'
    }

    It 'writes the three file identity segments without an inner vector loop' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $identityStart = $moduleSource.IndexOf('function Get-MvpBuildSetId')
        $identitySource = $moduleSource.Substring(
            $identityStart,
            $moduleSource.IndexOf('function Write-MvpBuildSetJson') - $identityStart)
        $fileLoopStart = $identitySource.IndexOf('foreach ($file in $Files)')
        $fileLoopSource = $identitySource.Substring(
            $fileLoopStart,
            $identitySource.IndexOf('$writer.Flush()') - $fileLoopStart)

        $fileLoopSource | Should Match '\$encoding\.GetBytes\(\[string\]\$file\.relative_path\)'
        $fileLoopSource | Should Match '\$encoding\.GetBytes\(\[string\]\$file\.sha256\)'
        $fileLoopSource | Should Match '\$encoding\.GetBytes\(\[string\]\[int64\]\$file\.byte_length\)'
        @([regex]::Matches($fileLoopSource, '\$writer\.Write\(\[int64\]\$bytes\.LongLength\)')).Count | Should Be 3
        $fileLoopSource | Should Not Match '\$segments|foreach \(\$segment'
    }

    It 'returns the sorted snapshot path list without an exact reference-array copy' {
        $module = Get-Module -Name MvpBuildSet -ErrorAction Stop
        $snapshotRoot = Join-Path $TestDrive 'snapshot-list-handoff'
        [IO.Directory]::CreateDirectory((Join-Path $snapshotRoot 'src')) | Out-Null
        [IO.File]::WriteAllText((Join-Path $snapshotRoot 'z.rs'), '')
        [IO.File]::WriteAllText((Join-Path $snapshotRoot 'src\a.rs'), '')

        $files = & $module {
            param($Root)
            Get-MvpBuildSetSnapshotFilesNoFollow -SnapshotRoot $Root
        } $snapshotRoot

        ($files -is [Collections.Generic.List[string]]) | Should Be $true
        $files.Count | Should Be 2
        $files[0] | Should Be 'src/a.rs'
        $files[1] | Should Be 'z.rs'
    }

    It 'keeps the sorted snapshot list typed through inventory validation' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $snapshotStart = $moduleSource.IndexOf('function Get-MvpBuildSetSnapshotFilesNoFollow')
        $snapshotSource = $moduleSource.Substring(
            $snapshotStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles') - $snapshotStart)
        $inventoryStart = $moduleSource.IndexOf('function Assert-MvpBuildSetInventory')
        $inventorySource = $moduleSource.Substring(
            $inventoryStart,
            $moduleSource.IndexOf('function New-MvpProductBuildSet') - $inventoryStart)
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        $snapshotSource | Should Match 'return ,\$files'
        $snapshotSource | Should Not Match '\.ToArray\(\)'
        $inventorySource | Should Match '\[Collections\.Generic\.List\[string\]\]\$ActualFiles'
        $validatorSource | Should Match '\[Collections\.Generic\.List\[string\]\]\$actualFiles =\s*Get-MvpBuildSetSnapshotFilesNoFollow'
        $validatorSource | Should Not Match '\$actualFiles = @\(Get-MvpBuildSetSnapshotFilesNoFollow'
    }

    It 'returns tracked file descriptors through one typed list into publication' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)
        $publisherStart = $moduleSource.IndexOf('function New-MvpProductBuildSet')
        $publisherSource = $moduleSource.Substring(
            $publisherStart,
            $moduleSource.IndexOf('function Assert-MvpProductBuildSet') - $publisherStart)

        $trackedSource | Should Match '\$entries = \$script:MvpBuildSetUtf8\.GetString\('
        $trackedSource | Should Match '\.Split\(\s*\$script:MvpBuildSetNulSeparator,\s*\[StringSplitOptions\]::RemoveEmptyEntries\)'
        $trackedSource | Should Match '\$paths = \[Collections\.Generic\.List\[string\]\]::new\(\$entries\.Length\)'
        $trackedSource | Should Match 'foreach \(\$entry in \$entries\)'
        $trackedSource | Should Match '\$pathBuffer = \$null'
        $trackedSource | Should Match '\$pathCapture = \$null'
        $trackedSource | Should Match '\$entries = \$null'
        $trackedSource | Should Match '\$files = \[Collections\.Generic\.List\[object\]\]::new\(\$paths\.Count\)'
        $trackedSource | Should Match 'return ,\$files'
        $trackedSource | Should Not Match 'return @\(\$files\)|\.Split\(\[char\]0\)|\$entry\.Length -eq 0'
        $publisherSource | Should Match '\[Collections\.Generic\.List\[object\]\]\$files = Get-MvpBuildSetTrackedFiles'
        $publisherSource | Should Match '(?m)^\s+files = \$files$'
        $publisherSource | Should Not Match 'files = @\(\$files\)'
    }

    It 'hashes typed tracked lists and parsed object arrays through one enumerable contract' {
        $module = Get-Module -Name MvpBuildSet -ErrorAction Stop
        $entries = @(
            [pscustomobject]@{ relative_path = 'Cargo.toml'; sha256 = ('A' * 64); byte_length = 31 },
            [pscustomobject]@{ relative_path = 'src/lib.rs'; sha256 = ('B' * 64); byte_length = 47 }
        )
        $typedEntries = [Collections.Generic.List[object]]::new([object[]]$entries)
        $arrayIdentity = & $module {
            param($Files)
            Get-MvpBuildSetId `
                -GitRevision ('a' * 40) `
                -DirtyOverlaySha256 ('C' * 64) `
                -Files $Files
        } $entries
        $listIdentity = & $module {
            param($Files)
            Get-MvpBuildSetId `
                -GitRevision ('a' * 40) `
                -DirtyOverlaySha256 ('C' * 64) `
                -Files $Files
        } $typedEntries
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $identityStart = $moduleSource.IndexOf('function Get-MvpBuildSetId')
        $identitySource = $moduleSource.Substring(
            $identityStart,
            $moduleSource.IndexOf('function Write-MvpBuildSetJson') - $identityStart)

        $listIdentity | Should Be $arrayIdentity
        $identitySource | Should Match '\[Collections\.Generic\.IEnumerable\[object\]\]\$Files'
        $identitySource | Should Not Match '\[object\[\]\]\$Files'
    }

    It 'binds the parsed manifest file array without a wrapper copy' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        $validatorSource | Should Match '\[object\[\]\]\$files = \$manifest\.files'
        $validatorSource | Should Not Match '\$files = @\(\$manifest\.files\)'
    }

    It 'reuses the verified manifest file array in the returned receipt' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        $validatorSource | Should Match '(?m)^\s+files = \$files$'
        $validatorSource | Should Not Match 'files = @\(\$files\)'
    }

    It 'caches manifest file identity fields across validation checks' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        $validatorSource | Should Match '\$expectedSha256 = \[string\]\$file\.sha256'
        $validatorSource | Should Match '\$expectedByteLength = \$file\.byte_length'
        $validatorSource | Should Match '\$expectedSha256 -notmatch'
        $validatorSource | Should Match '\[int64\]\$expectedByteLength -ne \[int64\]\$item\.Length'
        $validatorSource | Should Match '\$expectedSha256 -ne \$actualSha256'
        @([regex]::Matches($validatorSource, '\$file\.sha256')).Count | Should Be 1
        @([regex]::Matches($validatorSource, '\$file\.byte_length')).Count | Should Be 1
    }

    It 'checks LFS materialization and hashes each tracked file through one stream' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)

        @([regex]::Matches($trackedSource, '\[byte\[\]\]::new\(128\)')).Count | Should Be 1
        $trackedSource | Should Match '\$contentStream = \$item\.OpenRead\(\)'
        $trackedSource | Should Match '\$contentStream\.Read\(\s*\$materializedFilePrefixBuffer,\s*0,\s*\$materializedFilePrefixBufferLength\)'
        $trackedSource | Should Match '\$contentStream\.Position = 0'
        $trackedSource | Should Match '\$contentHasher\.ComputeHash\(\$contentStream\)'
        $trackedSource | Should Not Match '\[Math\]::Min\(|\$contentStream\.Length'
        $trackedSource | Should Not Match 'Assert-MvpBuildSetMaterializedFile|Get-MvpBuildSetFileSha256 -Path \$path'
        $moduleSource | Should Not Match 'function Assert-MvpBuildSetMaterializedFile'
    }

    It 'reuses LFS prefix length and ASCII decoding across tracked files' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)
        $fileLoopStart = $trackedSource.IndexOf('foreach ($relativePath in $paths)')
        $fileLoopSource = $trackedSource.Substring($fileLoopStart)

        $trackedSource | Should Match '\$materializedFilePrefixBufferLength = \[int\]\$materializedFilePrefixBuffer\.Length'
        $trackedSource | Should Match '\$materializedFilePrefixEncoding = \[Text\.Encoding\]::ASCII'
        $fileLoopSource | Should Match '\$materializedFilePrefixEncoding\.GetString\('
        $fileLoopSource | Should Not Match '\[Text\.Encoding\]::ASCII|\.LongLength'
    }

    It 'guards LFS text decoding with one cached prefix byte' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)
        $fileLoopStart = $trackedSource.IndexOf('foreach ($relativePath in $paths)')
        $fileLoopSource = $trackedSource.Substring($fileLoopStart)

        $trackedSource | Should Match '\$materializedFileLfsFirstByte = \[byte\]118'
        $fileLoopSource | Should Match 'if \(\$read -gt 0 -and\s*\$materializedFilePrefixBuffer\[0\] -eq \$materializedFileLfsFirstByte\) \{\s*\$text = \$materializedFilePrefixEncoding\.GetString\('
    }

    It 'reads tracked and verified file metadata from one FileInfo instance per path' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        foreach ($source in @($trackedSource, $validatorSource)) {
            $source | Should Match '\$item = \[IO\.FileInfo\]::new\(\$path\)'
            $source | Should Match '\$item\.Exists'
            $source | Should Match '\$contentStream = \$item\.OpenRead\(\)'
            $source | Should Not Match '\[IO\.File\]::Exists\(\$path\)|\[IO\.File\]::OpenRead\(\$path\)|Get-Item -LiteralPath \$path'
        }
    }

    It 'hashes tracked and verified file batches inline with one SHA256 instance' {
        $moduleSource = Get-Content -LiteralPath $buildSetModule -Raw
        $trackedStart = $moduleSource.IndexOf('function Get-MvpBuildSetTrackedFiles')
        $trackedSource = $moduleSource.Substring(
            $trackedStart,
            $moduleSource.IndexOf('function Get-MvpBuildSetId') - $trackedStart)
        $validatorStart = $moduleSource.IndexOf('function Assert-MvpProductBuildSet')
        $validatorSource = $moduleSource.Substring(
            $validatorStart,
            $moduleSource.IndexOf('Export-ModuleMember') - $validatorStart)

        foreach ($source in @($trackedSource, $validatorSource)) {
            @([regex]::Matches($source, '\[Security\.Cryptography\.SHA256\]::Create\(\)')).Count | Should Be 1
            $source | Should Match '\$contentHasher\.ComputeHash\(\$contentStream\)'
            $source | Should Match '\[BitConverter\]::ToString\(\s*\$contentHasher\.ComputeHash\(\$contentStream\)\)\.Replace\(''-'', ''''\)'
            $source | Should Match '\$contentHasher\.Dispose\(\)'
            $source | Should Not Match 'Get-MvpBuildSetFileSha256 -Path \$path|ConvertTo-MvpBuildSetUpperHex'
        }
    }

}
