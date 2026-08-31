$script:ZirconUiProfileWprScriptPath = $PSCommandPath

function Resolve-ZirconUiProfileNativeTool {
    param([Parameter(Mandatory = $true)][string]$Name)

    $command = Get-Command $Name -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -eq $command) {
        throw "UI sampled profiling requires $Name on PATH."
    }
    return $command.Source
}

function Invoke-ZirconUiProfileNativeTool {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    & $FilePath @Arguments | Out-Null
    return $LASTEXITCODE
}

function Test-ZirconUiSystemProfilePrivilege {
    if (-not $IsWindows -and $env:OS -ne "Windows_NT") {
        return $false
    }
    try {
        $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [Security.Principal.WindowsPrincipal]::new($identity)
        return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    }
    catch {
        return $false
    }
}

function Resolve-ZirconUiProfileEvidenceDirectory {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not [IO.Path]::IsPathRooted($Path)) {
        throw "UI profile evidence directory must be absolute."
    }
    $resolved = [IO.Path]::GetFullPath($Path).TrimEnd("\")
    $drive = [IO.Path]::GetPathRoot($resolved).TrimEnd("\")
    if ($drive -notin @("D:", "E:", "F:")) {
        throw "UI profile WPR artifacts must be written below D:, E:, or F:."
    }
    return $resolved
}

function Get-ZirconUiProfileEvidenceFingerprint {
    param([Parameter(Mandatory = $true)][string]$Path)

    $item = Get-Item -LiteralPath $Path -ErrorAction Stop
    return [pscustomobject]@{
        path = $item.FullName
        byte_length = $item.Length
        sha256 = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash
    }
}

function Format-ZirconUiProfileExitCode {
    param([Parameter(Mandatory = $true)][int]$ExitCode)

    $unsigned = [uint32]([int64]$ExitCode -band 0xffffffffL)
    return "$ExitCode (0x$($unsigned.ToString('X8')))"
}

function Get-ZirconUiProfileWprProductProcess {
    param([Parameter(Mandatory = $true)][psobject]$Capture)

    $property = $Capture.PSObject.Properties["product_process"]
    if ($null -eq $property) {
        return $null
    }
    return $property.Value
}

function Register-ZirconUiProfileWprProductProcess {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][psobject]$Capture,
        [Parameter(Mandatory = $true)][psobject]$Process,
        [Parameter(Mandatory = $true)][string]$ExecutablePath
    )

    $processId = [int64]$Process.Id
    if ($processId -le 0) {
        throw "WPR product attribution requires a positive process id."
    }
    $resolvedExecutablePath = [IO.Path]::GetFullPath($ExecutablePath)
    if (-not (Test-Path -LiteralPath $resolvedExecutablePath -PathType Leaf)) {
        throw "WPR product attribution executable is missing: '$resolvedExecutablePath'."
    }
    $startedUtc = ([datetime]$Process.StartTime).ToUniversalTime()
    $identity = [pscustomobject]@{
        process_id = $processId
        process_name = [string]$Process.ProcessName
        started_utc = $startedUtc.ToString("o")
        completed_utc = $null
        executable = Get-ZirconUiProfileEvidenceFingerprint -Path $resolvedExecutablePath
    }
    $property = $Capture.PSObject.Properties["product_process"]
    if ($null -eq $property) {
        $Capture | Add-Member -NotePropertyName product_process -NotePropertyValue $identity
    }
    else {
        $Capture.product_process = $identity
    }
}

function Complete-ZirconUiProfileWprProductProcess {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][psobject]$Capture,
        [Parameter(Mandatory = $true)][psobject]$Process
    )

    $identity = Get-ZirconUiProfileWprProductProcess -Capture $Capture
    if ($null -eq $identity) {
        throw "WPR product process must be registered before completion."
    }
    if ([int64]$Process.Id -ne [int64]$identity.process_id) {
        throw "WPR product process completion id does not match the registered process."
    }
    if (-not [bool]$Process.HasExited) {
        return $false
    }
    $completedUtc = ([datetime]$Process.ExitTime).ToUniversalTime()
    $startedUtc = [datetime]::Parse(
        [string]$identity.started_utc,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind
    )
    if ($completedUtc -lt $startedUtc) {
        throw "WPR product process completion precedes its registered start."
    }
    $identity.completed_utc = $completedUtc.ToString("o")
    return $true
}

