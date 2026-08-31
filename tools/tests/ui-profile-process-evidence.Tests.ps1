$script:ProcessEvidenceModule = Join-Path $PSScriptRoot '..\ui-profile-process-evidence.ps1'
if (Test-Path -LiteralPath $script:ProcessEvidenceModule) {
    . $script:ProcessEvidenceModule
}

function New-ValidProcessEvidence {
    return [pscustomobject][ordered]@{
        process_id = 4242
        elapsed_ms = 2000
        processor_time_delta_ms = 200
        cpu_core_utilization_percent = 10
        cpu_system_utilization_percent = 1.25
        logical_processor_count = 8
        start_working_set_bytes = 100000000
        end_working_set_bytes = 110000000
        peak_working_set_bytes = 120000000
        start_private_bytes = 80000000
        end_private_bytes = 90000000
        peak_private_bytes = 95000000
        quiescence_process_id = 4242
        quiescence_requested_ms = 2000
        quiescence_elapsed_ms = 2050
        quiescence_working_set_bytes = 108000000
        quiescence_private_bytes = 88000000
        quiescence_sampled = $true
    }
}

Describe 'ui profile process evidence' {
    It 'rejects CPU evidence that exceeds one average core' {
        $evidence = New-ValidProcessEvidence
        $evidence.processor_time_delta_ms = 2002
        $evidence.cpu_core_utilization_percent = 100.1
        $evidence.cpu_system_utilization_percent = 12.5125

        (Test-ZirconUiInteractionProcessEvidence -Interaction $evidence) | Should Be $false
    }

    It 'rejects inconsistent core and system CPU percentages' {
        $evidence = New-ValidProcessEvidence
        $evidence.cpu_system_utilization_percent = 5

        (Test-ZirconUiInteractionProcessEvidence -Interaction $evidence) | Should Be $false
    }

    It 'rejects excessive working-set or private-byte growth' {
        $evidence = New-ValidProcessEvidence
        $evidence.peak_working_set_bytes = $evidence.start_working_set_bytes + 100663297
        (Test-ZirconUiInteractionProcessEvidence -Interaction $evidence) | Should Be $false

        $evidence = New-ValidProcessEvidence
        $evidence.end_private_bytes = $evidence.start_private_bytes + 67108865
        $evidence.peak_private_bytes = $evidence.end_private_bytes
        (Test-ZirconUiInteractionProcessEvidence -Interaction $evidence) | Should Be $false
    }

    It 'rejects missing, cross-process, or excessive quiescence evidence' {
        $evidence = New-ValidProcessEvidence
        $evidence.quiescence_sampled = $false
        (Test-ZirconUiInteractionProcessEvidence -Interaction $evidence) | Should Be $false

        $evidence = New-ValidProcessEvidence
        $evidence.quiescence_process_id = $evidence.process_id + 1
        (Test-ZirconUiInteractionProcessEvidence -Interaction $evidence) | Should Be $false

        $evidence = New-ValidProcessEvidence
        $evidence.quiescence_working_set_bytes =
            $evidence.start_working_set_bytes + 67108865
        $evidence.peak_working_set_bytes = $evidence.quiescence_working_set_bytes
        (Test-ZirconUiInteractionProcessEvidence -Interaction $evidence) | Should Be $false
    }

    It 'rejects CPU time that exceeds the scenario operation budget' {
        $evidence = New-ValidProcessEvidence
        $evidence.processor_time_delta_ms = 300
        $evidence.cpu_core_utilization_percent = 15
        $evidence.cpu_system_utilization_percent = 1.875

        (Test-ZirconUiInteractionProcessEvidence `
                -Interaction $evidence `
                -OperationCount 1000 `
                -MaxCpuMsPerOperation 0.25) | Should Be $false
    }

    It 'accepts complete evidence at every process budget boundary' {
        $evidence = New-ValidProcessEvidence
        $evidence.processor_time_delta_ms = 2000
        $evidence.cpu_core_utilization_percent = 100
        $evidence.cpu_system_utilization_percent = 12.5
        $evidence.end_working_set_bytes = $evidence.start_working_set_bytes + 67108864
        $evidence.peak_working_set_bytes = $evidence.start_working_set_bytes + 100663296
        $evidence.end_private_bytes = $evidence.start_private_bytes + 67108864
        $evidence.peak_private_bytes = $evidence.start_private_bytes + 100663296

        (Test-ZirconUiInteractionProcessEvidence `
                -Interaction $evidence `
                -OperationCount 1000 `
                -MaxCpuMsPerOperation 2.0) | Should Be $true
    }
}
