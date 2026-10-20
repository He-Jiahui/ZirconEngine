$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$sourcePathResolver = Join-Path $repoRoot 'tools\maintenance\WindowsPathResolver.psm1'
$fixturePython = [IO.Path]::GetFullPath((Join-Path $repoRoot '.jenkins\runtime\python\python.exe'))
$fixturePowerShell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
Import-Module (Join-Path $repoRoot 'tools\mvp\MvpTestFixturePaths.psm1') -Force -ErrorAction Stop

function New-EditorBuildFixtureRoot {
    return New-MvpTestFixtureRoot -Prefix 'editor-build'
}

function Remove-EditorBuildFixtureRoot {
    param([Parameter(Mandatory)][string]$FixtureRoot)
    Remove-MvpTestFixtureRoot -Path $FixtureRoot
}

function Write-EditorFixtureText {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Value)
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Path)) | Out-Null
    [IO.File]::WriteAllText($Path, $Value, [Text.UTF8Encoding]::new($false))
}

function Invoke-EditorFixturePython {
    param([Parameter(Mandatory)][string[]]$Arguments)
    if (-not [IO.File]::Exists($fixturePython)) { throw "Managed fixture Python is missing: $fixturePython" }
    $previousPreference = $ErrorActionPreference
    Push-Location -LiteralPath $repoRoot
    try {
        # Preserve real native stderr and its exit code on Windows PowerShell 5.1.
        $ErrorActionPreference = 'Continue'
        $output = @(& $fixturePython -B -X utf8 @Arguments 2>&1)
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previousPreference
        Pop-Location
    }
    return @{ ExitCode = $exitCode; Output = @($output | ForEach-Object { [string]$_ }) }
}

function Invoke-EditorBuildFixture {
    param(
        [string]$OutputDirectory,
        [string]$TargetDir,
        [ValidateSet('reuse', 'compact', 'diagnostic')][string]$StorageMode = 'reuse',
        [ValidateSet('development', 'shipping')][string]$CargoProfile = 'development',
        [string]$SourceSnapshot,
        [string]$SourceSnapshotDigest,
        [switch]$Ephemeral
    )
    if ([string]::IsNullOrWhiteSpace($TargetDir)) { $TargetDir = Join-Path $fixtureRoot 'target' }
    $arguments = @(
        '-m', 'tools.build.build_editor', '--repo-root', $fixtureSource,
        '--target-dir', $TargetDir, '--storage-mode', $StorageMode,
        '--cargo-profile', $CargoProfile, '--jobs', '2',
        '--crt-directory', $fixtureCrtDirectory, '--skip-smoke-test'
    )
    if (-not [string]::IsNullOrWhiteSpace($OutputDirectory)) { $arguments += @('--output-directory', $OutputDirectory) }
    if (-not [string]::IsNullOrWhiteSpace($SourceSnapshot)) { $arguments += @('--source-snapshot', $SourceSnapshot) }
    if (-not [string]::IsNullOrWhiteSpace($SourceSnapshotDigest)) { $arguments += @('--source-snapshot-digest', $SourceSnapshotDigest) }
    if ($Ephemeral) { $arguments += '--ephemeral' }
    $result = Invoke-EditorFixturePython -Arguments $arguments
    if ($result.ExitCode -eq 0) { $result['Summary'] = ConvertFrom-Json -InputObject $result.Output[-1] -ErrorAction Stop }
    return $result
}

function Get-EditorFixtureCalls {
    if ([IO.File]::Exists($callLog)) {
        foreach ($line in [IO.File]::ReadAllLines($callLog)) { ConvertFrom-Json -InputObject $line -ErrorAction Stop }
    }
}

function Get-EditorFixtureReceipt {
    param([Parameter(Mandatory)]$Result)
    $path = Join-Path $Result.Summary.evidence_directory 'build-receipt.json'
    return ConvertFrom-Json -InputObject ([IO.File]::ReadAllText($path)) -ErrorAction Stop
}

function Assert-EditorFixtureCommands {
    param(
        [Parameter(Mandatory)][object[]]$Calls,
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Target,
        [ValidateSet('development', 'shipping')][string]$CargoProfile = 'development'
    )
    $Calls.Count | Should Be 2
    $feature = if ($CargoProfile -eq 'shipping') { 'shipping-editor' } else { 'target-editor-host' }
    $common = "--locked|--no-default-features|--features|$feature|--jobs|2|--message-format=json-render-diagnostics"
    if ($CargoProfile -eq 'shipping') { $common += '|--profile|shipping|--target|x86_64-pc-windows-msvc' }
    ($Calls[0].cargo_argv -join '|') | Should Be "build|-p|zircon_app|--bin|zircon_editor|$common"
    ($Calls[1].cargo_argv -join '|') | Should Be "rustc|-p|zircon_runtime|--lib|--crate-type|cdylib|$common"
    foreach ($call in $Calls) {
        $call.repo_root | Should Be $Source
        $call.target_dir | Should Be $Target
        $call.python_interpreter | Should Be $fixturePython
        $call.independent_preview | Should Be $true
    }
}

