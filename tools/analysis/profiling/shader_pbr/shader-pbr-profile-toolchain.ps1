Set-StrictMode -Version Latest

$script:ZirconShaderPbrCaptureToolchainSchemaVersion = 2
$script:ZirconShaderPbrCaptureToolchainKind = "zircon_shader_pbr_capture_toolchain"
$script:ZirconShaderPbrSupportedWgpuBackendSelectors = @("vulkan", "metal", "dx12", "gl", "webgpu")

function Get-ZirconShaderPbrToolchainRequiredProperty {
    param(
        [Parameter(Mandatory = $true)]$Object,
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "Shader PBR capture toolchain is missing $Description."
    }
    return $property.Value
}

function Get-ZirconShaderPbrToolchainRequiredString {
    param(
        [Parameter(Mandatory = $true)]$Object,
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $value = Get-ZirconShaderPbrToolchainRequiredProperty `
        -Object $Object `
        -Name $Name `
        -Description $Description
    if ([string]::IsNullOrWhiteSpace([string]$value)) {
        throw "Shader PBR capture toolchain has an empty $Description."
    }
    return [string]$value
}

function Get-ZirconShaderPbrToolchainRequiredStringSet {
    param(
        [Parameter(Mandatory = $true)]$Object,
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $value = Get-ZirconShaderPbrToolchainRequiredProperty `
        -Object $Object `
        -Name $Name `
        -Description $Description
    $values = @($value)
    if ($values.Count -eq 0) {
        throw "Shader PBR capture toolchain has an empty $Description."
    }
    $set = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($entry in $values) {
        if ([string]::IsNullOrWhiteSpace([string]$entry) -or -not $set.Add([string]$entry)) {
            throw "Shader PBR capture toolchain has an invalid $Description."
        }
    }
    return @($set | Sort-Object)
}

function Assert-ZirconShaderPbrToolchainFingerprint {
    param(
        [Parameter(Mandatory = $true)]$Fingerprint,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $declaredPath = Get-ZirconShaderPbrToolchainRequiredString `
        -Object $Fingerprint `
        -Name "path" `
        -Description "$Description path"
    if (-not [System.IO.Path]::IsPathRooted($declaredPath)) {
        throw "Shader PBR capture toolchain $Description path must be absolute."
    }
    $declaredSha256 = Get-ZirconShaderPbrToolchainRequiredString `
        -Object $Fingerprint `
        -Name "sha256" `
        -Description "$Description SHA-256"
    if ($declaredSha256 -notmatch '^[0-9a-f]{64}$') {
        throw "Shader PBR capture toolchain $Description SHA-256 is invalid."
    }
    $declaredByteLength = Get-ZirconShaderPbrToolchainRequiredProperty `
        -Object $Fingerprint `
        -Name "byte_length" `
        -Description "$Description byte length"
    try {
        $declaredByteLength = [int64]$declaredByteLength
    }
    catch {
        throw "Shader PBR capture toolchain $Description byte length is invalid."
    }
    if ($declaredByteLength -lt 1) {
        throw "Shader PBR capture toolchain $Description byte length is invalid."
    }
    $actual = Get-ZirconProfileRequiredFileFingerprint -Path $declaredPath -Description $Description
    if ([System.IO.Path]::GetFullPath($declaredPath) -ne [string]$actual.path -or
        $declaredSha256 -ne [string]$actual.sha256 -or
        $declaredByteLength -ne [int64]$actual.byte_length) {
        throw "Shader PBR capture toolchain $Description does not match its pinned fingerprint."
    }
    return $actual
}

function Resolve-ZirconShaderPbrCaptureToolchain {
    param([Parameter(Mandatory = $true)][string]$ManifestPath)

    $manifestFingerprint = Get-ZirconProfileRequiredFileFingerprint `
        -Path $ManifestPath `
        -Description "Shader PBR capture toolchain manifest"
    try {
        $manifest = Get-Content -LiteralPath $manifestFingerprint.path -Raw | ConvertFrom-Json
    }
    catch {
        throw "Shader PBR capture toolchain manifest is malformed: $($manifestFingerprint.path)"
    }
    if ($null -eq $manifest -or
        $manifest.schema_version -ne $script:ZirconShaderPbrCaptureToolchainSchemaVersion -or
        [string]$manifest.toolchain_kind -ne $script:ZirconShaderPbrCaptureToolchainKind) {
        throw "Shader PBR capture toolchain manifest has an unexpected schema: $($manifestFingerprint.path)"
    }

    $graphics = Get-ZirconShaderPbrToolchainRequiredProperty `
        -Object $manifest `
        -Name "graphics" `
        -Description "graphics policy"
    $wgpuBackend = Get-ZirconShaderPbrToolchainRequiredString `
        -Object $graphics `
        -Name "wgpu_backend" `
        -Description "WGPU backend"
    $evidenceBackend = Get-ZirconShaderPbrToolchainRequiredString `
        -Object $graphics `
        -Name "evidence_backend" `
        -Description "evidence backend"
    if ($wgpuBackend -cnotin $script:ZirconShaderPbrSupportedWgpuBackendSelectors) {
        throw "Shader PBR capture toolchain has an unsupported WGPU backend selector '$wgpuBackend'."
    }
    $permittedBackends = Get-ZirconShaderPbrToolchainRequiredStringSet `
        -Object $graphics `
        -Name "permitted_backends" `
        -Description "permitted backends"
    $unsupportedBackends = Get-ZirconShaderPbrToolchainRequiredStringSet `
        -Object $graphics `
        -Name "unsupported_backends" `
        -Description "unsupported backends"
    if ($wgpuBackend -notin $permittedBackends -or $wgpuBackend -in $unsupportedBackends) {
        throw "Shader PBR capture toolchain backend policy does not permit '$wgpuBackend'."
    }
    if (@($permittedBackends | Where-Object { $_ -in $unsupportedBackends }).Count -ne 0) {
        throw "Shader PBR capture toolchain backend policy overlaps permitted and unsupported backends."
    }
    # The selector reaches WGPU_BACKEND; the viewer records RenderBackend::backend_name().
    $expectedEvidenceBackend = "wgpu($($wgpuBackend.ToLowerInvariant()))"
    if (-not [string]::Equals(
            $evidenceBackend,
            $expectedEvidenceBackend,
            [System.StringComparison]::Ordinal
        )) {
        throw "Shader PBR capture toolchain evidence backend must be '$expectedEvidenceBackend' for WGPU backend '$wgpuBackend'."
    }

    $renderDoc = $null
    $renderDocProperty = $manifest.PSObject.Properties["renderdoc"]
    if ($null -ne $renderDocProperty -and $null -ne $renderDocProperty.Value) {
        $renderDocObject = $renderDocProperty.Value
        $renderDocDll = Assert-ZirconShaderPbrToolchainFingerprint `
            -Fingerprint (Get-ZirconShaderPbrToolchainRequiredProperty `
                -Object $renderDocObject `
                -Name "dll" `
                -Description "RenderDoc DLL") `
            -Description "RenderDoc DLL"
        if ([System.IO.Path]::GetExtension([string]$renderDocDll.path) -ne ".dll") {
            throw "Shader PBR capture toolchain RenderDoc DLL must have a .dll extension."
        }
        if (-not [string]::Equals(
                [System.IO.Path]::GetFileName([string]$renderDocDll.path),
                "renderdoc.dll",
                [System.StringComparison]::OrdinalIgnoreCase
            )) {
            throw "Shader PBR capture toolchain RenderDoc DLL must be named renderdoc.dll."
        }
        $renderDocCommand = Assert-ZirconShaderPbrToolchainFingerprint `
            -Fingerprint (Get-ZirconShaderPbrToolchainRequiredProperty `
                -Object $renderDocObject `
                -Name "command" `
                -Description "RenderDoc replay command") `
            -Description "RenderDoc replay command"
        if ([System.IO.Path]::GetExtension([string]$renderDocCommand.path) -ne ".exe") {
            throw "Shader PBR capture toolchain RenderDoc replay command must have a .exe extension."
        }
        if (-not [string]::Equals(
                [System.IO.Path]::GetFileName([string]$renderDocCommand.path),
                "renderdoccmd.exe",
                [System.StringComparison]::OrdinalIgnoreCase
            )) {
            throw "Shader PBR capture toolchain RenderDoc replay command must be named renderdoccmd.exe."
        }
        $renderDoc = [pscustomobject]@{
            dll = $renderDocDll
            command = $renderDocCommand
        }
    }

    return [pscustomobject]@{
        manifest = $manifestFingerprint
        graphics = [pscustomobject]@{
            wgpu_backend = $wgpuBackend
            evidence_backend = $evidenceBackend
            permitted_backends = @($permittedBackends)
            unsupported_backends = @($unsupportedBackends)
        }
        renderdoc = $renderDoc
    }
}
