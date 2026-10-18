$ErrorActionPreference = 'Stop'

# Intentionally avoid advanced parameter binding: it consumes native options
# such as Cargo's -p as PowerShell common-parameter abbreviations.
[string[]]$commandArguments = @($args)
$RequireMatchingTests = $false
$VsDevCmdPath = ''
$commandIndex = 0
while ($commandIndex -lt $commandArguments.Count) {
    if ($commandArguments[$commandIndex] -eq '-RequireMatchingTests') {
        $RequireMatchingTests = $true
        $commandIndex++
    } elseif ($commandArguments[$commandIndex] -eq '-VsDevCmdPath') {
        $commandIndex++
        if ($commandIndex -ge $commandArguments.Count) { throw '-VsDevCmdPath requires a path.' }
        $VsDevCmdPath = $commandArguments[$commandIndex]
        $commandIndex++
    } else {
        break
    }
}
if ($commandArguments.Count - $commandIndex -lt 2) {
    throw 'Pass command arguments after the executable.'
}
$Executable = $commandArguments[$commandIndex]
[string[]]$ExecutableArguments = $commandArguments[($commandIndex + 1)..($commandArguments.Count - 1)]

if ([string]::IsNullOrWhiteSpace($VsDevCmdPath)) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) {
        throw 'Visual Studio Installer vswhere.exe was not found on this CI runner.'
    }
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($installation)) {
        throw 'No Visual Studio installation with x64 C++ build tools is available.'
    }
    $devCommand = Join-Path $installation 'Common7\Tools\VsDevCmd.bat'
} else {
    if (-not [IO.Path]::IsPathFullyQualified($VsDevCmdPath) -or $VsDevCmdPath -match '[%"\r\n]') {
        throw 'Pass an absolute VsDevCmd.bat path without command expansion characters.'
    }
    $devCommand = [IO.Path]::GetFullPath($VsDevCmdPath)
}
if (-not (Test-Path -LiteralPath $devCommand -PathType Leaf)) {
    throw "Visual Studio developer command environment is missing: $devCommand"
}

$environmentCommand = 'call "' + $devCommand + '" -arch=amd64 >nul && set'
$environmentLines = & $env:ComSpec /d /s /c $environmentCommand
if ($LASTEXITCODE -ne 0) {
    throw 'VsDevCmd.bat failed to initialize the x64 MSVC environment.'
}
$developerPath = $null
foreach ($line in $environmentLines) {
    $separator = $line.IndexOf('=')
    if ($separator -le 0) { continue }
    $name = $line.Substring(0, $separator)
    $value = $line.Substring($separator + 1)
    if ($name -match '^=[A-Za-z]:$') { continue }
    if ($name -ieq 'PATH') {
        # Preserve the PATH updated by VsDevCmd when the inherited environment also contains
        # a differently cased Path entry. Importing that stale entry last loses MSVC tools.
        if ($name -ceq 'PATH' -or $null -eq $developerPath) { $developerPath = $value }
        continue
    }
    Set-Item -Path "Env:$name" -Value $value
}
if ($null -ne $developerPath) { $env:PATH = $developerPath }

if (-not $env:VSCMD_VER -or -not (Get-Command cl.exe -ErrorAction SilentlyContinue) -or -not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
    throw 'VsDevCmd did not provide the MSVC compiler and linker.'
}

if ($RequireMatchingTests) {
    $listArguments = @($ExecutableArguments)
    if ($ExecutableArguments -contains '--') {
        $listArguments += '--list'
    } else {
        $listArguments += @('--', '--list')
    }
    $listing = & $Executable @listArguments 2>&1
    $listExitCode = $LASTEXITCODE
    $listing | ForEach-Object { Write-Output ([string]$_) }
    if ($listExitCode -ne 0) {
        exit $listExitCode
    }
    $matchingTests = @(
        $listing | Where-Object { ([string]$_) -match '^\s*\S.*: test$' }
    )
    if ($matchingTests.Count -eq 0) {
        throw 'The requested Cargo test filter matched zero tests; refusing a vacuous CI pass.'
    }
    Write-Output "Verified $($matchingTests.Count) matching test(s); executing them now."

    $testOutput = & $Executable @ExecutableArguments 2>&1
    $testExitCode = $LASTEXITCODE
    $testOutput | ForEach-Object { Write-Output ([string]$_) }
    if ($testExitCode -ne 0) {
        exit $testExitCode
    }
    $passedSummary = @(
        $testOutput | Where-Object {
            ([string]$_) -match '(?i)test result: ok\.\s+([1-9][0-9]*) passed;'
        }
    )
    if ($passedSummary.Count -eq 0) {
        throw 'The filtered Cargo test run completed without any passed tests.'
    }
    exit 0
}

& $Executable @ExecutableArguments
exit $LASTEXITCODE
