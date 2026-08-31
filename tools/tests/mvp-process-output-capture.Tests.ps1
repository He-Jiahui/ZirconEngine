$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$captureModule = Join-Path $repoRoot 'tools\mvp\MvpProcessOutputCapture.psm1'
$supervisorModule = Join-Path $repoRoot 'tools\mvp\StagedProcessSupervisor.psm1'
$journalModule = Join-Path $repoRoot 'tools\mvp\MvpProcessLifecycleJournal.psm1'
Import-Module $captureModule -Force -ErrorAction Stop

function New-MvpTestOutputReader {
    param([Parameter(Mandatory)][string]$Text)

    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    $stream = [IO.MemoryStream]::new($bytes)
    return [pscustomobject]@{
        stream = $stream
        reader = [IO.StreamReader]::new($stream, [Text.UTF8Encoding]::new($false), $false)
    }
}

Describe 'MVP process output capture budgets' {
    It 'retains two independent tails inside one shared capacity' {
        $stdoutInput = New-MvpTestOutputReader -Text (('A' * 48) + ('S' * 32))
        $stderrInput = New-MvpTestOutputReader -Text (('B' * 48) + ('E' * 32))
        $retainedBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes 1024
        $tailBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes 64
        try {
            $stdoutTask = Start-MvpProcessOutputCapture `
                -Reader $stdoutInput.reader `
                -OutputPath (Join-Path $TestDrive 'shared-tail.stdout.log') `
                -MaximumRetainedBytes 1024 `
                -TailOutputPath (Join-Path $TestDrive 'shared-tail.stdout.tail.log') `
                -MaximumTailBytes 32 `
                -RetainedBudget $retainedBudget `
                -TailBudget $tailBudget
            $stderrTask = Start-MvpProcessOutputCapture `
                -Reader $stderrInput.reader `
                -OutputPath (Join-Path $TestDrive 'shared-tail.stderr.log') `
                -MaximumRetainedBytes 1024 `
                -TailOutputPath (Join-Path $TestDrive 'shared-tail.stderr.tail.log') `
                -MaximumTailBytes 32 `
                -RetainedBudget $retainedBudget `
                -TailBudget $tailBudget
            $stdout = $stdoutTask.GetAwaiter().GetResult()
            $stderr = $stderrTask.GetAwaiter().GetResult()

            $stdout.MaximumTailBytes | Should Be 32
            $stderr.MaximumTailBytes | Should Be 32
            ($stdout.TailRetainedBytes + $stderr.TailRetainedBytes) | Should Be 64
            $tailBudget.MaximumBytes | Should Be 64
            $tailBudget.RemainingBytes | Should Be 0
            [IO.File]::ReadAllText((Join-Path $TestDrive 'shared-tail.stdout.tail.log')) | Should Be ('S' * 32)
            [IO.File]::ReadAllText((Join-Path $TestDrive 'shared-tail.stderr.tail.log')) | Should Be ('E' * 32)
        }
        finally {
            $stdoutInput.reader.Dispose()
            $stderrInput.reader.Dispose()
            $stdoutInput.stream.Dispose()
            $stderrInput.stream.Dispose()
        }
    }

    It 'rejects a capture whose requested tail capacity exceeds the shared remainder' {
        $firstInput = New-MvpTestOutputReader -Text 'first'
        $secondInput = New-MvpTestOutputReader -Text 'second'
        $retainedBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes 64
        $tailBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes 32
        try {
            $firstTask = Start-MvpProcessOutputCapture `
                -Reader $firstInput.reader `
                -OutputPath (Join-Path $TestDrive 'exhaust-first.log') `
                -MaximumRetainedBytes 64 `
                -TailOutputPath (Join-Path $TestDrive 'exhaust-first.tail.log') `
                -MaximumTailBytes 32 `
                -RetainedBudget $retainedBudget `
                -TailBudget $tailBudget
            $null = $firstTask.GetAwaiter().GetResult()

            { Start-MvpProcessOutputCapture `
                    -Reader $secondInput.reader `
                    -OutputPath (Join-Path $TestDrive 'exhaust-second.log') `
                    -MaximumRetainedBytes 64 `
                    -TailOutputPath (Join-Path $TestDrive 'exhaust-second.tail.log') `
                    -MaximumTailBytes 1 `
                    -RetainedBudget $retainedBudget `
                    -TailBudget $tailBudget } |
                Should Throw 'shared tail output budget'
        }
        finally {
            $firstInput.reader.Dispose()
            $secondInput.reader.Dispose()
            $firstInput.stream.Dispose()
            $secondInput.stream.Dispose()
        }
    }

    It 'keeps the shared retained-prefix budget independent from tail capacity' {
        $stdoutInput = New-MvpTestOutputReader -Text ('X' * 20)
        $stderrInput = New-MvpTestOutputReader -Text ('Y' * 20)
        $retainedBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes 10
        $tailBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes 8
        try {
            $stdoutTask = Start-MvpProcessOutputCapture `
                -Reader $stdoutInput.reader `
                -OutputPath (Join-Path $TestDrive 'prefix.stdout.log') `
                -MaximumRetainedBytes 20 `
                -TailOutputPath (Join-Path $TestDrive 'prefix.stdout.tail.log') `
                -MaximumTailBytes 4 `
                -RetainedBudget $retainedBudget `
                -TailBudget $tailBudget
            $stderrTask = Start-MvpProcessOutputCapture `
                -Reader $stderrInput.reader `
                -OutputPath (Join-Path $TestDrive 'prefix.stderr.log') `
                -MaximumRetainedBytes 20 `
                -TailOutputPath (Join-Path $TestDrive 'prefix.stderr.tail.log') `
                -MaximumTailBytes 4 `
                -RetainedBudget $retainedBudget `
                -TailBudget $tailBudget
            $stdout = $stdoutTask.GetAwaiter().GetResult()
            $stderr = $stderrTask.GetAwaiter().GetResult()

            ($stdout.RetainedBytes + $stderr.RetainedBytes) | Should Be 10
            ($stdout.DroppedBytes + $stderr.DroppedBytes) | Should Be 30
            ($stdout.TailRetainedBytes + $stderr.TailRetainedBytes) | Should Be 8
        }
        finally {
            $stdoutInput.reader.Dispose()
            $stderrInput.reader.Dispose()
            $stdoutInput.stream.Dispose()
            $stderrInput.stream.Dispose()
        }
    }

    It 'reserves the tail ring from the caller supplied atomic budget' {
        $source = Get-Content -LiteralPath $captureModule -Raw

        $source | Should Match 'MvpProcessOutputCaptureBudget tailBudget'
        $source | Should Match 'tailBudget\.ReserveExact\(\(int\)maximumTailBytes\)'
        $source | Should Match 'MaximumTailBytes'
        $source | Should Not Match 'new MvpProcessOutputCapture\(\(int\)maximumTailBytes\)'
    }

    It 'passes one 64 KiB tail budget to both supervised streams and receipts each capacity' {
        $supervisorSource = Get-Content -LiteralPath $supervisorModule -Raw
        $journalSource = Get-Content -LiteralPath $journalModule -Raw

        $supervisorSource | Should Match 'MvpSupervisorMaximumTailOutputBytes = 65536'
        @([regex]::Matches($supervisorSource, '-TailBudget \$tailOutputBudget')).Count | Should Be 2
        $supervisorSource | Should Match 'New-MvpProcessOutputCaptureBudget -MaximumBytes \$script:MvpSupervisorMaximumTailOutputBytes'
        @([regex]::Matches($journalSource, 'tail_capacity_bytes = \[Int64\]\$Std(?:out|err)Capture\.MaximumTailBytes')).Count |
            Should Be 2
    }
}
