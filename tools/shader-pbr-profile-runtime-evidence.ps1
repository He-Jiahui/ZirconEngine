Set-StrictMode -Version Latest

function Get-ZirconShaderPbrRuntimeProfileEvidence {
    param(
        [Parameter(Mandatory = $true)][string]$ProfileRoot,
        [Parameter(Mandatory = $true)][string]$SessionId
    )

    if (-not (Test-Path -LiteralPath $ProfileRoot -PathType Container)) {
        throw "Shader PBR measured run did not export its Zircon runtime profile: $ProfileRoot"
    }
    $requiredFiles = [ordered]@{
        timeline = "timeline.zrtrace.json"
        hotspots = "hotspots.json"
        counter_hotspots = "counter_hotspots.json"
        summary = "summary.md"
    }
    $fingerprints = [ordered]@{}
    foreach ($entry in $requiredFiles.GetEnumerator()) {
        $matches = @(Get-ChildItem -LiteralPath $ProfileRoot -Recurse -File -Filter $entry.Value)
        if ($matches.Count -ne 1) {
            throw "Shader PBR measured run expected exactly one $($entry.Value) beneath $ProfileRoot, found $($matches.Count)."
        }
        $fingerprints[$entry.Key] = Get-ZirconProfileRequiredFileFingerprint `
            -Path $matches[0].FullName `
            -Description "Shader PBR Zircon runtime profile $($entry.Key)"
    }
    try {
        $timeline = Get-Content -LiteralPath $fingerprints.timeline.path -Raw |
            ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "Shader PBR Zircon runtime profile timeline is malformed: $($fingerprints.timeline.path)"
    }
    if ([string]$timeline.session_id -ne $SessionId) {
        throw "Shader PBR Zircon runtime profile session mismatch: expected=$SessionId actual=$($timeline.session_id)"
    }
    $activeProperty = $timeline.PSObject.Properties["active"]
    $featureEnabledProperty = $timeline.PSObject.Properties["feature_enabled"]
    if ($null -eq $activeProperty -or $null -eq $featureEnabledProperty -or
        $activeProperty.Value -isnot [bool] -or
        $featureEnabledProperty.Value -isnot [bool] -or
        [bool]$activeProperty.Value -or -not [bool]$featureEnabledProperty.Value) {
        throw "Shader PBR Zircon runtime profile is not a completed enabled capture for session $SessionId."
    }
    $timelineOutputRootProperty = $timeline.PSObject.Properties["output_root"]
    if ($null -eq $timelineOutputRootProperty -or
        [string]::IsNullOrWhiteSpace([string]$timelineOutputRootProperty.Value)) {
        throw "Shader PBR Zircon runtime profile is missing its output root for session $SessionId."
    }
    $expectedOutputRoot = [System.IO.Path]::GetFullPath($ProfileRoot)
    try {
        $timelineOutputRoot = [System.IO.Path]::GetFullPath(
            [string]$timelineOutputRootProperty.Value
        )
    }
    catch {
        throw "Shader PBR Zircon runtime profile output root is malformed for session $SessionId."
    }
    if (-not $timelineOutputRoot.Equals(
        $expectedOutputRoot,
        [System.StringComparison]::OrdinalIgnoreCase
    )) {
        throw "Shader PBR Zircon runtime profile output root mismatch: expected=$expectedOutputRoot actual=$timelineOutputRoot"
    }
    $retentionRecords = @($timeline.recorder_retention)
    if ($retentionRecords.Count -eq 0) {
        throw "Shader PBR Zircon runtime profile is missing recorder retention evidence for session $SessionId."
    }
    foreach ($retention in $retentionRecords) {
        $retentionStreams = [ordered]@{
            frames = $retention.frames
            spans = $retention.spans
            counters = $retention.counters
        }
        foreach ($entry in $retentionStreams.GetEnumerator()) {
            $streamRetention = $entry.Value
            if ($null -eq $streamRetention -or
                $null -eq $streamRetention.PSObject.Properties["overwritten"]) {
                throw "Shader PBR Zircon runtime profile is missing $($entry.Key) retention evidence for session $SessionId."
            }
            $overwritten = [System.Convert]::ToUInt64(
                $streamRetention.PSObject.Properties["overwritten"].Value
            )
            if ($overwritten -ne 0) {
                throw "Shader PBR Zircon runtime profile lost $($entry.Key) samples for session $SessionId."
            }
        }
    }
    $shaderStageCounts = [ordered]@{}
    foreach ($stage in @(
        "material_requirement_admission",
        "mesh_source_build",
        "module_include_resolution",
        "template_assembly",
        "source_hash",
        "naga_validation",
        "disk_cache_lookup",
        "disk_cache_write",
        "wgpu_pipeline_error_scope_pop"
    )) {
        $shaderStageCounts[$stage] = @(
            $timeline.spans | Where-Object {
                [string]$_.category -eq "shader_pipeline" -and [string]$_.name -eq $stage
            }
        ).Count
    }

    return [pscustomobject]@{
        schema = "zircon_shader_pbr_runtime_profile_v1"
        session_id = $SessionId
        output_root = [System.IO.Path]::GetFullPath($ProfileRoot)
        span_count = @($timeline.spans).Count
        counter_count = @($timeline.counters).Count
        recorder_retention = $retentionRecords
        shader_pipeline_stage_counts = [pscustomobject]$shaderStageCounts
        artifacts = [pscustomobject]$fingerprints
    }
}
