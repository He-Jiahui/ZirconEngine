Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Remove-TestTreeWithRetry {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    for ($attempt = 0; $attempt -lt 50; $attempt++) {
        if (-not (Test-Path -LiteralPath $Path)) {
            return
        }
        try {
            Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction Stop
            return
        }
        catch {
            if ($attempt -eq 49) {
                throw
            }
            Start-Sleep -Milliseconds 100
        }
    }
}

function Convert-CimProcessCreationDateToUtc {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [object]$CreationDate
    )

    if ($CreationDate -is [datetime]) {
        return $CreationDate.ToUniversalTime()
    }

    $creationText = [string]$CreationDate
    if ($creationText -match '^\d{14}\.\d{6}[+-]\d{3}$') {
        return [System.Management.ManagementDateTimeConverter]::ToDateTime($creationText).ToUniversalTime()
    }

    throw "Unrecognized Win32_Process CreationDate value: $creationText"
}

function Get-FixtureProcessTreeSnapshot {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [System.Diagnostics.Process]$Process,
        [Parameter(Mandatory)]
        [string]$FixtureRepoRoot,
        [Parameter(Mandatory)]
        [string]$ExpectedExecutablePath
    )

    $Process.Refresh()
    if ($Process.HasExited) {
        return [pscustomobject]@{
            RootInstance = $null
            Descendants = @()
        }
    }

    $rootId = $Process.Id
    $rootCreatedUtc = $Process.StartTime.ToUniversalTime()
    $snapshot = @(Get-CimInstance -ClassName Win32_Process -ErrorAction Stop)
    $rootInstances = @($snapshot | Where-Object { [int]$_.ProcessId -eq $rootId })
    if ($rootInstances.Count -ne 1) {
        throw "Could not identify the live fixture process instance for pid $rootId."
    }
    $rootInstance = $rootInstances[0]

    $cimRootCreatedUtc = Convert-CimProcessCreationDateToUtc -CreationDate $rootInstance.CreationDate
    if ([Math]::Abs(($cimRootCreatedUtc - $rootCreatedUtc).TotalSeconds) -gt 2) {
        throw "The live pid $rootId no longer matches the Start-Process fixture identity."
    }

    $expectedPath = [System.IO.Path]::GetFullPath($ExpectedExecutablePath)
    $livePath = [System.IO.Path]::GetFullPath([string]$rootInstance.ExecutablePath)
    if (-not [string]::Equals($expectedPath, $livePath, [StringComparison]::OrdinalIgnoreCase)) {
        throw "The live pid $rootId does not use the fixture's expected executable."
    }
    $commandLine = [string]$rootInstance.CommandLine
    if ([string]::IsNullOrWhiteSpace($commandLine) -or $commandLine.IndexOf($FixtureRepoRoot, [StringComparison]::OrdinalIgnoreCase) -lt 0 -or $commandLine.IndexOf("tools.session_coordinator", [StringComparison]::OrdinalIgnoreCase) -lt 0) {
        throw "The live pid $rootId command line does not identify this fixture's coordinator."
    }

    $descendants = [System.Collections.Generic.List[object]]::new()
    $pendingParents = [System.Collections.Generic.Queue[object]]::new()
    $pendingParents.Enqueue([pscustomobject]@{
        ProcessId = $rootId
        CreatedUtc = $rootCreatedUtc
    })
    $seenProcessIds = @{}
    $seenProcessIds[$rootId] = $true

    while ($pendingParents.Count -gt 0) {
        $parent = $pendingParents.Dequeue()
        foreach ($candidate in $snapshot) {
            if ([int]$candidate.ParentProcessId -ne [int]$parent.ProcessId) {
                continue
            }
            $childId = [int]$candidate.ProcessId
            if ($seenProcessIds.ContainsKey($childId)) {
                continue
            }

            $childCreatedUtc = Convert-CimProcessCreationDateToUtc -CreationDate $candidate.CreationDate
            if ($childCreatedUtc -lt $parent.CreatedUtc) {
                continue
            }

            $seenProcessIds[$childId] = $true
            $descendants.Add([pscustomobject]@{
                ProcessId = $childId
                CreationDateTicks = [long]$childCreatedUtc.Ticks
            })
            $pendingParents.Enqueue([pscustomobject]@{
                ProcessId = $childId
                CreatedUtc = $childCreatedUtc
            })
        }
    }

    return [pscustomobject]@{
        RootInstance = $rootInstance
        RootCreationDateTicks = [long]$cimRootCreatedUtc.Ticks
        Descendants = $descendants.ToArray()
    }
}

