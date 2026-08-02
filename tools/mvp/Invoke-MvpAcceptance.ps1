[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$StagingRoot,
    [Parameter(Mandatory)]
    [string]$EvidenceRoot,
    [string]$ExpectedSourceFingerprint,
    [string]$ProfileContractSummaryPath,
    [string]$WorkspaceSummaryPath,
    [switch]$RequireProjectCreationEvidence,
    [switch]$RequireAuthoringAutomation,
    [switch]$RequireReopenAutomation,
    [switch]$RequireProductEvidence,
    [switch]$RequireF5Evidence,
    [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($RequireF5Evidence) {
    # The F5 product claim is indivisible: creation, authoring, reopen, and visual evidence
    # must all describe the same staged project rather than being selected independently.
    $RequireProjectCreationEvidence = $true
    $RequireAuthoringAutomation = $true
    $RequireReopenAutomation = $true
    $RequireProductEvidence = $true
    if ([string]::IsNullOrWhiteSpace($ProfileContractSummaryPath)) {
        throw 'RequireF5Evidence requires an explicit ProfileContractSummaryPath.'
    }
    if ([string]::IsNullOrWhiteSpace($WorkspaceSummaryPath)) {
        throw 'RequireF5Evidence requires an explicit WorkspaceSummaryPath.'
    }
}

Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceNativeFileSystem.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpBuildSummaryEvidence.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceStagingProjection.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpAcceptanceStagingSnapshot.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProjectOpenEvidence.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProjectSaveEvidence.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpPersistenceComparison.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpScenePersistenceEvidence.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpStagingPreflightEvidence.psm1') -Force -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'MvpProcessTimingEvidence.psm1') -Force -ErrorAction Stop

function Get-MvpFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($hasher.ComputeHash($stream) | ForEach-Object { $_.ToString('X2') })
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Resolve-MvpStagedEvidenceFile {
    param(
        [Parameter(Mandatory)][string]$RelativePath,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$Label
    )

    if ([IO.Path]::IsPathRooted($RelativePath) -or $RelativePath -match '(^|[\\/])\.\.([\\/]|$)') {
        throw "$Label has unsafe relative path '$RelativePath'."
    }
    $candidate = Join-Path $StagingRoot $RelativePath
    if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) {
        throw "$Label '$RelativePath' does not exist in the staging root."
    }
    $resolved = (Resolve-Path -LiteralPath $candidate).Path
    $prefix = $StagingRoot.TrimEnd([char[]]@('\', '/')) + [IO.Path]::DirectorySeparatorChar
    if (-not $resolved.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label '$RelativePath' escapes the staging root."
    }
    return $resolved
}

function Assert-MvpStagedFileEvidence {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$Label
    )

    $relativePath = [string](Get-MvpRequiredProperty -Value $Evidence -Name 'path' -Label $Label)
    $path = Resolve-MvpStagedEvidenceFile -RelativePath $relativePath -StagingRoot $StagingRoot -Label $Label
    $expectedHash = [string](Get-MvpRequiredProperty -Value $Evidence -Name 'sha256' -Label $Label)
    if ($expectedHash -notmatch '^[0-9A-Fa-f]{64}$') {
        throw "$Label has invalid sha256 '$expectedHash'."
    }
    $actualHash = Get-MvpFileSha256 -Path $path
    if (-not $actualHash.Equals($expectedHash, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label hash mismatch for '$relativePath'."
    }
    $expectedSize = [Int64](Get-MvpRequiredProperty -Value $Evidence -Name 'size_bytes' -Label $Label)
    $actualSize = (Get-Item -LiteralPath $path).Length
    if ($actualSize -ne $expectedSize) {
        throw "$Label size mismatch for '$relativePath'."
    }
    return $path
}

function Get-MvpAcceptancePngEvidence {
    param([Parameter(Mandatory)][string]$Path)

    if ($null -eq ('ZirconMvpAcceptancePngEvidence' -as [type])) {
        Add-Type -AssemblyName System.Drawing -ErrorAction Stop
        $drawingReferences = @(
            [Drawing.Bitmap].Assembly.Location
            [Drawing.Rectangle].Assembly.Location
            [Security.Cryptography.SHA256].Assembly.Location
        ) | Select-Object -Unique
        Add-Type -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Security.Cryptography;

public sealed class ZirconMvpAcceptancePngEvidence
{
    public int Width { get; private set; }
    public int Height { get; private set; }
    public long NonBackgroundPixels { get; private set; }
    public long NonTransparentPixels { get; private set; }
    public string PixelSha256 { get; private set; }

    public static ZirconMvpAcceptancePngEvidence Inspect(string path)
    {
        using (var source = new Bitmap(path))
        using (var normalized = new Bitmap(source.Width, source.Height, PixelFormat.Format32bppArgb))
        {
            if (source.Width <= 0 || source.Height <= 0)
            {
                throw new InvalidOperationException("PNG dimensions must be positive.");
            }
            using (var graphics = Graphics.FromImage(normalized))
            {
                graphics.DrawImageUnscaled(source, 0, 0);
            }
            var bounds = new Rectangle(0, 0, normalized.Width, normalized.Height);
            var bitmapData = normalized.LockBits(bounds, ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
            try
            {
                var stride = bitmapData.Stride;
                var bytes = checked(Math.Abs(stride) * normalized.Height);
                var pixels = new byte[bytes];
                Marshal.Copy(bitmapData.Scan0, pixels, 0, bytes);
                var evidence = new ZirconMvpAcceptancePngEvidence
                {
                    Width = normalized.Width,
                    Height = normalized.Height,
                };
                var canonicalPixels = new byte[checked(normalized.Width * normalized.Height * 4)];
                var background = 0;
                var backgroundSet = false;
                for (var y = 0; y < normalized.Height; y++)
                {
                    var row = stride >= 0 ? y * stride : (normalized.Height - 1 - y) * -stride;
                    for (var x = 0; x < normalized.Width; x++)
                    {
                        var offset = row + x * 4;
                        var argb = pixels[offset] | (pixels[offset + 1] << 8) |
                            (pixels[offset + 2] << 16) | (pixels[offset + 3] << 24);
                        if (!backgroundSet)
                        {
                            background = argb;
                            backgroundSet = true;
                        }
                        if (argb != background)
                        {
                            evidence.NonBackgroundPixels++;
                        }
                    if (pixels[offset + 3] != 0)
                    {
                        evidence.NonTransparentPixels++;
                    }
                    Buffer.BlockCopy(pixels, offset, canonicalPixels, (y * normalized.Width + x) * 4, 4);
                }
            }
            using (var hasher = SHA256.Create())
            {
                evidence.PixelSha256 = BitConverter.ToString(hasher.ComputeHash(canonicalPixels)).Replace("-", string.Empty);
            }
            return evidence;
            }
            finally
            {
                normalized.UnlockBits(bitmapData);
            }
        }
    }
}
'@ -ReferencedAssemblies $drawingReferences -ErrorAction Stop
    }

    return [ZirconMvpAcceptancePngEvidence]::Inspect($Path)
}

function Get-MvpRequiredProperty {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property -or $null -eq $property.Value -or
        ($property.Value -is [string] -and [string]::IsNullOrWhiteSpace($property.Value))) {
        throw "$Label is missing '$Name'."
    }
    return $property.Value
}

function Read-MvpJsonObject {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label '$Path' does not exist."
    }
    try {
        $json = Get-Content -LiteralPath $Path -Raw -Encoding UTF8
        $convertFromJson = Get-Command ConvertFrom-Json -ErrorAction Stop
        if ($convertFromJson.Parameters.ContainsKey('DateKind')) {
            return $json | ConvertFrom-Json -DateKind String
        }
        return $json | ConvertFrom-Json
    }
    catch {
        throw "$Label '$Path' is not valid JSON: $($_.Exception.Message)"
    }
}

function Read-MvpProcessJournal {
    param([Parameter(Mandatory)][string]$StagingRoot)

    $journalPath = Resolve-MvpStagedEvidenceFile `
        -RelativePath 'logs/process-execution-journal.jsonl' `
        -StagingRoot $StagingRoot `
        -Label 'Process execution journal'
    $entries = @{}
    $lines = @(Get-Content -LiteralPath $journalPath -Encoding UTF8)
    if ($lines.Count -eq 0) {
        throw 'Process execution journal contains no entries.'
    }
    for ($index = 0; $index -lt $lines.Count; $index++) {
        $line = [string]$lines[$index]
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }
        try {
            $entry = $line | ConvertFrom-Json -ErrorAction Stop
        }
        catch {
            throw "Process execution journal line $($index + 1) is not valid JSON: $($_.Exception.Message)"
        }
        $phase = [string](Get-MvpRequiredProperty -Value $entry -Name 'phase' -Label "Process execution journal line $($index + 1)")
        if ($entries.ContainsKey($phase)) {
            throw "Process execution journal contains duplicate phase '$phase'."
        }
        foreach ($propertyName in @('started_at_utc', 'ended_at_utc', 'exit_code', 'outcome')) {
            $null = Get-MvpRequiredProperty -Value $entry -Name $propertyName -Label "Process execution journal phase '$phase'"
        }
        [DateTimeOffset]$startedAt = [DateTimeOffset]::MinValue
        [DateTimeOffset]$endedAt = [DateTimeOffset]::MinValue
        if (-not [DateTimeOffset]::TryParse([string]$entry.started_at_utc, [ref]$startedAt) -or
            -not [DateTimeOffset]::TryParse([string]$entry.ended_at_utc, [ref]$endedAt) -or
            $endedAt -lt $startedAt) {
            throw "Process execution journal phase '$phase' has invalid timing evidence."
        }
        $entries[$phase] = $entry
    }
    if ($entries.Count -eq 0) {
        throw 'Process execution journal contains no non-empty entries.'
    }
    return $entries
}

function Assert-MvpProcessJournalRun {
    param(
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)][hashtable]$Journal,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not $Journal.ContainsKey($Phase)) {
        throw "$Label has no process journal entry for phase '$Phase'."
    }
    $entry = $Journal[$Phase]
    foreach ($propertyName in @('started_at_utc', 'ended_at_utc', 'exit_code')) {
        $summaryValue = [string](Get-MvpRequiredProperty -Value $Run -Name $propertyName -Label $Label)
        $journalValue = [string](Get-MvpRequiredProperty -Value $entry -Name $propertyName -Label "Process journal phase '$Phase'")
        if ($summaryValue -ne $journalValue) {
            throw "$Label '$propertyName' '$summaryValue' differs from process journal '$journalValue'."
        }
    }
    if ([string]$entry.outcome -ne 'exited') {
        throw "$Label process journal outcome '$($entry.outcome)' is not 'exited'."
    }
    if ([Int64]$entry.exit_code -ne 0) {
        throw "$Label process journal exit_code '$($entry.exit_code)' is not 0."
    }
}

function Assert-MvpUniqueProductArtifact {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][System.Collections.Generic.HashSet[string]]$Paths,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    if (-not $Paths.Add($Path)) {
        throw "$Label reuses a product-attempt artifact path '$Path'."
    }
}

function Assert-MvpExpectedProductArtifactPath {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$ExpectedRelativePath,
        [Parameter(Mandatory)][string]$Label
    )

    $actualRelativePath = [string](Get-MvpRequiredProperty -Value $Evidence -Name 'path' -Label $Label)
    $normalizedActualPath = $actualRelativePath.Replace('\', '/')
    if ($normalizedActualPath -ne $ExpectedRelativePath) {
        throw "$Label path '$actualRelativePath' must be '$ExpectedRelativePath'."
    }
}

function Assert-MvpExpectedProductDiagnosticPath {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Product,
        [Parameter(Mandatory)][string]$Attempt,
        [Parameter(Mandatory)][string]$Label
    )

    $actualRelativePath = [string](Get-MvpRequiredProperty -Value $Evidence -Name 'path' -Label $Label)
    $normalizedActualPath = $actualRelativePath.Replace('\', '/')
    $expectedPrefix = "logs/$Product-$Attempt.diagnostics/"
    if (-not $normalizedActualPath.StartsWith($expectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label path '$actualRelativePath' must be under '$expectedPrefix'."
    }
}

function Get-MvpProductDiagnosticText {
    param(
        [Parameter(Mandatory)]$Run,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][AllowEmptyCollection()][System.Collections.Generic.HashSet[string]]$ArtifactPaths
    )

    $diagnosticLogsProperty = $Run.PSObject.Properties['diagnostic_logs']
    if ($null -eq $diagnosticLogsProperty -or @($diagnosticLogsProperty.Value).Count -eq 0) {
        throw "$Label does not contain diagnostic log evidence."
    }
    $text = [Text.StringBuilder]::new()
    foreach ($diagnosticLog in @($diagnosticLogsProperty.Value)) {
        $path = Assert-MvpStagedFileEvidence -Evidence $diagnosticLog -StagingRoot $StagingRoot -Label "$Label diagnostic log"
        $product = [string](Get-MvpRequiredProperty -Value $Run -Name 'product' -Label $Label)
        $attempt = [string](Get-MvpRequiredProperty -Value $Run -Name 'attempt' -Label $Label)
        Assert-MvpExpectedProductDiagnosticPath `
            -Evidence $diagnosticLog `
            -Product $product `
            -Attempt $attempt `
            -Label "$Label diagnostic log"
        Assert-MvpUniqueProductArtifact -Paths $ArtifactPaths -Path $path -Label "$Label diagnostic log"
        $null = $text.AppendLine((Get-Content -LiteralPath $path -Raw -Encoding UTF8))
    }
    return $text.ToString()
}

function Assert-MvpProductDiagnosticMarkers {
    param(
        [Parameter(Mandatory)][string]$DiagnosticText,
        [Parameter(Mandatory)][ValidateSet('runtime', 'editor')][string]$Product,
        [Parameter(Mandatory)][string]$Label,
        [switch]$RequireCapture
    )

    $markers = if ($Product -eq 'runtime') {
        @('runtime_first_frame_presented', 'runtime_process_teardown_complete')
    }
    else {
        @('editor_first_frame_presented', 'editor_process_teardown_complete')
    }
    if ($RequireCapture) {
        $markers += if ($Product -eq 'runtime') {
            'runtime_product_frame_capture_written'
        }
        else {
            'editor_product_frame_capture_written'
        }
    }
    foreach ($marker in $markers) {
        if ($DiagnosticText.IndexOf($marker, [StringComparison]::Ordinal) -lt 0) {
            throw "$Label diagnostic logs do not contain '$marker'."
        }
    }
}

function Get-MvpRuntimeProductDiagnosticsFromLog {
    param(
        [Parameter(Mandatory)][string]$DiagnosticText,
        [Parameter(Mandatory)][string]$Label
    )

    $diagnostic = @(
        $DiagnosticText -split '\r?\n' |
            Where-Object { $_.IndexOf('runtime_product_frame_diagnostics ', [StringComparison]::Ordinal) -ge 0 }
    ) | Select-Object -Last 1
    if ([string]::IsNullOrWhiteSpace($diagnostic)) {
        throw "$Label diagnostic logs do not contain runtime_product_frame_diagnostics."
    }
    $adapterMatch = [regex]::Match($diagnostic, '(?:^|\s)render_adapter=(.*?)\s+render_adapter_type=')
    if (-not $adapterMatch.Success -or [string]::IsNullOrWhiteSpace($adapterMatch.Groups[1].Value)) {
        throw "$Label runtime diagnostic is missing a usable render_adapter."
    }
    $fields = [ordered]@{ render_adapter = $adapterMatch.Groups[1].Value.Trim() }
    foreach ($name in @(
        'frame_index', 'viewport', 'project_identity', 'scene_uri', 'selected_model_resource_id',
        'selected_material_resource_id', 'render_backend', 'render_adapter_type',
        'device_max_bind_groups', 'device_max_texture_dimension_2d', 'device_max_texture_array_layers',
        'device_max_sampled_textures_per_shader_stage', 'device_max_storage_buffers_per_shader_stage',
        'device_max_storage_buffer_binding_size', 'graph_executed_pass_count', 'mesh_draw_count',
        'directional_light_count', 'material_fallback_count', 'material_validation_error_count',
        'input_viewport_resize_count',
        'input_pointer_move_count', 'input_mouse_button_press_count', 'input_mouse_button_release_count',
        'input_keyboard_press_count', 'input_keyboard_release_count'
    )) {
        $match = [regex]::Match($diagnostic, '(?:^|\s)' + [regex]::Escape($name) + '=([^\s]+)')
        if (-not $match.Success -or [string]::IsNullOrWhiteSpace($match.Groups[1].Value)) {
            throw "$Label runtime diagnostic is missing '$name'."
        }
        $fields[$name] = $match.Groups[1].Value
    }
    $derivedRun = [pscustomobject]@{ runtime_product_diagnostics = [pscustomobject]$fields }
    return Assert-MvpRuntimeProductDiagnostics -Run $derivedRun
}

function Get-MvpEditorProductDiagnosticsFromLog {
    param(
        [Parameter(Mandatory)][string]$DiagnosticText,
        [Parameter(Mandatory)][string]$ExpectedProjectRoot,
        [Parameter(Mandatory)][string]$ProjectRelativeRoot,
        [Parameter(Mandatory)][string]$Label
    )

    $diagnostic = @(
        $DiagnosticText -split '\r?\n' |
            Where-Object { $_.IndexOf('editor_product_frame_diagnostics ', [StringComparison]::Ordinal) -ge 0 }
    ) | Select-Object -Last 1
    if ([string]::IsNullOrWhiteSpace($diagnostic)) {
        throw "$Label diagnostic logs do not contain editor_product_frame_diagnostics."
    }
    $fields = [ordered]@{}
    foreach ($name in @('project_path', 'selected_node_id', 'selected_node_name', 'inspector_translation_x', 'inspector_translation_y', 'inspector_translation_z')) {
        $match = [regex]::Match($diagnostic, '(?:^|\s)' + [regex]::Escape($name) + '=([^\s]+)')
        if (-not $match.Success -or $match.Groups[1].Value -match '%(?![0-9A-Fa-f]{2})') {
            throw "$Label editor diagnostic is missing or malformed '$name'."
        }
        $value = [Uri]::UnescapeDataString($match.Groups[1].Value)
        if ([string]::IsNullOrWhiteSpace($value)) {
            throw "$Label editor diagnostic has an empty '$name'."
        }
        $fields[$name] = $value
    }
    [UInt64]$selectedNodeId = 0
    if (-not [UInt64]::TryParse([string]$fields.selected_node_id, [ref]$selectedNodeId) -or $selectedNodeId -eq 0) {
        throw "$Label editor diagnostic has invalid selected_node_id '$($fields.selected_node_id)'."
    }
    $reportedProjectRoot = [IO.Path]::GetFullPath([string]$fields.project_path)
    $canonicalExpectedRoot = [IO.Path]::GetFullPath($ExpectedProjectRoot)
    if (-not $reportedProjectRoot.Equals($canonicalExpectedRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label editor diagnostic project_path '$reportedProjectRoot' differs from staged project '$canonicalExpectedRoot'."
    }
    $fields.project_path = $ProjectRelativeRoot.Replace('\', '/')
    $fields.selected_node_id = $selectedNodeId
    return [pscustomobject]$fields
}

function Assert-MvpDiagnosticsMatchSummary {
    param(
        [Parameter(Mandatory)]$Summary,
        [Parameter(Mandatory)]$Derived,
        [Parameter(Mandatory)][string]$Label
    )

    foreach ($property in $Derived.PSObject.Properties) {
        $summaryValue = [string](Get-MvpRequiredProperty -Value $Summary -Name $property.Name -Label "$Label summary")
        $derivedValue = [string]$property.Value
        if ($summaryValue -ne $derivedValue) {
            throw "$Label summary '$($property.Name)' '$summaryValue' differs from captured diagnostic '$derivedValue'."
        }
    }
}

function Assert-MvpStagingManifestIntegrity {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)]$Manifest
    )

    $entriesProperty = $Manifest.PSObject.Properties['entries']
    if ($null -eq $entriesProperty) {
        throw "Staging manifest is missing 'entries'."
    }
    $entries = @($entriesProperty.Value)
    if ($entries.Count -eq 0) {
        throw 'Staging manifest does not contain staged entries.'
    }

    $stagingRootPrefix = if (
        $StagingRoot.EndsWith([IO.Path]::DirectorySeparatorChar) -or
        $StagingRoot.EndsWith([IO.Path]::AltDirectorySeparatorChar)
    ) {
        $StagingRoot
    }
    else {
        $StagingRoot + [IO.Path]::DirectorySeparatorChar
    }
    $logicalIds = @{}
    $targetPaths = @{}
    [Int64]$entryBytes = 0
    foreach ($entry in $entries) {
        $logicalId = [string](Get-MvpRequiredProperty -Value $entry -Name 'logical_id' -Label 'Staging manifest entry')
        $relativePath = [string](Get-MvpRequiredProperty -Value $entry -Name 'target_relative_path' -Label "Staging manifest entry '$logicalId'")
        if ([IO.Path]::IsPathRooted($relativePath) -or $relativePath -match '(^|[\\/])\.\.([\\/]|$)') {
            throw "Staging manifest entry '$logicalId' has unsafe target_relative_path '$relativePath'."
        }
        if ($targetPaths.ContainsKey($relativePath)) {
            throw "Staging manifest has duplicate target_relative_path '$relativePath'."
        }
        if ($logicalIds.ContainsKey($logicalId)) {
            throw "Staging manifest has duplicate logical_id '$logicalId'."
        }
        $targetPaths[$relativePath] = $true
        $logicalIds[$logicalId] = $true
        $targetPath = Join-Path $StagingRoot $relativePath
        if (-not (Test-Path -LiteralPath $targetPath -PathType Leaf)) {
            throw "Staging manifest entry '$logicalId' target '$relativePath' is missing."
        }
        $resolvedTargetPath = (Resolve-Path -LiteralPath $targetPath).Path
        if (-not $resolvedTargetPath.StartsWith($stagingRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Staging manifest entry '$logicalId' target '$relativePath' escapes the staging root."
        }
        $expectedHash = [string](Get-MvpRequiredProperty -Value $entry -Name 'sha256' -Label "Staging manifest entry '$logicalId'")
        if ($expectedHash -notmatch '^[0-9A-Fa-f]{64}$') {
            throw "Staging manifest entry '$logicalId' has invalid sha256 '$expectedHash'."
        }
        $actualHash = Get-MvpFileSha256 -Path $resolvedTargetPath
        if (-not $actualHash.Equals($expectedHash, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Staging manifest entry '$logicalId' hash mismatch for '$relativePath'."
        }
        $expectedSize = [Int64](Get-MvpRequiredProperty -Value $entry -Name 'size_bytes' -Label "Staging manifest entry '$logicalId'")
        if ($expectedSize -lt 0) {
            throw "Staging manifest entry '$logicalId' has negative size_bytes '$expectedSize'."
        }
        [decimal]$nextEntryBytes = [decimal]$entryBytes + [decimal]$expectedSize
        if ($nextEntryBytes -gt [Int64]::MaxValue) {
            throw 'Staging manifest entry sizes exceed the supported 64-bit byte budget.'
        }
        $entryBytes = [Int64]$nextEntryBytes
        $actualSize = (Get-Item -LiteralPath $resolvedTargetPath).Length
        if ($actualSize -ne $expectedSize) {
            throw "Staging manifest entry '$logicalId' size mismatch for '$relativePath'."
        }
    }
    $null = Assert-MvpStagingPreflightEvidence `
        -Manifest $Manifest `
        -EntryBytes $entryBytes `
        -StagingRoot $StagingRoot

    return $entries.Count
}

function ConvertTo-MvpUInt64 {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    [UInt64]$parsed = 0
    if (-not [UInt64]::TryParse([string]$Value, [ref]$parsed)) {
        throw "$Label has non-numeric '$Name' value '$Value'."
    }
    return $parsed
}

function Assert-MvpRuntimeProductDiagnostics {
    param([Parameter(Mandatory)]$Run)

    $diagnostics = Get-MvpRequiredProperty -Value $Run -Name 'runtime_product_diagnostics' -Label "Runtime product run"
    foreach ($name in @(
        'project_identity',
        'scene_uri',
        'selected_model_resource_id',
        'selected_material_resource_id',
        'render_backend',
        'render_adapter',
        'render_adapter_type'
    )) {
        $value = [string](Get-MvpRequiredProperty -Value $diagnostics -Name $name -Label 'Runtime product diagnostics')
        if ($value -eq 'unavailable') {
            throw "Runtime product diagnostics has no usable '$name' evidence."
        }
    }

    $viewport = [string](Get-MvpRequiredProperty -Value $diagnostics -Name 'viewport' -Label 'Runtime product diagnostics')
    if ($viewport -notmatch '^[1-9][0-9]*x[1-9][0-9]*$') {
        throw "Runtime product diagnostics has invalid viewport '$viewport'."
    }
    $null = ConvertTo-MvpUInt64 -Value (Get-MvpRequiredProperty -Value $diagnostics -Name 'frame_index' -Label 'Runtime product diagnostics') -Name 'frame_index' -Label 'Runtime product diagnostics'

    foreach ($name in @(
        'device_max_bind_groups',
        'device_max_texture_dimension_2d',
        'device_max_texture_array_layers',
        'device_max_sampled_textures_per_shader_stage',
        'device_max_storage_buffers_per_shader_stage',
        'device_max_storage_buffer_binding_size',
        'graph_executed_pass_count',
        'mesh_draw_count',
        'directional_light_count',
        'input_viewport_resize_count',
        'input_pointer_move_count',
        'input_mouse_button_press_count',
        'input_mouse_button_release_count',
        'input_keyboard_press_count',
        'input_keyboard_release_count'
    )) {
        $count = ConvertTo-MvpUInt64 -Value (Get-MvpRequiredProperty -Value $diagnostics -Name $name -Label 'Runtime product diagnostics') -Name $name -Label 'Runtime product diagnostics'
        if ($count -eq 0) {
            throw "Runtime product diagnostics has non-positive '$name' evidence."
        }
    }

    foreach ($name in @('material_fallback_count', 'material_validation_error_count')) {
        $count = ConvertTo-MvpUInt64 -Value (Get-MvpRequiredProperty -Value $diagnostics -Name $name -Label 'Runtime product diagnostics') -Name $name -Label 'Runtime product diagnostics'
        if ($count -ne 0) {
            throw "Runtime product diagnostics has $name '$count' instead of 0."
        }
    }

    return $diagnostics
}

function Resolve-MvpStableRuntimeDiagnosticValue {
    param(
        [Parameter(Mandatory)]$Runs,
        [Parameter(Mandatory)][string]$Name
    )

    $runtimeRuns = @($Runs | Where-Object { $_.product -eq 'runtime' })
    $values = @()
    foreach ($run in $runtimeRuns) {
        $diagnosticsProperty = $run.PSObject.Properties['runtime_product_diagnostics']
        if ($null -eq $diagnosticsProperty -or $null -eq $diagnosticsProperty.Value) {
            continue
        }
        $values += Get-MvpRequiredProperty `
            -Value $diagnosticsProperty.Value `
            -Name $Name `
            -Label "Runtime product run $($run.attempt) diagnostics"
    }

    if ($values.Count -eq 0) {
        return $null
    }
    if ($values.Count -ne $runtimeRuns.Count) {
        throw "Runtime product diagnostics provide '$Name' evidence for only a subset of runtime runs."
    }
    $uniqueValues = @($values | Sort-Object -Unique)
    if ($uniqueValues.Count -ne 1) {
        throw "Runtime product diagnostics disagree on '$Name': $($uniqueValues -join ', ')."
    }
    return $uniqueValues[0]
}

function Resolve-MvpStableRuntimeRenderBackend {
    param([Parameter(Mandatory)]$Runs)

    return Resolve-MvpStableRuntimeDiagnosticValue -Runs $Runs -Name 'render_backend'
}

function Resolve-MvpStableRuntimeRenderDevice {
    param([Parameter(Mandatory)]$Runs)

    $adapter = Resolve-MvpStableRuntimeDiagnosticValue -Runs $Runs -Name 'render_adapter'
    if ($null -eq $adapter) {
        return $null
    }

    $limits = [ordered]@{}
    foreach ($limit in @(
        @{ Output = 'max_bind_groups'; Input = 'device_max_bind_groups' },
        @{ Output = 'max_texture_dimension_2d'; Input = 'device_max_texture_dimension_2d' },
        @{ Output = 'max_texture_array_layers'; Input = 'device_max_texture_array_layers' },
        @{ Output = 'max_sampled_textures_per_shader_stage'; Input = 'device_max_sampled_textures_per_shader_stage' },
        @{ Output = 'max_storage_buffers_per_shader_stage'; Input = 'device_max_storage_buffers_per_shader_stage' },
        @{ Output = 'max_storage_buffer_binding_size'; Input = 'device_max_storage_buffer_binding_size' }
    )) {
        $value = Resolve-MvpStableRuntimeDiagnosticValue -Runs $Runs -Name $limit.Input
        $limits[$limit.Output] = ConvertTo-MvpUInt64 `
            -Value $value `
            -Name $limit.Input `
            -Label 'Runtime product diagnostics'
    }

    return [ordered]@{
        adapter = [string]$adapter
        adapter_type = [string](Resolve-MvpStableRuntimeDiagnosticValue -Runs $Runs -Name 'render_adapter_type')
        limits = $limits
    }
}

function Assert-MvpPngCaptureEvidence {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$Label
    )

    $capturePath = Assert-MvpStagedFileEvidence -Evidence $Evidence -StagingRoot $StagingRoot -Label $Label
    $actual = Get-MvpAcceptancePngEvidence -Path $capturePath
    foreach ($propertyName in @('width', 'height', 'non_background_pixels', 'non_transparent_pixels', 'pixel_sha256')) {
        $expected = Get-MvpRequiredProperty -Value $Evidence -Name $propertyName -Label $Label
        $actualValue = switch ($propertyName) {
            'width' { $actual.Width }
            'height' { $actual.Height }
            'non_background_pixels' { $actual.NonBackgroundPixels }
            'non_transparent_pixels' { $actual.NonTransparentPixels }
            'pixel_sha256' { $actual.PixelSha256 }
        }
        if ($propertyName -eq 'pixel_sha256') {
            if ([string]$actualValue -notmatch '^[0-9A-F]{64}$' -or
                -not ([string]$actualValue).Equals([string]$expected, [StringComparison]::OrdinalIgnoreCase)) {
                throw "$Label '$propertyName' '$actualValue' differs from staged evidence '$expected'."
            }
            continue
        }
        $expected = [Int64]$expected
        if ($actualValue -ne $expected) {
            throw "$Label '$propertyName' '$actualValue' differs from staged evidence '$expected'."
        }
    }
    if ($actual.NonTransparentPixels -le 0 -or $actual.NonBackgroundPixels -lt 100) {
        throw "$Label is blank or visually insufficient."
    }
    return $capturePath
}

function Assert-MvpProductEvidence {
    param(
        [Parameter(Mandatory)]$Runs,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$ExpectedStagingRoot,
        [Parameter(Mandatory)][hashtable]$ProcessJournal
    )

    $artifactPaths = [System.Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    foreach ($run in @($Runs)) {
        $product = [string](Get-MvpRequiredProperty -Value $run -Name 'product' -Label 'Product run')
        $attempt = [string](Get-MvpRequiredProperty -Value $run -Name 'attempt' -Label "Product '$product' run")
        $label = "Product '$product' attempt $attempt"
        $stdoutPath = Assert-MvpStagedFileEvidence `
            -Evidence (Get-MvpRequiredProperty -Value $run -Name 'stdout' -Label $label) `
            -StagingRoot $StagingRoot `
            -Label "$label stdout"
        Assert-MvpExpectedProductArtifactPath `
            -Evidence $run.stdout `
            -ExpectedRelativePath "logs/$product-$attempt.stdout.log" `
            -Label "$label stdout"
        Assert-MvpUniqueProductArtifact -Paths $artifactPaths -Path $stdoutPath -Label "$label stdout"
        $stderrPath = Assert-MvpStagedFileEvidence `
            -Evidence (Get-MvpRequiredProperty -Value $run -Name 'stderr' -Label $label) `
            -StagingRoot $StagingRoot `
            -Label "$label stderr"
        Assert-MvpExpectedProductArtifactPath `
            -Evidence $run.stderr `
            -ExpectedRelativePath "logs/$product-$attempt.stderr.log" `
            -Label "$label stderr"
        Assert-MvpUniqueProductArtifact -Paths $artifactPaths -Path $stderrPath -Label "$label stderr"
        $diagnosticText = Get-MvpProductDiagnosticText `
            -Run $run `
            -StagingRoot $StagingRoot `
            -Label $label `
            -ArtifactPaths $artifactPaths
        Assert-MvpProcessJournalRun `
            -Run $run `
            -Journal $ProcessJournal `
            -Phase "$product-$attempt" `
            -Label $label
        Assert-MvpProductDiagnosticMarkers -DiagnosticText $diagnosticText -Product $product -Label $label
        foreach ($propertyName in @('first_frame_presented', 'teardown_complete')) {
            if ((Get-MvpRequiredProperty -Value $run -Name $propertyName -Label $label) -ne $true) {
                throw "$label summary does not record '$propertyName=true'."
            }
        }
        if ($product -eq 'editor') {
            $editorCaptureProperty = $run.PSObject.Properties['editor_window_capture']
            if ($null -ne $editorCaptureProperty -and $null -ne $editorCaptureProperty.Value) {
                $capturePath = Assert-MvpPngCaptureEvidence `
                    -Evidence $editorCaptureProperty.Value `
                    -StagingRoot $StagingRoot `
                    -Label "$label editor window capture"
                Assert-MvpExpectedProductArtifactPath `
                    -Evidence $editorCaptureProperty.Value `
                    -ExpectedRelativePath 'captures/editor-after-reopen.png' `
                    -Label "$label editor window capture"
                Assert-MvpUniqueProductArtifact -Paths $artifactPaths -Path $capturePath -Label "$label editor window capture"
                Assert-MvpProductDiagnosticMarkers `
                    -DiagnosticText $diagnosticText `
                    -Product $product `
                    -Label $label `
                    -RequireCapture
            }
            $editorDiagnosticsProperty = $run.PSObject.Properties['editor_product_diagnostics']
            if ($null -ne $editorDiagnosticsProperty -and $null -ne $editorDiagnosticsProperty.Value) {
                $projectRelativeRoot = [string](Get-MvpRequiredProperty -Value $run -Name 'project' -Label $label)
                $derivedEditorDiagnostics = Get-MvpEditorProductDiagnosticsFromLog `
                    -DiagnosticText $diagnosticText `
                    -ExpectedProjectRoot (Join-Path $ExpectedStagingRoot $projectRelativeRoot) `
                    -ProjectRelativeRoot $projectRelativeRoot `
                    -Label $label
                Assert-MvpDiagnosticsMatchSummary `
                    -Summary $editorDiagnosticsProperty.Value `
                    -Derived $derivedEditorDiagnostics `
                    -Label "$label editor diagnostics"
            }
            continue
        }

        $capture = Get-MvpRequiredProperty -Value $run -Name 'frame_capture' -Label $label
        $capturePath = Assert-MvpPngCaptureEvidence `
            -Evidence $capture `
            -StagingRoot $StagingRoot `
            -Label "$label frame capture"
        Assert-MvpExpectedProductArtifactPath `
            -Evidence $capture `
            -ExpectedRelativePath "captures/runtime-$attempt.png" `
            -Label "$label frame capture"
        Assert-MvpUniqueProductArtifact -Paths $artifactPaths -Path $capturePath -Label "$label frame capture"
        Assert-MvpProductDiagnosticMarkers `
            -DiagnosticText $diagnosticText `
            -Product $product `
            -Label $label `
            -RequireCapture
        $derivedRuntimeDiagnostics = Get-MvpRuntimeProductDiagnosticsFromLog `
            -DiagnosticText $diagnosticText `
            -Label $label
        Assert-MvpDiagnosticsMatchSummary `
            -Summary (Get-MvpRequiredProperty -Value $run -Name 'runtime_product_diagnostics' -Label $label) `
            -Derived $derivedRuntimeDiagnostics `
            -Label "$label runtime diagnostics"
    }
}

function Assert-MvpAutomationProcessEvidence {
    param(
        [Parameter(Mandatory)]$Automation,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$ExpectedStagingRoot,
        [Parameter(Mandatory)]$StagingManifest,
        [Parameter(Mandatory)][string]$RequestLogicalId,
        [Parameter(Mandatory)][string]$StagedProjectRoot,
        [Parameter(Mandatory)][string]$Label,
        [AllowNull()][hashtable]$ProcessJournal,
        [string]$ProcessPhase
    )

    $requestEvidence = Get-MvpRequiredProperty -Value $Automation -Name 'automation_request' -Label $Label
    $requestPath = Assert-MvpStagedFileEvidence -Evidence $requestEvidence -StagingRoot $StagingRoot -Label "$Label request"
    $requestEntries = @($StagingManifest.entries | Where-Object { $_.logical_id -eq $RequestLogicalId })
    if ($requestEntries.Count -ne 1) {
        throw "$Label must bind exactly one '$RequestLogicalId' staging manifest entry; found $($requestEntries.Count)."
    }
    $requestEntry = $requestEntries[0]
    $requestRelativePath = [string](Get-MvpRequiredProperty -Value $requestEvidence -Name 'path' -Label "$Label request")
    $manifestRelativePath = [string](Get-MvpRequiredProperty -Value $requestEntry -Name 'target_relative_path' -Label "Staging manifest entry '$RequestLogicalId'")
    if ($requestRelativePath.Replace('\', '/') -ne $manifestRelativePath.Replace('\', '/')) {
        throw "$Label request path '$requestRelativePath' differs from staging manifest '$manifestRelativePath'."
    }
    foreach ($propertyName in @('sha256', 'size_bytes')) {
        $actual = [string](Get-MvpRequiredProperty -Value $requestEvidence -Name $propertyName -Label "$Label request")
        $expected = [string](Get-MvpRequiredProperty -Value $requestEntry -Name $propertyName -Label "Staging manifest entry '$RequestLogicalId'")
        if (-not $actual.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) {
            throw "$Label request '$propertyName' differs from staging manifest input."
        }
    }
    $null = $requestPath

    $stdoutPath = Assert-MvpStagedFileEvidence `
        -Evidence (Get-MvpRequiredProperty -Value $Automation -Name 'stdout' -Label $Label) `
        -StagingRoot $StagingRoot `
        -Label "$Label stdout"
    $null = Assert-MvpStagedFileEvidence `
        -Evidence (Get-MvpRequiredProperty -Value $Automation -Name 'stderr' -Label $Label) `
        -StagingRoot $StagingRoot `
        -Label "$Label stderr"
    $diagnosticLogsProperty = $Automation.PSObject.Properties['diagnostic_logs']
    if ($null -eq $diagnosticLogsProperty -or @($diagnosticLogsProperty.Value).Count -eq 0) {
        throw "$Label does not contain diagnostic log evidence."
    }
    foreach ($diagnosticLog in @($diagnosticLogsProperty.Value)) {
        $null = Assert-MvpStagedFileEvidence -Evidence $diagnosticLog -StagingRoot $StagingRoot -Label "$Label diagnostic log"
    }
    $exitCode = [Int64](Get-MvpRequiredProperty -Value $Automation -Name 'exit_code' -Label $Label)
    if ($exitCode -ne 0) {
        throw "$Label has exit_code '$exitCode' instead of 0."
    }
    if ($null -ne $ProcessJournal) {
        if ([string]::IsNullOrWhiteSpace($ProcessPhase)) {
            throw "$Label requires a process journal phase."
        }
        Assert-MvpProcessJournalRun `
            -Run $Automation `
            -Journal $ProcessJournal `
            -Phase $ProcessPhase `
            -Label $Label
    }

    $capturedReport = Read-MvpJsonObject -Path $stdoutPath -Label "$Label stdout"
    if ($null -eq $Automation.PSObject.Properties['records']) {
        throw "$Label lost records before stdout comparison; available=$($Automation.PSObject.Properties.Name -join ',')."
    }
    foreach ($propertyName in @(
        'project_path',
        'project_identity',
        'manifest_identity',
        'scene_uri',
        'selected_model_resource_id',
        'selected_material_resource_id',
        'opened_project_inspection_generation',
        'records',
        'snapshot'
    )) {
        if ($propertyName -eq 'project_path') {
            $capturedProjectPath = [string](Get-MvpRequiredProperty -Value $capturedReport -Name $propertyName -Label "$Label stdout")
            if ([IO.Path]::IsPathRooted($capturedProjectPath)) {
                $expectedProjectPath = [IO.Path]::GetFullPath((Join-Path $ExpectedStagingRoot $StagedProjectRoot))
                $resolvedCapturedProjectPath = [IO.Path]::GetFullPath($capturedProjectPath)
                if (-not $resolvedCapturedProjectPath.Equals($expectedProjectPath, [StringComparison]::OrdinalIgnoreCase)) {
                    throw "$Label captured stdout project_path '$capturedProjectPath' differs from staged project '$expectedProjectPath'."
                }
            }
            elseif ($capturedProjectPath -ne $StagedProjectRoot) {
                throw "$Label captured stdout project_path '$capturedProjectPath' differs from staged project root '$StagedProjectRoot'."
            }
            continue
        }
        $reported = ConvertTo-Json -InputObject (Get-MvpRequiredProperty -Value $Automation -Name $propertyName -Label $Label) -Depth 64
        $captured = ConvertTo-Json -InputObject (Get-MvpRequiredProperty -Value $capturedReport -Name $propertyName -Label "$Label stdout") -Depth 64
        if ($reported -ne $captured) {
            throw "$Label '$propertyName' differs from its captured stdout report."
        }
    }
}

function Assert-MvpProjectCreationEvidence {
    param(
        [Parameter(Mandatory)]$ProjectCreation,
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$ExpectedStagingRoot,
        [Parameter(Mandatory)][string]$StagedProjectRoot,
        [AllowNull()][hashtable]$ProcessJournal
    )

    $label = 'Project creation'
    $exitCode = [Int64](Get-MvpRequiredProperty -Value $ProjectCreation -Name 'exit_code' -Label $label)
    if ($exitCode -ne 0) {
        throw "$label has exit_code '$exitCode' instead of 0."
    }
    foreach ($propertyName in @('first_frame_presented', 'teardown_complete')) {
        if ((Get-MvpRequiredProperty -Value $ProjectCreation -Name $propertyName -Label $label) -ne $true) {
            throw "$label does not record '$propertyName=true'."
        }
    }
    $stdoutPath = Assert-MvpStagedFileEvidence -Evidence (Get-MvpRequiredProperty -Value $ProjectCreation -Name 'stdout' -Label $label) -StagingRoot $StagingRoot -Label "$label stdout"
    Assert-MvpExpectedProductArtifactPath -Evidence $ProjectCreation.stdout -ExpectedRelativePath 'logs/editor-create.stdout.log' -Label "$label stdout"
    $stderrPath = Assert-MvpStagedFileEvidence -Evidence (Get-MvpRequiredProperty -Value $ProjectCreation -Name 'stderr' -Label $label) -StagingRoot $StagingRoot -Label "$label stderr"
    Assert-MvpExpectedProductArtifactPath -Evidence $ProjectCreation.stderr -ExpectedRelativePath 'logs/editor-create.stderr.log' -Label "$label stderr"
    $diagnosticLogsProperty = $ProjectCreation.PSObject.Properties['diagnostic_logs']
    if ($null -eq $diagnosticLogsProperty -or @($diagnosticLogsProperty.Value).Count -eq 0) {
        throw "$label does not contain diagnostic log evidence."
    }
    $diagnosticText = (@($diagnosticLogsProperty.Value) | ForEach-Object {
        $path = Assert-MvpStagedFileEvidence -Evidence $_ -StagingRoot $StagingRoot -Label "$label diagnostic log"
        $relativePath = [string](Get-MvpRequiredProperty -Value $_ -Name 'path' -Label "$label diagnostic log")
        if (-not $relativePath.Replace('\', '/').StartsWith('logs/editor-create.diagnostics/', [StringComparison]::OrdinalIgnoreCase)) {
            throw "$label diagnostic log path '$relativePath' must be under 'logs/editor-create.diagnostics/'."
        }
        Get-Content -LiteralPath $path -Raw -ErrorAction Stop
    }) -join [Environment]::NewLine
    $projectRoot = Join-Path $StagingRoot $StagedProjectRoot
    $expectedProjectRoot = Join-Path $ExpectedStagingRoot $StagedProjectRoot
    if ($null -ne $ProcessJournal) {
        Assert-MvpProcessJournalRun -Run $ProjectCreation -Journal $ProcessJournal -Phase 'editor-create' -Label $label
    }
    Assert-MvpProductDiagnosticMarkers `
        -DiagnosticText $diagnosticText `
        -Product 'editor' `
        -Label $label `
        -RequireCapture
    $editorCapture = Get-MvpRequiredProperty -Value $ProjectCreation -Name 'editor_window_capture' -Label $label
    $capturePath = Assert-MvpPngCaptureEvidence `
        -Evidence $editorCapture `
        -StagingRoot $StagingRoot `
        -Label "$label editor window capture"
    $null = $capturePath
    Assert-MvpExpectedProductArtifactPath `
        -Evidence $editorCapture `
        -ExpectedRelativePath 'captures/editor-before-edit.png' `
        -Label "$label editor window capture"
    $derivedEditorDiagnostics = Get-MvpEditorProductDiagnosticsFromLog `
        -DiagnosticText $diagnosticText `
        -ExpectedProjectRoot $expectedProjectRoot `
        -ProjectRelativeRoot $StagedProjectRoot `
        -Label $label
    Assert-MvpDiagnosticsMatchSummary `
        -Summary (Get-MvpRequiredProperty -Value $ProjectCreation -Name 'editor_product_diagnostics' -Label $label) `
        -Derived $derivedEditorDiagnostics `
        -Label "$label editor diagnostics"
    Assert-MvpEditorProjectOpenEvidence `
        -Evidence (Get-MvpRequiredProperty -Value $ProjectCreation -Name 'project_open' -Label $label) `
        -DiagnosticText $diagnosticText `
        -StagingRoot $StagingRoot `
        -ProjectRoot $projectRoot `
        -ExpectedProjectRoot $expectedProjectRoot | Out-Null
}

function Assert-MvpF5EditorWindowEvidence {
    param(
        [Parameter(Mandatory)]$ProjectCreation,
        [Parameter(Mandatory)]$ProductRuns,
        [Parameter(Mandatory)][string]$StagingRoot
    )

    $beforeEditCapture = Get-MvpRequiredProperty `
        -Value $ProjectCreation `
        -Name 'editor_window_capture' `
        -Label 'Project creation'
    $beforeEditPath = [string](Get-MvpRequiredProperty `
        -Value $beforeEditCapture `
        -Name 'path' `
        -Label 'Project creation editor window capture')
    if ($beforeEditPath.Replace('\', '/') -ne 'captures/editor-before-edit.png') {
        throw "Project creation editor window capture must be 'captures/editor-before-edit.png', found '$beforeEditPath'."
    }
    $null = Assert-MvpPngCaptureEvidence `
        -Evidence $beforeEditCapture `
        -StagingRoot $StagingRoot `
        -Label 'Project creation editor window capture'
    $beforeEditDiagnostics = Get-MvpRequiredProperty `
        -Value $ProjectCreation `
        -Name 'editor_product_diagnostics' `
        -Label 'Project creation'

    $afterReopenRuns = @(
        $ProductRuns | Where-Object {
            $_.product -eq 'editor' -and
            $null -ne $_.PSObject.Properties['editor_window_capture'] -and
            $null -ne $_.editor_window_capture
        }
    )
    if ($afterReopenRuns.Count -ne 1) {
        throw "F5 evidence requires exactly one editor window capture after reopen; found $($afterReopenRuns.Count)."
    }
    $afterReopenCapture = Get-MvpRequiredProperty `
        -Value $afterReopenRuns[0] `
        -Name 'editor_window_capture' `
        -Label 'Reopened editor product'
    $afterReopenPath = [string](Get-MvpRequiredProperty `
        -Value $afterReopenCapture `
        -Name 'path' `
        -Label 'Reopened editor window capture')
    if ($afterReopenPath.Replace('\', '/') -ne 'captures/editor-after-reopen.png') {
        throw "Reopened editor window capture must be 'captures/editor-after-reopen.png', found '$afterReopenPath'."
    }
    $null = Assert-MvpPngCaptureEvidence `
        -Evidence $afterReopenCapture `
        -StagingRoot $StagingRoot `
        -Label 'Reopened editor window capture'
    $afterReopenDiagnostics = Get-MvpRequiredProperty `
        -Value $afterReopenRuns[0] `
        -Name 'editor_product_diagnostics' `
        -Label 'Reopened editor product'
    $projectOpen = Get-MvpRequiredProperty -Value $ProjectCreation -Name 'project_open' -Label 'Project creation'
    $expectedEditorProjectPath = [string](Get-MvpRequiredProperty -Value $projectOpen -Name 'project_root' -Label 'Project creation project-open evidence')

    foreach ($diagnosticField in @(
        @{ name = 'project_path'; expected = $expectedEditorProjectPath },
        @{ name = 'selected_node_id'; expected = '3' },
        @{ name = 'selected_node_name'; expected = 'Cube' },
        @{ name = 'inspector_translation_y'; expected = '0' },
        @{ name = 'inspector_translation_z'; expected = '0' }
    )) {
        $beforeValue = [string](Get-MvpRequiredProperty -Value $beforeEditDiagnostics -Name $diagnosticField.name -Label 'Project creation editor diagnostics')
        $afterValue = [string](Get-MvpRequiredProperty -Value $afterReopenDiagnostics -Name $diagnosticField.name -Label 'Reopened editor diagnostics')
        if ($beforeValue -ne $diagnosticField.expected -or $afterValue -ne $diagnosticField.expected) {
            throw "Editor before/after diagnostics do not preserve '$($diagnosticField.name)' as '$($diagnosticField.expected)'."
        }
    }
    [double]$beforeTranslationX = 0
    [double]$afterTranslationX = 0
    if (-not [double]::TryParse([string]$beforeEditDiagnostics.inspector_translation_x, [ref]$beforeTranslationX) -or $beforeTranslationX -ne 0.0) {
        throw 'Project creation editor diagnostics must show Cube Inspector X equal to 0.'
    }
    if (-not [double]::TryParse([string]$afterReopenDiagnostics.inspector_translation_x, [ref]$afterTranslationX) -or $afterTranslationX -ne 42.0) {
        throw 'Reopened editor diagnostics must show persisted Cube Inspector X equal to 42.'
    }
    $beforePixelSha256 = [string](Get-MvpRequiredProperty -Value $beforeEditCapture -Name 'pixel_sha256' -Label 'Project creation editor window capture')
    $afterPixelSha256 = [string](Get-MvpRequiredProperty -Value $afterReopenCapture -Name 'pixel_sha256' -Label 'Reopened editor window capture')
    if ($beforePixelSha256.Equals($afterPixelSha256, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Editor before/after window captures have identical decoded pixels despite the persisted Cube Inspector X change.'
    }
}

function Assert-MvpSuccessfulProductRuns {
    param(
        [Parameter(Mandatory)]$Runs,
        [Parameter(Mandatory)][string]$StagedProjectRoot,
        [switch]$RequireRuntimeProductDiagnostics
    )

    $runs = @($Runs)
    if ($runs.Count -eq 0) {
        throw 'Startup summary does not contain product runs.'
    }

    $observedProducts = @{}
    $productAttemptCounts = @{}
    $productAttempts = @{}
    $runtimeRepeatInvariantNames = @(
        'project_identity',
        'scene_uri',
        'selected_model_resource_id',
        'selected_material_resource_id',
        'render_backend',
        'render_adapter',
        'render_adapter_type',
        'viewport',
        'device_max_bind_groups',
        'device_max_texture_dimension_2d',
        'device_max_texture_array_layers',
        'device_max_sampled_textures_per_shader_stage',
        'device_max_storage_buffers_per_shader_stage',
        'device_max_storage_buffer_binding_size',
        'graph_executed_pass_count',
        'mesh_draw_count',
        'directional_light_count'
    )
    $runtimeRepeatBaseline = @{}
    $runtimeRunsForDiagnostics = @($runs | Where-Object {
            $productProperty = $_.PSObject.Properties['product']
            $null -ne $productProperty -and [string]$productProperty.Value -eq 'runtime'
        })
    $runtimeRunsWithDiagnostics = @($runtimeRunsForDiagnostics | Where-Object {
            $diagnosticsProperty = $_.PSObject.Properties['runtime_product_diagnostics']
            $null -ne $diagnosticsProperty -and $null -ne $diagnosticsProperty.Value
        })
    $validateRuntimeDiagnostics = $RequireRuntimeProductDiagnostics -or
        $runtimeRunsWithDiagnostics.Count -ne 0
    if ($validateRuntimeDiagnostics -and
        $runtimeRunsWithDiagnostics.Count -ne $runtimeRunsForDiagnostics.Count) {
        throw 'Runtime product runs are missing runtime_product_diagnostics for a subset of attempts.'
    }
    foreach ($run in $runs) {
        $product = [string](Get-MvpRequiredProperty -Value $run -Name 'product' -Label 'Product run')
        if ($product -notin @('runtime', 'editor')) {
            throw "Startup summary contains unsupported product '$product'."
        }
        $runProject = [string](Get-MvpRequiredProperty -Value $run -Name 'project' -Label "Product '$product' run")
        if ($runProject -ne $StagedProjectRoot) {
            throw "Product '$product' staged project root '$runProject' differs from expected '$StagedProjectRoot'."
        }
        $attempt = ConvertTo-MvpUInt64 -Value (Get-MvpRequiredProperty -Value $run -Name 'attempt' -Label "Product '$product' run") -Name 'attempt' -Label "Product '$product' run"
        if ($attempt -eq 0) {
            throw "Product '$product' has invalid zero attempt."
        }
        if (-not $productAttempts.ContainsKey($product)) {
            $productAttempts[$product] = [System.Collections.Generic.HashSet[UInt64]]::new()
        }
        if (-not $productAttempts[$product].Add($attempt)) {
            throw "Product '$product' contains duplicate attempt $attempt."
        }
        $observedProducts[$product] = $true
        if ($productAttemptCounts.ContainsKey($product)) {
            $productAttemptCounts[$product] = [int]$productAttemptCounts[$product] + 1
        }
        else {
            $productAttemptCounts[$product] = 1
        }
        $exitCode = [int](Get-MvpRequiredProperty -Value $run -Name 'exit_code' -Label "Product '$product' run")
        if ($exitCode -ne 0) {
            throw "Product '$product' exited with code $exitCode."
        }
        if ($run.first_frame_presented -ne $true) {
            throw "Product '$product' did not report a presented first frame."
        }
        if ($run.teardown_complete -ne $true) {
            throw "Product '$product' did not report completed teardown."
        }
        if ($product -eq 'runtime' -and $validateRuntimeDiagnostics) {
            $runtimeDiagnostics = Assert-MvpRuntimeProductDiagnostics -Run $run
            foreach ($name in $runtimeRepeatInvariantNames) {
                $value = [string](Get-MvpRequiredProperty -Value $runtimeDiagnostics -Name $name -Label 'Runtime product diagnostics')
                if ($runtimeRepeatBaseline.ContainsKey($name) -and $runtimeRepeatBaseline[$name] -ne $value) {
                    throw "Runtime product diagnostics '$name' differs between attempts: expected '$($runtimeRepeatBaseline[$name])', found '$value'."
                }
                $runtimeRepeatBaseline[$name] = $value
            }
        }
    }
    if (-not ($observedProducts.ContainsKey('runtime') -and $observedProducts.ContainsKey('editor'))) {
        throw 'Startup summary must contain successful runtime and editor product runs.'
    }

    $repeatFailures = @(
        foreach ($requiredProduct in @('runtime', 'editor')) {
            $attemptCount = if ($productAttemptCounts.ContainsKey($requiredProduct)) {
                [int]$productAttemptCounts[$requiredProduct]
            }
            else {
                0
            }

            if ($attemptCount -lt 2) {
                "$requiredProduct=$attemptCount"
            }
        }
    )
    if ($repeatFailures.Count -ne 0) {
        throw "Startup summary must contain at least two successful runtime and editor product runs: $($repeatFailures -join ', ')."
    }

    $attemptSequenceFailures = @(
        foreach ($requiredProduct in @('runtime', 'editor')) {
            $attempts = $productAttempts[$requiredProduct]
            $missingAttempts = @(
                foreach ($expectedAttempt in [UInt64[]]@(1, 2)) {
                    if (-not $attempts.Contains($expectedAttempt)) {
                        $expectedAttempt
                    }
                }
            )
            if ($missingAttempts.Count -ne 0) {
                "$requiredProduct missing attempts $($missingAttempts -join ',')"
            }
        }
    )
    if ($attemptSequenceFailures.Count -ne 0) {
        throw "Startup summary must contain unique attempts 1 and 2 for runtime and editor: $($attemptSequenceFailures -join '; ')."
    }

    return $runs
}

function Get-MvpAuthoringAutomationRecord {
    param(
        [Parameter(Mandatory)]$Records,
        [Parameter(Mandatory)][string]$BindingPath,
        [Parameter(Mandatory)][string]$Label
    )

    $matches = @(
        foreach ($record in @($Records)) {
            $bindingPathProperty = $record.PSObject.Properties['binding_path']
            if ($null -ne $bindingPathProperty -and [string]$bindingPathProperty.Value -eq $BindingPath) {
                $record
            }
        }
    )
    if ($matches.Count -ne 1) {
        throw "Authoring automation report must contain exactly one $Label binding '$BindingPath'; found $($matches.Count)."
    }

    return $matches[0]
}

function Assert-MvpAutomationProjectOpening {
    param(
        [Parameter(Mandatory)]$Automation,
        [Parameter(Mandatory)][string]$StagedProjectRoot,
        [Parameter(Mandatory)][string]$ResolvedStagingRoot,
        [Parameter(Mandatory)][string]$ExpectedStagingRoot,
        [Parameter(Mandatory)][string]$Label
    )

    $reportedProjectPath = [string](Get-MvpRequiredProperty -Value $Automation -Name 'project_path' -Label $Label)
    if ([IO.Path]::IsPathRooted($reportedProjectPath)) {
        $expectedProjectPath = [IO.Path]::GetFullPath((Join-Path $ExpectedStagingRoot $StagedProjectRoot))
        $resolvedReportedProjectPath = [IO.Path]::GetFullPath($reportedProjectPath)
        if (-not $resolvedReportedProjectPath.Equals($expectedProjectPath, [StringComparison]::OrdinalIgnoreCase)) {
            throw "$Label project_path '$reportedProjectPath' differs from staged project '$expectedProjectPath'."
        }
    }
    elseif ($reportedProjectPath -ne $StagedProjectRoot) {
        throw "$Label project_path '$reportedProjectPath' differs from staged project root '$StagedProjectRoot'."
    }

    $openedGeneration = ConvertTo-MvpUInt64 `
        -Value (Get-MvpRequiredProperty -Value $Automation -Name 'opened_project_inspection_generation' -Label $Label) `
        -Name 'opened_project_inspection_generation' `
        -Label $Label
    if ($openedGeneration -eq 0) {
        throw "$Label has a zero opened_project_inspection_generation."
    }
}

function Assert-MvpAutomationSnapshot {
    param(
        [Parameter(Mandatory)]$Automation,
        [Parameter(Mandatory)][string]$Label
    )

    $snapshot = Get-MvpRequiredProperty -Value $Automation -Name 'snapshot' -Label $Label
    if ($snapshot.project_open -ne $true) {
        throw "$Label snapshot did not retain an open project."
    }
    $sceneEntryCount = ConvertTo-MvpUInt64 `
        -Value (Get-MvpRequiredProperty -Value $snapshot -Name 'scene_entry_count' -Label "$Label snapshot") `
        -Name 'scene_entry_count' `
        -Label "$Label snapshot"
    if ($sceneEntryCount -lt 3) {
        throw "$Label snapshot has scene_entry_count '$sceneEntryCount' instead of the RenderableEmpty minimum of 3."
    }
    $selectedNodeId = ConvertTo-MvpUInt64 `
        -Value (Get-MvpRequiredProperty -Value $snapshot -Name 'selected_node_id' -Label "$Label snapshot") `
        -Name 'selected_node_id' `
        -Label "$Label snapshot"
    if ($selectedNodeId -eq 0) {
        throw "$Label snapshot has a zero selected_node_id."
    }
    $selectedNodeName = [string](Get-MvpRequiredProperty -Value $snapshot -Name 'selected_node_name' -Label "$Label snapshot")
    if ([string]::IsNullOrWhiteSpace($selectedNodeName)) {
        throw "$Label snapshot has no selected_node_name."
    }
    $translationProperty = $snapshot.PSObject.Properties['inspector_translation']
    if ($null -eq $translationProperty) {
        throw "$Label snapshot is missing 'inspector_translation'."
    }
    $translation = @($translationProperty.Value)
    if ($translation.Count -ne 3 -or @($translation | Where-Object { [string]::IsNullOrWhiteSpace([string]$_) }).Count -ne 0) {
        throw "$Label snapshot has invalid inspector_translation evidence."
    }
    $null = Assert-MvpAutomationSceneSnapshot `
        -Snapshot $snapshot `
        -SceneEntryCount $sceneEntryCount `
        -SelectedNodeId $selectedNodeId `
        -SelectedNodeName $selectedNodeName `
        -InspectorTranslation $translation `
        -Label "$Label snapshot"

    return $snapshot
}

function Assert-MvpBaselineAutomation {
    param(
        [Parameter(Mandatory)]$Automation,
        [Parameter(Mandatory)][string]$StagedProjectRoot,
        [Parameter(Mandatory)][string]$ResolvedStagingRoot,
        [Parameter(Mandatory)][string]$ExpectedStagingRoot,
        [Parameter(Mandatory)]$StagingManifest,
        [AllowNull()][hashtable]$ProcessJournal
    )

    $label = 'Pre-authoring baseline automation report'
    Assert-MvpAutomationProcessEvidence `
        -Automation $Automation `
        -StagingRoot $ResolvedStagingRoot `
        -ExpectedStagingRoot $ExpectedStagingRoot `
        -StagingManifest $StagingManifest `
        -RequestLogicalId 'reopen-automation-request' `
        -StagedProjectRoot $StagedProjectRoot `
        -Label $label `
        -ProcessJournal $ProcessJournal `
        -ProcessPhase 'editor-baseline'
    Assert-MvpAutomationProjectOpening `
        -Automation $Automation `
        -StagedProjectRoot $StagedProjectRoot `
        -ResolvedStagingRoot $ResolvedStagingRoot `
        -ExpectedStagingRoot $ExpectedStagingRoot `
        -Label $label
    $snapshot = Assert-MvpAutomationSnapshot -Automation $Automation -Label $label
    if ([string]$snapshot.selected_node_id -ne '3' -or [string]$snapshot.selected_node_name -ne 'Cube') {
        throw 'Pre-authoring baseline must select the RenderableEmpty Cube identity (3/Cube).'
    }
    [double]$translationX = 0
    if (-not [double]::TryParse([string]$snapshot.inspector_translation[0], [ref]$translationX) -or $translationX -ne 0.0) {
        throw 'Pre-authoring baseline must observe the template Cube X transform value 0.'
    }
    $records = @((Get-MvpRequiredProperty -Value $Automation -Name 'records' -Label $label))
    $selection = Get-MvpAuthoringAutomationRecord `
        -Records $records `
        -BindingPath 'Hierarchy/SelectCube:onClick' `
        -Label 'baseline scene selection'
    $source = [string](Get-MvpRequiredProperty -Value $selection -Name 'source' -Label $label)
    if ($source -ne 'Cli') {
        throw "$label selection record has source '$source' instead of 'Cli'."
    }

    return $Automation
}

function Assert-MvpAuthoringAutomation {
    param(
        [Parameter(Mandatory)]$Automation,
        [Parameter(Mandatory)][string]$StagedProjectRoot,
        [Parameter(Mandatory)][string]$ResolvedStagingRoot,
        [Parameter(Mandatory)][string]$ExpectedStagingRoot,
        [Parameter(Mandatory)]$StagingManifest,
        [AllowNull()][hashtable]$ProcessJournal
    )

    Assert-MvpAutomationProcessEvidence `
        -Automation $Automation `
        -StagingRoot $ResolvedStagingRoot `
        -ExpectedStagingRoot $ExpectedStagingRoot `
        -StagingManifest $StagingManifest `
        -RequestLogicalId 'authoring-automation-request' `
        -StagedProjectRoot $StagedProjectRoot `
        -Label 'Authoring automation report' `
        -ProcessJournal $ProcessJournal `
        -ProcessPhase 'editor-authoring'
    Assert-MvpAutomationProjectOpening `
        -Automation $Automation `
        -StagedProjectRoot $StagedProjectRoot `
        -ResolvedStagingRoot $ResolvedStagingRoot `
        -ExpectedStagingRoot $ExpectedStagingRoot `
        -Label 'Authoring automation report'
    $authoringSnapshot = Assert-MvpAutomationSnapshot -Automation $Automation -Label 'Authoring automation report'
    if ([string]$authoringSnapshot.selected_node_id -ne '3' -or [string]$authoringSnapshot.selected_node_name -ne 'Cube') {
        throw 'Authoring automation snapshot must select the RenderableEmpty Cube identity (3/Cube).'
    }
    [double]$authoringTranslationX = 0
    if (-not [double]::TryParse([string]$authoringSnapshot.inspector_translation[0], [ref]$authoringTranslationX) -or $authoringTranslationX -ne 42.0) {
        throw 'Authoring automation snapshot must retain the requested X transform value 42.'
    }

    $recordsProperty = $Automation.PSObject.Properties['records']
    if ($null -eq $recordsProperty) {
        throw "Authoring automation report is missing 'records'; available=$($Automation.PSObject.Properties.Name -join ',')."
    }
    $records = @($recordsProperty.Value)
    if ($records.Count -eq 0) {
        throw 'Authoring automation report does not contain binding records.'
    }

    $selection = Get-MvpAuthoringAutomationRecord `
        -Records $records `
        -BindingPath 'Hierarchy/SelectCube:onClick' `
        -Label 'scene selection'
    $transform = Get-MvpAuthoringAutomationRecord `
        -Records $records `
        -BindingPath 'Inspector/TransformPositionXCommit:onSubmit' `
        -Label 'transform commit'
    $save = Get-MvpAuthoringAutomationRecord `
        -Records $records `
        -BindingPath 'WorkbenchMenuBar/SaveProject:onClick' `
        -Label 'project save'

    foreach ($record in @($selection, $transform, $save)) {
        $source = [string](Get-MvpRequiredProperty -Value $record -Name 'source' -Label 'Authoring automation record')
        if ($source -ne 'Cli') {
            throw "Authoring automation record '$([string]$record.binding_path)' has source '$source' instead of 'Cli'."
        }
    }

    $transformOperation = [string](Get-MvpRequiredProperty -Value $transform -Name 'operation_id' -Label 'Authoring transform record')
    if ($transformOperation -ne 'inspector.field.apply_batch') {
        throw "Authoring transform record operation_id '$transformOperation' is not 'inspector.field.apply_batch'."
    }
    $transformTransaction = ConvertTo-MvpUInt64 `
        -Value (Get-MvpRequiredProperty -Value $transform -Name 'transaction_id' -Label 'Authoring transform record') `
        -Name 'transaction_id' `
        -Label 'Authoring transform record'
    if ($transformTransaction -eq 0) {
        throw 'Authoring transform record has a zero transaction_id.'
    }

    $saveOperation = [string](Get-MvpRequiredProperty -Value $save -Name 'operation_id' -Label 'Authoring save record')
    if ($saveOperation -ne 'file.project.save') {
        throw "Authoring save record operation_id '$saveOperation' is not 'file.project.save'."
    }
    $saveGeneration = ConvertTo-MvpUInt64 `
        -Value (Get-MvpRequiredProperty -Value $save -Name 'save_generation' -Label 'Authoring save record') `
        -Name 'save_generation' `
        -Label 'Authoring save record'
    if ($saveGeneration -eq 0) {
        throw 'Authoring save record has a zero save_generation.'
    }
    $projectSaveDiagnosticText = [Text.StringBuilder]::new()
    foreach ($diagnosticLog in @($Automation.diagnostic_logs)) {
        $diagnosticPath = Assert-MvpStagedFileEvidence `
            -Evidence $diagnosticLog `
            -StagingRoot $ResolvedStagingRoot `
            -Label 'Authoring automation report diagnostic log'
        $null = $projectSaveDiagnosticText.AppendLine(
            (Get-Content -LiteralPath $diagnosticPath -Raw -Encoding UTF8)
        )
    }
    $projectSaveLifecycle = Assert-MvpProjectSaveLifecycleEvidence `
        -DiagnosticText $projectSaveDiagnosticText.ToString() `
        -SaveOperationId $saveOperation `
        -SaveGeneration $saveGeneration `
        -ExpectedProjectPath ([IO.Path]::GetFullPath((Join-Path $ExpectedStagingRoot $StagedProjectRoot)))

    $Automation | Add-Member `
        -NotePropertyName 'project_save_lifecycle' `
        -NotePropertyValue $projectSaveLifecycle `
        -Force

    return $Automation
}

function Assert-MvpReopenAutomation {
    param(
        [Parameter(Mandatory)]$Automations,
        [Parameter(Mandatory)]$AuthoringAutomation,
        [Parameter(Mandatory)][string]$StagedProjectRoot,
        [Parameter(Mandatory)][string]$ResolvedStagingRoot,
        [Parameter(Mandatory)][string]$ExpectedStagingRoot,
        [Parameter(Mandatory)]$StagingManifest,
        [AllowNull()][hashtable]$ProcessJournal
    )

    $reports = @($Automations)
    if ($reports.Count -ne 2) {
        throw "Startup summary must contain exactly two reopened-project automation reports; found $($reports.Count)."
    }
    $authoringSnapshot = Assert-MvpAutomationSnapshot -Automation $AuthoringAutomation -Label 'Authoring automation report'
    $reopenedInspectionGenerations = [Collections.Generic.List[UInt64]]::new()
    $reopenedProcessEvidencePaths = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    foreach ($index in 0..($reports.Count - 1)) {
        $report = $reports[$index]
        $label = "Reopened-project automation report $($index + 1)"
        Assert-MvpAutomationProcessEvidence `
            -Automation $report `
            -StagingRoot $ResolvedStagingRoot `
            -ExpectedStagingRoot $ExpectedStagingRoot `
            -StagingManifest $StagingManifest `
            -RequestLogicalId 'reopen-automation-request' `
            -StagedProjectRoot $StagedProjectRoot `
            -Label $label `
            -ProcessJournal $ProcessJournal `
            -ProcessPhase "editor-reopen-$($index + 1)"
        $processEvidence = @(
            [pscustomobject]@{
                name = 'stdout'
                evidence = Get-MvpRequiredProperty -Value $report -Name 'stdout' -Label $label
            },
            [pscustomobject]@{
                name = 'stderr'
                evidence = Get-MvpRequiredProperty -Value $report -Name 'stderr' -Label $label
            }
        )
        foreach ($diagnosticLog in @($report.diagnostic_logs)) {
            $processEvidence += [pscustomobject]@{
                name = 'diagnostic log'
                evidence = $diagnosticLog
            }
        }
        foreach ($item in $processEvidence) {
            $relativePath = [string](Get-MvpRequiredProperty `
                -Value $item.evidence `
                -Name 'path' `
                -Label "$label $($item.name)")
            $candidatePath = if ([IO.Path]::IsPathRooted($relativePath)) {
                $relativePath
            }
            else {
                Join-Path $ResolvedStagingRoot $relativePath
            }
            $canonicalPath = [IO.Path]::GetFullPath($candidatePath)
            if (-not $reopenedProcessEvidencePaths.Add($canonicalPath)) {
                throw "$label reopened-project process evidence contains duplicate path '$relativePath'."
            }
        }
        Assert-MvpAutomationProjectOpening `
            -Automation $report `
            -StagedProjectRoot $StagedProjectRoot `
            -ResolvedStagingRoot $ResolvedStagingRoot `
            -ExpectedStagingRoot $ExpectedStagingRoot `
            -Label $label
        $reopenedInspectionGenerations.Add((ConvertTo-MvpUInt64 `
            -Value (Get-MvpRequiredProperty -Value $report -Name 'opened_project_inspection_generation' -Label $label) `
            -Name 'opened_project_inspection_generation' `
            -Label $label))
        $snapshot = Assert-MvpAutomationSnapshot -Automation $report -Label $label
        $recordsProperty = $report.PSObject.Properties['records']
        if ($null -eq $recordsProperty) {
            throw "$label is missing 'records'."
        }
        $selection = Get-MvpAuthoringAutomationRecord `
            -Records @($recordsProperty.Value) `
            -BindingPath 'Hierarchy/SelectCube:onClick' `
            -Label 'persisted scene selection'
        $source = [string](Get-MvpRequiredProperty -Value $selection -Name 'source' -Label $label)
        if ($source -ne 'Cli') {
            throw "$label selection record has source '$source' instead of 'Cli'."
        }
        foreach ($propertyName in @('selected_node_id', 'selected_node_name')) {
            $expected = [string](Get-MvpRequiredProperty -Value $authoringSnapshot -Name $propertyName -Label 'Authoring automation snapshot')
            $actual = [string](Get-MvpRequiredProperty -Value $snapshot -Name $propertyName -Label "$label snapshot")
            if ($actual -ne $expected) {
                throw "$label snapshot '$propertyName' '$actual' differs from authoring snapshot '$expected'."
            }
        }
        $expectedTranslation = @($authoringSnapshot.inspector_translation)
        $actualTranslation = @($snapshot.inspector_translation)
        if (($actualTranslation -join '|') -ne ($expectedTranslation -join '|')) {
            throw "$label snapshot inspector_translation differs from the authoring snapshot."
        }
        $null = Assert-MvpSceneSnapshotMatch `
            -ExpectedSnapshot $authoringSnapshot `
            -ActualSnapshot $snapshot `
            -Label "$label snapshot"
        foreach ($propertyName in @('selected_model_resource_id', 'selected_material_resource_id')) {
            $expected = [string](Get-MvpRequiredProperty -Value $AuthoringAutomation -Name $propertyName -Label 'Authoring automation report')
            $actual = [string](Get-MvpRequiredProperty -Value $report -Name $propertyName -Label $label)
            if ($actual -ne $expected) {
                throw "$label '$propertyName' '$actual' differs from authoring value '$expected'."
            }
        }
    }

    $projectSaveLifecycle = Get-MvpRequiredProperty `
        -Value $AuthoringAutomation `
        -Name 'project_save_lifecycle' `
        -Label 'Authoring automation report'
    $projectSaveLifecycle | Add-Member `
        -NotePropertyName 'reopened_inspection_generations' `
        -NotePropertyValue $reopenedInspectionGenerations.ToArray() `
        -Force

    return $reports
}

function Assert-MvpPostAuthoringRuntime {
    param([Parameter(Mandatory)]$ProductRuns)

    $afterAuthoringRuns = @(
        $ProductRuns | Where-Object {
            $_.product -eq 'runtime' -and [string]$_.attempt -eq '3'
        }
    )
    if ($afterAuthoringRuns.Count -ne 1) {
        throw "Startup summary must contain exactly one runtime attempt 3 after authoring and reopen; found $($afterAuthoringRuns.Count)."
    }
    $null = Assert-MvpRuntimeProductDiagnostics -Run $afterAuthoringRuns[0]
    return $afterAuthoringRuns[0]
}

function Assert-MvpF5ProjectIdentity {
    param(
        [Parameter(Mandatory)]$ProjectCreation,
        [Parameter(Mandatory)]$ProductRuns,
        [Parameter(Mandatory)]$BaselineAutomation,
        [Parameter(Mandatory)]$AuthoringAutomation,
        [Parameter(Mandatory)]$ReopenAutomation,
        [Parameter(Mandatory)][string]$StagedProjectRoot
    )

    $projectOpen = Get-MvpRequiredProperty -Value $ProjectCreation -Name 'project_open' -Label 'Project creation'
    $manifestIdentity = [string](Get-MvpRequiredProperty -Value $projectOpen -Name 'manifest_identity' -Label 'Project creation project-open evidence')
    $sceneUri = [string](Get-MvpRequiredProperty -Value $projectOpen -Name 'scene_uri' -Label 'Project creation project-open evidence')
    $runtimeRuns = @($ProductRuns | Where-Object { $_.product -eq 'runtime' })
    if ($runtimeRuns.Count -eq 0) {
        throw 'F5 project identity evidence requires at least one runtime product run.'
    }
    $runtimeDiagnostics = Get-MvpRequiredProperty -Value $runtimeRuns[0] -Name 'runtime_product_diagnostics' -Label 'Runtime product run'
    $projectIdentity = [string](Get-MvpRequiredProperty -Value $runtimeDiagnostics -Name 'project_identity' -Label 'Runtime product diagnostics')
    $modelResourceId = [string](Get-MvpRequiredProperty -Value $BaselineAutomation -Name 'selected_model_resource_id' -Label 'Pre-authoring baseline automation report')
    $materialResourceId = [string](Get-MvpRequiredProperty -Value $BaselineAutomation -Name 'selected_material_resource_id' -Label 'Pre-authoring baseline automation report')
    if ([string]::IsNullOrWhiteSpace($modelResourceId) -or [string]::IsNullOrWhiteSpace($materialResourceId)) {
        throw 'Pre-authoring baseline does not identify the selected Cube model and material resources.'
    }

    foreach ($runtimeIndex in 0..($runtimeRuns.Count - 1)) {
        $runtimeLabel = "Runtime product run $($runtimeIndex + 1)"
        $diagnostics = Get-MvpRequiredProperty -Value $runtimeRuns[$runtimeIndex] -Name 'runtime_product_diagnostics' -Label $runtimeLabel
        foreach ($field in @(
            @{ name = 'project_identity'; expected = $projectIdentity },
            @{ name = 'scene_uri'; expected = $sceneUri },
            @{ name = 'selected_model_resource_id'; expected = $modelResourceId },
            @{ name = 'selected_material_resource_id'; expected = $materialResourceId }
        )) {
            $actual = [string](Get-MvpRequiredProperty -Value $diagnostics -Name $field.name -Label "$runtimeLabel diagnostics")
            if ($actual -ne $field.expected) {
                throw "$runtimeLabel '$($field.name)' '$actual' differs from canonical '$($field.expected)'."
            }
        }
    }

    $reports = @($BaselineAutomation) + @($AuthoringAutomation) + @($ReopenAutomation)
    foreach ($index in 0..($reports.Count - 1)) {
        $report = $reports[$index]
        $label = if ($index -eq 0) {
            'Pre-authoring baseline automation report'
        }
        elseif ($index -eq 1) {
            'Authoring automation report'
        }
        else {
            "Reopened-project automation report $($index - 1)"
        }
        foreach ($identityField in @(
            @{ name = 'project_identity'; expected = $projectIdentity },
            @{ name = 'manifest_identity'; expected = $manifestIdentity },
            @{ name = 'scene_uri'; expected = $sceneUri }
        )) {
            $actual = [string](Get-MvpRequiredProperty -Value $report -Name $identityField.name -Label $label)
            if ($actual -ne $identityField.expected) {
                throw "$label '$($identityField.name)' '$actual' differs from canonical '$($identityField.expected)'."
            }
        }
        $reportModelResourceId = [string](Get-MvpRequiredProperty -Value $report -Name 'selected_model_resource_id' -Label $label)
        $reportMaterialResourceId = [string](Get-MvpRequiredProperty -Value $report -Name 'selected_material_resource_id' -Label $label)
        if ([string]::IsNullOrWhiteSpace($reportModelResourceId) -or [string]::IsNullOrWhiteSpace($reportMaterialResourceId)) {
            throw "$label does not identify the selected Cube model and material resources."
        }
        if ($reportModelResourceId -ne $modelResourceId) {
            throw "$label selected_model_resource_id differs from the pre-authoring baseline."
        }
        if ($reportMaterialResourceId -ne $materialResourceId) {
            throw "$label selected_material_resource_id differs from the pre-authoring baseline."
        }
    }

    return [ordered]@{
        project_root = $StagedProjectRoot
        project_identity = $projectIdentity
        manifest_identity = $manifestIdentity
        scene_uri = $sceneUri
        selected_entity_id = 3
        selected_entity_name = 'Cube'
        model_resource_id = $modelResourceId
        material_resource_id = $materialResourceId
    }
}

function Write-MvpAcceptanceManifest {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Value
    )

    $temporaryPath = "$Path.partial-$([guid]::NewGuid().ToString('N'))"
    $temporaryFile = $null
    $expectedContentBytes = [Text.UTF8Encoding]::new($false).GetBytes(
        ($Value | ConvertTo-Json -Depth 64))
    try {
        $temporaryFile = Write-MvpAcceptanceNewFileNoFollow `
            -Path $temporaryPath `
            -ContentBytes $expectedContentBytes `
            -PassThruDetails
        $contentBytes = [byte[]]$temporaryFile.content_bytes
        $temporaryIdentity = [string]$temporaryFile.identity
        if ([string]::IsNullOrWhiteSpace($temporaryIdentity)) {
            throw "Acceptance manifest temporary file '$temporaryPath' did not retain a stable identity."
        }
        Move-MvpAcceptanceNewFileNoFollow `
            -SourcePath $temporaryPath `
            -DestinationPath $Path `
            -ExpectedSourceIdentity $temporaryIdentity
    }
    finally {
        if ($null -ne $temporaryFile -and
            -not [string]::IsNullOrWhiteSpace([string]$temporaryFile.identity) -and
            (Test-Path -LiteralPath $temporaryPath -PathType Leaf)) {
            Remove-MvpAcceptanceFileNoFollow `
                -Path $temporaryPath `
                -ExpectedIdentity ([string]$temporaryFile.identity)
        }
    }
    Write-Output -NoEnumerate $contentBytes
}

function Add-MvpEvidenceProjectionOwnedFile {
    param(
        [Parameter(Mandatory)]$Projection,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$RelativePath,
        [Parameter(Mandatory)][byte[]]$ContentBytes
    )

    if ([IO.Path]::IsPathRooted($RelativePath) -or $RelativePath -match '(^|[\\/])\.\.([\\/]|$)') {
        throw "Acceptance evidence output has unsafe relative path '$RelativePath'."
    }
    Add-MvpAcceptanceStagingProjectionOwnedFile `
        -Projection $Projection `
        -Path (Join-Path $EvidenceRoot $RelativePath.Replace('/', '\\')) `
        -ContentBytes $ContentBytes
}

function Add-MvpBuildSummaryEvidenceProjection {
    param(
        [Parameter(Mandatory)]$Projection,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)]$Summary
    )

    foreach ($artifact in @($Summary) + @($Summary.gate_artifacts)) {
        Add-MvpEvidenceProjectionOwnedFile `
            -Projection $Projection `
            -EvidenceRoot $EvidenceRoot `
            -RelativePath ([string]$artifact.relative_path) `
            -ContentBytes ([byte[]]$artifact.content_bytes)
    }
}

function Publish-MvpAcceptanceEvidencePackage {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)]$Manifest,
        $BuildSummaries = @()
    )

    $partialRoot = "$EvidenceRoot.partial-$([guid]::NewGuid().ToString('N'))"
    $evidenceParent = Split-Path -Parent $EvidenceRoot
    $partialIdentity = $null
    $partialWriteLease = $null
    $partialSnapshotLease = $null
    try {
        if (-not [string]::IsNullOrWhiteSpace($evidenceParent)) {
            New-Item -ItemType Directory -Force -Path $evidenceParent | Out-Null
        }
        New-Item -ItemType Directory -Force -Path $partialRoot | Out-Null
        $partialWriteLease = Open-MvpAcceptanceStagingWriteLease -SnapshotRoot $partialRoot
        $partialIdentity = $partialWriteLease.root_identity
        $stagingProjection = Copy-MvpAcceptanceStagingItems `
            -SourceRoot $StagingRoot `
            -DestinationRoot $partialRoot `
            -ExcludedSourcePaths $acceptanceStagingSnapshotLease.marker_paths `
            -PassThruProjection
        foreach ($buildSummary in @($BuildSummaries)) {
            Copy-MvpBuildSummaryEvidence -Summary $buildSummary -EvidenceRoot $partialRoot
            Add-MvpBuildSummaryEvidenceProjection `
                -Projection $stagingProjection.projection `
                -EvidenceRoot $partialRoot `
                -Summary $buildSummary
        }
        if ($null -ne $Manifest['baseline_automation']) {
            $comparisonArtifacts = @(Write-MvpPersistenceComparisonEvidence `
                -EvidenceRoot $partialRoot `
                -BaselineAutomation $Manifest['baseline_automation'] `
                -AuthoringAutomation $Manifest['authoring_automation'] `
                -ReopenAutomation $Manifest['reopen_automation'])
            foreach ($artifact in $comparisonArtifacts) {
                Add-MvpEvidenceProjectionOwnedFile `
                    -Projection $stagingProjection.projection `
                    -EvidenceRoot $partialRoot `
                    -RelativePath ([string]$artifact.relative_path) `
                    -ContentBytes ([byte[]]$artifact.content_bytes)
            }
        }

        $partialPrefix = [IO.Path]::GetFullPath($partialRoot).TrimEnd('\') + [IO.Path]::DirectorySeparatorChar
        $evidenceFiles = @(
            Get-ChildItem -LiteralPath $partialRoot -Recurse -File |
                Sort-Object FullName |
                ForEach-Object {
                    $fullPath = [IO.Path]::GetFullPath($_.FullName)
                    if (-not $fullPath.StartsWith($partialPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                        throw "Evidence package file '$fullPath' escapes its partial root."
                    }
                    [ordered]@{
                        path = $fullPath.Substring($partialPrefix.Length).Replace('\', '/')
                        sha256 = Get-MvpFileSha256 -Path $fullPath
                        size_bytes = $_.Length
                    }
                }
        )
        if ($evidenceFiles.Count -eq 0) {
            throw 'Acceptance evidence package contains no source evidence files.'
        }
        $Manifest['evidence_layout_version'] = 1
        $Manifest['staging_manifest'] = 'staging-manifest.json'
        $Manifest['startup_summary'] = 'startup-summary.json'
        $Manifest['evidence_files'] = $evidenceFiles
        $manifestBytes = Write-MvpAcceptanceManifest `
            -Path (Join-Path $partialRoot 'manifest.json') `
            -Value $Manifest
        Add-MvpEvidenceProjectionOwnedFile `
            -Projection $stagingProjection.projection `
            -EvidenceRoot $partialRoot `
            -RelativePath 'manifest.json' `
            -ContentBytes $manifestBytes

        if (Test-Path -LiteralPath $EvidenceRoot) {
            $existingEvidenceIdentity = Get-MvpAcceptanceNoFollowDirectoryIdentity -Path $EvidenceRoot
            Remove-MvpAcceptanceEmptyDirectoryNoFollow `
                -Path $EvidenceRoot `
                -ExpectedIdentity $existingEvidenceIdentity
        }
        $partialSnapshotLease = Open-MvpAcceptanceStagingSnapshotLease `
            -SnapshotRoot $partialRoot `
            -ExpectedRootIdentity $partialIdentity
        Assert-MvpAcceptanceStagingProjection `
            -Root $partialRoot `
            -Projection $stagingProjection.projection `
            -ExcludedPaths $partialSnapshotLease.marker_paths
        Prepare-MvpAcceptanceStagingSnapshotLeaseForPublication `
            -Lease $partialSnapshotLease `
            -StagingWriteLease $partialWriteLease
        Assert-MvpAcceptanceStagingProjection `
            -Root $partialRoot `
            -Projection $stagingProjection.projection `
            -ExcludedPaths $partialSnapshotLease.marker_paths
        $partialSourceHandle = Take-MvpAcceptanceStagingWriteLeaseRootHandle `
            -Lease $partialWriteLease
        Move-MvpAcceptanceStagingDirectoryNoFollow `
            -SourcePath $partialRoot `
            -DestinationPath $EvidenceRoot `
            -ExpectedSourceIdentity $partialIdentity `
            -SourceHandle $partialSourceHandle `
            -ExcludedSourcePaths $partialSnapshotLease.marker_paths
    }
    finally {
        if ($null -ne $partialSnapshotLease) {
            Close-MvpAcceptanceStagingSnapshotLease -Lease $partialSnapshotLease
        }
        if ($null -ne $partialWriteLease) {
            Close-MvpAcceptanceStagingWriteLease -Lease $partialWriteLease
        }
        if (-not [string]::IsNullOrWhiteSpace($partialIdentity)) {
            Remove-MvpAcceptanceStagingSnapshot `
                -SnapshotRoot $partialRoot `
                -ExpectedRootIdentity $partialIdentity
        }
    }

    return Join-Path $EvidenceRoot 'manifest.json'
}

$acceptanceStagingSnapshotRoot = $null
$acceptanceStagingSnapshotIdentity = $null
$acceptanceStagingSnapshotLease = $null
try {
$resolvedEvidenceRoot = [IO.Path]::GetFullPath($EvidenceRoot)
$stagingSnapshot = New-MvpAcceptanceStagingSnapshot -StagingRoot $StagingRoot -PassThru
$acceptanceStagingSnapshotRoot = [string]$stagingSnapshot.snapshot_root
$acceptanceStagingSnapshotIdentity = [string]$stagingSnapshot.snapshot_identity
if ([string]::IsNullOrWhiteSpace($acceptanceStagingSnapshotIdentity)) {
    throw 'Acceptance staging snapshot did not return a published directory identity.'
}
$acceptanceStagingSnapshotLease = Open-MvpAcceptanceStagingSnapshotLease `
    -SnapshotRoot $acceptanceStagingSnapshotRoot `
    -ExpectedRootIdentity $acceptanceStagingSnapshotIdentity
$expectedStagingRoot = [string]$stagingSnapshot.source_root
if ($resolvedEvidenceRoot.Equals($expectedStagingRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'EvidenceRoot must be separate from StagingRoot.'
}
$stagingRootPrefix = if (
    $expectedStagingRoot.EndsWith([IO.Path]::DirectorySeparatorChar) -or
    $expectedStagingRoot.EndsWith([IO.Path]::AltDirectorySeparatorChar)
) {
    $expectedStagingRoot
}
else {
    $expectedStagingRoot + [IO.Path]::DirectorySeparatorChar
}
if ($resolvedEvidenceRoot.StartsWith($stagingRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'EvidenceRoot must be outside StagingRoot.'
}
if (Test-Path -LiteralPath $resolvedEvidenceRoot) {
    $existingEvidence = @(Get-ChildItem -LiteralPath $resolvedEvidenceRoot -Force)
    if ($existingEvidence.Count -ne 0) {
        throw "EvidenceRoot '$resolvedEvidenceRoot' must be empty for a new acceptance run."
    }
}

$resolvedStagingRoot = $acceptanceStagingSnapshotRoot

$stagingManifestPath = Join-Path $resolvedStagingRoot 'staging-manifest.json'
$startupSummaryPath = Join-Path $resolvedStagingRoot 'startup-summary.json'
$stagingManifest = Read-MvpJsonObject -Path $stagingManifestPath -Label 'Staging manifest'
$stagingEntryCount = Assert-MvpStagingManifestIntegrity -StagingRoot $resolvedStagingRoot -Manifest $stagingManifest
$startupSummary = Read-MvpJsonObject -Path $startupSummaryPath -Label 'Startup summary'

$runId = [string](Get-MvpRequiredProperty -Value $stagingManifest -Name 'run_id' -Label 'Staging manifest')
$sourceFingerprint = [string](Get-MvpRequiredProperty -Value $stagingManifest -Name 'source_fingerprint' -Label 'Staging manifest')
$toolchain = [string](Get-MvpRequiredProperty -Value $stagingManifest -Name 'toolchain' -Label 'Staging manifest')
$target = [string](Get-MvpRequiredProperty -Value $stagingManifest -Name 'target' -Label 'Staging manifest')
$startupRunId = [string](Get-MvpRequiredProperty -Value $startupSummary -Name 'run_id' -Label 'Startup summary')
$startupSourceFingerprint = [string](Get-MvpRequiredProperty -Value $startupSummary -Name 'source_fingerprint' -Label 'Startup summary')
if ($startupRunId -ne $runId) {
    throw "Startup summary run_id '$startupRunId' differs from staging manifest run_id '$runId'."
}
if ($startupSourceFingerprint -ne $sourceFingerprint) {
    throw 'Startup summary source_fingerprint differs from the staging manifest.'
}
if (-not [string]::IsNullOrWhiteSpace($ExpectedSourceFingerprint) -and $ExpectedSourceFingerprint -ne $sourceFingerprint) {
    throw "Staging manifest source_fingerprint '$sourceFingerprint' differs from expected source fingerprint '$ExpectedSourceFingerprint'."
}

$validatedBuildSummaries = @()
$buildSummaryManifest = $null
if ($RequireF5Evidence) {
    $profileBuildSummary = Assert-MvpBuildSummaryEvidence `
        -Path $ProfileContractSummaryPath `
        -ExpectedKind 'profile-contract' `
        -ExpectedSourceFingerprint $sourceFingerprint
    $workspaceBuildSummary = Assert-MvpBuildSummaryEvidence `
        -Path $WorkspaceSummaryPath `
        -ExpectedKind 'workspace' `
        -ExpectedSourceFingerprint $sourceFingerprint
    $validatedBuildSummaries = @($profileBuildSummary, $workspaceBuildSummary)
    $buildSummaryManifest = [ordered]@{
        profile_contract = $profileBuildSummary.manifest_evidence
        workspace = $workspaceBuildSummary.manifest_evidence
    }
}

$stagedProjectRoot = [string](Get-MvpRequiredProperty -Value $startupSummary -Name 'staged_project_root' -Label 'Startup summary')
if ([IO.Path]::IsPathRooted($stagedProjectRoot) -or $stagedProjectRoot -match '(^|[\\/])\.\.([\\/]|$)' -or
    -not ($stagedProjectRoot -eq 'project' -or $stagedProjectRoot -match '^project[\\/]')) {
    throw "Startup summary must identify the staged project root as a safe relative path under 'project'; found '$stagedProjectRoot'."
}
if (-not (Test-Path -LiteralPath (Join-Path $resolvedStagingRoot $stagedProjectRoot) -PathType Container)) {
    throw "Startup summary staged_project_root '$stagedProjectRoot' does not exist under the staging root."
}
$processJournal = $null
$processJournalEvidence = $null
$productRuns = Assert-MvpSuccessfulProductRuns `
    -Runs $startupSummary.products `
    -StagedProjectRoot $stagedProjectRoot `
    -RequireRuntimeProductDiagnostics:$RequireProductEvidence
if ($RequireProductEvidence -or $RequireProjectCreationEvidence) {
    # The summary establishes run shape; journal and diagnostic evidence below independently
    # corroborate every claimed successful product result before it is published.
    $processJournal = Read-MvpProcessJournal -StagingRoot $resolvedStagingRoot
    $processJournalPath = Resolve-MvpStagedEvidenceFile `
        -RelativePath 'logs/process-execution-journal.jsonl' `
        -StagingRoot $resolvedStagingRoot `
        -Label 'Process execution journal'
    $processJournalEvidence = [ordered]@{
        path = 'logs/process-execution-journal.jsonl'
        sha256 = Get-MvpFileSha256 -Path $processJournalPath
        size_bytes = (Get-Item -LiteralPath $processJournalPath).Length
    }
    if ($RequireProductEvidence) {
        Assert-MvpProductEvidence `
            -Runs $productRuns `
            -StagingRoot $resolvedStagingRoot `
            -ExpectedStagingRoot $expectedStagingRoot `
            -ProcessJournal $processJournal
    }
}
$renderBackend = Resolve-MvpStableRuntimeRenderBackend -Runs $productRuns
$renderDevice = Resolve-MvpStableRuntimeRenderDevice -Runs $productRuns
$projectCreation = $null
$projectCreationProperty = $startupSummary.PSObject.Properties['project_creation']
if ($null -ne $projectCreationProperty -and $null -ne $projectCreationProperty.Value) {
    $projectCreation = $projectCreationProperty.Value
    Assert-MvpProjectCreationEvidence `
        -ProjectCreation $projectCreation `
        -StagingRoot $resolvedStagingRoot `
        -ExpectedStagingRoot $expectedStagingRoot `
        -StagedProjectRoot $stagedProjectRoot `
        -ProcessJournal $processJournal
}
elseif ($RequireProjectCreationEvidence) {
    throw "Startup summary is missing 'project_creation'."
}
if ($RequireReopenAutomation -and -not $RequireAuthoringAutomation) {
    throw 'RequireReopenAutomation requires RequireAuthoringAutomation so reopened state has a validated authoring predecessor.'
}
$authoringAutomation = $null
$baselineAutomation = $null
$projectIdentityEvidence = $null
if ($RequireF5Evidence) {
    $baselineAutomation = Assert-MvpBaselineAutomation `
        -Automation (Get-MvpRequiredProperty -Value $startupSummary -Name 'baseline_automation' -Label 'Startup summary') `
        -StagedProjectRoot $stagedProjectRoot `
        -ResolvedStagingRoot $resolvedStagingRoot `
        -ExpectedStagingRoot $expectedStagingRoot `
        -StagingManifest $stagingManifest `
        -ProcessJournal $processJournal
}
if ($RequireAuthoringAutomation) {
    $authoringAutomation = Assert-MvpAuthoringAutomation `
        -Automation (Get-MvpRequiredProperty -Value $startupSummary -Name 'authoring_automation' -Label 'Startup summary') `
        -StagedProjectRoot $stagedProjectRoot `
        -ResolvedStagingRoot $resolvedStagingRoot `
        -ExpectedStagingRoot $expectedStagingRoot `
        -StagingManifest $stagingManifest `
        -ProcessJournal $processJournal
}
if ($RequireF5Evidence) {
    $null = Assert-MvpExpectedAuthoringSceneDelta `
        -BaselineSnapshot $baselineAutomation.snapshot `
        -AuthoringSnapshot $authoringAutomation.snapshot
}
$reopenAutomation = @()
if ($RequireReopenAutomation) {
    $reopenAutomationProperty = $startupSummary.PSObject.Properties['reopen_automation']
    if ($null -eq $reopenAutomationProperty -or $null -eq $reopenAutomationProperty.Value) {
        throw "Startup summary is missing 'reopen_automation'."
    }
    $reopenAutomation = Assert-MvpReopenAutomation `
        -Automations $reopenAutomationProperty.Value `
        -AuthoringAutomation $authoringAutomation `
        -StagedProjectRoot $stagedProjectRoot `
        -ResolvedStagingRoot $resolvedStagingRoot `
        -ExpectedStagingRoot $expectedStagingRoot `
        -StagingManifest $stagingManifest `
        -ProcessJournal $processJournal
    $null = Assert-MvpPostAuthoringRuntime -ProductRuns $productRuns
    if ($RequireProductEvidence) {
        if ($null -eq $projectCreation) {
            throw 'F5 editor window evidence requires project_creation evidence.'
        }
        Assert-MvpF5EditorWindowEvidence `
            -ProjectCreation $projectCreation `
            -ProductRuns $productRuns `
            -StagingRoot $resolvedStagingRoot
    }
}
if ($RequireF5Evidence) {
    $projectIdentityEvidence = Assert-MvpF5ProjectIdentity `
        -ProjectCreation $projectCreation `
        -ProductRuns $productRuns `
        -BaselineAutomation $baselineAutomation `
        -AuthoringAutomation $authoringAutomation `
        -ReopenAutomation $reopenAutomation `
        -StagedProjectRoot $stagedProjectRoot
    Assert-MvpF5ProcessTimingEvidence `
        -ProjectCreation $projectCreation `
        -ProductRuns $productRuns `
        -BaselineAutomation $baselineAutomation `
        -AuthoringAutomation $authoringAutomation `
        -ReopenAutomation $reopenAutomation
}
$stagingManifestHash = Get-MvpFileSha256 -Path $stagingManifestPath
$startupSummaryHash = Get-MvpFileSha256 -Path $startupSummaryPath

$manifest = [ordered]@{
    schema_version = if ($RequireF5Evidence) { 2 } else { 1 }
    run_id = $runId
    source_fingerprint = $sourceFingerprint
    toolchain = $toolchain
    target = $target
    render_backend = $renderBackend
    render_adapter = if ($null -eq $renderDevice) { $null } else { $renderDevice.adapter }
    render_adapter_type = if ($null -eq $renderDevice) { $null } else { $renderDevice.adapter_type }
    render_device_limits = if ($null -eq $renderDevice) { $null } else { $renderDevice.limits }
    created_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
    staging_manifest_sha256 = $stagingManifestHash
    startup_summary_sha256 = $startupSummaryHash
    staging_entry_count = $stagingEntryCount
    staged_project_root = $stagedProjectRoot
    project_identity = $projectIdentityEvidence
    project_creation = $projectCreation
    process_execution_journal = $processJournalEvidence
    product_runs = $productRuns
    baseline_automation = $baselineAutomation
    authoring_automation = $authoringAutomation
    reopen_automation = $reopenAutomation
}
if ($RequireF5Evidence) {
    $manifest['build_summaries'] = $buildSummaryManifest
}
$manifestPath = Publish-MvpAcceptanceEvidencePackage `
    -StagingRoot $resolvedStagingRoot `
    -EvidenceRoot $resolvedEvidenceRoot `
    -Manifest $manifest `
    -BuildSummaries $validatedBuildSummaries

$result = [ordered]@{
    run_id = $runId
    source_fingerprint = $sourceFingerprint
    toolchain = $toolchain
    target = $target
    render_backend = $renderBackend
    render_adapter = if ($null -eq $renderDevice) { $null } else { $renderDevice.adapter }
    render_adapter_type = if ($null -eq $renderDevice) { $null } else { $renderDevice.adapter_type }
    render_device_limits = if ($null -eq $renderDevice) { $null } else { $renderDevice.limits }
    staging_manifest_sha256 = $stagingManifestHash
    startup_summary_sha256 = $startupSummaryHash
    staging_entry_count = $stagingEntryCount
    staged_project_root = $stagedProjectRoot
    project_identity = $projectIdentityEvidence
    project_creation = $projectCreation
    process_execution_journal = $processJournalEvidence
    product_runs = $productRuns
    baseline_automation = $baselineAutomation
    authoring_automation = $authoringAutomation
    reopen_automation = $reopenAutomation
    build_summaries = $buildSummaryManifest
    manifest = $manifestPath
}
if ($Json) {
    $result | ConvertTo-Json -Depth 64
}
else {
    $result
}
}
finally {
    if ($null -ne $acceptanceStagingSnapshotLease) {
        Close-MvpAcceptanceStagingSnapshotLease -Lease $acceptanceStagingSnapshotLease
    }
    if (-not [string]::IsNullOrWhiteSpace($acceptanceStagingSnapshotIdentity)) {
        Remove-MvpAcceptanceStagingSnapshot `
            -SnapshotRoot $acceptanceStagingSnapshotRoot `
            -ExpectedRootIdentity $acceptanceStagingSnapshotIdentity
    }
}
