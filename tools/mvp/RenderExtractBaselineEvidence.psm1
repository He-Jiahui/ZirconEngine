Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot '..\WindowsPathResolver.psm1') -DisableNameChecking -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
$script:RenderExtractUpperHexDigits = [char[]]'0123456789ABCDEF'

function ConvertTo-RenderExtractEvidenceSha256 {
    param([Parameter(Mandatory)][byte[]]$HashBytes)

    [char[]]$characters = [char[]]::new($HashBytes.Length * 2)
    for ($index = 0; $index -lt $HashBytes.Length; $index++) {
        $value = [int]$HashBytes[$index]
        $characters[$index * 2] = $script:RenderExtractUpperHexDigits[$value -shr 4]
        $characters[($index * 2) + 1] = $script:RenderExtractUpperHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function Get-RenderExtractFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $hasher = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.FileStream]::new($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        return ConvertTo-RenderExtractEvidenceSha256 -HashBytes $hasher.ComputeHash($stream)
    }
    finally {
        $stream.Dispose()
        $hasher.Dispose()
    }
}

function Get-RenderExtractReportProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Get-RenderExtractReportArrayProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value) {
        throw "$Label is missing '$Name'."
    }
    return @($property.Value | Where-Object { $null -ne $_ })
}

function Get-RenderExtractReportProject {
    param([Parameter(Mandatory)]$Summary)

    $project = Get-RenderExtractReportProperty -Value $Summary -Name 'project' -Label 'Baseline summary'
    $runtimeArgument = [string](Get-RenderExtractReportProperty -Value $project -Name 'runtime_argument' -Label 'Baseline project')
    if ($runtimeArgument -ne '.') {
        throw "Baseline project runtime_argument must be '.', got '$runtimeArgument'."
    }
    $physicalIdentity = [string](Get-RenderExtractReportProperty -Value $project -Name 'physical_identity' -Label 'Baseline project')
    if ([string]::IsNullOrWhiteSpace($physicalIdentity)) {
        throw 'Baseline project physical_identity must not be empty.'
    }

    $scaleProjectProperty = $project.PSObject.Properties['scale_project']
    $scaleProject = $null
    if ($null -ne $scaleProjectProperty -and $null -ne $scaleProjectProperty.Value) {
        $primitiveCount = [int](Get-RenderExtractReportProperty `
                -Value $scaleProjectProperty.Value `
                -Name 'primitive_count' `
                -Label 'Baseline scale project')
        if ($primitiveCount -lt 1 -or $primitiveCount -gt 100000) {
            throw "Baseline scale project primitive_count '$primitiveCount' is outside 1..100000."
        }
        $sceneVirtualPath = [string](Get-RenderExtractReportProperty `
                -Value $scaleProjectProperty.Value `
                -Name 'scene_virtual_path' `
                -Label 'Baseline scale project')
        if ($sceneVirtualPath -ne 'res://scenes/main.scene.toml') {
            throw "Baseline scale project scene_virtual_path must be 'res://scenes/main.scene.toml'."
        }
        $scaleProject = [ordered]@{
            primitive_count = $primitiveCount
            scene_virtual_path = $sceneVirtualPath
        }
    }

    return [ordered]@{
        runtime_argument = $runtimeArgument
        physical_identity = $physicalIdentity
        scale_project = $scaleProject
    }
}

function Assert-RenderExtractBaselineEvidenceDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $storage = Resolve-MvpArtifactStoragePath `
        -Path $Path `
        -NamespaceId 'render-extract-baselines'
    return [pscustomobject]@{
        OperationalPath = $storage.operation_path
        DisplayPath = $storage.display_path
        StoragePolicy = $storage
    }
}

