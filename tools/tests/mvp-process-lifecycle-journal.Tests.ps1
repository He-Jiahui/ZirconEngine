$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$journalModule = Join-Path $repoRoot 'tools\mvp\MvpProcessLifecycleJournal.psm1'
Import-Module $journalModule -Force -ErrorAction Stop

function New-MvpTestProcessJournalLine {
    param([Parameter(Mandatory)][ValidateRange(1, [Int32]::MaxValue)][int]$Sequence)

    return ([ordered]@{
            schema_version = 1
            event_stream_kind = 'zircon.mvp-process-lifecycle-event'
            sequence = $Sequence
            event_sha256 = ('{0:x64}' -f $Sequence)
        } | ConvertTo-Json -Compress)
}

function Get-MvpTestProcessJournalPath {
    param([Parameter(Mandatory)][string]$StageRoot)

    $logRoot = Join-Path $StageRoot 'logs'
    [IO.Directory]::CreateDirectory($logRoot) | Out-Null
    return Join-Path $logRoot 'process-execution-journal.jsonl'
}

Describe 'MVP process lifecycle journal streaming resume' {
    It 'resumes from the final complete event without whole-file text materialization' {
        $journalPath = Get-MvpTestProcessJournalPath -StageRoot $TestDrive
        $builder = [Text.StringBuilder]::new()
        foreach ($sequence in 1..4096) {
            $builder.Append((New-MvpTestProcessJournalLine -Sequence $sequence)).Append("`n") | Out-Null
        }
        [IO.File]::WriteAllText($journalPath, $builder.ToString(), [Text.UTF8Encoding]::new($false))

        $state = New-MvpProcessJournalState `
            -StageRoot $TestDrive `
            -MaximumJournalBytes 1048576 `
            -MaximumArchivedSegments 2
        $source = Get-Content -LiteralPath $journalModule -Raw

        $state.next_sequence | Should Be 4097
        $state.previous_event_sha256 | Should Be ('{0:x64}' -f 4096)
        $state.journal_offset_bytes | Should Be ([IO.FileInfo]::new($journalPath).Length)
        $source | Should Match '\[IO\.StreamReader\]'
        $source | Should Match '\.ReadLine\(\)'
        $source | Should Not Match '\[IO\.File\]::ReadAllText\(\$JournalPath'
    }

    It 'rejects a terminal JSON object that is missing its line terminator' {
        $journalPath = Get-MvpTestProcessJournalPath -StageRoot $TestDrive
        [IO.File]::WriteAllText(
            $journalPath,
            (New-MvpTestProcessJournalLine -Sequence 1),
            [Text.UTF8Encoding]::new($false))

        { New-MvpProcessJournalState `
                -StageRoot $TestDrive `
                -MaximumJournalBytes 1048576 `
                -MaximumArchivedSegments 2 } |
            Should Throw 'complete event line'
    }

    It 'rejects invalid UTF-8 before an otherwise valid terminal event' {
        $journalPath = Get-MvpTestProcessJournalPath -StageRoot $TestDrive
        $terminalBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (New-MvpTestProcessJournalLine -Sequence 1) + "`n")
        $bytes = [byte[]]::new($terminalBytes.Length + 2)
        $bytes[0] = 0xff
        $bytes[1] = 0x0a
        [Buffer]::BlockCopy($terminalBytes, 0, $bytes, 2, $terminalBytes.Length)
        [IO.File]::WriteAllBytes($journalPath, $bytes)

        { New-MvpProcessJournalState `
                -StageRoot $TestDrive `
                -MaximumJournalBytes 1048576 `
                -MaximumArchivedSegments 2 } |
            Should Throw 'invalid UTF-8'
    }

    It 'preserves the empty journal cursor' {
        $journalPath = Get-MvpTestProcessJournalPath -StageRoot $TestDrive
        [IO.File]::WriteAllBytes($journalPath, [byte[]]::new(0))

        $state = New-MvpProcessJournalState `
            -StageRoot $TestDrive `
            -MaximumJournalBytes 1048576 `
            -MaximumArchivedSegments 2

        $state.next_sequence | Should Be 1
        $state.previous_event_sha256 | Should Be $null
        $state.journal_offset_bytes | Should Be 0
    }

    It 'reads a tail slice into one exact byte buffer without a MemoryStream copy' {
        $journalPath = Get-MvpTestProcessJournalPath -StageRoot $TestDrive
        $firstLine = (New-MvpTestProcessJournalLine -Sequence 1) + "`n"
        $secondLine = (New-MvpTestProcessJournalLine -Sequence 2) + "`n"
        [IO.File]::WriteAllText(
            $journalPath,
            ($firstLine + $secondLine),
            [Text.UTF8Encoding]::new($false))
        $firstLineBytes = [Text.UTF8Encoding]::new($false).GetByteCount($firstLine)

        $tail = Get-MvpProcessJournalTail `
            -StageRoot $TestDrive `
            -JournalSegment 0 `
            -JournalOffsetBytes $firstLineBytes `
            -MaximumJournalBytes 1048576 `
            -MaximumArchivedSegments 2
        $source = Get-Content -LiteralPath $journalModule -Raw

        $tail.content | Should Be $secondLine
        $tail.next_journal_offset_bytes | Should Be ([IO.FileInfo]::new($journalPath).Length)
        $source | Should Match '\[byte\[\]\]::new\(\[int\]\$remaining\)'
        $source | Should Not Match '\[IO\.MemoryStream\]::new\(\)'
        $source | Should Not Match '\$content\.ToArray\(\)'
    }

    It 'resolves a tail segment without rescanning the active journal resume cursor' {
        $source = Get-Content -LiteralPath $journalModule -Raw
        $tailFunction = [regex]::Match(
            $source,
            '(?s)function Get-MvpProcessJournalTail \{.*?(?=\r?\nfunction Write-MvpProcessJournalEntry)')

        $tailFunction.Success | Should Be $true
        $tailFunction.Value | Should Match 'Get-MvpProcessJournalCurrentSegment'
        $tailFunction.Value | Should Not Match 'New-MvpProcessJournalState'
        @([regex]::Matches($source, 'Get-MvpProcessJournalCurrentSegment')).Count | Should Be 3
    }

    It 'reuses UTF-8 encoders and keeps the optional pruned archive path scalar' {
        $source = Get-Content -LiteralPath $journalModule -Raw
        $writeFunction = [regex]::Match(
            $source,
            '(?s)function Write-MvpProcessJournalEntry \{.*?(?=\r?\nExport-ModuleMember)')

        $source | Should Match '\$script:MvpProcessJournalUtf8 = \[Text\.UTF8Encoding\]::new\(\$false\)'
        $source | Should Match '\$script:MvpProcessJournalStrictUtf8 = \[Text\.UTF8Encoding\]::new\(\$false, \$true\)'
        $writeFunction.Value | Should Match '\$prunedArchivePath = \$null'
        $writeFunction.Value | Should Not Match '\[Text\.UTF8Encoding\]::new\('
        $writeFunction.Value | Should Not Match '\$prunedArchivePaths'
    }

    It 'selects the single oldest archive in one bounded pass during rotation' {
        $source = Get-Content -LiteralPath $journalModule -Raw
        $writeFunction = [regex]::Match(
            $source,
            '(?s)function Write-MvpProcessJournalEntry \{.*?(?=\r?\nExport-ModuleMember)')

        $writeFunction.Value | Should Match '\.EnumerateFiles\('
        $writeFunction.Value | Should Match '\$oldestArchivedSegment'
        $writeFunction.Value | Should Match '\$archiveCount -gt \(\$maximumArchivedSegments \+ 1\)'
        $writeFunction.Value | Should Not Match 'Sort-Object'
        $writeFunction.Value | Should Not Match '\$archiveRecords'
        $writeFunction.Value | Should Not Match 'Select-Object'
    }

    It 'hashes event JSON through one pooled UTF8 buffer without changing the digest' {
        $text = '{"event":"pooled-hash","sequence":42}'
        $expected = Get-MvpProcessJournalSha256 `
            -Bytes ([Text.UTF8Encoding]::new($false).GetBytes($text))
        $module = Get-Module MvpProcessLifecycleJournal
        $actual = & $module {
            param([string]$Value)
            Get-MvpProcessJournalStringSha256 -Text $Value
        } $text
        $source = Get-Content -LiteralPath $journalModule -Raw

        $actual | Should Be $expected
        $source | Should Match 'function Get-MvpProcessJournalStringSha256'
        $source | Should Match "'System\.Buffers\.ArrayPool\x601\[System\.Byte\]' -as \[type\]"
        $source | Should Match 'MvpProcessJournalByteArrayPool\.Rent'
        $source | Should Match '\[byte\[\]\]::new\(\$bufferLength\)'
        $source | Should Not Match '\[Text\.Encoding\]::UTF8\.GetBytes\(\$payloadJson\)'
        $source | Should Not Match '\$script:MvpProcessJournalUtf8\.GetBytes\(\$retentionManifest\)'
    }

    It 'serializes the launch environment without copying its complete reference array' {
        $source = Get-Content -LiteralPath $journalModule -Raw
        $writeFunction = [regex]::Match(
            $source,
            '(?s)function Write-MvpProcessJournalEntry \{.*?(?=\r?\nExport-ModuleMember)')

        $writeFunction.Value | Should Match 'environment_variables = \$LaunchIdentity\.environment_variables'
        $writeFunction.Value | Should Not Match 'environment_variables = @\(\$LaunchIdentity\.environment_variables\)'
    }
}
