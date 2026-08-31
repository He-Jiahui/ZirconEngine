$script:ProfileCaptureManifest = Join-Path $PSScriptRoot "..\profile-capture-manifest.ps1"
$script:ToolchainScript = Join-Path $PSScriptRoot "..\shader-pbr-profile-toolchain.ps1"

if (Test-Path -LiteralPath $script:ProfileCaptureManifest) {
    . $script:ProfileCaptureManifest
}
if (Test-Path -LiteralPath $script:ToolchainScript) {
    . $script:ToolchainScript
}

Describe "shader PBR capture toolchain contract" {
    It "requires a versioned backend policy and pinned RenderDoc capture and replay binaries" {
        Get-Command Resolve-ZirconShaderPbrCaptureToolchain -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $renderDocDll = Join-Path $TestDrive "renderdoc.dll"
        Set-Content -LiteralPath $renderDocDll -Value "renderdoc fixture" -Encoding UTF8
        $renderDocFingerprint = Get-ZirconProfileRequiredFileFingerprint `
            -Path $renderDocDll `
            -Description "RenderDoc fixture"
        $renderDocCommand = Join-Path $TestDrive "renderdoccmd.exe"
        Set-Content -LiteralPath $renderDocCommand -Value "renderdoc command fixture" -Encoding UTF8
        $renderDocCommandFingerprint = Get-ZirconProfileRequiredFileFingerprint `
            -Path $renderDocCommand `
            -Description "RenderDoc command fixture"
        $toolchainPath = Join-Path $TestDrive "capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "dx12"
                evidence_backend = "wgpu(dx12)"
                permitted_backends = @("dx12")
                unsupported_backends = @("vulkan", "gl", "metal")
            }
            renderdoc = [ordered]@{
                dll = $renderDocFingerprint
                command = $renderDocCommandFingerprint
            }
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        $toolchain = Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath

        $toolchain.manifest.sha256 | Should Match '^[0-9a-f]{64}$'
        $toolchain.graphics.wgpu_backend | Should Be "dx12"
        $toolchain.graphics.evidence_backend | Should Be "wgpu(dx12)"
        $toolchain.renderdoc.dll.sha256 | Should Be $renderDocFingerprint.sha256
        $toolchain.renderdoc.command.sha256 | Should Be $renderDocCommandFingerprint.sha256

        Set-Content -LiteralPath $renderDocDll -Value "replaced RenderDoc fixture" -Encoding UTF8
        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "does not match its pinned fingerprint"
    }

    It "rejects a pinned capture library that is not the RenderDoc DLL" {
        $otherDll = Join-Path $TestDrive "other-capture.dll"
        Set-Content -LiteralPath $otherDll -Value "other capture fixture" -Encoding UTF8
        $renderDocCommand = Join-Path $TestDrive "renderdoccmd.exe"
        Set-Content -LiteralPath $renderDocCommand -Value "renderdoc command fixture" -Encoding UTF8
        $toolchainPath = Join-Path $TestDrive "capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "dx12"
                evidence_backend = "wgpu(dx12)"
                permitted_backends = @("dx12")
                unsupported_backends = @("vulkan", "gl", "metal")
            }
            renderdoc = [ordered]@{
                dll = (Get-ZirconProfileRequiredFileFingerprint -Path $otherDll -Description "other capture fixture")
                command = (Get-ZirconProfileRequiredFileFingerprint -Path $renderDocCommand -Description "RenderDoc command fixture")
            }
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "must be named renderdoc.dll"
    }

    It "rejects a replay command that differs from its pinned fingerprint" {
        $renderDocDll = Join-Path $TestDrive "renderdoc.dll"
        Set-Content -LiteralPath $renderDocDll -Value "renderdoc fixture" -Encoding UTF8
        $renderDocCommand = Join-Path $TestDrive "renderdoccmd.exe"
        Set-Content -LiteralPath $renderDocCommand -Value "renderdoc command fixture" -Encoding UTF8
        $toolchainPath = Join-Path $TestDrive "capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "dx12"
                evidence_backend = "wgpu(dx12)"
                permitted_backends = @("dx12")
                unsupported_backends = @("vulkan", "gl", "metal")
            }
            renderdoc = [ordered]@{
                dll = (Get-ZirconProfileRequiredFileFingerprint -Path $renderDocDll -Description "RenderDoc fixture")
                command = (Get-ZirconProfileRequiredFileFingerprint -Path $renderDocCommand -Description "RenderDoc command fixture")
            }
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        Set-Content -LiteralPath $renderDocCommand -Value "replaced renderdoc command fixture" -Encoding UTF8
        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "does not match its pinned fingerprint"
    }

    It "rejects a pinned replay executable that is not the RenderDoc command" {
        $renderDocDll = Join-Path $TestDrive "renderdoc.dll"
        Set-Content -LiteralPath $renderDocDll -Value "renderdoc fixture" -Encoding UTF8
        $otherCommand = Join-Path $TestDrive "other-replay.exe"
        Set-Content -LiteralPath $otherCommand -Value "other replay fixture" -Encoding UTF8
        $toolchainPath = Join-Path $TestDrive "capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "dx12"
                evidence_backend = "wgpu(dx12)"
                permitted_backends = @("dx12")
                unsupported_backends = @("vulkan", "gl", "metal")
            }
            renderdoc = [ordered]@{
                dll = (Get-ZirconProfileRequiredFileFingerprint -Path $renderDocDll -Description "RenderDoc fixture")
                command = (Get-ZirconProfileRequiredFileFingerprint -Path $otherCommand -Description "other replay fixture")
            }
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "must be named renderdoccmd.exe"
    }

    It "rejects a RenderDoc capture declaration without a pinned replay command" {
        $renderDocDll = Join-Path $TestDrive "renderdoc.dll"
        Set-Content -LiteralPath $renderDocDll -Value "renderdoc fixture" -Encoding UTF8
        $toolchainPath = Join-Path $TestDrive "capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "dx12"
                evidence_backend = "wgpu(dx12)"
                permitted_backends = @("dx12")
                unsupported_backends = @("vulkan", "gl", "metal")
            }
            renderdoc = [ordered]@{
                dll = (Get-ZirconProfileRequiredFileFingerprint -Path $renderDocDll -Description "RenderDoc fixture")
            }
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "missing RenderDoc replay command"
    }

    It "rejects a backend outside its explicit policy" {
        Get-Command Resolve-ZirconShaderPbrCaptureToolchain -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $toolchainPath = Join-Path $TestDrive "invalid-capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "vulkan"
                evidence_backend = "wgpu(vulkan)"
                permitted_backends = @("dx12")
                unsupported_backends = @("vulkan")
            }
            renderdoc = $null
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "backend policy"
    }

    It "rejects a selector used where the renderer evidence name is required" {
        $toolchainPath = Join-Path $TestDrive "selector-labelled-capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "dx12"
                evidence_backend = "dx12"
                permitted_backends = @("dx12")
                unsupported_backends = @("vulkan", "gl", "metal")
            }
            renderdoc = $null
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "must be 'wgpu(dx12)'"
    }

    It "rejects a multi-backend selector that cannot bind one adapter evidence name" {
        $toolchainPath = Join-Path $TestDrive "multi-backend-capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "dx12,vulkan"
                evidence_backend = "wgpu(dx12,vulkan)"
                permitted_backends = @("dx12,vulkan")
                unsupported_backends = @("gl", "metal")
            }
            renderdoc = $null
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "unsupported WGPU backend selector"
    }

    It "rejects evidence labelled for a different graphics backend" {
        $toolchainPath = Join-Path $TestDrive "mixed-backend-capture-toolchain.json"
        [ordered]@{
            schema_version = 2
            toolchain_kind = "zircon_shader_pbr_capture_toolchain"
            graphics = [ordered]@{
                wgpu_backend = "dx12"
                evidence_backend = "wgpu(vulkan)"
                permitted_backends = @("dx12")
                unsupported_backends = @("vulkan", "gl", "metal")
            }
            renderdoc = $null
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $toolchainPath -Encoding UTF8

        {
            Resolve-ZirconShaderPbrCaptureToolchain -ManifestPath $toolchainPath
        } | Should Throw "must be 'wgpu(dx12)'"
    }
}