function Test-RenderExtractPathWithinDirectory {
    param(
        [Parameter(Mandatory)][string]$CandidatePath,
        [Parameter(Mandatory)][string]$RootPath
    )

    $trimCharacters = [char[]]@([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    $candidate = [IO.Path]::GetFullPath($CandidatePath).TrimEnd($trimCharacters)
    $root = [IO.Path]::GetFullPath($RootPath).TrimEnd($trimCharacters)
    return $candidate.StartsWith("$root$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::OrdinalIgnoreCase)
}

function Write-RenderExtractBaselineReportFileNew {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Content
    )

    $stream = $null
    $writer = $null
    try {
        try {
            $stream = [IO.FileStream]::new(
                $Path,
                [IO.FileMode]::CreateNew,
                [IO.FileAccess]::Write,
                [IO.FileShare]::None
            )
        }
        catch [IO.IOException] {
            throw "Refusing to overwrite existing render-extract report: $Path"
        }
        $writer = [IO.StreamWriter]::new($stream, [Text.UTF8Encoding]::new($false))
        $stream = $null
        $writer.Write($Content)
    }
    finally {
        if ($null -ne $writer) {
            $writer.Dispose()
        }
        elseif ($null -ne $stream) {
            $stream.Dispose()
        }
    }
}

function Get-RenderExtractFileEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Kind,
        [Parameter(Mandatory)][string]$LogicalId,
        [Parameter(Mandatory)][int]$Attempt
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if (-not [IO.File]::Exists($resolution.OperationalPath)) {
        throw "Render-extract baseline run '$LogicalId' attempt $Attempt is missing ${Kind}: $($resolution.DisplayPath)"
    }
    $file = [IO.FileInfo]::new($resolution.OperationalPath)
    if ($file.Length -le 0) {
        throw "Render-extract baseline run '$LogicalId' attempt $Attempt exported an empty ${Kind}: $($resolution.DisplayPath)"
    }
    return [ordered]@{
        logical_id = $LogicalId
        attempt = $Attempt
        kind = $Kind
        path = $resolution.DisplayPath
        bytes = $file.Length
        sha256 = Get-RenderExtractFileSha256 -Path $resolution.OperationalPath
    }
}

function Read-RenderExtractJsonEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if (-not [IO.File]::Exists($resolution.OperationalPath)) {
        throw "$Label does not exist: $($resolution.DisplayPath)"
    }
    [byte[]]$bytes = [IO.File]::ReadAllBytes($resolution.OperationalPath)
    if ($bytes.Length -eq 0) {
        throw "$Label is empty: $($resolution.DisplayPath)"
    }
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $sha256 = ConvertTo-RenderExtractEvidenceSha256 -HashBytes $hasher.ComputeHash($bytes)
    }
    finally {
        $hasher.Dispose()
    }
    try {
        $text = ([Text.UTF8Encoding]::new($false)).GetString($bytes)
        if ($text.Length -gt 0 -and $text[0] -eq [char]0xFEFF) {
            $text = $text.Substring(1)
        }
        $json = $text | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "$Label is not valid JSON: $($resolution.DisplayPath): $($_.Exception.Message)"
    }
    return [pscustomobject]@{
        json = $json
        path = $resolution.DisplayPath
        bytes = [Int64]$bytes.LongLength
        sha256 = $sha256
    }
}

function Get-RenderExtractProcessElapsedMilliseconds {
    param([Parameter(Mandatory)]$Run)

    try {
        $rawElapsed = Get-RenderExtractReportProperty `
                -Value $Run `
                -Name 'process_elapsed_ms' `
                -Label 'Baseline run'
        $numericTypes = @(
            [sbyte], [byte], [Int16], [UInt16], [int], [UInt32], [long], [UInt64],
            [single], [double], [decimal]
        )
        if ($null -eq $rawElapsed -or $numericTypes -notcontains $rawElapsed.GetType()) {
            throw 'value must be a JSON number'
        }
        $elapsed = [double]$rawElapsed
    }
    catch {
        throw "Baseline run has invalid process_elapsed_ms: $($_.Exception.Message)"
    }
    if ([double]::IsNaN($elapsed) -or [double]::IsInfinity($elapsed) -or $elapsed -lt 0) {
        throw 'Baseline run process_elapsed_ms must be a finite nonnegative number.'
    }
    return $elapsed
}

function Get-RenderExtractProcessId {
    [OutputType([Int64])]
    param([Parameter(Mandatory)]$Run)

    try {
        $rawProcessId = Get-RenderExtractReportProperty `
            -Value $Run `
            -Name 'process_id' `
            -Label 'Baseline run'
        $numericTypes = @(
            [sbyte], [byte], [Int16], [UInt16], [int], [UInt32], [long], [UInt64],
            [single], [double], [decimal]
        )
        if ($null -eq $rawProcessId -or $numericTypes -notcontains $rawProcessId.GetType()) {
            throw 'value must be a JSON number'
        }
        $numericProcessId = [double]$rawProcessId
        if ([double]::IsNaN($numericProcessId) -or [double]::IsInfinity($numericProcessId) -or
            $numericProcessId -ne [Math]::Truncate($numericProcessId)) {
            throw 'value must be a finite integer'
        }
        $processId = [Int64]$rawProcessId
    }
    catch {
        throw "Baseline run has invalid process_id: $($_.Exception.Message)"
    }
    if ($processId -lt 1 -or $processId -gt [Int32]::MaxValue) {
        throw 'Baseline run process_id must be a positive Windows process identifier.'
    }
    return $processId
}

Export-ModuleMember -Function @(
    'Get-RenderExtractReportProperty',
    'Get-RenderExtractReportArrayProperty',
    'Get-RenderExtractReportProject',
    'Assert-RenderExtractBaselineEvidenceDirectory',
    'Test-RenderExtractPathWithinDirectory',
    'Write-RenderExtractBaselineReportFileNew',
    'Get-RenderExtractFileEvidence',
    'Read-RenderExtractJsonEvidence',
    'Get-RenderExtractProcessElapsedMilliseconds',
    'Get-RenderExtractProcessId'
)