function Get-EditorFixtureCrtDirectory {
    $visualStudio = if ([string]::IsNullOrWhiteSpace($env:VSINSTALLDIR)) { 'E:\Visual Studio' } else { $env:VSINSTALLDIR }
    $redist = Join-Path $visualStudio 'VC\Redist\MSVC'
    $candidates = @(Get-ChildItem -LiteralPath $redist -Directory -ErrorAction Stop | Sort-Object Name | ForEach-Object {
        $candidate = Join-Path $_.FullName 'x64\Microsoft.VC143.CRT'
        if ([IO.Directory]::Exists($candidate)) { $candidate }
    })
    if ($candidates.Count -eq 0) { throw "MSVC x64 CRT redist is missing: $redist" }
    $selected = $candidates[-1]
    foreach ($name in @('msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')) {
        if (-not [IO.File]::Exists((Join-Path $selected $name))) { throw "MSVC CRT input is missing: $selected\$name" }
    }
    # Production copy_crt performs PE and Microsoft signature verification.
    return $selected
}

function New-EditorFixtureSource {
    param([Parameter(Mandatory)][string]$SourceRoot, [string]$Tag = 'fixture')
    $manifest = @('[workspace]', 'members = []', '') -join [Environment]::NewLine
    Write-EditorFixtureText -Path (Join-Path $SourceRoot 'Cargo.toml') -Value $manifest
    # This is a synthetic pre-activation checkout, not the canonical checkout's gate.
    Write-EditorFixtureText -Path (Join-Path $SourceRoot '.jenkins\state\entry-inventory.json') -Value '{"activation":{"soleEntryEnforcementApplied":false}}'
    $fonts = Join-Path $SourceRoot 'zircon_runtime\assets\fonts'
    Write-EditorFixtureText -Path (Join-Path $fonts 'fixture.txt') -Value "$Tag asset"
    foreach ($font in @('default', 'editor-ui')) {
        $descriptor = 'source = "{0}.ttc"{1}family = "Synthetic"{1}' -f $font, [Environment]::NewLine
        Write-EditorFixtureText -Path (Join-Path $fonts "$font.font.toml") -Value $descriptor
        Write-EditorFixtureText -Path (Join-Path $fonts "$font.ttc") -Value "$Tag packaged font"
    }
    Write-EditorFixtureText -Path (Join-Path $fonts 'editor-ui-sources\source.ttf') -Value "$Tag source font"
    Write-EditorFixtureText -Path (Join-Path $SourceRoot 'zircon_editor\assets\icons\fixture.svg') -Value "<svg>$Tag</svg>"
    $schemaDirectory = Join-Path $SourceRoot 'zircon_runtime_interface\src\runtime_build_set'
    [IO.Directory]::CreateDirectory($schemaDirectory) | Out-Null
    foreach ($name in @('interface_spec_v1.json', 'payload_schema_set_v1.json')) {
        Copy-Item -LiteralPath (Join-Path $repoRoot "zircon_runtime_interface\src\runtime_build_set\$name") -Destination $schemaDirectory
    }
    $profileDirectory = Join-Path $SourceRoot 'tools\export'
    [IO.Directory]::CreateDirectory($profileDirectory) | Out-Null
    Copy-Item -LiteralPath (Join-Path $repoRoot 'tools\export\product_build_profiles.toml') -Destination $profileDirectory
}

