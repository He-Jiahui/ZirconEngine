$script:ProductDirectoryScript = Join-Path $PSScriptRoot "..\profiling\ui\ui-profile-product-directory.ps1"
$script:CaptureScript = Join-Path $PSScriptRoot "..\profiling\ui\ui-profile-capture.ps1"
$script:CaptureManifest = Join-Path $PSScriptRoot "..\profiling\shared\profile-capture-manifest.ps1"
$script:ProfilePaths = Join-Path $PSScriptRoot "..\profiling\shared\profile-capture-paths.ps1"

if (Test-Path -LiteralPath $script:ProfilePaths) {
    . $script:ProfilePaths
}
if (Test-Path -LiteralPath $script:ProductDirectoryScript) {
    . $script:ProductDirectoryScript
}
if (Test-Path -LiteralPath $script:CaptureManifest) {
    . $script:CaptureManifest
}

Describe "UI profile product directory" {
    It "accepts an explicitly published editor bundle below an approved artifact root" {
        Get-Command Resolve-ZirconUiProfileProductDirectory -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $bundle = "E:\cargo-targets\editor-profile-$([guid]::NewGuid().ToString('N'))"
        Resolve-ZirconUiProfileProductDirectory `
            -ProductDirectory $bundle `
            -CargoTargetDir "C:\unmanaged-target" |
            Should Be $bundle
    }

    It "falls back to the managed Cargo profiling product when no bundle is supplied" {
        Resolve-ZirconUiProfileProductDirectory `
            -CargoTargetDir "E:\cargo-targets\zircon-engine\pool\fixture" |
            Should Be "E:\cargo-targets\zircon-engine\pool\fixture\profiling"
    }

    It "rejects unmanaged and root-only product directories" {
        foreach ($path in @(
                "C:\zircon-editor-bundle",
                "E:\cargo-targets",
                "E:\cargo-targets-sibling\bundle",
                "E:\cargo-targets\bundle\..\..\escape",
                "E:\ZirconBuilds\legacy-bundle"
            )) {
            {
                Resolve-ZirconUiProfileProductDirectory -ProductDirectory $path
            } | Should Throw "UI profile product directory must resolve below an approved managed product root."
        }
    }

    It "fails closed when neither an explicit bundle nor a managed Cargo target exists" {
        {
            Resolve-ZirconUiProfileProductDirectory -CargoTargetDir ""
        } | Should Throw "ProductDirectory or CARGO_TARGET_DIR must identify a managed UI profiling product."
    }

    It "wires the helper into capture orchestration and source binding" {
        $capture = Get-Content -LiteralPath $script:CaptureScript -Raw
        $capture | Should Match '\[string\]\$ProductDirectory'
        $capture | Should Match 'ui-profile-product-directory\.ps1'
        $capture | Should Match 'Resolve-ZirconUiProfileProductDirectory'

        $toolPaths = @(Get-ZirconProfileCaptureToolPaths)
        ($toolPaths -contains "tools/analysis/profiling/ui/ui-profile-product-directory.ps1") | Should Be $true
    }
}