function Stop-FixtureProcessAndWait {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [System.Diagnostics.Process]$Process,
        [Parameter(Mandatory)]
        [string]$FixtureRepoRoot,
        [Parameter(Mandatory)]
        [string]$ExpectedExecutablePath,
        [ValidateRange(1, 600000)]
        [int]$TimeoutMilliseconds = 5000
    )

    try {
        $Process.Refresh()
        if (-not $Process.HasExited) {
            $tree = Get-FixtureProcessTreeSnapshot -Process $Process -FixtureRepoRoot $FixtureRepoRoot -ExpectedExecutablePath $ExpectedExecutablePath
            if ($null -eq $tree.RootInstance) {
                throw "The fixture process exited before its live identity could be confirmed."
            }

            $taskkillOutput = @()
            $taskkillExitCode = $null
            $Process.Refresh()
            if (-not $Process.HasExited) {
                $currentRoot = @(Get-CimInstance -ClassName Win32_Process -Filter "ProcessId = $($Process.Id)" -ErrorAction Stop)
                if ($currentRoot.Count -eq 1 -and [long](Convert-CimProcessCreationDateToUtc -CreationDate $currentRoot[0].CreationDate).Ticks -eq [long]$tree.RootCreationDateTicks) {
                    # taskkill is used only after this Start-Process object, its
                    # creation time, executable, and unique fixture repo argument
                    # match Win32_Process. Recheck immediately before using its PID.
                    $currentPath = [System.IO.Path]::GetFullPath([string]$currentRoot[0].ExecutablePath)
                    $currentCommandLine = [string]$currentRoot[0].CommandLine
                    if (
                        [string]::Equals($currentPath, [System.IO.Path]::GetFullPath($ExpectedExecutablePath), [StringComparison]::OrdinalIgnoreCase) -and
                        $currentCommandLine.IndexOf($FixtureRepoRoot, [StringComparison]::OrdinalIgnoreCase) -ge 0 -and
                        $currentCommandLine.IndexOf("tools.session_coordinator", [StringComparison]::OrdinalIgnoreCase) -ge 0
                    ) {
                        $taskkill = Join-Path $env:SystemRoot "System32\taskkill.exe"
                        if (-not [System.IO.File]::Exists($taskkill)) {
                            throw "The Windows taskkill executable is unavailable: $taskkill"
                        }
                        $priorErrorActionPreference = $ErrorActionPreference
                        try {
                            $ErrorActionPreference = "Continue"
                            $taskkillOutput = @(& $taskkill /PID $Process.Id /T /F 2>&1)
                            $taskkillExitCode = $LASTEXITCODE
                        }
                        finally {
                            $ErrorActionPreference = $priorErrorActionPreference
                        }
                    }
                    else {
                        throw "The live pid $($Process.Id) no longer matches the coordinator fixture identity."
                    }
                }
                else {
                    $Process.Refresh()
                    if (-not $Process.HasExited) {
                        throw "The live pid $($Process.Id) no longer matches the captured fixture process tree."
                    }
                }
            }
            $taskkillDiagnostic = (@($taskkillOutput | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine).Trim()

            if (-not $Process.WaitForExit($TimeoutMilliseconds)) {
                throw "Fixture process did not exit within $TimeoutMilliseconds ms after tree termination (taskkill exit $taskkillExitCode): $taskkillDiagnostic"
            }

            $clock = [System.Diagnostics.Stopwatch]::StartNew()
            $remainingDescendants = @()
            while ($clock.ElapsedMilliseconds -lt $TimeoutMilliseconds) {
                $liveProcesses = @(Get-CimInstance -ClassName Win32_Process -ErrorAction Stop)
                $liveCreationDates = @{}
                foreach ($liveProcess in $liveProcesses) {
                    $liveCreationDates[[int]$liveProcess.ProcessId] = [long](Convert-CimProcessCreationDateToUtc -CreationDate $liveProcess.CreationDate).Ticks
                }
                $remainingDescendants = @()
                foreach ($descendant in $tree.Descendants) {
                    if ($liveCreationDates.ContainsKey([int]$descendant.ProcessId) -and [long]$liveCreationDates[[int]$descendant.ProcessId] -eq [long]$descendant.CreationDateTicks) {
                        $remainingDescendants += $descendant
                    }
                }
                if ($remainingDescendants.Count -eq 0) {
                    break
                }
                Start-Sleep -Milliseconds 100
            }
            $clock.Stop()

            if ($remainingDescendants.Count -gt 0) {
                $remainingIds = ($remainingDescendants | ForEach-Object { $_.ProcessId }) -join ", "
                throw "Fixture descendants remained alive after tree termination (pid(s) $remainingIds; taskkill exit $taskkillExitCode): $taskkillDiagnostic"
            }
        }
        else {
            throw "The fixture process had already exited before its process tree could be confirmed; refusing to remove the fixture."
        }

        if (-not $Process.WaitForExit($TimeoutMilliseconds)) {
            throw "Fixture process did not exit within $TimeoutMilliseconds ms."
        }

        $Process.Refresh()
        if (-not $Process.HasExited) {
            throw "Fixture process remained alive after its termination wait."
        }
    }
    catch {
        throw "Unable to confirm fixture process termination: $($_.Exception.Message)"
    }
}