function Write-EditorFixtureExecutor {
    param([Parameter(Mandatory)][string]$Path)
    # Only the local-cargo process boundary is synthetic. The canonical Python
    # main must build its commands, read these Cargo records, and package itself.
    $executor = @'
$ErrorActionPreference = 'Stop'
$received = @($args | ForEach-Object { [string]$_ })
if ($received.Count -lt 8 -or $received[0] -ne '-RepoRoot' -or
    $received[2] -ne '-TargetDir' -or $received[4] -ne '-PythonInterpreter' -or
    $received[6] -ne '-IndependentPreview') { throw 'Unexpected local-cargo invocation prefix.' }
$source = $received[1]
$target = $received[3]
$python = $received[5]
$cargo = @($received[7..($received.Count - 1)])
$packageIndex = [Array]::IndexOf($cargo, '-p')
if ($packageIndex -lt 0 -or $packageIndex + 1 -ge $cargo.Count) { throw 'Cargo package is missing.' }
$package = $cargo[$packageIndex + 1]
$record = [ordered]@{
    argv = $received; repo_root = $source; target_dir = $target
    python_interpreter = $python; independent_preview = $true; cargo_argv = $cargo
}
[IO.File]::AppendAllText($env:BUILD_EDITOR_TEST_LOG, (($record | ConvertTo-Json -Depth 6 -Compress) + [Environment]::NewLine), [Text.UTF8Encoding]::new($false))
if ($env:BUILD_EDITOR_TEST_FAIL_PACKAGE -eq $package) {
    Write-Output 'fixture runtime failure'
    exit 17
}
$name = if ($package -eq 'zircon_app' -and $cargo[0] -eq 'build') { 'zircon_editor.exe' }
    elseif ($package -eq 'zircon_runtime' -and $cargo[0] -eq 'rustc') { 'zircon_runtime.dll' }
    else { throw "Unexpected fixture Cargo package: $package" }
$cargoProfile = if ($cargo -contains '--profile') { 'shipping' } else { 'debug' }
$artifactDirectory = $target
if ($cargo -contains '--target') { $artifactDirectory = Join-Path $artifactDirectory 'x86_64-pc-windows-msvc' }
$artifactDirectory = Join-Path $artifactDirectory $cargoProfile
[IO.Directory]::CreateDirectory($artifactDirectory) | Out-Null
$artifact = Join-Path $artifactDirectory $name
$pe = New-Object byte[] 128
$pe[0] = 0x4d; $pe[1] = 0x5a
[BitConverter]::GetBytes([int]64).CopyTo($pe, 60)
$pe[64] = 0x50; $pe[65] = 0x45
[BitConverter]::GetBytes([UInt16]0x8664).CopyTo($pe, 68)
$characteristics = if ($name.EndsWith('.dll')) { [UInt16]0x2002 } else { [UInt16]0x0002 }
[BitConverter]::GetBytes($characteristics).CopyTo($pe, 86)
[IO.File]::WriteAllBytes($artifact, $pe)
@{ reason = 'compiler-artifact'; filenames = @($artifact) } | ConvertTo-Json -Compress
@{ reason = 'build-finished'; success = $true } | ConvertTo-Json -Compress
exit 0
'@
    Write-EditorFixtureText -Path $Path -Value $executor
}

function New-EditorOutsideFixtureDirectory {
    $ownedRoot = New-MvpTestFixtureRoot -Prefix 'editor-build-outside'
    $fixtureRoots.Add($ownedRoot)
    $directory = Join-Path $ownedRoot 'external-target'
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    return $directory
}

