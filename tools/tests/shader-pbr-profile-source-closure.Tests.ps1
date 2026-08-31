$script:ShaderPbrProfileSourceClosure = Join-Path $PSScriptRoot "..\shader-pbr-profile-source-closure.ps1"
$script:ShaderPbrProfileContract = Join-Path $PSScriptRoot "..\shader-pbr-profile-contract.ps1"

if (Test-Path -LiteralPath $script:ShaderPbrProfileSourceClosure) {
    . $script:ShaderPbrProfileSourceClosure
}
if (Test-Path -LiteralPath $script:ShaderPbrProfileContract) {
    . $script:ShaderPbrProfileContract
}

Describe "shader PBR viewer source closure" {
    It "discovers production Rust modules and excludes cfg(test) modules" {
        Get-Command Get-ZirconShaderPbrViewerProductionSourceClosure -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $viewerRoot = Join-Path $TestDrive "zircon_app\src\bin\zircon_shader_pbr_viewer"
        New-Item -ItemType Directory -Force -Path (Join-Path $viewerRoot "app") | Out-Null
        @"
mod app;
mod args; // a production module may retain a trailing comment.
#[cfg(test)]
mod app_tests;
#[cfg(test)] mod inline_app_tests;
#[path = "inline_path.rs"] mod inline_path;
"@ | Set-Content -LiteralPath (Join-Path $viewerRoot "main.rs") -Encoding UTF8
        @"
#[path = "base_pipeline_recheck.rs"]
mod base_pipeline_recheck;
mod state;
"@ | Set-Content -LiteralPath (Join-Path $viewerRoot "app.rs") -Encoding UTF8
        "" | Set-Content -LiteralPath (Join-Path $viewerRoot "args.rs") -Encoding UTF8
        "" | Set-Content -LiteralPath (Join-Path $viewerRoot "base_pipeline_recheck.rs") -Encoding UTF8
        "" | Set-Content -LiteralPath (Join-Path $viewerRoot "inline_path.rs") -Encoding UTF8
        "" | Set-Content -LiteralPath (Join-Path $viewerRoot "app\state.rs") -Encoding UTF8
        "" | Set-Content -LiteralPath (Join-Path $viewerRoot "app_tests.rs") -Encoding UTF8
        "" | Set-Content -LiteralPath (Join-Path $viewerRoot "inline_app_tests.rs") -Encoding UTF8

        $actual = @(Get-ZirconShaderPbrViewerProductionSourceClosure -RepoRoot $TestDrive)
        $expected = @(
            "zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs",
            "zircon_app/src/bin/zircon_shader_pbr_viewer/app/state.rs",
            "zircon_app/src/bin/zircon_shader_pbr_viewer/args.rs",
            "zircon_app/src/bin/zircon_shader_pbr_viewer/base_pipeline_recheck.rs",
            "zircon_app/src/bin/zircon_shader_pbr_viewer/inline_path.rs",
            "zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs"
        )
        ($actual -join "`n") | Should Be ($expected -join "`n")
    }

    It "rejects a production module that cannot be resolved inside the viewer root" {
        $viewerRoot = Join-Path $TestDrive "zircon_app\src\bin\zircon_shader_pbr_viewer"
        New-Item -ItemType Directory -Force -Path $viewerRoot | Out-Null
        "mod missing;" | Set-Content -LiteralPath (Join-Path $viewerRoot "main.rs") -Encoding UTF8

        {
            Get-ZirconShaderPbrViewerProductionSourceClosure -RepoRoot $TestDrive
        } | Should Throw "cannot resolve production module 'missing'"
    }

    It "emits a content-bound manifest for the complete production closure" {
        $viewerRoot = Join-Path $TestDrive "zircon_app\src\bin\zircon_shader_pbr_viewer"
        New-Item -ItemType Directory -Force -Path $viewerRoot | Out-Null
        New-Item -ItemType Directory -Force -Path (Join-Path $viewerRoot "app") | Out-Null
        "mod app;" | Set-Content -LiteralPath (Join-Path $viewerRoot "main.rs") -Encoding UTF8
        "pub(crate) mod render;" | Set-Content -LiteralPath (Join-Path $viewerRoot "app.rs") -Encoding UTF8
        "" | Set-Content -LiteralPath (Join-Path $viewerRoot "app\render.rs") -Encoding UTF8

        $actual = @(Get-ZirconShaderPbrViewerProductionSourceManifest -RepoRoot $TestDrive)
        $expectedPaths = @(
            "zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs",
            "zircon_app/src/bin/zircon_shader_pbr_viewer/app/render.rs",
            "zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs"
        )

        (@($actual.relative_path) -join "`n") | Should Be ($expectedPaths -join "`n")
        foreach ($record in $actual) {
            $sourcePath = Join-Path $TestDrive $record.relative_path
            [int64]$record.byte_length | Should Be ([int64](Get-Item -LiteralPath $sourcePath).Length)
            $record.sha256 | Should Be (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    }

    It "includes the complete recursive viewer closure in the critical source set" {
        $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
        $viewerPaths = @(Get-ZirconShaderPbrViewerProductionSourceClosure -RepoRoot $repoRoot)
        $criticalPaths = @(Get-ZirconShaderPbrProfileCriticalSourcePaths -RepoRoot $repoRoot)
        $criticalViewerPaths = @($criticalPaths | Where-Object {
            $_ -like "zircon_app/src/bin/zircon_shader_pbr_viewer/*"
        })

        ($criticalViewerPaths -join "`n") | Should Be ($viewerPaths -join "`n")
        $criticalPaths.Count | Should BeGreaterThan $viewerPaths.Count
        $criticalPaths.Count | Should Be ($criticalPaths | Select-Object -Unique).Count
    }

}