function Get-ZirconUiProfileWprProductRange {
    param(
        [Parameter(Mandatory = $true)][psobject]$Capture,
        [Parameter(Mandatory = $true)][psobject]$ProductProcess
    )

    if ([string]::IsNullOrWhiteSpace([string]$ProductProcess.completed_utc)) {
        return $null
    }
    $traceStartedUtc = [datetime]::Parse(
        [string]$Capture.started_utc,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind
    ).ToUniversalTime()
    $processStartedUtc = [datetime]::Parse(
        [string]$ProductProcess.started_utc,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind
    ).ToUniversalTime()
    $processCompletedUtc = [datetime]::Parse(
        [string]$ProductProcess.completed_utc,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::RoundtripKind
    ).ToUniversalTime()
    $startUs = [Math]::Max(
        0L,
        [long][Math]::Round(($processStartedUtc - $traceStartedUtc).TotalMilliseconds * 1000.0)
    )
    $endUs = [Math]::Max(
        $startUs + 1L,
        [long][Math]::Round(($processCompletedUtc - $traceStartedUtc).TotalMilliseconds * 1000.0)
    )
    return [pscustomobject]@{
        start_us = $startUs
        end_us = $endUs
    }
}

function Start-ZirconUiProfileWprCapture {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)][string]$ProfileDir)

    $resolvedProfileDir = Resolve-ZirconUiProfileEvidenceDirectory -Path $ProfileDir
    New-Item -ItemType Directory -Force -Path $resolvedProfileDir | Out-Null
    $temporaryDirectory = Join-Path $resolvedProfileDir "wpr-recording"
    New-Item -ItemType Directory -Force -Path $temporaryDirectory | Out-Null
    $tracePath = Join-Path $resolvedProfileDir "system.etl"
    $wprPath = Resolve-ZirconUiProfileNativeTool -Name "wpr.exe"
    $arguments = @("-start", "CPU", "-filemode", "-recordtempto", $temporaryDirectory)
    $startedUtc = [datetime]::UtcNow
    $exitCode = Invoke-ZirconUiProfileNativeTool -FilePath $wprPath -Arguments $arguments
    if ($exitCode -ne 0) {
        throw "WPR could not start sampled CPU capture; exit code $(Format-ZirconUiProfileExitCode -ExitCode $exitCode). Run the capture from an elevated Windows terminal with the system performance profile privilege."
    }
    return [pscustomobject]@{
        schema_version = 2
        profile_dir = $resolvedProfileDir
        temporary_directory = $temporaryDirectory
        trace_path = $tracePath
        wpr_path = $wprPath
        started_utc = $startedUtc.ToString("o")
        product_process = $null
    }
}