Describe 'Editor build Python owner and Windows publication contracts' {
    BeforeEach {
        $savedFixtureEnvironment = @{}
        foreach ($name in @('BUILD_EDITOR_TEST_LOG', 'BUILD_EDITOR_TEST_FAIL_PACKAGE')) {
            $savedFixtureEnvironment[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
        }
        $fixtureRoots = [Collections.Generic.List[string]]::new()
        $ownedFixtureRoot = New-EditorBuildFixtureRoot
        $fixtureRoots.Add($ownedFixtureRoot)
        # Keep publication inside an ordinary child of the instance held with DELETE access.
        $fixtureRoot = Join-Path $ownedFixtureRoot 'workspace'
        [IO.Directory]::CreateDirectory($fixtureRoot) | Out-Null
        $fixtureSource = Join-Path $fixtureRoot 'repo'
        $fixtureAssets = Join-Path $fixtureSource 'zircon_runtime\assets\fonts'
        $fixtureRuntimeBuildSet = Join-Path $fixtureSource 'zircon_runtime_interface\src\runtime_build_set'
        $callLog = Join-Path $fixtureRoot 'cargo-calls.jsonl'
        $fixtureCrtDirectory = Get-EditorFixtureCrtDirectory
        New-EditorFixtureSource -SourceRoot $fixtureSource
        Write-EditorFixtureExecutor -Path (Join-Path $fixtureSource 'tools\dev\local-cargo.ps1')
        $env:BUILD_EDITOR_TEST_LOG = $callLog
        $env:BUILD_EDITOR_TEST_FAIL_PACKAGE = $null
    }

    AfterEach {
        foreach ($name in @('BUILD_EDITOR_TEST_LOG', 'BUILD_EDITOR_TEST_FAIL_PACKAGE')) {
            [Environment]::SetEnvironmentVariable($name, $savedFixtureEnvironment[$name], 'Process')
        }
        # Remove the primary tree, including junctions, before their owned targets.
        foreach ($ownedRoot in $fixtureRoots) { Remove-EditorBuildFixtureRoot -FixtureRoot $ownedRoot }
    }

    It 'publishes matching editor, runtime, and asset identities after both real owner build calls succeed' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $result = Invoke-EditorBuildFixture -OutputDirectory $bundle
        if ($result.ExitCode -ne 0) { throw ($result.Output -join [Environment]::NewLine) }
        $result.ExitCode | Should Be 0
        $calls = @(Get-EditorFixtureCalls)
        Assert-EditorFixtureCommands -Calls $calls -Source $fixtureSource -Target (Join-Path $fixtureRoot 'target')
        $manifest = ConvertFrom-Json -InputObject ([IO.File]::ReadAllText((Join-Path $bundle 'zircon_runtime.dll.manifest.json')))
        $manifest.artifact.file_name | Should Be 'zircon_runtime.dll'
        $manifest.artifact.sha256 | Should Be (Get-FileHash -LiteralPath (Join-Path $bundle 'zircon_runtime.dll') -Algorithm SHA256).Hash.ToLowerInvariant()
        @($manifest.host_artifacts).Count | Should Be 1
        $manifest.host_artifacts[0].file_name | Should Be 'zircon_editor.exe'
        $manifest.host_artifacts[0].sha256 | Should Be (Get-FileHash -LiteralPath (Join-Path $bundle 'zircon_editor.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
        [IO.File]::Exists((Join-Path $bundle 'assets\fonts\default.ttc')) | Should Be $true
        [IO.File]::Exists((Join-Path $bundle 'assets\fonts\editor-ui-sources\source.ttf')) | Should Be $true
        [IO.File]::Exists((Join-Path $bundle 'assets\icons\fixture.svg')) | Should Be $true
        foreach ($name in @('msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')) {
            (Get-FileHash -LiteralPath (Join-Path $bundle $name) -Algorithm SHA256).Hash | Should Be (Get-FileHash -LiteralPath (Join-Path $fixtureCrtDirectory $name) -Algorithm SHA256).Hash
        }
        $receipt = Get-EditorFixtureReceipt -Result $result
        $receipt.status | Should Be 'preview_built'
        $receipt.build_set_id | Should Be $manifest.build_set_id
        $receipt.output_closure.digest | Should Be $result.Summary.bundle_digest
        $receipt.completed_publication | Should Be $true
        $receipt.full_build_acceptance | Should Be $false
        $receipt.native_ui_acceptance | Should Be $false
        [IO.File]::Exists((Join-Path $bundle 'build-receipt.json')) | Should Be $false
        [IO.Directory]::Exists((Join-Path $bundle 'temporary')) | Should Be $false
        @(Get-ChildItem -LiteralPath $fixtureRoot -Directory -Filter 'editor-bundle.staging-*').Count | Should Be 0
    }

    It 'builds shipping editor and runtime with matching BuildSet, essential assets, ZIP, and size evidence' {
        $bundle = Join-Path $fixtureRoot 'editor-shipping-bundle'
        $result = Invoke-EditorBuildFixture -OutputDirectory $bundle -CargoProfile shipping
        if ($result.ExitCode -ne 0) { throw ($result.Output -join [Environment]::NewLine) }
        $result.ExitCode | Should Be 0
        Assert-EditorFixtureCommands -Calls @(Get-EditorFixtureCalls) -Source $fixtureSource -Target (Join-Path $fixtureRoot 'target') -CargoProfile shipping
        $manifest = ConvertFrom-Json -InputObject ([IO.File]::ReadAllText((Join-Path $bundle 'zircon_runtime.dll.manifest.json')))
        $manifest.build_mode | Should Be 'release'
        @($manifest.runtime_features).Count | Should Be 1
        $manifest.runtime_features[0] | Should Be 'shipping-editor'
        $manifest.target.target_triple | Should Be 'x86_64-pc-windows-msvc'
        $manifest.artifact.sha256 | Should Be (Get-FileHash -LiteralPath (Join-Path $bundle 'zircon_runtime.dll') -Algorithm SHA256).Hash.ToLowerInvariant()
        $manifest.host_artifacts[0].sha256 | Should Be (Get-FileHash -LiteralPath (Join-Path $bundle 'zircon_editor.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
        [IO.File]::Exists((Join-Path $bundle 'assets\fonts\editor-ui.ttc')) | Should Be $true
        [IO.Directory]::Exists((Join-Path $bundle 'assets\fonts\editor-ui-sources')) | Should Be $false
        [IO.File]::Exists((Join-Path $bundle 'assets\icons\fixture.svg')) | Should Be $true
        $receipt = Get-EditorFixtureReceipt -Result $result
        $receipt.status | Should Be 'shipping_packaged'
        $receipt.archive.path | Should Be "$bundle.zip"
        $receipt.archive.sha256 | Should Be (Get-FileHash -LiteralPath "$bundle.zip" -Algorithm SHA256).Hash.ToLowerInvariant()
        $receipt.product_size_report_sha256 | Should Be (Get-FileHash -LiteralPath $receipt.product_size_report -Algorithm SHA256).Hash.ToLowerInvariant()
        $receipt.build_set_id | Should Be $manifest.build_set_id
        $receipt.shipping_acceptance | Should Be $false
    }

    It 'uses one actual sealed snapshot digest for both builds, assets, and Runtime BuildSet contracts' {
        $bundle = Join-Path $fixtureRoot 'snapshot-editor-bundle'
        $snapshot = Join-Path $fixtureRoot 'sealed\source'
        New-EditorFixtureSource -SourceRoot $snapshot -Tag snapshot
        [IO.File]::Delete((Join-Path $fixtureRuntimeBuildSet 'interface_spec_v1.json'))
        $digestScript = 'import sys; from pathlib import Path; from tools.build import build_editor as owner; print(owner.inventory_digest(owner.source_inventory(Path(sys.argv[1]))))'
        $digestResult = Invoke-EditorFixturePython -Arguments @('-c', $digestScript, $snapshot)
        $digestResult.ExitCode | Should Be 0
        $digest = $digestResult.Output[-1].Trim()
        $digest | Should Match '^[0-9a-f]{64}$'
        $result = Invoke-EditorBuildFixture -OutputDirectory $bundle -SourceSnapshot $snapshot -SourceSnapshotDigest $digest
        if ($result.ExitCode -ne 0) { throw ($result.Output -join [Environment]::NewLine) }
        Assert-EditorFixtureCommands -Calls @(Get-EditorFixtureCalls) -Source $snapshot -Target (Join-Path $fixtureRoot 'target')
        [IO.File]::ReadAllText((Join-Path $bundle 'assets\fonts\fixture.txt')) | Should Be 'snapshot asset'
        [IO.File]::ReadAllText((Join-Path $bundle 'assets\icons\fixture.svg')) | Should Be '<svg>snapshot</svg>'
        $receipt = Get-EditorFixtureReceipt -Result $result
        $receipt.source_digest | Should Be $digest
        $sourceHash = (Get-FileHash -LiteralPath (Join-Path $snapshot 'zircon_runtime_interface\src\runtime_build_set\payload_schema_set_v1.json') -Algorithm SHA256).Hash.ToLowerInvariant()
        $manifest = ConvertFrom-Json -InputObject ([IO.File]::ReadAllText((Join-Path $bundle 'zircon_runtime.dll.manifest.json')))
        $manifest.payload_schema_digest | Should Be $sourceHash
        $receipt.source_inventory.'source/zircon_runtime_interface/src/runtime_build_set/payload_schema_set_v1.json' | Should Be $sourceHash
    }

    It 'forwards one explicit physical target directory to both package builds' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $target = Join-Path $fixtureRoot 'explicit-target'
        $result = Invoke-EditorBuildFixture -OutputDirectory $bundle -TargetDir $target
        if ($result.ExitCode -ne 0) {
            $result.Output | ForEach-Object { Write-Host $_ }
        }
        $result.ExitCode | Should Be 0
        Assert-EditorFixtureCommands -Calls @(Get-EditorFixtureCalls) -Source $fixtureSource -Target $target
    }

    It 'selects one owned ephemeral target for both package builds' {
        $target = Join-Path $fixtureRoot 'requested-target'
        $result = Invoke-EditorBuildFixture -OutputDirectory (Join-Path $fixtureRoot 'editor-bundle') -TargetDir $target -Ephemeral
        if ($result.ExitCode -ne 0) {
            $result.Output | ForEach-Object { Write-Host $_ }
        }
        $result.ExitCode | Should Be 0
        $calls = @(Get-EditorFixtureCalls)
        $actual = [string]$calls[0].target_dir
        $actual | Should Match ('^' + [regex]::Escape($fixtureRoot) + '\\ephemeral-[0-9a-f]{32}\\target$')
        Assert-EditorFixtureCommands -Calls $calls -Source $fixtureSource -Target $actual
        [IO.Directory]::Exists($target) | Should Be $false
    }

    It 'rejects unsupported diagnostic and compact storage modes before invoking the executor' {
        foreach ($mode in @('diagnostic', 'compact')) {
            $bundle = Join-Path $fixtureRoot "editor-$mode"
            $result = Invoke-EditorBuildFixture -OutputDirectory $bundle -StorageMode $mode
            $result.ExitCode | Should Be 2
            ($result.Output -join [Environment]::NewLine) | Should Match 'no supported executor'
            [IO.Directory]::Exists($bundle) | Should Be $false
            [IO.File]::Exists($callLog) | Should Be $false
        }
    }

    It 'preserves owned staging and failure evidence without publishing when the runtime executor exits 17' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $sentinel = Join-Path $fixtureRoot 'unrelated.txt'
        Write-EditorFixtureText -Path $sentinel -Value 'keep me'
        $env:BUILD_EDITOR_TEST_FAIL_PACKAGE = 'zircon_runtime'
        $result = Invoke-EditorBuildFixture -OutputDirectory $bundle
        $result.ExitCode | Should Be 2
        ($result.Output -join [Environment]::NewLine) | Should Match 'Cargo failed \(17\)'
        [IO.Directory]::Exists($bundle) | Should Be $false
        Assert-EditorFixtureCommands -Calls @(Get-EditorFixtureCalls) -Source $fixtureSource -Target (Join-Path $fixtureRoot 'target')
        @(Get-ChildItem -LiteralPath $fixtureRoot -Directory -Filter 'editor-bundle.staging-*').Count | Should Be 1
        $evidence = @(Get-ChildItem -LiteralPath (Join-Path $fixtureRoot 'evidence') -Directory)
        $evidence.Count | Should Be 1
        [IO.File]::ReadAllText((Join-Path $evidence[0].FullName 'cargo-1.log')) | Should Match 'fixture runtime failure'
        [IO.File]::Exists((Join-Path $evidence[0].FullName 'build-receipt.json')) | Should Be $false
        [IO.File]::ReadAllText($sentinel) | Should Be 'keep me'
    }

    It 'does not publish when real Runtime BuildSet generation fails on a missing InterfaceSpec' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        [IO.File]::Delete((Join-Path $fixtureRuntimeBuildSet 'interface_spec_v1.json'))
        $result = Invoke-EditorBuildFixture -OutputDirectory $bundle
        # Runtime manifest SystemExit is the actual CLI exit, not main's ValueError exit 2.
        $result.ExitCode | Should Be 1
        ($result.Output -join [Environment]::NewLine) | Should Match 'Cannot read Runtime InterfaceSpec'
        [IO.Directory]::Exists($bundle) | Should Be $false
        @(Get-EditorFixtureCalls).Count | Should Be 2
        @(Get-ChildItem -LiteralPath $fixtureRoot -Directory -Filter 'editor-bundle.staging-*').Count | Should Be 1
    }

    It 'rejects reparse-point source assets before execution without touching their owned external target' {
        $outside = New-EditorOutsideFixtureDirectory
        $sentinel = Join-Path $outside 'sentinel.txt'
        Write-EditorFixtureText -Path $sentinel -Value 'outside'
        [IO.Directory]::Delete($fixtureAssets, $true)
        New-Item -ItemType Junction -Path $fixtureAssets -Target $outside | Out-Null
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        $result = Invoke-EditorBuildFixture -OutputDirectory $bundle
        $result.ExitCode | Should Be 2
        ($result.Output -join [Environment]::NewLine) | Should Match 'Reparse path rejected'
        [IO.Directory]::Exists($bundle) | Should Be $false
        [IO.File]::Exists($callLog) | Should Be $false
        [IO.File]::ReadAllText($sentinel) | Should Be 'outside'
    }

    It 'does not overwrite an existing output directory' {
        $bundle = Join-Path $fixtureRoot 'editor-bundle'
        Write-EditorFixtureText -Path (Join-Path $bundle 'sentinel.txt') -Value 'keep me'
        $result = Invoke-EditorBuildFixture -OutputDirectory $bundle
        $result.ExitCode | Should Be 2
        [IO.File]::ReadAllText((Join-Path $bundle 'sentinel.txt')) | Should Be 'keep me'
        [IO.File]::Exists($callLog) | Should Be $false
    }

    It 'requires the bundle parent to exist before invoking the executor' {
        $parent = Join-Path $fixtureRoot 'missing-parent'
        $result = Invoke-EditorBuildFixture -OutputDirectory (Join-Path $parent 'editor-bundle')
        $result.ExitCode | Should Be 2
        ($result.Output -join [Environment]::NewLine) | Should Match 'Output parent must exist'
        [IO.Directory]::Exists($parent) | Should Be $false
        [IO.File]::Exists($callLog) | Should Be $false
    }

    It 'rejects a relative output before invoking the executor' {
        $result = Invoke-EditorBuildFixture -OutputDirectory 'zircon-local\relative-editor-bundle'
        $result.ExitCode | Should Be 2
        ($result.Output -join [Environment]::NewLine) | Should Match 'approved physical build root'
        [IO.File]::Exists($callLog) | Should Be $false
    }

    It 'rejects a C drive output before invoking the executor' {
        $result = Invoke-EditorBuildFixture -OutputDirectory 'C:\cargo-targets\editor-bundle'
        $result.ExitCode | Should Be 2
        ($result.Output -join [Environment]::NewLine) | Should Match 'approved physical build root'
        [IO.File]::Exists($callLog) | Should Be $false
    }

    It 'rejects a drive-relative output before invoking the executor' {
        $result = Invoke-EditorBuildFixture -OutputDirectory 'C:editor-bundle'
        $result.ExitCode | Should Be 2
        ($result.Output -join [Environment]::NewLine) | Should Match 'approved physical build root'
        [IO.File]::Exists($callLog) | Should Be $false
    }

    It 'enforces the real Python publication closure, fresh sibling destination, and ordinary path contract' {
        $outside = New-EditorOutsideFixtureDirectory
        Write-EditorFixtureText -Path (Join-Path $outside 'sentinel.txt') -Value 'outside'
        $alias = Join-Path $fixtureRoot 'publication-alias'
        New-Item -ItemType Junction -Path $alias -Target $outside | Out-Null
        # The Python builder does not implement native handle-bound TOCTOU protection.
        # This case covers its actual path/closure contract; that stronger guarantee
        # remains unaccepted. Cases 16-20 separately exercise real native resolver APIs.
        $probe = @'
import sys
from pathlib import Path
from tools.build import build_editor as owner

base = Path(sys.argv[1]) / 'publication-contract'
base.mkdir()
alias = Path(sys.argv[2])
outside = Path(sys.argv[3])

def stage(name):
    path = base / name
    path.mkdir()
    (path / 'payload').write_bytes(b'fixture')
    return path, owner.editor_bundle_closure(path)

def reject(source, final, expected):
    before = owner.editor_bundle_closure(source)
    try:
        owner.publish_editor_bundle(source, final, expected)
    except (OSError, ValueError):
        pass
    else:
        raise AssertionError('Unsafe publication unexpectedly succeeded')
    assert source.is_dir()
    after = owner.editor_bundle_closure(source)
    assert after['files'] == before['files'] and after['digest'] == before['digest']

source, expected = stage('success-staging')
final = base / 'published'
published = owner.publish_editor_bundle(source, final, expected)
assert published['files'] == expected['files'] and published['digest'] == expected['digest']
assert not source.exists() and (final / 'payload').read_bytes() == b'fixture'
source, expected = stage('different-parent-staging')
other = base / 'other-parent'
other.mkdir()
reject(source, other / 'published', expected)
assert not (other / 'published').exists()
source, expected = stage('existing-staging')
existing = base / 'existing'
existing.mkdir()
(existing / 'sentinel').write_bytes(b'keep')
reject(source, existing, expected)
assert (existing / 'sentinel').read_bytes() == b'keep'
source, expected = stage('changed-staging')
(source / 'payload').write_bytes(b'changed')
reject(source, base / 'changed-final', expected)
assert not (base / 'changed-final').exists()
source, expected = stage('alias-staging')
reject(source, alias / 'published', expected)
assert not (outside / 'published').exists()
assert (outside / 'sentinel.txt').read_text(encoding='utf-8') == 'outside'
print('publication contract passed')
'@
        $result = Invoke-EditorFixturePython -Arguments @('-c', $probe, $fixtureRoot, $alias, $outside)
        if ($result.ExitCode -ne 0) { throw ($result.Output -join [Environment]::NewLine) }
        $result.ExitCode | Should Be 0
        ($result.Output -join [Environment]::NewLine) | Should Match 'publication contract passed'
        [IO.File]::Exists($callLog) | Should Be $false
    }

    It 'loads the versioned resolver interop when legacy types are already loaded' {
        $legacyProbe = Join-Path $fixtureRoot 'legacy-resolver-probe.ps1'
        $probe = @'
param([Parameter(Mandatory)][string]$ResolverPath)
Add-Type -TypeDefinition @"
namespace ZirconEngine.WindowsPathResolver
{
    public static class NativeMethods { public static string LegacyMarker() { return "legacy"; } }
    public static class NativeMethodsV2 { public static string PreviousMarker() { return "previous"; } }
    public static class NativeMethodsV3 { public static string PreviousVersionMarker() { return "previous-version"; } }
}
"@ -ErrorAction Stop
Import-Module $ResolverPath -Force -DisableNameChecking -ErrorAction Stop
[void](Resolve-ZirconWindowsPath -Path $PWD)
if ($null -eq ('ZirconEngine.WindowsPathResolver.NativeMethodsV4' -as [type])) {
    throw 'NativeMethodsV4 did not load after the legacy types.'
}
Write-Output 'NativeMethodsV4 loaded'
'@
        Write-EditorFixtureText -Path $legacyProbe -Value $probe
        $probeOutput = @(& $fixturePowerShell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $legacyProbe -ResolverPath $sourcePathResolver 2>&1)
        $LASTEXITCODE | Should Be 0
        ($probeOutput -join [Environment]::NewLine) | Should Match 'NativeMethodsV4 loaded'
    }

    It 'prevents replacement while an approved unpinned child directory lease is held' {
        Import-Module $sourcePathResolver -Force -DisableNameChecking -ErrorAction Stop
        # The fixture owner pins its root with DELETE access. Operate on a fresh
        # child so this test measures the resolver lease, not a conflicting owner.
        $operationRoot = Join-Path $fixtureRoot 'lease-operation'
        [IO.Directory]::CreateDirectory($operationRoot) | Out-Null
        $operational = (Resolve-ZirconWindowsPath -Path $operationRoot).OperationalPath
        $moved = "$operationRoot-replacement"
        $lease = Open-ZirconWindowsDirectoryLease -Path $operational -ExpectedOperationalPath $operational -DenyWrite
        try {
            $moveFailure = $null
            try { Move-Item -LiteralPath $operationRoot -Destination $moved -ErrorAction Stop }
            catch { $moveFailure = $_ }
            $moveFailure | Should Not BeNullOrEmpty
            [IO.Directory]::Exists($operationRoot) | Should Be $true
            [IO.Directory]::Exists($moved) | Should Be $false
        }
        finally {
            $lease.Dispose()
            if ([IO.Directory]::Exists($moved)) { [IO.Directory]::Move($moved, $operationRoot) }
        }
    }

    It 'marks an empty held staging directory for deletion through its lease' {
        Import-Module $sourcePathResolver -Force -DisableNameChecking -ErrorAction Stop
        $staging = Join-Path $fixtureRoot 'leased-empty-staging'
        [IO.Directory]::CreateDirectory($staging) | Out-Null
        $operational = (Resolve-ZirconWindowsPath -Path $staging).OperationalPath
        $options = @{ Path = $operational; ExpectedOperationalPath = $operational; ForMove = $true; DenyWrite = $true; NoFollow = $true }
        $lease = Open-ZirconWindowsDirectoryLease @options
        try { Remove-ZirconWindowsLeasedDirectory -Lease $lease }
        finally { $lease.Dispose() }
        [IO.Directory]::Exists($staging) | Should Be $false
    }

    It 'removes a held staging tree without following a junction to another owned fixture' {
        Import-Module $sourcePathResolver -Force -DisableNameChecking -ErrorAction Stop
        $staging = Join-Path $fixtureRoot 'leased-staging-tree'
        Write-EditorFixtureText -Path (Join-Path $staging 'nested\payload.txt') -Value 'staging'
        $outside = New-EditorOutsideFixtureDirectory
        $sentinel = Join-Path $outside 'sentinel.txt'
        Write-EditorFixtureText -Path $sentinel -Value 'outside'
        New-Item -ItemType Junction -Path (Join-Path $staging 'outside-link') -Target $outside | Out-Null
        $operational = (Resolve-ZirconWindowsPath -Path $staging).OperationalPath
        $options = @{ Path = $operational; ExpectedOperationalPath = $operational; ForMove = $true; DenyWrite = $true; NoFollow = $true }
        $lease = Open-ZirconWindowsDirectoryLease @options
        try { Remove-ZirconWindowsLeasedDirectoryTree -Lease $lease }
        finally { $lease.Dispose() }
        [IO.Directory]::Exists($staging) | Should Be $false
        [IO.Directory]::Exists($outside) | Should Be $true
        [IO.File]::ReadAllText($sentinel) | Should Be 'outside'
    }

    It 'rejects a resolved output parent outside an unpinned approved operation root without moving the source' {
        Import-Module $sourcePathResolver -Force -DisableNameChecking -ErrorAction Stop
        $operationRoot = Join-Path $fixtureRoot 'rename-operation'
        $source = Join-Path $operationRoot 'rename-source'
        Write-EditorFixtureText -Path (Join-Path $source 'payload.txt') -Value 'source'
        $outside = New-EditorOutsideFixtureDirectory
        $sentinel = Join-Path $outside 'sentinel.txt'
        Write-EditorFixtureText -Path $sentinel -Value 'outside'
        New-Item -ItemType Junction -Path (Join-Path $operationRoot 'outside-root') -Target $outside | Out-Null
        $approved = (Resolve-ZirconWindowsPath -Path $operationRoot).OperationalPath
        $moveFailure = $null
        try {
            $move = @{
                Source = (Resolve-ZirconWindowsPath -Path $source).OperationalPath
                Destination = Join-ZirconWindowsPath -Path $approved -ChildPath 'outside-root\published'
                ApprovedRoot = $approved
            }
            Move-ZirconWindowsPath @move
        }
        catch { $moveFailure = $_ }
        $moveFailure | Should Not BeNullOrEmpty
        $moveFailure.Exception.Message | Should Match 'outside the approved root'
        [IO.File]::ReadAllText((Join-Path $source 'payload.txt')) | Should Be 'source'
        [IO.Directory]::Exists((Join-Path $outside 'published')) | Should Be $false
        [IO.File]::ReadAllText($sentinel) | Should Be 'outside'
    }

    It 'rejects an approved-looking output junction outside the fixture operation root before execution' {
        $outside = New-EditorOutsideFixtureDirectory
        $sentinel = Join-Path $outside 'sentinel.txt'
        Write-EditorFixtureText -Path $sentinel -Value 'outside'
        $junction = Join-Path $fixtureRoot 'outside-build-root'
        New-Item -ItemType Junction -Path $junction -Target $outside | Out-Null
        $result = Invoke-EditorBuildFixture -OutputDirectory (Join-Path $junction 'editor-bundle')
        $result.ExitCode | Should Be 2
        ($result.Output -join [Environment]::NewLine) | Should Match 'reparse|alias'
        [IO.File]::Exists($callLog) | Should Be $false
        [IO.Directory]::Exists((Join-Path $outside 'editor-bundle')) | Should Be $false
        [IO.File]::ReadAllText($sentinel) | Should Be 'outside'
    }
}
