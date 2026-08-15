$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$capture = Join-Path $repoRoot 'tools\mvp\Capture-RenderExtractBaseline.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$manifestModule = Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1'
$originalTestMode = $env:RENDER_EXTRACT_BASELINE_TEST_MODE

Import-Module $resolverModule -Force -Global -ErrorAction Stop
Import-Module $manifestModule -Force -Global -ErrorAction Stop

try {
    $env:RENDER_EXTRACT_BASELINE_TEST_MODE = '1'
    . $capture
}
finally {
    $env:RENDER_EXTRACT_BASELINE_TEST_MODE = $originalTestMode
}

Describe 'Render-extract baseline capture plan' {
    BeforeEach {
        Import-Module $manifestModule -Force -ErrorAction Stop
        Import-Module $resolverModule -Force -ErrorAction Stop
    }

    It 'plans runtime cold and steady runs plus an editor cold first-frame run' {
        $runs = @(
            Get-RenderExtractBaselineRunPlan `
                -RepeatCount 3 `
                -WarmupPresentedFrameCount 60 `
                -MeasuredPresentedFrameCount 300
        )

        $runs.Count | Should Be 4
        $runs[0].logical_id | Should Be 'pipelined-first-frame'
        $runs[0].product | Should Be 'runtime'
        $runs[0].runtime_profile | Should Be 'runtime-pipelined'
        $runs[0].exit_after_first_frame | Should Be $true
        $runs[0].repeat_count | Should Be 3
        $runs[1].logical_id | Should Be 'pipelined-steady'
        $runs[1].runtime_profile | Should Be 'runtime-pipelined'
        $runs[1].warmup_presented_frame_count | Should Be 60
        $runs[1].measured_presented_frame_count | Should Be 300
        $runs[1].target_presented_frame_count | Should Be 360
        $runs[1].presented_frame_count | Should Be 360
        $runs[2].logical_id | Should Be 'synchronous-steady'
        $runs[2].runtime_profile | Should Be 'runtime'
        $runs[2].warmup_presented_frame_count | Should Be 60
        $runs[2].measured_presented_frame_count | Should Be 300
        $runs[2].target_presented_frame_count | Should Be 360
        $runs[2].presented_frame_count | Should Be 360
        $runs[3].logical_id | Should Be 'editor-first-frame'
        $runs[3].product | Should Be 'editor'
        $runs[3].runtime_profile | Should Be 'editor'
        $runs[3].exit_after_first_frame | Should Be $true
        $runs[3].presented_frame_count | Should BeNullOrEmpty
        $runs[3].warmup_presented_frame_count | Should Be 0
        $runs[3].measured_presented_frame_count | Should Be 1
        $runs[3].target_presented_frame_count | Should Be 1
    }

    It 'launches the product at the physical project root with the relative project argument' {
        $runtimeArguments = @(Get-RenderExtractBaselineProductArguments -Product 'runtime' -RuntimeProfile 'runtime-pipelined')
        $editorArguments = @(Get-RenderExtractBaselineProductArguments -Product 'editor' -RuntimeProfile 'editor')
        $captureSource = Get-Content -LiteralPath $capture -Raw
        $workingDirectoryPattern = [regex]::Escape('$startInfo.WorkingDirectory = $ProjectRoot.OperationalPath')
        $libraryPattern = [regex]::Escape("ZIRCON_RUNTIME_LIBRARY = 'zircon_runtime.dll'")

        $runtimeArguments | Should Be @('--project', '.', '--runtime-session-profile', 'runtime-pipelined')
        $editorArguments | Should Be @('--project', '.')
        $captureSource | Should Match $workingDirectoryPattern
        $captureSource | Should Match $libraryPattern
        $captureSource | Should Match 'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG'
        $captureSource | Should Match 'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME'
    }

    It 'keeps system ETW capture explicit and session-scoped' {
        $captureSource = Get-Content -LiteralPath $capture -Raw

        $captureSource | Should Match '\[switch\]\$UseWpr'
        $captureSource | Should Match 'Start-RenderExtractWprCapture'
        $captureSource | Should Match 'Stop-RenderExtractWprCapture'
        $captureSource | Should Match 'Join-ZirconWindowsPath -Path \$tracesRoot -ChildPath "\$sessionId.etl"'
        $captureSource | Should Match 'Join-ZirconWindowsPath -Path \$tracesRoot -ChildPath "\$sessionId.wpr-temp"'
        $captureSource | Should Match 'Start-RenderExtractWprCapture -TemporaryDirectory \$wprTemporaryDirectory'
        $captureSource | Should Match '& \$wpr.Source ''-start'' ''CPU'' ''-filemode'' ''-recordtempto'' \$TemporaryDirectory \| Out-Null'
        $captureSource | Should Match '& \$WprPath ''-stop'' \$TracePath \| Out-Null'
    }

    It 'refuses to start WPR without a caller-owned temporary directory' {
        $missingDirectory = Join-Path $TestDrive 'missing-wpr-recording-directory'

        { Start-RenderExtractWprCapture -TemporaryDirectory $missingDirectory } |
            Should Throw 'temporary directory does not exist'
    }

    It 'binds every baseline process to the profiling manifest artifact hashes' {
        $artifactDirectory = Join-Path $TestDrive 'profile-inputs'
        $runtimeDirectory = Join-Path $artifactDirectory 'runtime'
        $editorDirectory = Join-Path $artifactDirectory 'editor'
        [IO.Directory]::CreateDirectory($runtimeDirectory) | Out-Null
        [IO.Directory]::CreateDirectory($editorDirectory) | Out-Null
        $runtimeExecutablePath = Join-Path $runtimeDirectory 'zircon_runtime.exe'
        $runtimeLibraryPath = Join-Path $runtimeDirectory 'zircon_runtime.dll'
        $editorExecutablePath = Join-Path $editorDirectory 'zircon_editor.exe'
        $editorLibraryPath = Join-Path $editorDirectory 'zircon_runtime.dll'
        [IO.File]::WriteAllBytes($runtimeExecutablePath, [byte[]](1, 2, 3, 4))
        [IO.File]::WriteAllBytes($runtimeLibraryPath, [byte[]](5, 6, 7, 8, 9))
        [IO.File]::WriteAllBytes($editorExecutablePath, [byte[]](10, 11, 12, 13, 14, 15))
        [IO.File]::WriteAllBytes($editorLibraryPath, [byte[]](16, 17, 18, 19, 20, 21, 22))
        $manifestPath = Join-Path $artifactDirectory 'render-extract-profiling-inputs.json'
        $manifest = [ordered]@{
            schema_version = 2
            source_fingerprint = ('A' * 64)
            cargo_profile = 'profiling'
            artifacts = @(
                [ordered]@{
                    logical_id = 'runtime-profile-executable'
                    product = 'runtime'
                    package = 'zircon_app'
                    bin = 'zircon_runtime'
                    features = 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
                    path = $runtimeExecutablePath
                    bytes = 4
                    sha256 = Get-MvpProductInputFileSha256 -Path $runtimeExecutablePath
                },
                [ordered]@{
                    logical_id = 'runtime-profile-library'
                    product = 'runtime'
                    package = 'zircon_runtime'
                    bin = $null
                    features = 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
                    path = $runtimeLibraryPath
                    bytes = 5
                    sha256 = Get-MvpProductInputFileSha256 -Path $runtimeLibraryPath
                },
                [ordered]@{
                    logical_id = 'editor-profile-executable'
                    product = 'editor'
                    package = 'zircon_app'
                    bin = 'zircon_editor'
                    features = 'target-editor-host,profiling'
                    path = $editorExecutablePath
                    bytes = 6
                    sha256 = Get-MvpProductInputFileSha256 -Path $editorExecutablePath
                },
                [ordered]@{
                    logical_id = 'editor-profile-library'
                    product = 'editor'
                    package = 'zircon_runtime'
                    bin = $null
                    features = 'target-editor-host,profiling'
                    path = $editorLibraryPath
                    bytes = 7
                    sha256 = Get-MvpProductInputFileSha256 -Path $editorLibraryPath
                }
            )
        }
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))

        $input = Resolve-RenderExtractProfilingInput -ManifestPath $manifestPath -ExpectedSourceFingerprint ('A' * 64)

        $input.runtime.executable_path | Should Be (Resolve-ZirconWindowsPath -Path $runtimeExecutablePath).OperationalPath
        $input.runtime.library_path | Should Be (Resolve-ZirconWindowsPath -Path $runtimeLibraryPath).OperationalPath
        $input.editor.executable_path | Should Be (Resolve-ZirconWindowsPath -Path $editorExecutablePath).OperationalPath
        $input.editor.library_path | Should Be (Resolve-ZirconWindowsPath -Path $editorLibraryPath).OperationalPath
        $input.manifest_sha256 | Should Match '^[0-9A-F]{64}$'
        $input.runtime.executable_sha256 | Should Be $manifest.artifacts[0].sha256
        $input.runtime.library_sha256 | Should Be $manifest.artifacts[1].sha256
        $input.editor.executable_sha256 | Should Be $manifest.artifacts[2].sha256
        $input.editor.library_sha256 | Should Be $manifest.artifacts[3].sha256

        $otherDirectory = Join-Path $TestDrive 'other-profile-inputs'
        New-Item -ItemType Directory -Path $otherDirectory | Out-Null
        $otherLibraryPath = Join-Path $otherDirectory 'zircon_runtime.dll'
        [IO.File]::WriteAllBytes($otherLibraryPath, [byte[]](9, 8, 7, 6, 5))
        $manifest.artifacts[1].path = $otherLibraryPath
        $manifest.artifacts[1].sha256 = Get-MvpProductInputFileSha256 -Path $otherLibraryPath
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))

        $failure = $null
        try {
            Resolve-RenderExtractProfilingInput -ManifestPath $manifestPath -ExpectedSourceFingerprint ('A' * 64) | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'same directory'
    }

    It 'requires each product pair to live in its manifest managed product directory' {
        $manifestDirectory = Join-Path $TestDrive 'profile-manifest'
        $artifactDirectory = Join-Path $TestDrive 'profile-artifacts'
        [IO.Directory]::CreateDirectory($manifestDirectory) | Out-Null
        [IO.Directory]::CreateDirectory($artifactDirectory) | Out-Null
        $executablePath = Join-Path $artifactDirectory 'zircon_runtime.exe'
        $libraryPath = Join-Path $artifactDirectory 'zircon_runtime.dll'
        [IO.File]::WriteAllBytes($executablePath, [byte[]](1, 2, 3, 4))
        [IO.File]::WriteAllBytes($libraryPath, [byte[]](5, 6, 7, 8))
        $manifestPath = Join-Path $manifestDirectory 'render-extract-profiling-inputs.json'
        $manifest = [ordered]@{
            schema_version = 2
            source_fingerprint = ('A' * 64)
            cargo_profile = 'profiling'
            artifacts = @(
                [ordered]@{
                    logical_id = 'runtime-profile-executable'
                    product = 'runtime'
                    package = 'zircon_app'
                    bin = 'zircon_runtime'
                    features = 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
                    path = $executablePath
                    bytes = 4
                    sha256 = Get-MvpProductInputFileSha256 -Path $executablePath
                },
                [ordered]@{
                    logical_id = 'runtime-profile-library'
                    product = 'runtime'
                    package = 'zircon_runtime'
                    bin = $null
                    features = 'target-client,platform-winit,input-gamepad,gamepad-gilrs,profiling'
                    path = $libraryPath
                    bytes = 4
                    sha256 = Get-MvpProductInputFileSha256 -Path $libraryPath
                },
                [ordered]@{
                    logical_id = 'editor-profile-executable'
                    product = 'editor'
                    package = 'zircon_app'
                    bin = 'zircon_editor'
                    features = 'target-editor-host,profiling'
                    path = $executablePath
                    bytes = 4
                    sha256 = Get-MvpProductInputFileSha256 -Path $executablePath
                },
                [ordered]@{
                    logical_id = 'editor-profile-library'
                    product = 'editor'
                    package = 'zircon_runtime'
                    bin = $null
                    features = 'target-editor-host,profiling'
                    path = $libraryPath
                    bytes = 4
                    sha256 = Get-MvpProductInputFileSha256 -Path $libraryPath
                }
            )
        }
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))

        { Resolve-RenderExtractProfilingInput -ManifestPath $manifestPath -ExpectedSourceFingerprint ('A' * 64) } |
            Should Throw 'managed product directory'
    }

    It 'rejects profiling input identity changes between capture attempts' {
        $expected = [pscustomobject]@{
            manifest_path = 'E:\ZirconBuilds\mvp-perf-inputs\run\render-extract-profiling-inputs.json'
            manifest_sha256 = 'A' * 64
            runtime = [pscustomobject]@{
                executable_path = 'E:\ZirconBuilds\mvp-perf-inputs\run\runtime\zircon_runtime.exe'
                executable_sha256 = 'B' * 64
                library_path = 'E:\ZirconBuilds\mvp-perf-inputs\run\runtime\zircon_runtime.dll'
                library_sha256 = 'C' * 64
            }
            editor = [pscustomobject]@{
                executable_path = 'E:\ZirconBuilds\mvp-perf-inputs\run\editor\zircon_editor.exe'
                executable_sha256 = 'D' * 64
                library_path = 'E:\ZirconBuilds\mvp-perf-inputs\run\editor\zircon_runtime.dll'
                library_sha256 = 'E' * 64
            }
        }
        $actual = $expected.PSObject.Copy()
        $actual.editor = $expected.editor.PSObject.Copy()
        $actual.editor.executable_sha256 = 'F' * 64

        { Assert-RenderExtractProfilingInputIdentity -Expected $expected -Actual $actual } |
            Should Throw 'changed during baseline capture'
    }

    It 'freezes verified product inputs into invocation-local product directories before launch' {
        $sourceDirectory = Join-Path $TestDrive 'mutable-profile-inputs'
        [IO.Directory]::CreateDirectory($sourceDirectory) | Out-Null
        $manifestPath = Join-Path $sourceDirectory 'render-extract-profiling-inputs.json'
        $runtimeExecutablePath = Join-Path $sourceDirectory 'zircon_runtime.exe'
        $runtimeLibraryPath = Join-Path $sourceDirectory 'runtime-zircon_runtime.dll'
        $editorExecutablePath = Join-Path $sourceDirectory 'zircon_editor.exe'
        $editorLibraryPath = Join-Path $sourceDirectory 'editor-zircon_runtime.dll'
        $editorAssetRoot = Join-Path $sourceDirectory 'editor-assets'
        $runtimeAssetRoot = Join-Path $sourceDirectory 'runtime-assets'
        $engineAssetPath = Join-Path $editorAssetRoot 'ui\editor\host\editor_main_frame.zui'
        $fontAssetPath = Join-Path $runtimeAssetRoot 'fonts\default.font.toml'
        $editorSharedAssetPath = Join-Path $editorAssetRoot 'shared\version.txt'
        $runtimeSharedAssetPath = Join-Path $runtimeAssetRoot 'shared\version.txt'
        [IO.Directory]::CreateDirectory((Split-Path -Parent $engineAssetPath)) | Out-Null
        [IO.Directory]::CreateDirectory((Split-Path -Parent $fontAssetPath)) | Out-Null
        [IO.Directory]::CreateDirectory((Split-Path -Parent $editorSharedAssetPath)) | Out-Null
        [IO.Directory]::CreateDirectory((Split-Path -Parent $runtimeSharedAssetPath)) | Out-Null
        [IO.File]::WriteAllBytes($manifestPath, [byte[]](10, 11, 12))
        [IO.File]::WriteAllBytes($runtimeExecutablePath, [byte[]](1, 2, 3, 4))
        [IO.File]::WriteAllBytes($runtimeLibraryPath, [byte[]](5, 6, 7, 8))
        [IO.File]::WriteAllBytes($editorExecutablePath, [byte[]](9, 10, 11, 12))
        [IO.File]::WriteAllBytes($editorLibraryPath, [byte[]](13, 14, 15, 16))
        [IO.File]::WriteAllText($engineAssetPath, 'frozen editor asset', [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($fontAssetPath, 'frozen runtime font', [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($editorSharedAssetPath, 'matching duplicate', [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($runtimeSharedAssetPath, 'matching duplicate', [Text.UTF8Encoding]::new($false))
        $input = [pscustomobject]@{
            manifest_path = $manifestPath
            manifest_sha256 = Get-MvpProductInputFileSha256 -Path $manifestPath
            runtime = [pscustomobject]@{
                executable_path = $runtimeExecutablePath
                executable_sha256 = Get-MvpProductInputFileSha256 -Path $runtimeExecutablePath
                library_path = $runtimeLibraryPath
                library_sha256 = Get-MvpProductInputFileSha256 -Path $runtimeLibraryPath
            }
            editor = [pscustomobject]@{
                executable_path = $editorExecutablePath
                executable_sha256 = Get-MvpProductInputFileSha256 -Path $editorExecutablePath
                library_path = $editorLibraryPath
                library_sha256 = Get-MvpProductInputFileSha256 -Path $editorLibraryPath
            }
        }
        $invocationId = 'A' * 32

        $frozen = New-RenderExtractFrozenProfilingInput `
            -ProfilingInput $input `
            -EngineAssetRoots @($editorAssetRoot, $runtimeAssetRoot) `
            -OutputDirectory $TestDrive `
            -InvocationId $invocationId

        $frozenDirectory = Join-Path (Join-Path $TestDrive 'inputs') $invocationId
        $frozen.manifest_path | Should Be (Join-Path $frozenDirectory 'render-extract-profiling-inputs.json')
        $frozen.runtime.executable_path | Should Be (Join-Path (Join-Path $frozenDirectory 'runtime') 'zircon_runtime.exe')
        $frozen.runtime.library_path | Should Be (Join-Path (Join-Path $frozenDirectory 'runtime') 'zircon_runtime.dll')
        $frozen.editor.executable_path | Should Be (Join-Path (Join-Path $frozenDirectory 'editor') 'zircon_editor.exe')
        $frozen.editor.library_path | Should Be (Join-Path (Join-Path $frozenDirectory 'editor') 'zircon_runtime.dll')
        [IO.File]::WriteAllBytes($runtimeExecutablePath, [byte[]](9, 9, 9, 9))
        Get-MvpProductInputFileSha256 -Path $frozen.runtime.executable_path | Should Be $input.runtime.executable_sha256
        Get-MvpProductInputFileSha256 -Path $frozen.runtime.library_path | Should Be $input.runtime.library_sha256
        Get-MvpProductInputFileSha256 -Path $frozen.editor.executable_path | Should Be $input.editor.executable_sha256
        Get-MvpProductInputFileSha256 -Path $frozen.editor.library_path | Should Be $input.editor.library_sha256
        Get-MvpProductInputFileSha256 -Path $frozen.manifest_path | Should Be $input.manifest_sha256
        $frozen.runtime.asset_root_path | Should Be (Join-Path (Join-Path $frozenDirectory 'runtime') 'assets')
        $frozen.editor.asset_root_path | Should Be (Join-Path (Join-Path $frozenDirectory 'editor') 'assets')
        $frozen.runtime.asset_file_count | Should Be 3
        $frozen.editor.asset_file_count | Should Be 3
        Get-Content -Raw -LiteralPath (Join-Path $frozen.editor.asset_root_path 'ui\editor\host\editor_main_frame.zui') |
            Should Be 'frozen editor asset'
        Get-Content -Raw -LiteralPath (Join-Path $frozen.editor.asset_root_path 'fonts\default.font.toml') |
            Should Be 'frozen runtime font'
        Get-Content -Raw -LiteralPath (Join-Path $frozen.runtime.asset_root_path 'shared\version.txt') |
            Should Be 'matching duplicate'

        [IO.File]::WriteAllText($runtimeSharedAssetPath, 'conflicting duplicate', [Text.UTF8Encoding]::new($false))
        {
            New-RenderExtractFrozenProfilingInput `
                -ProfilingInput $input `
                -EngineAssetRoots @($editorAssetRoot, $runtimeAssetRoot) `
                -OutputDirectory $TestDrive `
                -InvocationId ('B' * 32)
        } | Should Throw "conflicting file 'shared/version.txt'"
        [IO.Directory]::Exists((Join-Path (Join-Path $TestDrive 'inputs') ('B' * 32))) | Should Be $false
    }

    It 'revalidates frozen executable library and asset hashes immediately before product launch' {
        $productDirectory = Join-Path $TestDrive 'frozen-runtime'
        [IO.Directory]::CreateDirectory($productDirectory) | Out-Null
        $executablePath = Join-Path $productDirectory 'zircon_runtime.exe'
        $libraryPath = Join-Path $productDirectory 'zircon_runtime.dll'
        [IO.File]::WriteAllBytes($executablePath, [byte[]](1, 2, 3, 4))
        [IO.File]::WriteAllBytes($libraryPath, [byte[]](5, 6, 7, 8))
        $productInput = [pscustomobject]@{
            executable_path = $executablePath
            executable_sha256 = Get-MvpProductInputFileSha256 -Path $executablePath
            library_path = $libraryPath
            library_sha256 = Get-MvpProductInputFileSha256 -Path $libraryPath
            asset_root_path = Join-Path $productDirectory 'assets'
            asset_manifest_path = Join-Path $productDirectory 'asset-manifest.json'
        }
        $assetPath = Join-Path $productInput.asset_root_path 'ui\editor\host\editor_main_frame.zui'
        [IO.Directory]::CreateDirectory((Split-Path -Parent $assetPath)) | Out-Null
        [IO.File]::WriteAllText($assetPath, 'asset', [Text.UTF8Encoding]::new($false))
        $productInput | Add-Member -NotePropertyName asset_files -NotePropertyValue @(
            [pscustomobject]@{
                relative_path = 'ui/editor/host/editor_main_frame.zui'
                bytes = [IO.FileInfo]::new($assetPath).Length
                sha256 = Get-MvpProductInputFileSha256 -Path $assetPath
            }
        )
        [IO.File]::WriteAllText($productInput.asset_manifest_path, '{"schema_version":1}', [Text.UTF8Encoding]::new($false))
        $productInput | Add-Member -NotePropertyName asset_manifest_sha256 -NotePropertyValue (Get-MvpProductInputFileSha256 -Path $productInput.asset_manifest_path)

        $actual = Assert-RenderExtractFrozenProductInput `
            -ProductInput $productInput `
            -Product 'runtime'

        $actual.executable_sha256 | Should Be $productInput.executable_sha256
        $actual.library_sha256 | Should Be $productInput.library_sha256
        $actual.asset_file_count | Should Be 1

        [IO.File]::WriteAllBytes($executablePath, [byte[]](9, 9, 9, 9))
        { Assert-RenderExtractFrozenProductInput -ProductInput $productInput -Product 'runtime' } |
            Should Throw 'executable changed before process launch'

        [IO.File]::WriteAllBytes($executablePath, [byte[]](1, 2, 3, 4))
        [IO.File]::WriteAllBytes($libraryPath, [byte[]](8, 8, 8, 8))
        { Assert-RenderExtractFrozenProductInput -ProductInput $productInput -Product 'runtime' } |
            Should Throw 'runtime library changed before process launch'

        [IO.File]::WriteAllBytes($libraryPath, [byte[]](5, 6, 7, 8))
        [IO.File]::WriteAllText($assetPath, 'tampered asset', [Text.UTF8Encoding]::new($false))
        { Assert-RenderExtractFrozenProductInput -ProductInput $productInput -Product 'runtime' } |
            Should Throw 'asset inventory changed before process launch'
    }

    It 'publishes the source-bound percentile report after preserving the raw summary' {
        $captureSource = Get-Content -LiteralPath $capture -Raw
        $reporterPattern = [regex]::Escape("& (Join-Path `$PSScriptRoot 'Write-RenderExtractBaselineReport.ps1') -BaselineSummaryPath `$summaryPath | Out-Null")
        $invocationPattern = [regex]::Escape('invocation_id = $sessionLease.InvocationId')

        $captureSource | Should Match $reporterPattern
        $captureSource | Should Match $invocationPattern
        $captureSource | Should Match '\$peakWorkingSetBytes = \[Int64\]\$process.PeakWorkingSet64'
        $captureSource | Should Match '\$totalProcessorTime = \$process.TotalProcessorTime'
        $captureSource | Should Match '\[TimeSpan\]\$totalProcessorTime\).TotalMilliseconds'
        $captureSource | Should Match '\$exitCode = \$assignedProcess.TryGetExitCode\(\)'
        $captureSource | Should Match '\$processId = \[Int64\]\$process.Id'
        $captureSource | Should Match 'process_id = \$processId'
        $captureSource | Should Match 'process_elapsed_ms = \$processElapsedMs'
        $captureSource | Should Match 'schema_version = 4'
        $assetFreezeIndex = $captureSource.IndexOf('$actualProductHashes = Assert-RenderExtractFrozenProductInput')
        $wprIndex = $captureSource.IndexOf('$wprPath = Start-RenderExtractWprCapture')
        $stopwatchStartIndex = $captureSource.IndexOf('$processStopwatch.Start()')
        $processStartIndex = $captureSource.IndexOf('Start-RenderExtractBaselineAssignedProcess -Job $processJob -StartInfo $startInfo')
        $stopwatchStopIndex = $captureSource.IndexOf('$processStopwatch.Stop()')
        $outputDrainIndex = $captureSource.IndexOf('$assignedProcess.StandardOutput.ReadToEndAsync()')
        $wprIndex | Should BeGreaterThan $assetFreezeIndex
        $stopwatchStartIndex | Should BeGreaterThan $wprIndex
        $processStartIndex | Should BeGreaterThan $stopwatchStartIndex
        $stopwatchStopIndex | Should BeGreaterThan $processStartIndex
        $outputDrainIndex | Should BeGreaterThan $processStartIndex
    }

    It 'accepts only an empty physical perf evidence root on the plan-owned E drive' {
        $accepted = Assert-RenderExtractBaselineOutputDirectory -Path 'E:\ZirconBuilds\mvp-perf\baseline-contract'
        $resolution = Resolve-ZirconWindowsPath -Path 'E:\ZirconBuilds\mvp-perf\baseline-contract'

        $accepted | Should Be $resolution.OperationalPath

        $failure = $null
        try {
            Assert-RenderExtractBaselineOutputDirectory -Path 'C:\ZirconBuilds\mvp-perf\baseline-contract'
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'mvp-perf'

        $otherDriveFailure = $null
        try {
            Assert-RenderExtractBaselineOutputDirectory -Path 'D:\ZirconBuilds\mvp-perf\baseline-contract'
        }
        catch {
            $otherDriveFailure = $_
        }

        $otherDriveFailure | Should Not BeNullOrEmpty
        $otherDriveFailure.Exception.Message | Should Match 'E:\\ZirconBuilds\\mvp-perf'
    }

    It 'rejects the repository source template as a mutable capture project' {
        $templateProject = Join-Path $repoRoot 'templates\projects\renderable-empty'
        $failure = $null

        try {
            Assert-RenderExtractBaselineProjectDirectory -Path $templateProject | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'created project or example project'
    }

    It 'rejects a capture project outside approved artifact drives before it can create state' {
        $failure = $null

        try {
            Assert-RenderExtractBaselineProjectDirectory -Path 'C:\ZirconBuilds\render-extract-project' | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'approved D:, E:, or F: drive'
    }

    It 'accepts an ordinary project without generated-scale metadata' {
        $projectDirectory = Join-Path $TestDrive 'ordinary-capture-project'
        [IO.Directory]::CreateDirectory($projectDirectory) | Out-Null
        [IO.File]::WriteAllText((Join-Path $projectDirectory 'zircon-project.toml'), 'format_version = 2')

        $metadata = Get-RenderExtractScaleProjectMetadata `
            -ProjectRoot (Resolve-ZirconWindowsPath -Path $projectDirectory) `
            -ExpectedSourceFingerprint ('A' * 64)

        $metadata | Should BeNullOrEmpty
    }

    It 'rejects generated-scale metadata from a different source snapshot' {
        $projectDirectory = Join-Path $TestDrive 'stale-scale-capture-project'
        [IO.Directory]::CreateDirectory($projectDirectory) | Out-Null
        [IO.File]::WriteAllText((Join-Path $projectDirectory 'zircon-project.toml'), 'format_version = 2')
        [IO.File]::WriteAllText(
            (Join-Path $projectDirectory 'render-extract-scale-project.json'),
            (([ordered]@{
                        schema_version = 1
                        source_fingerprint = 'B' * 64
                        primitive_count = 1000
                        scene_virtual_path = 'res://scenes/main.scene.toml'
                        model_virtual_path = 'assets/models/cube.obj'
                        material_virtual_path = 'assets/materials/default.zmaterial'
                    }) | ConvertTo-Json),
            [Text.UTF8Encoding]::new($false)
        )

        {
            Get-RenderExtractScaleProjectMetadata `
                -ProjectRoot (Resolve-ZirconWindowsPath -Path $projectDirectory) `
                -ExpectedSourceFingerprint ('A' * 64)
        } | Should Throw 'different source snapshot'
    }

    It 'does not create a baseline evidence root when profiling input preflight fails' {
        $outputDirectory = Join-Path $TestDrive 'capture-preflight-failure-output'
        $projectDirectory = Join-Path $TestDrive 'capture-preflight-project'
        [IO.Directory]::CreateDirectory($projectDirectory) | Out-Null
        [IO.File]::WriteAllText((Join-Path $projectDirectory 'zircon-project.toml'), 'schema_version = 1')
        Mock Assert-RenderExtractBaselineOutputDirectory {
            param($Path)
            $outputDirectory
        }
        Mock Assert-RenderExtractBaselineProjectDirectory {
            param($Path)
            [pscustomobject]@{
                OperationalPath = 'E:\ZirconBuilds\mvp-perf\capture-preflight-project'
                DisplayPath = 'E:\ZirconBuilds\mvp-perf\capture-preflight-project'
            }
        }
        Mock Get-MvpSourceFingerprint { 'A' * 64 }
        Mock Resolve-RenderExtractProfilingInput { throw 'profiling input preflight failed' }
        $failure = $null

        try {
            Invoke-RenderExtractBaselineCapture `
                -ManifestPath (Join-Path $TestDrive 'missing-input.json') `
                -ProjectPath $projectDirectory `
                -EvidenceOutputDirectory 'E:\ZirconBuilds\mvp-perf\contract' | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'profiling input preflight failed'
        [IO.Directory]::Exists($outputDirectory) | Should Be $false
    }

    It 'holds one exclusive baseline session lease and removes only that lease on close' {
        $outputDirectory = Join-Path $TestDrive 'exclusive-baseline-session'
        $firstLease = New-RenderExtractBaselineOutputSessionLease -Path $outputDirectory
        $secondFailure = $null

        try {
            New-RenderExtractBaselineOutputSessionLease -Path $outputDirectory | Out-Null
        }
        catch {
            $secondFailure = $_
        }
        finally {
            $firstLease.Stream.Dispose()
        }

        $secondFailure | Should Not BeNullOrEmpty
        $secondFailure.Exception.Message | Should Match 'already active or changed after preflight'
        [IO.Directory]::Exists($outputDirectory) | Should Be $true
        [IO.File]::Exists($firstLease.Path) | Should Be $false
    }

    It 'rejects foreign output created before the baseline session lease without deleting it' {
        $outputDirectory = Join-Path $TestDrive 'foreign-baseline-session'
        $foreignEvidence = Join-Path $outputDirectory 'foreign-evidence.txt'
        [IO.Directory]::CreateDirectory($outputDirectory) | Out-Null
        [IO.File]::WriteAllText($foreignEvidence, 'preserve')
        $failure = $null

        try {
            New-RenderExtractBaselineOutputSessionLease -Path $outputDirectory | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'must remain empty'
        [IO.File]::Exists($foreignEvidence) | Should Be $true
    }

    It 'refuses to overwrite an existing capture text artifact' {
        $artifactPath = Join-Path $TestDrive 'existing-capture-output.log'
        [IO.File]::WriteAllText($artifactPath, 'foreign-evidence', [Text.UTF8Encoding]::new($false))
        $failure = $null

        try {
            Write-RenderExtractBaselineTextFileNew -Path $artifactPath -Content 'capture-output'
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'Refusing to overwrite existing render-extract evidence'
        [IO.File]::ReadAllText($artifactPath) | Should Be 'foreign-evidence'
    }

    It 'preserves a product start failure instead of replacing it during process cleanup' {
        $outputDirectory = Join-Path $TestDrive ("capture-start-failure-" + [guid]::NewGuid().ToString('N'))
        $missingExecutable = Join-Path $TestDrive 'missing-zircon-runtime.exe'
        $run = (Get-RenderExtractBaselineRunPlan -RepeatCount 3 -WarmupPresentedFrameCount 60 -MeasuredPresentedFrameCount 300)[0]
        $failure = $null

        try {
            Invoke-RenderExtractBaselineProcess `
                -ProfilingInput ([pscustomobject]@{
                        runtime = [pscustomobject]@{
                            executable_path = $missingExecutable
                            executable_sha256 = 'A' * 64
                            library_path = (Join-Path $TestDrive 'zircon_runtime.dll')
                            library_sha256 = 'B' * 64
                        }
                    }) `
                -ProjectRoot (Resolve-ZirconWindowsPath -Path $TestDrive) `
                -Run $run `
                -Attempt 1 `
                -InvocationId 'product-start-failure' `
                -OutputDirectory $outputDirectory `
                -TimeoutSeconds 1 `
                -MaxProfileFrames 1 `
                -MaxProfileSpans 1 `
                -MaxProfileCounters 1 | Out-Null
        }
        catch {
            $failure = $_
        }
        finally {
            if ([IO.Directory]::Exists($outputDirectory)) {
                Remove-Item -LiteralPath $outputDirectory -Recurse -Force
            }
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Not Match 'No process is associated'
        $failure.Exception.Message | Should Match 'missing-zircon-runtime\.exe'
    }

    It 'rejects a tampered frozen executable at the capture process boundary before launch' {
        $outputDirectory = Join-Path $TestDrive ("capture-tampered-product-" + [guid]::NewGuid().ToString('N'))
        $executablePath = Join-Path $TestDrive 'frozen-zircon-runtime.exe'
        $libraryPath = Join-Path $TestDrive 'frozen-zircon-runtime.dll'
        [IO.File]::WriteAllBytes($executablePath, [byte[]](1, 2, 3, 4))
        [IO.File]::WriteAllBytes($libraryPath, [byte[]](5, 6, 7, 8))
        $expectedExecutableSha256 = Get-MvpProductInputFileSha256 -Path $executablePath
        $run = (Get-RenderExtractBaselineRunPlan -RepeatCount 3 -WarmupPresentedFrameCount 60 -MeasuredPresentedFrameCount 300)[0]
        [IO.File]::WriteAllBytes($executablePath, [byte[]](9, 9, 9, 9))
        $failure = $null

        try {
            Invoke-RenderExtractBaselineProcess `
                -ProfilingInput ([pscustomobject]@{
                        runtime = [pscustomobject]@{
                            executable_path = $executablePath
                            executable_sha256 = $expectedExecutableSha256
                            library_path = $libraryPath
                            library_sha256 = Get-MvpProductInputFileSha256 -Path $libraryPath
                        }
                    }) `
                -ProjectRoot (Resolve-ZirconWindowsPath -Path $TestDrive) `
                -Run $run `
                -Attempt 1 `
                -InvocationId 'tampered-product' `
                -OutputDirectory $outputDirectory `
                -TimeoutSeconds 1 `
                -MaxProfileFrames 1 `
                -MaxProfileSpans 1 `
                -MaxProfileCounters 1 | Out-Null
        }
        catch {
            $failure = $_
        }
        finally {
            if ([IO.Directory]::Exists($outputDirectory)) {
                Remove-Item -LiteralPath $outputDirectory -Recurse -Force
            }
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'executable changed before process launch'
        $failure.Exception.Message | Should Not Match 'not a valid Win32 application|Windows did not start'
    }

    It 'terminates a live product process tree before releasing its process object' {
        $process = Start-Process `
            -FilePath 'powershell.exe' `
            -ArgumentList @('-NoProfile', '-Command', 'Start-Sleep -Seconds 30') `
            -WindowStyle Hidden `
            -PassThru
        try {
            Stop-RenderExtractBaselineProcessTree -Process $process -SessionId 'cleanup-test'

            $process.HasExited | Should Be $true
            $captureSource = Get-Content -LiteralPath $capture -Raw
            $cleanupIndex = $captureSource.IndexOf('Stop-RenderExtractBaselineProcessTree -Process $process')
            $disposeIndex = $captureSource.IndexOf('$assignedProcess.Dispose()')
            $cleanupIndex | Should BeGreaterThan -1
            $disposeIndex | Should BeGreaterThan $cleanupIndex
        }
        finally {
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
            }
            $process.Dispose()
        }
    }

    It 'terminates descendants through the process job after the root process exits' {
        $childPidPath = Join-Path $TestDrive ("render-extract-child-" + [guid]::NewGuid().ToString('N') + '.txt')
        $parentScriptPath = Join-Path $TestDrive ("render-extract-parent-" + [guid]::NewGuid().ToString('N') + '.ps1')
        $escapedChildPidPath = $childPidPath.Replace("'", "''")
        [IO.File]::WriteAllText(
            $parentScriptPath,
            "`$child = Start-Process powershell.exe -ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 30' -WindowStyle Hidden -PassThru`n[IO.File]::WriteAllText('$escapedChildPidPath', [string]`$child.Id)",
            [Text.UTF8Encoding]::new($false)
        )
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = (Get-Command powershell.exe -ErrorAction Stop).Source
        $startInfo.UseShellExecute = $false
        $startInfo.CreateNoWindow = $true
        $startInfo.WorkingDirectory = $TestDrive
        $startInfo.Arguments = "-NoProfile -File `"$parentScriptPath`""
        $assignedProcess = $null
        $process = $null
        $job = New-RenderExtractBaselineProcessJob
        $child = $null
        try {
            $assignedProcess = Start-RenderExtractBaselineAssignedProcess -Job $job -StartInfo $startInfo
            $process = $assignedProcess.Process
            $process.WaitForExit(10000) | Should Be $true
            [IO.File]::Exists($childPidPath) | Should Be $true
            $child = [Diagnostics.Process]::GetProcessById([int][IO.File]::ReadAllText($childPidPath))
            $child.HasExited | Should Be $false

            Stop-RenderExtractBaselineProcessJob -Job $job -SessionId 'descendant-cleanup-test'

            $child.WaitForExit(5000) | Should Be $true
        }
        finally {
            $job.Dispose()
            if ($null -ne $child) {
                if (-not $child.HasExited) {
                    Stop-Process -Id $child.Id -Force -ErrorAction SilentlyContinue
                }
                $child.Dispose()
            }
            if ($null -ne $process -and -not $process.HasExited) {
                Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
            }
            if ($null -ne $assignedProcess) {
                $assignedProcess.Dispose()
            }
        }

        $captureSource = Get-Content -LiteralPath $capture -Raw
        $assignIndex = $captureSource.IndexOf('Start-RenderExtractBaselineAssignedProcess -Job $processJob -StartInfo $startInfo')
        $outputDrainIndex = $captureSource.IndexOf('$assignedProcess.StandardOutput.ReadToEndAsync()')
        $assignIndex | Should BeGreaterThan -1
        $outputDrainIndex | Should BeGreaterThan $assignIndex
        $captureSource | Should Match 'if \(\$processStarted\)'
        $captureSource | Should Match 'Stop-RenderExtractBaselineProcessJob -Job \$processJob'
        $cleanupIndex = $captureSource.IndexOf('Stop-RenderExtractBaselineProcessJob -Job $processJob')
        $statisticsIndex = $captureSource.IndexOf('$peakWorkingSetBytes = [Int64]$process.PeakWorkingSet64')
        $wprStopIndex = $captureSource.IndexOf('Stop-RenderExtractWprCapture -WprPath $wprPath')
        $cleanupIndex | Should BeGreaterThan -1
        $cleanupIndex | Should BeGreaterThan $statisticsIndex
        $wprStopIndex | Should BeGreaterThan $cleanupIndex
    }

    It 'acquires the product process and output pipes before resuming a fast-exit product' {
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = (Get-Command powershell.exe -ErrorAction Stop).Source
        $startInfo.UseShellExecute = $false
        $startInfo.CreateNoWindow = $true
        $startInfo.WorkingDirectory = $TestDrive
        $startInfo.Arguments = '-NoProfile -Command "Write-Output fast-exit-product"'
        $job = New-RenderExtractBaselineProcessJob
        $assignedProcess = $null
        try {
            $assignedProcess = Start-RenderExtractBaselineAssignedProcess -Job $job -StartInfo $startInfo
            $stdoutTask = $assignedProcess.StandardOutput.ReadToEndAsync()
            $assignedProcess.Process.WaitForExit(5000) | Should Be $true
            [Threading.Tasks.Task]::WaitAll(@($stdoutTask), 5000) | Should Be $true
            $stdoutTask.GetAwaiter().GetResult().Trim() | Should Be 'fast-exit-product'
            Stop-RenderExtractBaselineProcessJob -Job $job -SessionId 'fast-exit-product'
        }
        finally {
            $job.Dispose()
            if ($null -ne $assignedProcess) {
                $assignedProcess.Dispose()
            }
        }

        $jobModuleSource = Get-Content -LiteralPath (Join-Path $repoRoot 'tools\mvp\RenderExtractProcessJob.psm1') -Raw
        $processIndex = $jobModuleSource.IndexOf('process = Process.GetProcessById')
        $stdoutIndex = $jobModuleSource.IndexOf('stdout = ReaderFromHandle')
        $stderrIndex = $jobModuleSource.IndexOf('stderr = ReaderFromHandle')
        $resumeIndex = $jobModuleSource.IndexOf('ResumeThread(processInformation.Thread)')
        $processIndex | Should BeGreaterThan -1
        $stdoutIndex | Should BeGreaterThan $processIndex
        $stderrIndex | Should BeGreaterThan $stdoutIndex
        $resumeIndex | Should BeGreaterThan $stderrIndex
    }

    It 'waits for the successful product job to become empty before publishing a run' {
        $outputDirectory = Join-Path $TestDrive ("capture-successful-job-" + [guid]::NewGuid().ToString('N'))
        $invocationId = 'successful-product-job'
        $run = (Get-RenderExtractBaselineRunPlan -RepeatCount 3 -WarmupPresentedFrameCount 60 -MeasuredPresentedFrameCount 300)[0]
        $sessionId = "$($run.logical_id)-1"
        $childPidPath = Join-Path $TestDrive ("successful-job-child-" + [guid]::NewGuid().ToString('N') + '.txt')
        $parentScriptPath = Join-Path $TestDrive ("successful-job-parent-" + [guid]::NewGuid().ToString('N') + '.ps1')
        $escapedChildPidPath = $childPidPath.Replace("'", "''")
        [IO.File]::WriteAllText(
            $parentScriptPath,
            "`$child = Start-Process powershell.exe -ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 30' -WindowStyle Hidden -PassThru`n[IO.File]::WriteAllText('$escapedChildPidPath', [string]`$child.Id)",
            [Text.UTF8Encoding]::new($false)
        )
        $profileDirectory = Join-Path (Join-Path (Join-Path $outputDirectory 'profiles') $invocationId) $sessionId
        $capturePath = Join-Path (Join-Path (Join-Path $outputDirectory 'captures') $invocationId) "$sessionId.png"
        [IO.Directory]::CreateDirectory($profileDirectory) | Out-Null
        [IO.Directory]::CreateDirectory((Split-Path -Parent $capturePath)) | Out-Null
        foreach ($name in @('timeline.zrtrace.json', 'hotspots.json', 'counter_hotspots.json', 'summary.md')) {
            [IO.File]::WriteAllText((Join-Path $profileDirectory $name), '{}', [Text.UTF8Encoding]::new($false))
        }
        [IO.File]::WriteAllBytes($capturePath, [byte[]](137, 80, 78, 71))
        Mock Assert-RenderExtractFrozenProductInput {
            [pscustomobject]@{
                executable_sha256 = 'A' * 64
                library_sha256 = 'B' * 64
                asset_manifest_sha256 = 'C' * 64
                asset_file_count = 1
                asset_bytes = 1
            }
        }
        Mock Start-RenderExtractBaselineAssignedProcess {
            param($Job, $StartInfo)
            $replacement = [Diagnostics.ProcessStartInfo]::new()
            $replacement.FileName = (Get-Command powershell.exe -ErrorAction Stop).Source
            $replacement.UseShellExecute = $false
            $replacement.CreateNoWindow = $true
            $replacement.WorkingDirectory = $TestDrive
            $replacement.Arguments = "-NoProfile -File `"$parentScriptPath`""
            $Job.StartAssigned($replacement)
        }
        $child = $null
        try {
            $result = Invoke-RenderExtractBaselineProcess `
                -ProfilingInput ([pscustomobject]@{
                        manifest_sha256 = 'D' * 64
                        runtime = [pscustomobject]@{
                            executable_path = (Join-Path $TestDrive 'placeholder-zircon-runtime.exe')
                            executable_sha256 = 'A' * 64
                            library_path = (Join-Path $TestDrive 'placeholder-zircon-runtime.dll')
                            library_sha256 = 'B' * 64
                        }
                    }) `
                -ProjectRoot (Resolve-ZirconWindowsPath -Path $TestDrive) `
                -Run $run `
                -Attempt 1 `
                -InvocationId $invocationId `
                -OutputDirectory $outputDirectory `
                -TimeoutSeconds 10 `
                -MaxProfileFrames 1 `
                -MaxProfileSpans 1 `
                -MaxProfileCounters 1

            $result.exit_code | Should Be 0
            [IO.File]::Exists($childPidPath) | Should Be $true
            $child = Get-Process -Id ([int][IO.File]::ReadAllText($childPidPath)) -ErrorAction SilentlyContinue
            $child | Should BeNullOrEmpty
        }
        finally {
            if ($null -ne $child) {
                if (-not $child.HasExited) {
                    Stop-Process -Id $child.Id -Force -ErrorAction SilentlyContinue
                }
                $child.Dispose()
            }
            if ([IO.Directory]::Exists($outputDirectory)) {
                Remove-Item -LiteralPath $outputDirectory -Recurse -Force
            }
        }
    }

    It 'preserves a product start failure when WPR cleanup also fails' {
        $outputDirectory = Join-Path $TestDrive ("capture-wpr-start-failure-" + [guid]::NewGuid().ToString('N'))
        $missingExecutable = Join-Path $TestDrive 'missing-wpr-zircon-runtime.exe'
        $run = (Get-RenderExtractBaselineRunPlan -RepeatCount 3 -WarmupPresentedFrameCount 60 -MeasuredPresentedFrameCount 300)[0]
        Mock Assert-RenderExtractFrozenProductInput {
            [pscustomobject]@{
                executable_sha256 = 'A' * 64
                library_sha256 = 'B' * 64
                asset_manifest_sha256 = 'C' * 64
                asset_file_count = 1
                asset_bytes = 1
            }
        }
        Mock Start-RenderExtractWprCapture { 'wpr.exe' }
        Mock Stop-RenderExtractWprCapture { throw 'WPR stop failed' }
        Mock Start-RenderExtractBaselineAssignedProcess {
            param($Job, $StartInfo)
            throw "Windows did not start '$($StartInfo.FileName)' in the render-extract process job: fixture launch failure"
        }
        $failure = $null

        try {
            Invoke-RenderExtractBaselineProcess `
                -ProfilingInput ([pscustomobject]@{
                        runtime = [pscustomobject]@{
                            executable_path = $missingExecutable
                            executable_sha256 = 'A' * 64
                            library_path = (Join-Path $TestDrive 'zircon_runtime.dll')
                            library_sha256 = 'B' * 64
                        }
                    }) `
                -ProjectRoot (Resolve-ZirconWindowsPath -Path $TestDrive) `
                -Run $run `
                -Attempt 1 `
                -InvocationId 'wpr-start-failure' `
                -OutputDirectory $outputDirectory `
                -TimeoutSeconds 1 `
                -MaxProfileFrames 1 `
                -MaxProfileSpans 1 `
                -MaxProfileCounters 1 `
                -UseWpr | Out-Null
        }
        catch {
            $failure = $_
        }
        finally {
            if ([IO.Directory]::Exists($outputDirectory)) {
                Remove-Item -LiteralPath $outputDirectory -Recurse -Force
            }
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match 'missing-wpr-zircon-runtime\.exe'
        $failure.Exception.Message | Should Match 'WPR cleanup also failed: WPR stop failed'
    }
}
