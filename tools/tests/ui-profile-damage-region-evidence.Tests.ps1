$script:EvidenceModule = Join-Path $PSScriptRoot '..\ui-profile-counter-evidence.ps1'
$script:CaptureScript = Join-Path $PSScriptRoot '..\ui-profile-capture.ps1'
$script:CaptureManifestScript = Join-Path $PSScriptRoot '..\profile-capture-manifest.ps1'
. $script:EvidenceModule

function Write-DamageTimeline {
    param(
        [string]$ProfileDir,
        [object[]]$Counters
    )

    New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
    [pscustomobject]@{ counters = @($Counters) } |
        ConvertTo-Json -Depth 5 |
        Set-Content -LiteralPath (Join-Path $ProfileDir 'timeline.zrtrace.json') -Encoding UTF8
}

function New-DamageCounterBatch {
    param(
        [int]$BatchCount,
        [double]$RectCount = 2,
        [double]$SourceRectCount = 2,
        [double]$SimplificationCount = 0,
        [double]$RepresentedArea = 200,
        [double]$BoundingArea = 1000,
        [double]$BoundingOverdrawArea = 800
    )

    $counters = @()
    for ($index = 0; $index -lt $BatchCount; $index++) {
        foreach ($entry in @(
                @{ metric = 'rect_count'; value = $RectCount },
                @{ metric = 'source_rect_count'; value = $SourceRectCount },
                @{ metric = 'simplification_count'; value = $SimplificationCount },
                @{ metric = 'represented_area'; value = $RepresentedArea },
                @{ metric = 'bounding_area'; value = $BoundingArea },
                @{ metric = 'bounding_overdraw_area'; value = $BoundingOverdrawArea }
            )) {
            $counters += [pscustomobject]@{
                    name = "ui.click.redraw_damage_$($entry.metric)"
                    value = [double]$entry.value
                    timestamp_us = [int64](($index * 10) + $counters.Count)
                }
        }
    }
    return $counters
}

function Write-DamageSourceManifest {
    param(
        [string]$ProfileDir,
        [int]$RunOrdinal,
        [int]$MeasuredRunCount,
        [string]$ScenarioName = 'viewport_toolbar_click',
        [string]$SourceBinding = 'stable'
    )

    $sourceHash = if ($SourceBinding -eq 'stable') {
        ('a' * 64) -join ''
    }
    else {
        ('b' * 64) -join ''
    }
    $runtimeHash = if ($SourceBinding -eq 'stable') {
        ('c' * 64) -join ''
    }
    else {
        ('d' * 64) -join ''
    }
    [pscustomobject]@{
        schema_version = 2
        session_id = "damage-run-$RunOrdinal"
        scenario = $ScenarioName
        input_fixture = $null
        repository = [pscustomobject]@{
            root = 'E:\Git\ZirconEngine'
            git = [pscustomobject]@{
                revision = ('1' * 40) -join ''
                dirty = $true
                dirty_entry_count = 1
                dirty_tree_sha256 = $sourceHash
            }
            critical_source_files = @(
                [pscustomobject]@{
                    relative_path = 'zircon_editor/src/ui/retained_host/host_contract/redraw.rs'
                    sha256 = $sourceHash
                    byte_length = 100
                }
            )
        }
        binaries = [pscustomobject]@{
            editor = [pscustomobject]@{ sha256 = $sourceHash; byte_length = 200 }
            runtime = [pscustomobject]@{ sha256 = $runtimeHash; byte_length = 300 }
        }
        capture = [pscustomobject]@{
            options = [pscustomobject]@{
                auto_click_count = 1000
                run_phase = 'measured'
                run_ordinal = $RunOrdinal
                measured_run_count = $MeasuredRunCount
                run_process_scope = 'within_process_warm_measure'
                within_process_warmup_present_count = 1
                max_counters = 65536
                max_frames = 2048
                max_spans = 65536
                use_tracy = $false
                use_wpr = $false
            }
            tool_files = @(
                [pscustomobject]@{
                    relative_path = 'tools/ui-profile-counter-evidence.ps1'
                    sha256 = $sourceHash
                    byte_length = 400
                }
            )
        }
    } | ConvertTo-Json -Depth 8 |
        Set-Content -LiteralPath (Join-Path $ProfileDir 'source_manifest.json') -Encoding UTF8
}

