$ErrorActionPreference = 'Stop'

Describe 'Window resize geometry evidence' {
    BeforeAll {
        $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
        . (Join-Path $repoRoot 'tools\ui-window-resize-geometry-evidence.ps1')

        function New-ResizeTimeline {
            param(
                [int64]$StalePresentCount = 0,
                [int64]$CoalescedInputCount = 2,
                [double[]]$MatchingLatenciesUs = @(4000, 8000, 12000),
                [double[]]$PrepareLatenciesUs = @(2000, 3000, 4000)
            )

            $counters = @(
                [pscustomobject]@{ name = 'ui.window_resize.window_metrics_received_count'; value = 6 },
                [pscustomobject]@{ name = 'ui.window_resize.duplicate_metrics_suppressed_count'; value = 1 },
                [pscustomobject]@{ name = 'ui.window_resize.geometry_coalesced_input_count'; value = $CoalescedInputCount },
                [pscustomobject]@{ name = 'ui.window_resize.matching_geometry_presented_input_count'; value = 3 },
                [pscustomobject]@{ name = 'ui.window_resize.geometry_fallback_input_count'; value = 0 },
                [pscustomobject]@{ name = 'ui.window_resize.geometry_prepare_count'; value = 3 },
                [pscustomobject]@{ name = 'ui.window_resize.geometry_commit_count'; value = 3 },
                [pscustomobject]@{ name = 'ui.window_resize.geometry_noop_count'; value = 0 },
                [pscustomobject]@{ name = 'ui.window_resize.geometry_fallback_transaction_count'; value = 0 },
                [pscustomobject]@{ name = 'ui.window_resize.stale_geometry_present_count'; value = $StalePresentCount },
                [pscustomobject]@{ name = 'ui.window_resize.geometry_full_hit_index_rebuild_count'; value = 0 },
                [pscustomobject]@{ name = 'ui.window_resize.geometry_full_command_rebuild_count'; value = 0 }
            )
            $counters += @($MatchingLatenciesUs | ForEach-Object {
                    [pscustomobject]@{ name = 'ui.window_resize.input_to_matching_geometry_us'; value = $_ }
                })
            $counters += @($PrepareLatenciesUs | ForEach-Object {
                    [pscustomobject]@{ name = 'ui.window_resize.geometry_prepare_us'; value = $_ }
                })
            return [pscustomobject]@{ counters = @($counters) }
        }
    }

    It 'accepts complete conserved geometry evidence within latency budgets' {
        $result = Test-ZirconWindowResizeGeometryEvidence -Timeline (New-ResizeTimeline)

        $result.ready | Should Be $true
        @($result.blockers).Count | Should Be 0
        $result.conservation.non_duplicate_inputs | Should Be 5
        $result.conservation.resolved_inputs | Should Be 5
        $result.latency.input_to_matching_geometry_us.p95 | Should Be 12000
        $result.latency.geometry_prepare_us.p95 | Should Be 4000
    }

    It 'rejects missing evidence instead of interpreting it as zero' {
        $timeline = New-ResizeTimeline
        $timeline.counters = @($timeline.counters | Where-Object {
                $_.name -ne 'ui.window_resize.stale_geometry_present_count'
            })

        $result = Test-ZirconWindowResizeGeometryEvidence -Timeline $timeline

        $result.ready | Should Be $false
        @($result.blockers.code) -contains 'missing_counter' | Should Be $true
    }

    It 'rejects stale presentation, unexplained inputs, and ordinary fallbacks' {
        $stale = Test-ZirconWindowResizeGeometryEvidence -Timeline (
            New-ResizeTimeline -StalePresentCount 1
        )
        $unexplained = Test-ZirconWindowResizeGeometryEvidence -Timeline (
            New-ResizeTimeline -CoalescedInputCount 1
        )
        $fallbackTimeline = New-ResizeTimeline
        ($fallbackTimeline.counters | Where-Object {
                $_.name -eq 'ui.window_resize.geometry_fallback_input_count'
            }).value = 1
        $fallback = Test-ZirconWindowResizeGeometryEvidence -Timeline $fallbackTimeline

        @($stale.blockers.code) -contains 'stale_geometry_presented' | Should Be $true
        @($unexplained.blockers.code) -contains 'resize_input_conservation_failed' | Should Be $true
        @($fallback.blockers.code) -contains 'ordinary_resize_fallback' | Should Be $true
    }

    It 'rejects unresolved prepare transactions and full retained rebuilds' {
        $transactionTimeline = New-ResizeTimeline
        ($transactionTimeline.counters | Where-Object {
                $_.name -eq 'ui.window_resize.geometry_prepare_count'
            }).value = 4
        $transactionTimeline.counters += [pscustomobject]@{
            name = 'ui.window_resize.geometry_prepare_us'
            value = 4000
        }
        $transaction = Test-ZirconWindowResizeGeometryEvidence -Timeline $transactionTimeline

        $rebuildTimeline = New-ResizeTimeline
        ($rebuildTimeline.counters | Where-Object {
                $_.name -eq 'ui.window_resize.geometry_full_hit_index_rebuild_count'
            }).value = 1
        ($rebuildTimeline.counters | Where-Object {
                $_.name -eq 'ui.window_resize.geometry_full_command_rebuild_count'
            }).value = 1
        $rebuild = Test-ZirconWindowResizeGeometryEvidence -Timeline $rebuildTimeline

        @($transaction.blockers.code) -contains 'geometry_transaction_conservation_failed' |
            Should Be $true
        @($rebuild.blockers.code) -contains 'full_hit_index_rebuild' | Should Be $true
        @($rebuild.blockers.code) -contains 'full_command_rebuild' | Should Be $true
    }

    It 'rejects incomplete or over-budget latency samples' {
        $missing = Test-ZirconWindowResizeGeometryEvidence -Timeline (
            New-ResizeTimeline -MatchingLatenciesUs @(4000, 8000)
        )
        $slow = Test-ZirconWindowResizeGeometryEvidence -Timeline (
            New-ResizeTimeline -MatchingLatenciesUs @(4000, 8000, 40000)
        )
        $slowPrepare = Test-ZirconWindowResizeGeometryEvidence -Timeline (
            New-ResizeTimeline -PrepareLatenciesUs @(2000, 3000, 9000)
        )

        @($missing.blockers.code) -contains 'matching_geometry_latency_membership_failed' |
            Should Be $true
        @($slow.blockers.code) -contains 'matching_geometry_latency_budget_exceeded' |
            Should Be $true
        @($slowPrepare.blockers.code) -contains 'geometry_prepare_latency_budget_exceeded' |
            Should Be $true
    }
}