function Stop-ZirconUiProfileWprCapture {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)][psobject]$Capture)

    $profileDir = Resolve-ZirconUiProfileEvidenceDirectory -Path ([string]$Capture.profile_dir)
    $tracePath = [IO.Path]::GetFullPath([string]$Capture.trace_path)
    $profilePrefix = $profileDir + [IO.Path]::DirectorySeparatorChar
    if (-not $tracePath.StartsWith($profilePrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "WPR ETL path must remain inside its UI profile session directory."
    }

    $stopArguments = @("-stop", $tracePath)
    $stopExitCode = Invoke-ZirconUiProfileNativeTool `
        -FilePath ([string]$Capture.wpr_path) `
        -Arguments $stopArguments
    if ($stopExitCode -ne 0) {
        throw "WPR could not stop sampled CPU capture; exit code $(Format-ZirconUiProfileExitCode -ExitCode $stopExitCode)."
    }
    if (-not (Test-Path -LiteralPath $tracePath -PathType Leaf) -or
        (Get-Item -LiteralPath $tracePath).Length -le 0) {
        throw "UI profile capture did not produce a nonempty WPR ETL at '$tracePath'."
    }

    $sampledProfilePath = Join-Path $profileDir "cpu-sampled-profile.txt"
    $xperfPath = Resolve-ZirconUiProfileNativeTool -Name "xperf.exe"
    $analysisArguments = @(
        "-i", $tracePath,
        "-symbols",
        "-target", "machine",
        "-o", $sampledProfilePath,
        "-a", "profile", "-detail"
    )
    $analysisExitCode = Invoke-ZirconUiProfileNativeTool `
        -FilePath $xperfPath `
        -Arguments $analysisArguments
    if ($analysisExitCode -ne 0) {
        throw "xperf could not export the sampled CPU profile; exit code $(Format-ZirconUiProfileExitCode -ExitCode $analysisExitCode)."
    }
    if (-not (Test-Path -LiteralPath $sampledProfilePath -PathType Leaf) -or
        (Get-Item -LiteralPath $sampledProfilePath).Length -le 0) {
        throw "xperf did not produce a nonempty sampled CPU profile at '$sampledProfilePath'."
    }

    $productProcess = Get-ZirconUiProfileWprProductProcess -Capture $Capture
    $productRange = if ($null -eq $productProcess) {
        $null
    }
    else {
        Get-ZirconUiProfileWprProductRange `
            -Capture $Capture `
            -ProductProcess $productProcess
    }
    if ($null -ne $productProcess -and $null -eq $productRange) {
        throw "WPR product attribution requires a complete product process lifetime."
    }
    $productSampledStacks = $null
    $productAnalysisCommand = $null
    if ($null -ne $productRange) {
        $productSampledStacksPath = Join-Path $profileDir "cpu-product-sampled-stacks.txt"
        $productAnalysisArguments = @(
            "-i", $tracePath,
            "-symbols",
            "-target", "machine",
            "-o", $productSampledStacksPath,
            "-a", "stack",
            "-pid", ([string]$productProcess.process_id),
            "-range", ([string]$productRange.start_us), ([string]$productRange.end_us),
            "-event", "PROFILE",
            "-butterfly", "1"
        )
        $productAnalysisExitCode = Invoke-ZirconUiProfileNativeTool `
            -FilePath $xperfPath `
            -Arguments $productAnalysisArguments
        if ($productAnalysisExitCode -ne 0) {
            throw "xperf could not export the product-filtered sampled stacks; exit code $(Format-ZirconUiProfileExitCode -ExitCode $productAnalysisExitCode)."
        }
        if (-not (Test-Path -LiteralPath $productSampledStacksPath -PathType Leaf) -or
            (Get-Item -LiteralPath $productSampledStacksPath).Length -le 0) {
            throw "xperf did not produce nonempty product-filtered sampled stacks at '$productSampledStacksPath'."
        }
        $productSampledStacks = Get-ZirconUiProfileEvidenceFingerprint `
            -Path $productSampledStacksPath
        $productAnalysisCommand = "xperf -i system.etl -symbols -target machine -o cpu-product-sampled-stacks.txt -a stack -pid $($productProcess.process_id) -range $($productRange.start_us) $($productRange.end_us) -event PROFILE -butterfly 1"
    }

    $hasProductAttribution = $null -ne $productSampledStacks
    $scriptFingerprint = if (Test-Path -LiteralPath $script:ZirconUiProfileWprScriptPath) {
        Get-ZirconUiProfileEvidenceFingerprint -Path $script:ZirconUiProfileWprScriptPath
    }
    else {
        $null
    }
    $receiptPath = Join-Path $profileDir "wpr-capture.json"
    $receipt = [ordered]@{
        schema_version = 2
        evidence_kind = "windows_sampled_cpu_profile"
        is_product_timing = $hasProductAttribution
        started_utc = [string]$Capture.started_utc
        stopped_utc = [datetime]::UtcNow.ToString("o")
        trace = Get-ZirconUiProfileEvidenceFingerprint -Path $tracePath
        sampled_profile = Get-ZirconUiProfileEvidenceFingerprint -Path $sampledProfilePath
        product_sampled_stacks = $productSampledStacks
        storage = [ordered]@{
            profile_directory = $profileDir
            recording_temporary_directory = [string]$Capture.temporary_directory
            system_drive_used = $false
        }
        recorder = [ordered]@{
            path = [string]$Capture.wpr_path
            arguments = @("-start", "CPU", "-filemode", "-recordtempto", [string]$Capture.temporary_directory)
        }
        analysis = [ordered]@{
            path = $xperfPath
            command = "xperf -i system.etl -symbols -target machine -o cpu-sampled-profile.txt -a profile -detail"
            product_command = $productAnalysisCommand
        }
        attribution = [ordered]@{
            scope = if ($hasProductAttribution) { "product_process" } else { "system" }
            product_process_filter_applied = $hasProductAttribution
            process_lifetime_range_applied = $hasProductAttribution
            process = $productProcess
            product_acceptance_requirement = if ($hasProductAttribution) {
                "Satisfied for sampled CPU attribution; latency and frame-time acceptance remain separate."
            }
            else {
                "Bind the sampled rows to zircon_editor.exe PID and process lifetime before treating this receipt as product timing."
            }
        }
        tool_binding = $scriptFingerprint
    }
    $receipt | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath $receiptPath -Encoding UTF8
    return $receiptPath
}