Describe 'UI profile bounded damage-region evidence' {
    It 'registers the evidence export in every completed capture run' {
        $source = Get-Content -LiteralPath $script:CaptureScript -Raw
        $moduleSource = Get-Content -LiteralPath $script:EvidenceModule -Raw
        $manifestSource = Get-Content -LiteralPath $script:CaptureManifestScript -Raw

        $source | Should Match 'Export-ZirconDamageRegionEvidence'
        $source | Should Match 'Export-ZirconDamageRegionTrialEvidence'
        $source | Should Match '-CounterScenarioName \(Resolve-InteractionScenarioName'
        $moduleSource | Should Match "ui_damage_region_evidence\.json"
        $moduleSource | Should Match "ui_damage_region_trial_evidence\.json"
        foreach ($path in @(
                'host_contract/redraw.rs',
                'host_contract/redraw/damage_region.rs',
                'host_contract/redraw/request.rs',
                'host_contract/redraw/request/constructors.rs',
                'host_contract/redraw/request/merge.rs',
                'host_contract/redraw/request/query.rs'
            )) {
            $manifestSource.Replace('\\', '/') | Should Match ([regex]::Escape($path))
        }
    }

    It 'exports an eligible three-rect trial decision from one complete 100-batch run' {
        $profileDir = "E:\zircon-profiles\pester-damage-$([guid]::NewGuid().ToString('N'))"
        try {
            Write-DamageTimeline -ProfileDir $profileDir `
                -Counters (New-DamageCounterBatch -BatchCount 100)

            $result = Export-ZirconDamageRegionEvidence `
                -ProfileDir $profileDir `
                -ScenarioName 'viewport_toolbar_click' `
                -CounterScenarioName 'click'

            $result.schema_version | Should Be 1
            $result.capture_scenario | Should Be 'viewport_toolbar_click'
            $result.counter_scenario | Should Be 'click'
            $result.has_region_samples | Should Be $true
            $result.sample_count | Should Be 100
            $result.total_source_rect_count | Should Be 200
            $result.total_simplification_count | Should Be 0
            $result.bounding_overdraw_ratio | Should Be 0.8
            $result.simplification_ratio | Should Be 0
            $result.eligible_for_multi_region_trial | Should Be $true
            Test-Path -LiteralPath (Join-Path $profileDir 'ui_damage_region_evidence.json') |
                Should Be $true
        }
        finally {
            if (Test-Path -LiteralPath $profileDir) {
                Remove-Item -LiteralPath $profileDir -Recurse -Force
            }
        }
    }

    It 'records high simplification pressure without treating measurement as a failure' {
        $profileDir = "E:\zircon-profiles\pester-damage-$([guid]::NewGuid().ToString('N'))"
        try {
            Write-DamageTimeline -ProfileDir $profileDir -Counters (
                New-DamageCounterBatch `
                    -BatchCount 100 `
                    -RectCount 3 `
                    -SourceRectCount 4 `
                    -SimplificationCount 1 `
                    -RepresentedArea 800 `
                    -BoundingArea 1000 `
                    -BoundingOverdrawArea 200
            )

            $result = Export-ZirconDamageRegionEvidence `
                -ProfileDir $profileDir `
                -ScenarioName 'viewport_toolbar_click' `
                -CounterScenarioName 'click'

            $result.has_region_samples | Should Be $true
            $result.sample_count | Should Be 100
            $result.simplification_ratio | Should Be 0.25
            $result.eligible_for_multi_region_trial | Should Be $false
        }
        finally {
            if (Test-Path -LiteralPath $profileDir) {
                Remove-Item -LiteralPath $profileDir -Recurse -Force
            }
        }
    }

    It 'fails closed when only part of the six-counter schema is present' {
        $profileDir = "E:\zircon-profiles\pester-damage-$([guid]::NewGuid().ToString('N'))"
        try {
            Write-DamageTimeline -ProfileDir $profileDir -Counters @(
                [pscustomobject]@{
                    name = 'ui.click.redraw_damage_rect_count'
                    value = 2
                    timestamp_us = 1
                }
            )

            $didThrow = $false
            $errorMessage = ''
            try {
                Export-ZirconDamageRegionEvidence `
                    -ProfileDir $profileDir `
                    -ScenarioName 'viewport_toolbar_click' `
                    -CounterScenarioName 'click' | Out-Null
            }
            catch {
                $didThrow = $true
                $errorMessage = $_.Exception.Message
            }
            $didThrow | Should Be $true
            $errorMessage | Should Match 'part of the six-counter schema'
        }
        finally {
            if (Test-Path -LiteralPath $profileDir) {
                Remove-Item -LiteralPath $profileDir -Recurse -Force
            }
        }
    }

    It 'fails closed when bounding and represented areas contradict overdraw' {
        $profileDir = "E:\zircon-profiles\pester-damage-$([guid]::NewGuid().ToString('N'))"
        try {
            Write-DamageTimeline -ProfileDir $profileDir -Counters (
                New-DamageCounterBatch `
                    -BatchCount 1 `
                    -RepresentedArea 200 `
                    -BoundingArea 1000 `
                    -BoundingOverdrawArea 700
            )

            $didThrow = $false
            $errorMessage = ''
            try {
                Export-ZirconDamageRegionEvidence `
                    -ProfileDir $profileDir `
                    -ScenarioName 'viewport_toolbar_click' `
                    -CounterScenarioName 'click' | Out-Null
            }
            catch {
                $didThrow = $true
                $errorMessage = $_.Exception.Message
            }
            $didThrow | Should Be $true
            $errorMessage | Should Match 'contradicts its bounding overdraw area'
        }
        finally {
            if (Test-Path -LiteralPath $profileDir) {
                Remove-Item -LiteralPath $profileDir -Recurse -Force
            }
        }
    }

    It 'exports a valid non-eligible artifact when a run has no region presents' {
        $profileDir = "E:\zircon-profiles\pester-damage-$([guid]::NewGuid().ToString('N'))"
        try {
            Write-DamageTimeline -ProfileDir $profileDir -Counters @(
                [pscustomobject]@{
                    name = 'ui.startup.redraw_full_frame'
                    value = 1
                    timestamp_us = 1
                }
            )

            $result = Export-ZirconDamageRegionEvidence `
                -ProfileDir $profileDir `
                -ScenarioName 'material_lab_startup' `
                -CounterScenarioName 'startup'

            $result.has_region_samples | Should Be $false
            $result.sample_count | Should Be 0
            $result.eligible_for_multi_region_trial | Should Be $false
        }
        finally {
            if (Test-Path -LiteralPath $profileDir) {
                Remove-Item -LiteralPath $profileDir -Recurse -Force
            }
        }
    }

    It 'recommends a trial only after three source-bound eligible measured runs' {
        $trialDir = "E:\zircon-profiles\pester-damage-trial-$([guid]::NewGuid().ToString('N'))"
        try {
            $profileDirs = @()
            foreach ($runOrdinal in 1..3) {
                $profileDir = Join-Path $trialDir "run-$runOrdinal"
                $profileDirs += $profileDir
                Write-DamageTimeline -ProfileDir $profileDir `
                    -Counters (New-DamageCounterBatch -BatchCount 100)
                Write-DamageSourceManifest `
                    -ProfileDir $profileDir `
                    -RunOrdinal $runOrdinal `
                    -MeasuredRunCount 3
                Export-ZirconDamageRegionEvidence `
                    -ProfileDir $profileDir `
                    -ScenarioName 'viewport_toolbar_click' `
                    -CounterScenarioName 'click' | Out-Null
            }

            $result = Export-ZirconDamageRegionTrialEvidence `
                -ProfileDirs $profileDirs `
                -OutputDir $trialDir `
                -ScenarioName 'viewport_toolbar_click'

            $result.schema_version | Should Be 1
            $result.run_count | Should Be 3
            $result.total_sample_count | Should Be 300
            $result.every_run_eligible | Should Be $true
            $result.trial_recommended | Should Be $true
            $result.performance_accepted | Should Be $false
            $result.source_binding_id | Should Match '^[0-9a-f]{64}$'
            Test-Path -LiteralPath (Join-Path $trialDir 'ui_damage_region_trial_evidence.json') |
                Should Be $true
        }
        finally {
            if (Test-Path -LiteralPath $trialDir) {
                Remove-Item -LiteralPath $trialDir -Recurse -Force
            }
        }
    }

    It 'keeps a complete two-run aggregate below the minimum trial count' {
        $trialDir = "E:\zircon-profiles\pester-damage-trial-$([guid]::NewGuid().ToString('N'))"
        try {
            $profileDirs = @()
            foreach ($runOrdinal in 1..2) {
                $profileDir = Join-Path $trialDir "run-$runOrdinal"
                $profileDirs += $profileDir
                Write-DamageTimeline -ProfileDir $profileDir `
                    -Counters (New-DamageCounterBatch -BatchCount 100)
                Write-DamageSourceManifest `
                    -ProfileDir $profileDir `
                    -RunOrdinal $runOrdinal `
                    -MeasuredRunCount 2
                Export-ZirconDamageRegionEvidence `
                    -ProfileDir $profileDir `
                    -ScenarioName 'viewport_toolbar_click' `
                    -CounterScenarioName 'click' | Out-Null
            }

            $result = Export-ZirconDamageRegionTrialEvidence `
                -ProfileDirs $profileDirs `
                -OutputDir $trialDir `
                -ScenarioName 'viewport_toolbar_click'

            $result.run_count | Should Be 2
            $result.every_run_eligible | Should Be $true
            $result.trial_recommended | Should Be $false
            $result.performance_accepted | Should Be $false
        }
        finally {
            if (Test-Path -LiteralPath $trialDir) {
                Remove-Item -LiteralPath $trialDir -Recurse -Force
            }
        }
    }

    It 'fails closed when one measured run has a different source binding' {
        $trialDir = "E:\zircon-profiles\pester-damage-trial-$([guid]::NewGuid().ToString('N'))"
        try {
            $profileDirs = @()
            foreach ($runOrdinal in 1..3) {
                $profileDir = Join-Path $trialDir "run-$runOrdinal"
                $profileDirs += $profileDir
                Write-DamageTimeline -ProfileDir $profileDir `
                    -Counters (New-DamageCounterBatch -BatchCount 100)
                Write-DamageSourceManifest `
                    -ProfileDir $profileDir `
                    -RunOrdinal $runOrdinal `
                    -MeasuredRunCount 3 `
                    -SourceBinding $(if ($runOrdinal -eq 3) { 'changed' } else { 'stable' })
                Export-ZirconDamageRegionEvidence `
                    -ProfileDir $profileDir `
                    -ScenarioName 'viewport_toolbar_click' `
                    -CounterScenarioName 'click' | Out-Null
            }

            $didThrow = $false
            $errorMessage = ''
            try {
                Export-ZirconDamageRegionTrialEvidence `
                    -ProfileDirs $profileDirs `
                    -OutputDir $trialDir `
                    -ScenarioName 'viewport_toolbar_click' | Out-Null
            }
            catch {
                $didThrow = $true
                $errorMessage = $_.Exception.Message
            }
            $didThrow | Should Be $true
            $errorMessage | Should Match 'same source binding'
            Test-Path -LiteralPath (Join-Path $trialDir 'ui_damage_region_trial_evidence.json') |
                Should Be $false
        }
        finally {
            if (Test-Path -LiteralPath $trialDir) {
                Remove-Item -LiteralPath $trialDir -Recurse -Force
            }
        }
    }
}
