$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$supervisorModule = Join-Path $repoRoot 'tools\mvp\StagedProcessSupervisor.psm1'
$journalModule = Join-Path $repoRoot 'tools\mvp\MvpProcessLifecycleJournal.psm1'
$environmentPolicyModule = Join-Path $repoRoot 'tools\mvp\MvpProcessEnvironmentPolicy.psm1'
$artifactBudgetModule = Join-Path $repoRoot 'tools\mvp\MvpRunArtifactBudget.psm1'
$qualificationContextModule = Join-Path $repoRoot 'tools\mvp\MvpProcessQualificationContext.psm1'
$scenarioRegistryModule = Join-Path $repoRoot 'tools\mvp\MvpScenarioRegistry.psm1'

Import-Module $environmentPolicyModule -Force -ErrorAction Stop
Import-Module $artifactBudgetModule -Force -ErrorAction Stop
Import-Module $scenarioRegistryModule -Force -ErrorAction Stop
Import-Module $qualificationContextModule -Force -ErrorAction Stop
Import-Module $supervisorModule -Force -ErrorAction Stop

$testEnvironmentPolicy = New-MvpProcessEnvironmentPolicy `
    -PolicyId 'test.supervisor.v1' `
    -InheritedNames @('ComSpec', 'PATH', 'PATHEXT', 'SystemRoot', 'TEMP', 'TMP', 'WINDIR') `
    -DeclaredNames @('ZIRCON_LOG_FILTER')
$testScenarioRegistry = Read-MvpScenarioRegistry -Path (Join-Path $repoRoot 'tools\mvp\mvp-scenario-registry.json')
$testScenarioRegistryReceipt = Get-MvpScenarioRegistryReceipt -Registry $testScenarioRegistry
$testRuntimeScenario = Get-MvpScenarioRegistration `
    -Registry $testScenarioRegistry `
    -ScenarioId 'mvp.runtime-first-frame.v1'

function New-MvpTestSupervisorQualificationContext {
    param([Parameter(Mandatory)][string]$RunId)

    return New-MvpProcessQualificationContext `
        -RunId $RunId `
        -SourceFingerprint ('A' * 64) `
        -BuildSetId ('B' * 64) `
        -ScenarioRegistryReceipt $testScenarioRegistryReceipt `
        -ScenarioRegistration $testRuntimeScenario `
        -ScenarioVariant 'host.default'
}

Describe 'Staged process supervisor SHA-256 encoding' {
    It 'reads bounded diagnostic files into one exact-size byte snapshot' {
        $tokens = $null
        $errors = $null
        $ast = [Management.Automation.Language.Parser]::ParseFile(
            $supervisorModule,
            [ref]$tokens,
            [ref]$errors
        )
        $functionAst = $ast.Find({
                param($node)

                return $node -is [Management.Automation.Language.FunctionDefinitionAst] -and
                    $node.Name -eq 'Read-MvpSupervisedBoundedDiagnosticFile'
            }, $true)

        $errors.Count | Should Be 0
        $functionAst | Should Not BeNullOrEmpty
        $functionSource = $functionAst.Extent.Text
        $functionSource | Should Match '\$bytes = \[byte\[\]\]::new\(\[int\]\$length\)'
        $functionSource | Should Match '\$stream\.Read\(\$bytes, \$offset, \$bytes\.Length - \$offset\)'
        $functionSource | Should Not Match '\[IO\.MemoryStream\]::new\(\)'
        $functionSource | Should Not Match '\.ToArray\(\)'
    }

    It 'uses a fixed lower-case character buffer for supervisor digests' {
        $module = Get-Module -Name StagedProcessSupervisor -ErrorAction Stop
        $bytes = [byte[]]@(0x00, 0x0F, 0x10, 0x7F, 0x80, 0xF0, 0xFF)

        $encoded = & $module {
            param([byte[]]$Value)

            ConvertTo-MvpSupervisorLowerHex -Bytes $Value
        } $bytes

        $encoded | Should Be '000f107f80f0ff'
        $emptyDigest = & $module {
            [byte[]]$emptyBytes = @()
            Get-MvpSupervisorSha256 -Bytes $emptyBytes
        }
        $emptyDigest | Should Be 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'
        $moduleSource = Get-Content -LiteralPath $supervisorModule -Raw
        $journalSource = Get-Content -LiteralPath $journalModule -Raw
        $moduleSource | Should Match 'MvpProcessLifecycleJournal\.psm1'
        $journalSource | Should Match '\[char\[\]\]::new\(\$Bytes\.Length \* 2\)'
        $journalSource | Should Not Match 'ForEach-Object \{ \$_.ToString\(''x2''\) \}'
    }

    It 'reuses one process-owned progress result and skips the empty probe array' {
        $source = Get-Content -LiteralPath $supervisorModule -Raw
        $startFunction = [regex]::Match(
            $source,
            '(?s)function Start-MvpSupervisedProcess \{.*?(?=\r?\nfunction Write-MvpSupervisedProcessHeartbeat)')
        $probeFunction = [regex]::Match(
            $source,
            '(?s)function Invoke-MvpSupervisedProcessProgressProbe \{.*?(?=\r?\nfunction Complete-MvpSupervisedProcess)')

        $startFunction.Success | Should Be $true
        $probeFunction.Success | Should Be $true
        $startFunction.Value | Should Match 'progress_probe_result_scratch = \[pscustomobject\]@\{'
        $probeFunction.Value | Should Match '\$result = \$ProcessState\.progress_probe_result_scratch'
        $probeFunction.Value | Should Match '\$reportedProgress = & \$ProgressProbe \$ProcessState'
        $probeFunction.Value | Should Match '\$reportedProgress -is \[array\]'
        $probeFunction.Value | Should Not Match '@\(& \$ProgressProbe'
        $probeFunction.Value | Should Not Match 'return \[pscustomobject\]@\{'
    }

    It 'returns a scalar timestamp from each progress journal write' {
        $source = Get-Content -LiteralPath $supervisorModule -Raw
        $writerFunction = [regex]::Match(
            $source,
            '(?s)function Write-MvpSupervisedProcessProgress \{.*?(?=\r?\nfunction Write-MvpSupervisedProcessFailure)')
        $probeFunction = [regex]::Match(
            $source,
            '(?s)function Invoke-MvpSupervisedProcessProgressProbe \{.*?(?=\r?\nfunction Complete-MvpSupervisedProcess)')

        $writerFunction.Value | Should Match 'return \$recordedAtUtc'
        $writerFunction.Value | Should Not Match 'return \[pscustomobject\]@\{'
        $probeFunction.Value | Should Match '\$recordedAtUtc = Write-MvpSupervisedProcessProgress'
        $probeFunction.Value | Should Match '\$EmittedProgressNames\.Add\(\$progressName\)'
        $probeFunction.Value | Should Match '\$lastProgressRecordedAtUtc = \$recordedAtUtc'
        $probeFunction.Value | Should Not Match '\$progress = Write-MvpSupervisedProcessProgress'
    }
}

Describe 'Staged process supervisor lifecycle journal' {
    It 'writes a heartbeat bound to the started process before its terminal receipt' {
        $state = $null
        try {
            $deviceStageRoot = '\\?\' + [IO.Path]::GetFullPath($TestDrive).TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "ping -n 3 127.0.0.1 > nul & echo %ZIRCON_MVP_UNAPPROVED_PARENT_ENV% & echo %ZIRCON_LOG_FILTER%"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true
            $startInfo.EnvironmentVariables['ZIRCON_MVP_UNAPPROVED_PARENT_ENV'] = 'parent-secret-value'

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $deviceStageRoot `
                -RunId 'supervisor-heartbeat-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-heartbeat-fixture') `
                -Phase 'heartbeat-fixture' `
                -StdoutPath (Join-Path $TestDrive 'stdout.log') `
                -StderrPath (Join-Path $TestDrive 'stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -MaximumJournalBytes 4096 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -DeclaredEnvironment @{ ZIRCON_LOG_FILTER = 'allowed-fixture-value' } `
                -HeartbeatIntervalMilliseconds 50
            @(Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 10) | Should Be 0

            $journalPaths = @(Get-ChildItem -LiteralPath (Join-Path $TestDrive 'logs') -Filter 'process-execution-journal*.jsonl' | Sort-Object Name)
            $journalPaths.Count | Should BeGreaterThan 1
            $events = @($journalPaths | ForEach-Object {
                Get-Content -LiteralPath $_.FullName | ForEach-Object { $_ | ConvertFrom-Json }
            } | Sort-Object `
                @{ Expression = { [int]$_.journal_segment } }, `
                @{ Expression = { [Int64]$_.journal_offset_bytes } })
            $stdout = [IO.File]::ReadAllText((Join-Path $TestDrive 'stdout.log'))
            $stdoutTail = [IO.File]::ReadAllText((Join-Path $TestDrive 'stdout.tail.log'))
            $started = @($events | Where-Object { $_.event_kind -eq 'started' })
            $heartbeats = @($events | Where-Object { $_.event_kind -eq 'heartbeat' })
            $exits = @($events | Where-Object { $_.event_kind -eq 'exit' })
            $cleanups = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $started.Count | Should Be 1
            $heartbeats.Count | Should BeGreaterThan 0
            $exits.Count | Should Be 1
            $cleanups.Count | Should Be 1
            $terminal.Count | Should Be 1
            $events | ForEach-Object { $_.run_id | Should Be 'supervisor-heartbeat-fixture' }
            $events | ForEach-Object {
                $_.schema_version | Should Be 1
                $_.event_stream_kind | Should Be 'zircon.mvp-process-lifecycle-event'
                ([int]$_.journal_segment -ge 0) | Should Be $true
                ([Int64]$_.journal_offset_bytes -ge 0) | Should Be $true
            }
            $stdout | Should Not Match 'parent-secret-value'
            $stdout | Should Match 'allowed-fixture-value'
            $stdoutTail | Should Match 'allowed-fixture-value'
            $started[0].executable_sha256 | Should Match '^[0-9a-f]{64}$'
            $started[0].working_directory | Should Be ([string]$TestDrive)
            $started[0].arguments_sha256 | Should Match '^[0-9a-f]{64}$'
            $started[0].environment_sha256 | Should Match '^[0-9a-f]{64}$'
            $started[0].environment_policy_schema_version | Should Be 1
            $started[0].environment_policy_kind | Should Be 'zircon.mvp-process-environment-policy'
            $started[0].environment_policy_id | Should Be 'test.supervisor.v1'
            $started[0].resource_limits.maximum_active_process_count | Should Be 8
            $started[0].resource_limits.maximum_job_memory_bytes | Should Be 4294967296
            $started[0].resource_limits.maximum_cpu_rate_per_ten_thousand | Should Be 7500
            $declaredEnvironment = @($started[0].environment_variables | Where-Object { $_.name -eq 'ZIRCON_LOG_FILTER' })
            $declaredEnvironment.Count | Should Be 1
            $declaredEnvironment[0].source | Should Be 'scenario_declared'
            $declaredEnvironment[0].sensitivity | Should Be 'non_sensitive'
            $declaredEnvironment[0].value_sha256 | Should Match '^[0-9a-f]{64}$'
            @($started[0].environment_variables | Where-Object { $_.name -eq 'ZIRCON_MVP_UNAPPROVED_PARENT_ENV' }).Count | Should Be 0
            $heartbeats | ForEach-Object {
                $_.process_id | Should Be $started[0].process_id
                $_.process_started_at_utc | Should Be $started[0].process_started_at_utc
                $_.executable_sha256 | Should Be $started[0].executable_sha256
                $_.arguments_sha256 | Should Be $started[0].arguments_sha256
                $_.environment_sha256 | Should Be $started[0].environment_sha256
                [Int64]$_.elapsed_milliseconds | Should BeGreaterThan 0
            }
            $terminal[0].process_id | Should Be $started[0].process_id
            $terminal[0].process_started_at_utc | Should Be $started[0].process_started_at_utc
            $terminal[0].executable_sha256 | Should Be $started[0].executable_sha256
            $terminal[0].arguments_sha256 | Should Be $started[0].arguments_sha256
            $terminal[0].environment_sha256 | Should Be $started[0].environment_sha256
            $exits[0].process_id | Should Be $started[0].process_id
            $exits[0].process_started_at_utc | Should Be $started[0].process_started_at_utc
            $exits[0].exit_code | Should Be 0
            $cleanups[0].process_id | Should Be $started[0].process_id
            $cleanups[0].process_started_at_utc | Should Be $started[0].process_started_at_utc
            $cleanups[0].job_empty | Should Be $true
            $cleanups[0].outcome | Should Be 'exited'
            $terminal[0].stdout.tail_file_name | Should Be 'stdout.tail.log'
            $terminal[0].stderr.tail_file_name | Should Be 'stderr.tail.log'
            ([Int64]$terminal[0].stdout.tail_retained_bytes -le [Int64]$state.maximum_tail_log_bytes) | Should Be $true
            ([Int64]$terminal[0].stderr.tail_retained_bytes -le [Int64]$state.maximum_tail_log_bytes) | Should Be $true
            $jobLimits = $state.process_job.GetLimitSnapshot()
            $jobLimits.ActiveProcessLimit | Should Be 8
            $jobLimits.JobMemoryLimitBytes | Should Be 4294967296
            $jobLimits.CpuRatePerTenThousand | Should Be 7500
            $firstSegment = [int]$events[0].journal_segment
            $tail = Get-MvpSupervisedJournalTail `
                -StageRoot $deviceStageRoot `
                -JournalSegment $firstSegment `
                -JournalOffsetBytes 0
            $tail.content | Should Match '"event_kind":"started"'
            $tail.journal_segment | Should Be $firstSegment
            ($tail.next_journal_offset_bytes -gt 0) | Should Be $true
            for ($eventIndex = 0; $eventIndex -lt $events.Count; $eventIndex++) {
                $events[$eventIndex].sequence | Should Be ($eventIndex + 1)
                $events[$eventIndex].event_sha256 | Should Match '^[0-9a-f]{64}$'
                if ($eventIndex -eq 0) {
                    $events[$eventIndex].previous_event_sha256 | Should Be $null
                }
                else {
                    $events[$eventIndex].previous_event_sha256 | Should Be $events[$eventIndex - 1].event_sha256
                }
            }
            [array]::IndexOf($events, $started[0]) | Should BeLessThan ([array]::IndexOf($events, $heartbeats[0]))
            [array]::IndexOf($events, $heartbeats[-1]) | Should BeLessThan ([array]::IndexOf($events, $exits[0]))
            [array]::IndexOf($events, $exits[0]) | Should BeLessThan ([array]::IndexOf($events, $cleanups[0]))
            [array]::IndexOf($events, $cleanups[0]) | Should BeLessThan ([array]::IndexOf($events, $terminal[0]))
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'continues the journal sequence and hash chain across process states' {
        $firstState = $null
        $secondState = $null
        try {
            foreach ($attempt in @(1, 2)) {
                $startInfo = [Diagnostics.ProcessStartInfo]::new()
                $startInfo.FileName = $env:ComSpec
                $startInfo.Arguments = ('/d /s /c "echo supervisor-journal-resume-{0}"' -f $attempt)
                $startInfo.WorkingDirectory = $TestDrive
                $startInfo.UseShellExecute = $false
                $startInfo.RedirectStandardOutput = $true
                $startInfo.RedirectStandardError = $true

                $state = Start-MvpSupervisedProcess `
                    -StartInfo $startInfo `
                    -StageRoot $TestDrive `
                    -RunId "supervisor-journal-resume-$attempt" `
                    -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId "supervisor-journal-resume-$attempt") `
                    -Phase "journal-resume-$attempt" `
                    -StdoutPath (Join-Path $TestDrive "journal-resume-$attempt.stdout.log") `
                    -StderrPath (Join-Path $TestDrive "journal-resume-$attempt.stderr.log") `
                    -MaximumRetainedLogBytes 1024 `
                    -EnvironmentPolicy $testEnvironmentPolicy `
                    -HeartbeatIntervalMilliseconds 50
                @(Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 10) | Should Be 0
                if ($attempt -eq 1) {
                    $firstState = $state
                    Close-MvpSupervisedProcessState -ProcessState $firstState
                    $firstState = $null
                }
                else {
                    $secondState = $state
                }
            }

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -like 'journal-resume-*' } |
                    Sort-Object { [int]$_.sequence })
            $firstTerminal = @($events | Where-Object { $_.phase -eq 'journal-resume-1' -and $_.event_kind -eq 'terminal' })
            $secondStarted = @($events | Where-Object { $_.phase -eq 'journal-resume-2' -and $_.event_kind -eq 'started' })

            $firstTerminal.Count | Should Be 1
            $secondStarted.Count | Should Be 1
            $firstSequence = [int]$events[0].sequence
            for ($eventIndex = 0; $eventIndex -lt $events.Count; $eventIndex++) {
                $events[$eventIndex].sequence | Should Be ($firstSequence + $eventIndex)
                if ($eventIndex -gt 0) {
                    $events[$eventIndex].previous_event_sha256 | Should Be $events[$eventIndex - 1].event_sha256
                }
            }
            $secondStarted[0].previous_event_sha256 | Should Be $firstTerminal[0].event_sha256
        }
        finally {
            if ($null -ne $firstState) {
                Close-MvpSupervisedProcessState -ProcessState $firstState
            }
            if ($null -ne $secondState) {
                Close-MvpSupervisedProcessState -ProcessState $secondState
            }
        }
    }

    It 'rejects an incompatible lifecycle event schema before launching a process' {
        $stageRoot = Join-Path $TestDrive 'journal-schema-rejection-stage'
        $logRoot = Join-Path $stageRoot 'logs'
        [IO.Directory]::CreateDirectory($logRoot) | Out-Null
        $incompatibleEvent = [ordered]@{
            schema_version = 2
            event_stream_kind = 'zircon.mvp-process-lifecycle-event'
            sequence = 1
            event_sha256 = ('0' * 64)
        }
        [IO.File]::WriteAllText(
            (Join-Path $logRoot 'process-execution-journal.jsonl'),
            (($incompatibleEvent | ConvertTo-Json -Compress) + [Environment]::NewLine),
            [Text.UTF8Encoding]::new($false)
        )
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = $env:ComSpec
        $startInfo.Arguments = '/d /s /c "echo should-not-launch"'
        $startInfo.WorkingDirectory = $stageRoot
        $startInfo.UseShellExecute = $false
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true

        $schemaRejected = $false
        try {
            Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $stageRoot `
                -RunId 'supervisor-journal-schema-rejection' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-journal-schema-rejection') `
                -Phase 'journal-schema-rejection' `
                -StdoutPath (Join-Path $stageRoot 'schema-rejection.stdout.log') `
                -StderrPath (Join-Path $stageRoot 'schema-rejection.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy | Out-Null
        }
        catch {
            $schemaRejected = $_.Exception.Message -match 'unsupported schema version'
        }
        $schemaRejected | Should Be $true
    }

    It 'bounds archived journal segments with a hashed retention receipt' {
        $state = $null
        try {
            $stageRoot = Join-Path $TestDrive 'journal-retention-stage'
            [IO.Directory]::CreateDirectory($stageRoot) | Out-Null
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "ping -n 6 127.0.0.1 > nul"'
            $startInfo.WorkingDirectory = $stageRoot
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $stageRoot `
                -RunId 'supervisor-journal-retention-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-journal-retention-fixture') `
                -Phase 'journal-retention-fixture' `
                -StdoutPath (Join-Path $stageRoot 'journal-retention.stdout.log') `
                -StderrPath (Join-Path $stageRoot 'journal-retention.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -MaximumJournalBytes 4096 `
                -MaximumArchivedJournalSegments 2 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            @(Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 15) | Should Be 0

            $logRoot = Join-Path $stageRoot 'logs'
            $archives = @(Get-ChildItem -LiteralPath $logRoot -Filter 'process-execution-journal.??????.jsonl' -File)
            $journalPaths = @(Get-ChildItem -LiteralPath $logRoot -Filter 'process-execution-journal*.jsonl' -File)
            $events = @($journalPaths | ForEach-Object {
                Get-Content -LiteralPath $_.FullName | ForEach-Object { $_ | ConvertFrom-Json }
            })
            $retentionEvents = @($events | Where-Object { $null -ne $_.PSObject.Properties['retention'] })
            $latestRetention = @($retentionEvents | Sort-Object { [int]$_.sequence } | Select-Object -Last 1)

            ($archives.Count -le 2) | Should Be $true
            $retentionEvents.Count | Should BeGreaterThan 0
            $latestRetention.Count | Should Be 1
            $latestRetention[0].retention.maximum_archived_segments | Should Be 2
            ([int]$latestRetention[0].retention.pruned_segment_count -gt 0) | Should Be $true
            ([int]$latestRetention[0].retention.pruned_from_segment -le [int]$latestRetention[0].retention.pruned_through_segment) | Should Be $true
            $latestRetention[0].retention.pruned_segments_sha256 | Should Match '^[0-9a-f]{64}$'
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'records cleanup after timing out a job-bound process' {
        $state = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "ping -n 30 127.0.0.1 > nul"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-timeout-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-timeout-fixture') `
                -Phase 'timeout-fixture' `
                -StdoutPath (Join-Path $TestDrive 'timeout.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'timeout.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            $timedOut = $false
            try {
                Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 1 | Out-Null
            }
            catch [TimeoutException] {
                $timedOut = $true
            }
            $timedOut | Should Be $true

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'timeout-fixture' })
            $exit = @($events | Where-Object { $_.event_kind -eq 'exit' })
            $cleanup = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $exit.Count | Should Be 1
            $cleanup.Count | Should Be 1
            $terminal.Count | Should Be 1
            $exit[0].root_process_exited | Should Be $true
            $cleanup[0].job_empty | Should Be $true
            $cleanup[0].outcome | Should Be 'timed_out'
            $terminal[0].outcome | Should Be 'timed_out'
            [array]::IndexOf($events, $exit[0]) | Should BeLessThan ([array]::IndexOf($events, $cleanup[0]))
            [array]::IndexOf($events, $cleanup[0]) | Should BeLessThan ([array]::IndexOf($events, $terminal[0]))
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'classifies stalled semantic progress before the process deadline' {
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = (Get-Command powershell.exe -ErrorAction Stop).Source
        $startInfo.WorkingDirectory = $TestDrive
        $startInfo.Arguments = '-NoProfile -Command "Start-Sleep -Seconds 10"'
        $startInfo.UseShellExecute = $false
        $startInfo.CreateNoWindow = $true
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true
        $state = $null
        try {
            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-progress-stalled-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-progress-stalled-fixture') `
                -Phase 'progress-stalled-fixture' `
                -StdoutPath (Join-Path $TestDrive 'progress-stalled.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'progress-stalled.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            $stalled = $false
            try {
                Complete-MvpSupervisedProcess `
                    -ProcessState $state `
                    -TimeoutSeconds 10 `
                    -ProgressProbe { param($unusedState) $null } `
                    -ProgressInactivityTimeoutSeconds 1 | Out-Null
            }
            catch [InvalidOperationException] {
                $stalled = $_.Exception.Message -match 'progress_stalled'
            }
            $stalled | Should Be $true

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'progress-stalled-fixture' })
            $failure = @($events | Where-Object { $_.event_kind -eq 'supervisor_failure' })
            $exit = @($events | Where-Object { $_.event_kind -eq 'exit' })
            $cleanup = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $failure.Count | Should Be 1
            $failure[0].failure_kind | Should Be 'progress_stalled'
            $exit.Count | Should Be 1
            $exit[0].root_process_exited | Should Be $true
            $cleanup.Count | Should Be 1
            $cleanup[0].job_empty | Should Be $true
            $cleanup[0].outcome | Should Be 'supervisor_failed'
            $terminal.Count | Should Be 1
            $terminal[0].outcome | Should Be 'supervisor_failed'
            $terminal[0].supervisor_failure.kind | Should Be 'progress_stalled'
            [array]::IndexOf($events, $failure[0]) | Should BeLessThan ([array]::IndexOf($events, $exit[0]))
            [array]::IndexOf($events, $exit[0]) | Should BeLessThan ([array]::IndexOf($events, $cleanup[0]))
            [array]::IndexOf($events, $cleanup[0]) | Should BeLessThan ([array]::IndexOf($events, $terminal[0]))
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'terminates a process that exceeds its shared run artifact budget' {
        $stageRoot = Join-Path $TestDrive 'artifact-quota-stage'
        [IO.Directory]::CreateDirectory($stageRoot) | Out-Null
        $artifactBudget = New-MvpRunArtifactBudget `
            -Root $stageRoot `
            -PolicyId 'test.supervisor-artifacts.v1' `
            -MaximumAdditionalBytes 16384 `
            -MaximumAdditionalFileCount 32
        $startInfo = [Diagnostics.ProcessStartInfo]::new()
        $startInfo.FileName = $env:ComSpec
        $startInfo.Arguments = '/d /s /c "ping -n 30 127.0.0.1 > nul"'
        $startInfo.WorkingDirectory = $stageRoot
        $startInfo.UseShellExecute = $false
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true
        $state = $null
        try {
            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $stageRoot `
                -RunId 'supervisor-artifact-quota-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-artifact-quota-fixture') `
                -Phase 'artifact-quota-fixture' `
                -StdoutPath (Join-Path $stageRoot 'artifact-quota.stdout.log') `
                -StderrPath (Join-Path $stageRoot 'artifact-quota.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -ArtifactBudget $artifactBudget `
                -HeartbeatIntervalMilliseconds 50
            [IO.File]::WriteAllBytes(
                (Join-Path $stageRoot 'oversized-artifact.bin'),
                [byte[]]::new(32768)
            )
            $quotaRejected = $false
            try {
                Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 10 | Out-Null
            }
            catch [InvalidOperationException] {
                $quotaRejected = $_.Exception.Message -match 'artifact_quota_exceeded'
            }
            $quotaRejected | Should Be $true

            $events = @(Get-Content -LiteralPath (Join-Path $stageRoot 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'artifact-quota-fixture' })
            $started = @($events | Where-Object { $_.event_kind -eq 'started' })
            $failure = @($events | Where-Object { $_.event_kind -eq 'supervisor_failure' })
            $cleanup = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $started.Count | Should Be 1
            $started[0].artifact_budget_schema_version | Should Be 1
            $started[0].artifact_budget_kind | Should Be 'zircon.mvp-run-artifact-budget'
            $started[0].artifact_budget_id | Should Be 'test.supervisor-artifacts.v1'
            $started[0].artifact_budget_baseline_sha256 | Should Match '^[0-9a-f]{64}$'
            $failure.Count | Should Be 1
            $failure[0].failure_kind | Should Be 'artifact_quota_exceeded'
            $cleanup[0].outcome | Should Be 'supervisor_failed'
            $terminal[0].outcome | Should Be 'supervisor_failed'
            $terminal[0].supervisor_failure.kind | Should Be 'artifact_quota_exceeded'
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'cancels a job-bound process through the caller cancellation probe' {
        $state = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "ping -n 30 127.0.0.1 > nul"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-cancel-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-cancel-fixture') `
                -Phase 'cancel-fixture' `
                -StdoutPath (Join-Path $TestDrive 'cancel.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'cancel.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            $cancelled = $false
            try {
                Complete-MvpSupervisedProcess `
                    -ProcessState $state `
                    -TimeoutSeconds 10 `
                    -CancellationProbe { param($unusedState) $true } `
                    -CancellationReason 'fixture_cancel' | Out-Null
            }
            catch [OperationCanceledException] {
                $cancelled = $true
            }
            $cancelled | Should Be $true

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'cancel-fixture' })
            $cancellations = @($events | Where-Object { $_.event_kind -eq 'cancellation_requested' })
            $exit = @($events | Where-Object { $_.event_kind -eq 'exit' })
            $cleanup = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $cancellations.Count | Should Be 1
            $exit.Count | Should Be 1
            $cleanup.Count | Should Be 1
            $terminal.Count | Should Be 1
            $cancellations[0].cancellation_reason | Should Be 'fixture_cancel'
            $cleanup[0].job_empty | Should Be $true
            $cleanup[0].outcome | Should Be 'cancelled'
            $terminal[0].outcome | Should Be 'cancelled'
            $terminal[0].cancellation.reason | Should Be 'fixture_cancel'
            [array]::IndexOf($events, $cancellations[0]) | Should BeLessThan ([array]::IndexOf($events, $exit[0]))
            [array]::IndexOf($events, $exit[0]) | Should BeLessThan ([array]::IndexOf($events, $cleanup[0]))
            [array]::IndexOf($events, $cleanup[0]) | Should BeLessThan ([array]::IndexOf($events, $terminal[0]))
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'records a crash classification for a nonzero product exit' {
        $state = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "exit /b 23"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-crash-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-crash-fixture') `
                -Phase 'crash-fixture' `
                -StdoutPath (Join-Path $TestDrive 'crash.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'crash.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            @(Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 10) | Should Be 23

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'crash-fixture' })
            $exit = @($events | Where-Object { $_.event_kind -eq 'exit' })
            $crashes = @($events | Where-Object { $_.event_kind -eq 'crash' })
            $cleanup = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $exit.Count | Should Be 1
            $crashes.Count | Should Be 1
            $cleanup.Count | Should Be 1
            $terminal.Count | Should Be 1
            $exit[0].exit_code | Should Be 23
            $crashes[0].crash_kind | Should Be 'nonzero_exit'
            $crashes[0].exit_code | Should Be 23
            $cleanup[0].outcome | Should Be 'crashed'
            $terminal[0].outcome | Should Be 'crashed'
            $terminal[0].crash.kind | Should Be 'nonzero_exit'
            $terminal[0].crash.exit_code | Should Be 23
            [array]::IndexOf($events, $exit[0]) | Should BeLessThan ([array]::IndexOf($events, $crashes[0]))
            [array]::IndexOf($events, $crashes[0]) | Should BeLessThan ([array]::IndexOf($events, $cleanup[0]))
            [array]::IndexOf($events, $cleanup[0]) | Should BeLessThan ([array]::IndexOf($events, $terminal[0]))
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'terminates live descendants without masking a nonzero root exit' {
        $state = $null
        $childProcessId = $null
        try {
            $fixtureScriptPath = Join-Path $TestDrive 'crash-with-descendant.ps1'
            $childProcessIdPath = Join-Path $TestDrive 'crash-descendant.pid'
            $rootExitTimestampPath = Join-Path $TestDrive 'crash-root-exit.utc'
            @'
$child = Start-Process `
    -FilePath $env:ComSpec `
    -ArgumentList '/d /s /c "ping -n 30 127.0.0.1 > nul"' `
    -PassThru `
    -WindowStyle Hidden
[IO.File]::WriteAllText($args[0], [string]$child.Id)
[IO.File]::WriteAllText($args[1], [DateTimeOffset]::UtcNow.ToString('o'))
exit 23
'@ | Set-Content -LiteralPath $fixtureScriptPath -Encoding UTF8

            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = (Get-Process -Id $PID).Path
            $startInfo.Arguments = "-NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$fixtureScriptPath`" `"$childProcessIdPath`" `"$rootExitTimestampPath`""
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-crash-descendant-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-crash-descendant-fixture') `
                -Phase 'crash-descendant-fixture' `
                -StdoutPath (Join-Path $TestDrive 'crash-descendant.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'crash-descendant.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50

            @(Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 10) | Should Be 23

            Test-Path -LiteralPath $childProcessIdPath | Should Be $true
            Test-Path -LiteralPath $rootExitTimestampPath | Should Be $true
            $childProcessId = [int](Get-Content -LiteralPath $childProcessIdPath -Raw)
            $rootExitTimestamp = [DateTimeOffset]::ParseExact(
                (Get-Content -LiteralPath $rootExitTimestampPath -Raw),
                'o',
                [Globalization.CultureInfo]::InvariantCulture)
            ([DateTimeOffset]::UtcNow - $rootExitTimestamp).TotalSeconds | Should BeLessThan 2
            $state.process_job.IsEmpty() | Should Be $true
            @(Get-Process -Id $childProcessId -ErrorAction SilentlyContinue).Count | Should Be 0

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'crash-descendant-fixture' })
            $cleanup = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $cleanup.Count | Should Be 1
            $terminal.Count | Should Be 1
            $cleanup[0].job_empty | Should Be $true
            $cleanup[0].outcome | Should Be 'crashed'
            $terminal[0].exit_code | Should Be 23
            $terminal[0].outcome | Should Be 'crashed'
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
            if ($null -ne $childProcessId) {
                Stop-Process -Id $childProcessId -Force -ErrorAction SilentlyContinue
            }
        }
    }

    It 'records a changed phase progress signal before its terminal receipt' {
        $state = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "ping -n 3 127.0.0.1 > nul"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-progress-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-progress-fixture') `
                -Phase 'progress-fixture' `
                -StdoutPath (Join-Path $TestDrive 'progress.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'progress.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            @(Complete-MvpSupervisedProcess `
                    -ProcessState $state `
                    -TimeoutSeconds 10 `
                    -ProgressProbe {
                        param($unusedState)
                        'fixture_ready'
                    }) | Should Be 0

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'progress-fixture' })
            $progress = @($events | Where-Object { $_.event_kind -eq 'progress' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $progress.Count | Should Be 1
            $progress[0].progress_name | Should Be 'fixture_ready'
            ([Int64]$progress[0].elapsed_milliseconds -gt 0) | Should Be $true
            $terminal.Count | Should Be 1
            $terminal[0].phase_progress.last_name | Should Be 'fixture_ready'
            [array]::IndexOf($events, $progress[0]) | Should BeLessThan ([array]::IndexOf($events, $terminal[0]))
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'writes a terminal receipt after a progress probe failure' {
        $state = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "ping -n 30 127.0.0.1 > nul"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-progress-probe-failure-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-progress-probe-failure-fixture') `
                -Phase 'progress-probe-failure-fixture' `
                -StdoutPath (Join-Path $TestDrive 'progress-probe-failure.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'progress-probe-failure.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            $failed = $false
            try {
                Complete-MvpSupervisedProcess `
                    -ProcessState $state `
                    -TimeoutSeconds 10 `
                    -ProgressProbe { throw 'fixture progress probe detail' } | Out-Null
            }
            catch [InvalidOperationException] {
                $failed = $true
            }
            $failed | Should Be $true

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'progress-probe-failure-fixture' })
            $failure = @($events | Where-Object { $_.event_kind -eq 'supervisor_failure' })
            $exit = @($events | Where-Object { $_.event_kind -eq 'exit' })
            $cleanup = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $failure.Count | Should Be 1
            $exit.Count | Should Be 1
            $cleanup.Count | Should Be 1
            $terminal.Count | Should Be 1
            $failure[0].failure_kind | Should Be 'progress_probe_failed'
            $failure[0].failure_message_sha256 | Should Match '^[0-9a-f]{64}$'
            $cleanup[0].job_empty | Should Be $true
            $cleanup[0].outcome | Should Be 'supervisor_failed'
            $terminal[0].outcome | Should Be 'supervisor_failed'
            $terminal[0].supervisor_failure.kind | Should Be 'progress_probe_failed'
            $terminal[0].supervisor_failure.message_sha256 | Should Match '^[0-9a-f]{64}$'
            ($events | ConvertTo-Json -Depth 16 -Compress) | Should Not Match 'fixture progress probe detail'
            [array]::IndexOf($events, $failure[0]) | Should BeLessThan ([array]::IndexOf($events, $exit[0]))
            [array]::IndexOf($events, $exit[0]) | Should BeLessThan ([array]::IndexOf($events, $cleanup[0]))
            [array]::IndexOf($events, $cleanup[0]) | Should BeLessThan ([array]::IndexOf($events, $terminal[0]))
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'writes a terminal receipt after a cancellation probe failure' {
        $state = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "ping -n 30 127.0.0.1 > nul"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-cancel-probe-failure-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-cancel-probe-failure-fixture') `
                -Phase 'cancel-probe-failure-fixture' `
                -StdoutPath (Join-Path $TestDrive 'cancel-probe-failure.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'cancel-probe-failure.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            $failed = $false
            try {
                Complete-MvpSupervisedProcess `
                    -ProcessState $state `
                    -TimeoutSeconds 10 `
                    -CancellationProbe { throw 'fixture cancellation probe detail' } | Out-Null
            }
            catch [InvalidOperationException] {
                $failed = $true
            }
            $failed | Should Be $true

            $events = @(Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'cancel-probe-failure-fixture' })
            $failure = @($events | Where-Object { $_.event_kind -eq 'supervisor_failure' })
            $exit = @($events | Where-Object { $_.event_kind -eq 'exit' })
            $cleanup = @($events | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $failure.Count | Should Be 1
            $exit.Count | Should Be 1
            $cleanup.Count | Should Be 1
            $terminal.Count | Should Be 1
            $failure[0].failure_kind | Should Be 'cancellation_probe_failed'
            $failure[0].failure_message_sha256 | Should Match '^[0-9a-f]{64}$'
            $cleanup[0].job_empty | Should Be $true
            $cleanup[0].outcome | Should Be 'supervisor_failed'
            $terminal[0].outcome | Should Be 'supervisor_failed'
            $terminal[0].supervisor_failure.kind | Should Be 'cancellation_probe_failed'
            $terminal[0].supervisor_failure.message_sha256 | Should Match '^[0-9a-f]{64}$'
            ($events | ConvertTo-Json -Depth 16 -Compress) | Should Not Match 'fixture cancellation probe detail'
            [array]::IndexOf($events, $failure[0]) | Should BeLessThan ([array]::IndexOf($events, $exit[0]))
            [array]::IndexOf($events, $exit[0]) | Should BeLessThan ([array]::IndexOf($events, $cleanup[0]))
            [array]::IndexOf($events, $cleanup[0]) | Should BeLessThan ([array]::IndexOf($events, $terminal[0]))
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'enforces one retained-output budget across stdout and stderr' {
        $state = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "(for /L %i in (1,1,32) do @echo stdout-shared-budget-%i) & (for /L %i in (1,1,32) do @echo stderr-shared-budget-%i 1>&2)"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-shared-output-budget-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-shared-output-budget-fixture') `
                -Phase 'shared-output-budget-fixture' `
                -StdoutPath (Join-Path $TestDrive 'shared.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'shared.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 50
            @(Complete-MvpSupervisedProcess -ProcessState $state -TimeoutSeconds 30) | Should Be 0

            $terminal = @(
                Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'shared-output-budget-fixture' -and $_.event_kind -eq 'terminal' }
            )
            $retainedBytes = [Int64]$terminal[0].stdout.retained_bytes + [Int64]$terminal[0].stderr.retained_bytes
            $droppedBytes = [Int64]$terminal[0].stdout.dropped_bytes + [Int64]$terminal[0].stderr.dropped_bytes

            $terminal.Count | Should Be 1
            ($retainedBytes -le 1024) | Should Be $true
            $terminal[0].retained_output_budget.maximum_bytes | Should Be 1024
            $terminal[0].retained_output_budget.retained_bytes | Should Be $retainedBytes
            $terminal[0].retained_output_budget.dropped_bytes | Should Be $droppedBytes
            $droppedBytes | Should BeGreaterThan 0
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }

    It 'records every ordered progress milestone for a short-lived process before exit' {
        $state = $null
        try {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $env:ComSpec
            $startInfo.Arguments = '/d /s /c "exit /b 0"'
            $startInfo.WorkingDirectory = $TestDrive
            $startInfo.UseShellExecute = $false
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true

            $state = Start-MvpSupervisedProcess `
                -StartInfo $startInfo `
                -StageRoot $TestDrive `
                -RunId 'supervisor-terminal-progress-fixture' `
                -QualificationContext (New-MvpTestSupervisorQualificationContext -RunId 'supervisor-terminal-progress-fixture') `
                -Phase 'terminal-progress-fixture' `
                -StdoutPath (Join-Path $TestDrive 'terminal-progress.stdout.log') `
                -StderrPath (Join-Path $TestDrive 'terminal-progress.stderr.log') `
                -MaximumRetainedLogBytes 1024 `
                -EnvironmentPolicy $testEnvironmentPolicy `
                -HeartbeatIntervalMilliseconds 1000
            @(Complete-MvpSupervisedProcess `
                    -ProcessState $state `
                    -TimeoutSeconds 10 `
                    -ProgressProbe { @('startup_ready', 'first_frame_presented', 'teardown_complete') }) | Should Be 0

            $events = @(
                Get-Content -LiteralPath (Join-Path $TestDrive 'logs\process-execution-journal.jsonl') |
                    ForEach-Object { $_ | ConvertFrom-Json } |
                    Where-Object { $_.phase -eq 'terminal-progress-fixture' }
            )
            $progress = @($events | Where-Object { $_.event_kind -eq 'progress' })
            $exit = @($events | Where-Object { $_.event_kind -eq 'exit' })
            $terminal = @($events | Where-Object { $_.event_kind -eq 'terminal' })

            $progress.Count | Should Be 3
            $progress[0].progress_name | Should Be 'startup_ready'
            $progress[1].progress_name | Should Be 'first_frame_presented'
            $progress[2].progress_name | Should Be 'teardown_complete'
            [array]::IndexOf($events, $progress[2]) | Should BeLessThan ([array]::IndexOf($events, $exit[0]))
            $terminal[0].phase_progress.last_name | Should Be 'teardown_complete'
        }
        finally {
            if ($null -ne $state) {
                Close-MvpSupervisedProcessState -ProcessState $state
            }
        }
    }
}
