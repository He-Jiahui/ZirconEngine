$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$generator = Join-Path $repoRoot 'tools\mvp\New-RenderExtractScaleProject.ps1'
$resolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
$manifestModule = Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1'
$originalTestMode = $env:RENDER_EXTRACT_SCALE_PROJECT_TEST_MODE

Import-Module $resolverModule -Force -Global -ErrorAction Stop
Import-Module $manifestModule -Force -Global -ErrorAction Stop

try {
    $env:RENDER_EXTRACT_SCALE_PROJECT_TEST_MODE = '1'
    . $generator
}
finally {
    $env:RENDER_EXTRACT_SCALE_PROJECT_TEST_MODE = $originalTestMode
}

Describe 'Render-extract scale project generator' {
    BeforeEach {
        Import-Module $manifestModule -Force -Global -ErrorAction Stop
        Import-Module $resolverModule -Force -Global -ErrorAction Stop
    }

    It 'rejects output roots outside the plan-owned E drive' {
        { Assert-RenderExtractScaleProjectDirectory -Path 'C:\ZirconBuilds\mvp-perf-projects\scale' } |
            Should Throw 'mvp-perf-projects'
    }

    It 'creates a project with relative virtual asset paths and distinct primitive owners' {
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-perf-projects' ('scale-project-test-' + [guid]::NewGuid().ToString('N'))
        try {
            $created = New-RenderExtractScaleProject `
                -ProjectRoot $projectRoot `
                -PrimitiveCount 4 `
                -SourceFingerprint ('A' * 64)
            $scenePath = Join-Path $projectRoot 'assets\scenes\main.scene.toml'
            $scene = [IO.File]::ReadAllText($scenePath)
            $manifest = [IO.File]::ReadAllText((Join-Path $projectRoot 'render-extract-scale-project.json')) | ConvertFrom-Json

            $created.project_root | Should Be (Resolve-ZirconWindowsPath -Path $projectRoot).DisplayPath
            $created.scene_virtual_path | Should Be 'res://scenes/main.scene.toml'
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
        }
    }

    It 'streams the generated scene instead of materializing one complete UTF-8 byte array' {
        $generatorSource = Get-Content -LiteralPath $generator -Raw

        $generatorSource | Should Match 'Write-RenderExtractScaleSceneFileNew'
        $generatorSource | Should Not Match 'GetBytes\s*\(\s*\(Get-RenderExtractScaleSceneContent'
    }

    It 'rejects a malformed source snapshot before publishing a scale project' {
        $projectRoot = Join-Path 'E:\ZirconBuilds\mvp-perf-projects' ('scale-project-invalid-fingerprint-' + [guid]::NewGuid().ToString('N'))

        { New-RenderExtractScaleProject -ProjectRoot $projectRoot -PrimitiveCount 1 -SourceFingerprint 'invalid' } |
            Should Throw 'source fingerprint'
        [IO.Directory]::Exists($projectRoot) | Should Be $false
    }
}
