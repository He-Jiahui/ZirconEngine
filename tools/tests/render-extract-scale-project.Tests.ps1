$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$generator = Join-Path $repoRoot 'tools\mvp\New-RenderExtractScaleProject.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$manifestModule = Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1'
$artifactStorageModule = Join-Path $repoRoot 'tools\mvp\MvpArtifactStoragePolicy.psm1'
$originalTestMode = $env:RENDER_EXTRACT_SCALE_PROJECT_TEST_MODE

Import-Module $resolverModule -Force -Global -ErrorAction Stop
Import-Module $manifestModule -Force -Global -ErrorAction Stop
Import-Module $artifactStorageModule -Force -Global -ErrorAction Stop

try {
    $env:RENDER_EXTRACT_SCALE_PROJECT_TEST_MODE = '1'
    . $generator
}
finally {
    $env:RENDER_EXTRACT_SCALE_PROJECT_TEST_MODE = $originalTestMode
}

function New-TestRenderExtractScaleSourceIdentity {
    param([Parameter(Mandatory)][string]$Root)

    $snapshotRoot = Join-Path $Root 'source'
    $sourceTemplateRoot = Join-Path $repoRoot 'templates\projects\renderable-empty'
    $snapshotTemplateRoot = Join-Path $snapshotRoot 'templates\projects\renderable-empty'
    [IO.Directory]::CreateDirectory($snapshotTemplateRoot) | Out-Null
    $sourcePrefix = $sourceTemplateRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    foreach ($sourcePath in [IO.Directory]::EnumerateFiles($sourceTemplateRoot, '*', [IO.SearchOption]::AllDirectories)) {
        $relativePath = $sourcePath.Substring($sourcePrefix.Length)
        if ($relativePath.Split([IO.Path]::DirectorySeparatorChar) |
            Where-Object { $_.StartsWith('.', [StringComparison]::Ordinal) }) {
            continue
        }
        $destinationPath = Join-Path $snapshotTemplateRoot $relativePath
        [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($destinationPath)) | Out-Null
        [IO.File]::Copy($sourcePath, $destinationPath, $false)
    }

    $snapshotPrefix = $snapshotRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    $files = @([IO.Directory]::EnumerateFiles($snapshotRoot, '*', [IO.SearchOption]::AllDirectories) |
        ForEach-Object {
            [pscustomobject]@{
                relative_path = $_.Substring($snapshotPrefix.Length).Replace('\', '/')
            }
        } |
        Sort-Object { $_.relative_path })
    $buildSetId = 'B' * 64
    return [pscustomobject]@{
        manifest_path = Join-Path $Root 'render-extract-profiling-inputs.json'
        build_set_id = $buildSetId
        build_set = [pscustomobject]@{
            build_set_id = $buildSetId
            snapshot_root = $snapshotRoot
            files = $files
        }
    }
}

Describe 'Render-extract scale project generator' {
    BeforeEach {
        Import-Module $manifestModule -Force -Global -ErrorAction Stop
        Import-Module $resolverModule -Force -Global -ErrorAction Stop
        Import-Module $artifactStorageModule -Force -Global -ErrorAction Stop
    }

    It 'rejects output roots outside the plan-owned E drive' {
        { Assert-RenderExtractScaleProjectDirectory -Path 'C:\ZirconBuilds\mvp-render-extract-scale-project-rejected' } |
            Should Throw 'approved'
    }

    It 'creates a project with relative virtual asset paths and distinct primitive owners' {
        $projectRoot = New-MvpArtifactStoragePath `
            -NamespaceId 'render-extract-scale-projects' `
            -InstanceId ('scale-project-test-' + [guid]::NewGuid().ToString('N'))
        $profilingInputRoot = Join-Path 'E:\ZirconBuilds' ('render-extract-scale-input-test-' + [guid]::NewGuid().ToString('N'))
        $profilingInput = New-TestRenderExtractScaleSourceIdentity -Root $profilingInputRoot
        try {
            $script:scaleSourceIdentity = [pscustomobject]@{
                build_set_id = $profilingInput.build_set_id
                build_set = $profilingInput.build_set
            }
            Mock Resolve-RenderExtractProfilingSourceIdentity { $script:scaleSourceIdentity }
            $created = New-RenderExtractScaleProject `
                -ProjectRoot $projectRoot `
                -PrimitiveCount 4 `
                -ProfilingInputManifestPath $profilingInput.manifest_path
            $scenePath = Join-Path $projectRoot 'assets\scenes\main.scene.toml'
            $scene = [IO.File]::ReadAllText($scenePath)
            $manifest = [IO.File]::ReadAllText((Join-Path $projectRoot 'render-extract-scale-project.json')) | ConvertFrom-Json

            $created.project_root | Should Be (Resolve-ZirconWindowsPath -Path $projectRoot).DisplayPath
            $created.scene_virtual_path | Should Be 'res://scenes/main.scene.toml'
            $created.build_set_id | Should Be $profilingInput.build_set_id
            $manifest.schema_version | Should Be 2
            $manifest.source_fingerprint | Should Be $profilingInput.build_set_id
            $manifest.build_set_id | Should Be $profilingInput.build_set_id
            $manifest.primitive_count | Should Be 4
            $manifest.scene_virtual_path | Should Be 'res://scenes/main.scene.toml'
            $manifest.model_virtual_path | Should Be 'assets/models/cube.obj'
            $manifest.material_virtual_path | Should Be 'assets/materials/default.zmaterial'
            ([regex]::Matches($scene, '(?m)^\[\[entities\]\]\r?$').Count) | Should Be 6
            ([regex]::Matches($scene, '(?m)^\[entities\.mesh\.model\]\r?$').Count) | Should Be 4
            $scene | Should Match 'name = "Cube_000001"'
            $scene | Should Match 'name = "Cube_000004"'
            $scene | Should Not Match '[A-Z]:\\'
        }
        finally {
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
            if ([IO.Directory]::Exists($profilingInputRoot)) {
                [IO.Directory]::Delete($profilingInputRoot, $true)
            }
        }
    }

    It 'streams the generated scene instead of materializing one complete UTF-8 byte array' {
        $generatorSource = Get-Content -LiteralPath $generator -Raw

        $generatorSource | Should Match 'Write-RenderExtractScaleSceneFileNew'
        $generatorSource | Should Not Match 'GetBytes\s*\(\s*\(Get-RenderExtractScaleSceneContent'
        $generatorSource | Should Match 'RenderExtractSourceIdentity\.psm1'
        $generatorSource | Should Not Match 'Get-MvpSourceFingerprint -RepositoryRoot \$repoRoot'
    }

    It 'rejects a profiling manifest that does not identify its verified BuildSet' {
        $projectRoot = New-MvpArtifactStoragePath `
            -NamespaceId 'render-extract-scale-projects' `
            -InstanceId ('scale-project-invalid-fingerprint-' + [guid]::NewGuid().ToString('N'))
        try {
            Mock Resolve-RenderExtractProfilingSourceIdentity {
                throw 'Profiling input source_fingerprint must equal its verified BuildSetId.'
            }

            {
                New-RenderExtractScaleProject `
                    -ProjectRoot $projectRoot `
                    -PrimitiveCount 1 `
                    -ProfilingInputManifestPath 'E:\ZirconBuilds\invalid-profiling-input.json'
            } | Should Throw 'verified BuildSetId'
            [IO.Directory]::Exists($projectRoot) | Should Be $false
        }
        finally {
            if ([IO.Directory]::Exists($projectRoot)) {
                [IO.Directory]::Delete($projectRoot, $true)
            }
        }
    }
}