function Resolve-TestFixtureRoot {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        [string]$FixtureRepoRoot
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $temporaryRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    $trimCharacters = [char[]]@([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    $temporaryRoot = $temporaryRoot.TrimEnd($trimCharacters)
    $fixtureName = [System.IO.Path]::GetFileName($fullPath)
    $fixtureParent = [System.IO.Path]::GetFullPath([System.IO.Path]::GetDirectoryName($fullPath))
    $normalizedRepo = [System.IO.Path]::GetFullPath($FixtureRepoRoot)
    $expectedRepo = [System.IO.Path]::GetFullPath((Join-Path $fullPath "repo"))

    if (-not [string]::Equals($fixtureParent, $temporaryRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to clean a fixture path outside the direct temp root: $fullPath"
    }
    if ($fixtureName -notmatch '^zircon-coordinator-smoke-[0-9a-f]{32}$') {
        throw "Refusing to clean a path that is not a randomized coordinator smoke fixture: $fullPath"
    }
    if (-not [string]::Equals($normalizedRepo, $expectedRepo, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to clean a fixture whose repo path does not match its root: $normalizedRepo"
    }
    if (Test-Path -LiteralPath $fullPath) {
        $fixtureItem = Get-Item -LiteralPath $fullPath -Force -ErrorAction Stop
        if (($fixtureItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Refusing to recursively clean a reparse-point fixture root: $fullPath"
        }
    }

    return $fullPath
}

function Write-FixtureProcessOutputDiagnostics {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$FixtureRoot,
        [Parameter(Mandatory)]
        [string]$StdoutPath,
        [Parameter(Mandatory)]
        [string]$StderrPath,
        [AllowEmptyCollection()]
        [Parameter(Mandatory)]
        [System.Collections.Generic.List[object]]$Errors,
        [ValidateRange(1, 65536)]
        [int]$MaximumTailBytes = 8192
    )

    $fullFixtureRoot = [System.IO.Path]::GetFullPath($FixtureRoot)
    $sections = [System.Collections.Generic.List[string]]::new()
    $sources = @(
        [pscustomobject]@{ Name = "coordinator stdout"; Path = $StdoutPath },
        [pscustomobject]@{ Name = "coordinator stderr"; Path = $StderrPath }
    )

    foreach ($source in $sources) {
        $stream = $null
        try {
            $fullPath = [System.IO.Path]::GetFullPath($source.Path)
            $parentPath = [System.IO.Path]::GetDirectoryName($fullPath)
            if (-not [string]::Equals($parentPath, $fullFixtureRoot, [StringComparison]::OrdinalIgnoreCase)) {
                $sections.Add("===== $($source.Name): omitted because the path is outside the fixture root =====")
                continue
            }
            if (-not [System.IO.File]::Exists($fullPath)) {
                $sections.Add("===== $($source.Name): unavailable (file not created) =====")
                continue
            }

            $stream = [System.IO.File]::Open(
                $fullPath,
                [System.IO.FileMode]::Open,
                [System.IO.FileAccess]::Read,
                ([System.IO.FileShare]::ReadWrite -bor [System.IO.FileShare]::Delete)
            )
            $startOffset = [Math]::Max([long]0, $stream.Length - $MaximumTailBytes)
            $stream.Position = $startOffset
            $byteCount = [int][Math]::Min([long]$MaximumTailBytes, $stream.Length - $startOffset)
            $buffer = New-Object byte[] $byteCount
            $bytesRead = 0
            while ($bytesRead -lt $byteCount) {
                $readNow = $stream.Read($buffer, $bytesRead, $byteCount - $bytesRead)
                if ($readNow -le 0) {
                    break
                }
                $bytesRead += $readNow
            }

            $tail = [System.Text.Encoding]::UTF8.GetString($buffer, 0, $bytesRead).Trim()
            if ([string]::IsNullOrWhiteSpace($tail)) {
                $tail = "<empty>"
            }
            if ($startOffset -gt 0) {
                $tail = "[Earlier output omitted; showing at most the last $MaximumTailBytes bytes.]`n$tail"
            }
            $sections.Add("===== $($source.Name) (last $bytesRead bytes; $fullPath) =====`n$tail")
        }
        catch {
            $sections.Add("===== $($source.Name) capture failed: $($_.Exception.Message) =====")
        }
        finally {
            if ($null -ne $stream) {
                $stream.Dispose()
            }
        }
    }

    try {
        [Console]::Error.WriteLine(($sections -join [Environment]::NewLine))
    }
    catch {
        $Errors.Add($_)
    }
}

function Invoke-FixtureTeardown {
    [CmdletBinding()]
    param(
        [AllowNull()]
        [System.Diagnostics.Process]$Process,
        [string]$FixtureRepoRoot,
        [string]$ExpectedExecutablePath,
        [switch]$CaptureProcessOutputDiagnostics,
        [string]$StdoutPath,
        [string]$StderrPath,
        [Parameter(Mandatory)]
        [string]$Path
    )

    $errors = [System.Collections.Generic.List[object]]::new()
    try {
        $safePath = Resolve-TestFixtureRoot -Path $Path -FixtureRepoRoot $FixtureRepoRoot
    }
    catch {
        $errors.Add($_)
        return [pscustomobject]@{
            Errors = $errors.ToArray()
        }
    }

    if ($null -ne $Process) {
        try {
            Stop-FixtureProcessAndWait -Process $Process -FixtureRepoRoot $FixtureRepoRoot -ExpectedExecutablePath $ExpectedExecutablePath
        }
        catch {
            $errors.Add($_)
            if ($CaptureProcessOutputDiagnostics) {
                try {
                    Write-FixtureProcessOutputDiagnostics `
                        -FixtureRoot $safePath `
                        -StdoutPath $StdoutPath `
                        -StderrPath $StderrPath `
                        -Errors $errors
                }
                catch {
                    $errors.Add($_)
                }
            }
            return [pscustomobject]@{
                Errors = $errors.ToArray()
            }
        }
    }

    if ($CaptureProcessOutputDiagnostics) {
        try {
            Write-FixtureProcessOutputDiagnostics `
                -FixtureRoot $safePath `
                -StdoutPath $StdoutPath `
                -StderrPath $StderrPath `
                -Errors $errors
        }
        catch {
            $errors.Add($_)
        }
    }

    if (Test-Path -LiteralPath $safePath) {
        try {
            Remove-TestTreeWithRetry -Path $safePath
        }
        catch {
            $errors.Add($_)
        }
    }

    return [pscustomobject]@{
        Errors = $errors.ToArray()
    }
}

Export-ModuleMember -Function Remove-TestTreeWithRetry, Stop-FixtureProcessAndWait, Invoke-FixtureTeardown
