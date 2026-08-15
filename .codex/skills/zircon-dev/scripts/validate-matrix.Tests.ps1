$script:ValidateMatrixScript = Join-Path $PSScriptRoot "validate-matrix.ps1"
$script:ValidateMatrixTestRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$script:OriginalValidateMatrixTestMode = $env:VALIDATE_MATRIX_TEST_MODE
$script:OriginalCargoTargetDir = $env:CARGO_TARGET_DIR
$script:ManagedPoolRegex = '[D-F]:\\(?:cargo-targets|targets|ZirconBuilds)\\zircon-engine\\pool\\[0-9a-f]{64}'

$env:VALIDATE_MATRIX_TEST_MODE = "1"
. $script:ValidateMatrixScript -DryRun -SkipBuild -SkipTest
$env:VALIDATE_MATRIX_TEST_MODE = $script:OriginalValidateMatrixTestMode

function Get-CiExportPlatformMatrix {
    $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\ci.yml"
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath
    $match = [regex]::Match($workflow, "(?m)^\s*export-platform:\s*\[(?<platforms>[^\]]+)\]")

    if (-not $match.Success) {
        throw "Could not find export-platform matrix in $workflowPath"
    }

    return $match.Groups["platforms"].Value.Split(",") |
        ForEach-Object { $_.Trim() } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
}

Describe "Validate matrix Windows PowerShell compatibility" {
    It "keeps the validator source ASCII when it has no UTF-8 byte-order mark" {
        $sourceBytes = [System.IO.File]::ReadAllBytes($script:ValidateMatrixScript)
        $hasUtf8ByteOrderMark = $sourceBytes.Length -ge 3 -and
            $sourceBytes[0] -eq 0xEF -and
            $sourceBytes[1] -eq 0xBB -and
            $sourceBytes[2] -eq 0xBF

        if (-not $hasUtf8ByteOrderMark) {
            (@($sourceBytes | Where-Object { $_ -gt 0x7F })).Count | Should Be 0
        }
    }
}

Describe "Validate matrix managed Cargo environment policy" {
    It "rejects a target that physically resolves outside the approved drives before coordinator acquisition" {
        $targetDirectory = Join-Path "C:\cargo-targets\zircon-engine" (
            "validate-matrix-disallowed-{0}" -f [guid]::NewGuid().ToString("N")
        )

        $failure = $null
        try {
            Resolve-ManagedCargoTargetPath -TargetDirectory $targetDirectory | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match "must physically resolve under D:, E:, or F:"
        Test-Path -LiteralPath $targetDirectory | Should Be $false
    }

    It "keeps manual and coordinator-managed target validation on the shared physical-path guard" {
        $source = Get-Content -Raw -Encoding UTF8 $script:ValidateMatrixScript

        $source | Should Match 'Resolve-ManagedCargoTargetPath\s+`?\s*-TargetDirectory\s+\$absoluteRequestedTarget'
        $source | Should Match 'Resolve-ManagedCargoTargetPath\s+-TargetDirectory\s+\$targetDir'
    }

    It "binds temporary and Cargo cache output to the managed target and restores the caller environment" {
        $targetDirectory = Join-Path "E:\cargo-targets\zircon-engine" (
            "validate-matrix-temporary-{0}" -f [guid]::NewGuid().ToString("N")
        )
        $names = @("CARGO_TARGET_DIR", "TEMP", "TMP", "TMPDIR", "CARGO_HOME", "SCCACHE_DIR")
        $previousValues = @{}
        foreach ($name in $names) {
            $previousValues[$name] = [Environment]::GetEnvironmentVariable($name, "Process")
            [Environment]::SetEnvironmentVariable($name, "C:\caller-$($name.ToLowerInvariant())", "Process")
        }

        try {
            $lease = $null
            try {
                $managedTargetResolution = Resolve-ManagedCargoTargetPath -TargetDirectory $targetDirectory
                $lease = Push-ManagedCargoEnvironment -TargetDirectory $targetDirectory

                $lease.TemporaryDisplayPath | Should Be (Join-Path $managedTargetResolution.DisplayPath "temporary")
                $lease.CargoHomeDisplayPath | Should Be (Join-Path $managedTargetResolution.DisplayPath "cargo-home")
                $lease.SccacheDisplayPath | Should Be (Join-Path $managedTargetResolution.DisplayPath "sccache")
                Test-Path -LiteralPath $lease.TemporaryOperationalPath -PathType Container | Should Be $true
                Test-Path -LiteralPath $lease.CargoHomeOperationalPath -PathType Container | Should Be $true
                Test-Path -LiteralPath $lease.SccacheOperationalPath -PathType Container | Should Be $true
                [Environment]::GetEnvironmentVariable("CARGO_TARGET_DIR", "Process") | Should Be $managedTargetResolution.OperationalPath
                foreach ($name in @("TEMP", "TMP", "TMPDIR")) {
                    [Environment]::GetEnvironmentVariable($name, "Process") | Should Be $lease.TemporaryOperationalPath
                }
                [Environment]::GetEnvironmentVariable("CARGO_HOME", "Process") | Should Be $lease.CargoHomeOperationalPath
                [Environment]::GetEnvironmentVariable("SCCACHE_DIR", "Process") | Should Be $lease.SccacheOperationalPath
            }
            finally {
                if ($null -ne $lease) {
                    Pop-ManagedCargoEnvironment -Lease $lease
                }
                if (Test-Path -LiteralPath $targetDirectory) {
                    Remove-Item -LiteralPath $targetDirectory -Recurse -Force
                }
            }
            foreach ($name in $names) {
                [Environment]::GetEnvironmentVariable($name, "Process") | Should Be "C:\caller-$($name.ToLowerInvariant())"
            }
        }
        finally {
            foreach ($name in $names) {
                [Environment]::SetEnvironmentVariable($name, $previousValues[$name], "Process")
            }
        }
    }
}

function Get-CiProfileFeatureMatrix {
    $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\profile-feature-contract.yml"
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath
    $caseMatches = [regex]::Matches(
        $workflow,
        "(?ms)^\s*-\s+label:\s*(?<label>[^\r\n]+)\s+package:\s*(?<package>[^\r\n]+)\s+features:\s*(?<features>[^\r\n]+)(?:\s+bin:\s*(?<bin>[^\r\n]+))?"
    )

    if ($caseMatches.Count -eq 0) {
        throw "Could not find profile-feature matrix cases in $workflowPath"
    }

    return $caseMatches | ForEach-Object {
        [pscustomobject]@{
            Label    = $_.Groups["label"].Value.Trim()
            Package  = $_.Groups["package"].Value.Trim()
            Features = $_.Groups["features"].Value.Trim()
            Bin      = $_.Groups["bin"].Value.Trim().Trim('"')
        }
    }
}

function Get-WorkflowAptPackages {
    param(
        [string]$WorkflowRelativePath,
        [string]$StepName
    )

    $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot $WorkflowRelativePath
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath
    $escapedStepName = [regex]::Escape($StepName)
    $stepMatch = [regex]::Match(
        $workflow,
        "(?ms)^\s*-\s+name:\s*$escapedStepName\s+run:\s*\|\s*(?<body>.*?)(?=^\s*-\s+(?:name|uses):|\z)"
    )
    if (-not $stepMatch.Success) {
        throw "Could not find workflow step '$StepName' in $workflowPath"
    }

    $packages = [System.Collections.Generic.List[string]]::new()
    $collecting = $false
    foreach ($line in ($stepMatch.Groups["body"].Value -split "\r?\n")) {
        $trimmed = $line.Trim()
        if ($trimmed -match '^sudo apt-get install -y\s*\\?$') {
            $collecting = $true
            continue
        }

        if (-not $collecting) {
            continue
        }

        if ([string]::IsNullOrWhiteSpace($trimmed)) {
            break
        }

        $package = ($trimmed -replace '\s*\\\s*$', '').Trim()
        if (-not [string]::IsNullOrWhiteSpace($package)) {
            $packages.Add($package) | Out-Null
        }

        if (-not $trimmed.EndsWith("\")) {
            break
        }
    }

    if ($packages.Count -eq 0) {
        throw "Could not find apt package list in workflow step '$StepName' in $workflowPath"
    }

    return $packages.ToArray()
}

function Assert-WorkflowHasContractScaffolding {
    param(
        [string]$WorkflowRelativePath,
        [string]$WorkflowText
    )

    $WorkflowText | Should Match "push:\s*\r?\n\s*branches:\s*\[main, master\]"
    $WorkflowText | Should Match "pull_request:\s*\r?\n\s*branches:\s*\[main, master\]"
    $WorkflowText | Should Match "CARGO_TERM_COLOR:\s*always"
    $WorkflowText | Should Match "RUST_BACKTRACE:\s*1"
    $WorkflowText | Should Match "fail-fast:\s*false"
    $WorkflowText | Should Match "actions/checkout@v5"
    $WorkflowText | Should Match "dtolnay/rust-toolchain@stable"
    $WorkflowText | Should Match "Swatinem/rust-cache@v2"
}

function Invoke-ValidateMatrixCli {
    param(
        [string[]]$Arguments,
        [switch]$PreserveCargoTargetDir
    )

    $powershell = Get-Command pwsh -ErrorAction SilentlyContinue
    if ($null -eq $powershell) {
        $powershell = Get-Command powershell -ErrorAction Stop
    }

    $commandArgs = @(
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        $script:ValidateMatrixScript
    ) + $Arguments
    $previousCargoTargetDir = $env:CARGO_TARGET_DIR
    $previousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        if (-not $PreserveCargoTargetDir) {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        }
        $output = & $powershell.Source @commandArgs 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousErrorActionPreference
        if ($null -eq $previousCargoTargetDir) {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        } else {
            $env:CARGO_TARGET_DIR = $previousCargoTargetDir
        }
    }

    return [pscustomobject]@{
        ExitCode = $exitCode
        Output   = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    }
}

function Invoke-ValidateMatrixCliWithCargoTargetDir {
    param(
        [string[]]$Arguments,
        [AllowNull()]
        [string]$CargoTargetDir = $null
    )

    $previousCargoTargetDir = $env:CARGO_TARGET_DIR
    try {
        if ($null -eq $CargoTargetDir) {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        } else {
            $env:CARGO_TARGET_DIR = $CargoTargetDir
        }
        return Invoke-ValidateMatrixCli -Arguments $Arguments -PreserveCargoTargetDir
    } finally {
        if ($null -eq $previousCargoTargetDir) {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        } else {
            $env:CARGO_TARGET_DIR = $previousCargoTargetDir
        }
    }
}

function Invoke-ValidateMatrixCliWithoutCargo {
    param([string[]]$Arguments)

    $powershell = Get-Command pwsh -ErrorAction SilentlyContinue
    if ($null -eq $powershell) {
        $powershell = Get-Command powershell -ErrorAction Stop
    }

    $scriptDir = Split-Path $script:ValidateMatrixScript -Parent
    $commandArgs = @(
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        $script:ValidateMatrixScript
    ) + $Arguments

    $previousPath = $env:PATH
    $previousCargoTargetDir = $env:CARGO_TARGET_DIR
    $previousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        $pythonDirectory = Split-Path (Get-Command python -ErrorAction Stop).Source -Parent
        $env:PATH = "$scriptDir;$pythonDirectory"
        $output = & $powershell.Source @commandArgs 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousErrorActionPreference
        $env:PATH = $previousPath
        if ($null -eq $previousCargoTargetDir) {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        } else {
            $env:CARGO_TARGET_DIR = $previousCargoTargetDir
        }
    }

    return [pscustomobject]@{
        ExitCode = $exitCode
        Output   = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    }
}

function Get-CargoFeatureValues {
    param(
        [string]$CargoTomlPath,
        [string]$FeatureName
    )

    $cargoToml = Get-Content -Raw -Encoding UTF8 $CargoTomlPath
    $featuresMatch = [regex]::Match($cargoToml, "(?ms)^\[features\]\s*(?<features>.*?)(?=^\[|\z)")
    if (-not $featuresMatch.Success) {
        throw "Could not find [features] in $CargoTomlPath"
    }

    $escapedFeatureName = [regex]::Escape($FeatureName)
    $featureMatch = [regex]::Match(
        $featuresMatch.Groups["features"].Value,
        "(?ms)^$escapedFeatureName\s*=\s*(?<value>\[[^\]]*\])"
    )

    if (-not $featureMatch.Success) {
        throw "Could not find feature '$FeatureName' in $CargoTomlPath"
    }

    return [regex]::Matches($featureMatch.Groups["value"].Value, '"(?<value>[^"]+)"') |
        ForEach-Object { $_.Groups["value"].Value }
}

function Get-CargoPackageStringValue {
    param(
        [string]$CargoTomlPath,
        [string]$Key
    )

    $cargoToml = Get-Content -Raw -Encoding UTF8 $CargoTomlPath
    $packageMatch = [regex]::Match($cargoToml, "(?ms)^\[package\]\s*(?<package>.*?)(?=^\[|\z)")
    if (-not $packageMatch.Success) {
        throw "Could not find [package] in $CargoTomlPath"
    }

    $escapedKey = [regex]::Escape($Key)
    $keyMatch = [regex]::Match(
        $packageMatch.Groups["package"].Value,
        "(?m)^$escapedKey\s*=\s*`"(?<value>[^`"]+)`""
    )

    if (-not $keyMatch.Success) {
        return $null
    }

    return $keyMatch.Groups["value"].Value
}

function Get-CargoPackageName {
    param([string]$CargoTomlPath)

    $packageName = Get-CargoPackageStringValue -CargoTomlPath $CargoTomlPath -Key "name"
    if ([string]::IsNullOrWhiteSpace($packageName)) {
        throw "Could not find package name in $CargoTomlPath"
    }

    return $packageName
}

function Get-RustEnumAsStrTokens {
    param(
        [string]$RustPath,
        [string]$EnumName
    )

    $source = Get-Content -Raw -Encoding UTF8 $RustPath
    $escapedEnumName = [regex]::Escape($EnumName)
    $implMatch = [regex]::Match($source, "(?ms)impl\s+$escapedEnumName\s*\{(?<body>.*?)^\}")
    if (-not $implMatch.Success) {
        throw "Could not find impl block for enum '$EnumName' in $RustPath"
    }

    $asStrMatch = [regex]::Match(
        $implMatch.Groups["body"].Value,
        "(?ms)pub\s+(?:const\s+)?fn\s+as_str\s*\([^)]*\)\s*->\s*&'static\s+str\s*\{(?<body>.*?)^\s*\}"
    )
    if (-not $asStrMatch.Success) {
        throw "Could not find as_str implementation for enum '$EnumName' in $RustPath"
    }

    return [regex]::Matches($asStrMatch.Groups["body"].Value, 'Self::[A-Za-z0-9_]+\s*=>\s*"(?<token>[^"]+)"') |
        ForEach-Object { $_.Groups["token"].Value }
}

function Get-RustMatchStringArms {
    param(
        [string]$RustPath,
        [string]$FunctionName
    )

    $source = Get-Content -Raw -Encoding UTF8 $RustPath
    $escapedFunctionName = [regex]::Escape($FunctionName)
    $functionMatch = [regex]::Match(
        $source,
        "(?ms)fn\s+$escapedFunctionName\s*\([^)]*\)\s*(?:->\s*[^{]+)?\{(?<body>.*?)^\}"
    )
    if (-not $functionMatch.Success) {
        throw "Could not find function '$FunctionName' in $RustPath"
    }

    return [regex]::Matches($functionMatch.Groups["body"].Value, '"(?<token>[^"]+)"\s*=>') |
        ForEach-Object { $_.Groups["token"].Value }
}

function Get-RustMatchArmBody {
    param(
        [string]$RustPath,
        [string]$FunctionName,
        [string]$ArmLiteral
    )

    $source = Get-Content -Raw -Encoding UTF8 $RustPath
    $escapedFunctionName = [regex]::Escape($FunctionName)
    $functionMatch = [regex]::Match(
        $source,
        "(?ms)fn\s+$escapedFunctionName\s*\([^)]*\)\s*(?:->\s*[^{]+)?\{(?<body>.*?)^\}"
    )
    if (-not $functionMatch.Success) {
        throw "Could not find function '$FunctionName' in $RustPath"
    }

    $escapedArmLiteral = [regex]::Escape($ArmLiteral)
    $armMatch = [regex]::Match(
        $functionMatch.Groups["body"].Value,
        "(?ms)`"$escapedArmLiteral`"\s*=>\s*(?<body>.*?)(?=^\s*(?:`"[^`"]+`"|_)\s*=>|^\s*\})"
    )
    if (-not $armMatch.Success) {
        throw "Could not find match arm '$ArmLiteral' in function '$FunctionName' in $RustPath"
    }

    return $armMatch.Groups["body"].Value
}

Describe "Coordinator Cargo target hard cutover" {
    It "contains no repo-local shared-slot implementation" {
    $source = Get-Content -Raw -Encoding UTF8 $script:ValidateMatrixScript

        $source | Should Not Match "Resolve-SharedCargoTarget"
        $source | Should Not Match "codex-shared-[ab]"
        $source | Should Not Match "target.manual-check"
        $source | Should Match "Resolve-CoordinatorCargoTarget"
    }

    It "does not acquire a reusable pool when toolchain identity cannot be established" {
        $client = Join-Path $script:ValidateMatrixTestRepoRoot "tools\zircon-session.ps1"
        $beforeRaw = & $client -Command cargo -RepoRoot $script:ValidateMatrixTestRepoRoot -Json list
        $beforeIds = @((($beforeRaw -join "`n") | ConvertFrom-Json).jobs | ForEach-Object job_id)
        $result = Invoke-ValidateMatrixCliWithoutCargo -Arguments @("-SkipTest")

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "rustc"
        $raw = & $client -Command cargo -RepoRoot $script:ValidateMatrixTestRepoRoot -Json list
        $jobs = (($raw -join "`n") | ConvertFrom-Json).jobs
        $ownerId = Resolve-ValidationSessionId -RepoRoot $script:ValidateMatrixTestRepoRoot
        $created = @($jobs | Where-Object {
            $beforeIds -notcontains $_.job_id -and $_.session_id -eq $ownerId
        })
        $created.Count | Should Be 0
    }
}

Describe "Coordinator supervisor role" {
    It "marks the long-lived validation wrapper as a supervisor when it starts its Cargo job" {
        $source = Get-Content -Raw -Encoding UTF8 $script:ValidateMatrixScript

        $startMatch = [regex]::Match(
            $source,
            '(?s)function Start-CoordinatorCargoTarget.*?\n\}'
        )

        $startMatch.Success | Should Be $true
        $startMatch.Value | Should Match '"--supervisor"'
        $startMatch.Value | Should Match '"cargo", "start"'
    }
}

Describe "Coordinator pre-start failure cleanup" {
    It "preserves an invalid coordinator target error without releasing the unstarted job" {
        $script:PreStartCoordinatorCalls = [System.Collections.Generic.List[string]]::new()
        Mock Resolve-ValidationSessionId { return "validate-matrix:test" }
        Mock New-CargoCompatibilityJson { return "{}" }
        Mock Invoke-SessionCoordinatorJson {
            $command = $Arguments -join " "
            $script:PreStartCoordinatorCalls.Add($command)
            if ($command -match '^session register') {
                return [pscustomobject]@{
                    session = [pscustomobject]@{ session_id = "validate-matrix:test" }
                }
            }
            return [pscustomobject]@{
                job = [pscustomobject]@{
                    job_id = "invalid-target-job"
                    target_dir = "C:\cargo-targets\zircon-engine\invalid-target-job"
                    dry_run = $false
                }
            }
        }

        $failure = $null
        try {
            Resolve-CoordinatorCargoTarget `
                -RepoRoot $script:ValidateMatrixTestRepoRoot `
                -LaneKind "test" `
                -WorkspaceManifest "Cargo.toml" | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match "must physically resolve under D:, E:, or F:"
        $script:PreStartCoordinatorCalls.Count | Should Be 2
        ($script:PreStartCoordinatorCalls -join "`n") | Should Not Match "cargo release"
    }

    It "preserves the primary error without releasing the current wrapper before cargo start" {
        Mock Resolve-CoordinatorCargoTarget {
            return [pscustomobject]@{
                SelectionMode     = "managed"
                JobId             = "pre-start-job"
                TargetDir         = "E:\cargo-targets\zircon-engine\pre-start-job"
                AbsoluteTargetDir = "E:\cargo-targets\zircon-engine\pre-start-job"
                Reason            = "coordinator managed test lane"
                OwnerId           = "validate-matrix:test"
                DryRun            = $false
            }
        }
        Mock Push-ManagedCargoEnvironment {
            throw "primary pre-start failure"
        }
        Mock Invoke-SessionCoordinatorJson {
            throw "cargo_process_tree_alive cleanup failure"
        }

        $failure = $null
        try {
            Invoke-ValidateMatrixMain | Out-Null
        }
        catch {
            $failure = $_
        }

        $failure | Should Not BeNullOrEmpty
        $failure.Exception.Message | Should Match "primary pre-start failure"
        $failure.Exception.Message | Should Not Match "cleanup failure"
        Assert-MockCalled Invoke-SessionCoordinatorJson -Times 0 -ParameterFilter {
            $Arguments[0] -eq "cargo" -and $Arguments[1] -eq "release"
        }
    }

    It "finishes and releases a coordinator job after cargo start" {
        $script:CoordinatorCompletionCalls = [System.Collections.Generic.List[string]]::new()
        Mock Invoke-SessionCoordinatorJson {
            $script:CoordinatorCompletionCalls.Add(($Arguments -join " "))
            return [pscustomobject]@{}
        }
        $target = [pscustomobject]@{
            JobId  = "started-job"
            OwnerId = "validate-matrix:test"
            DryRun = $false
        }

        Complete-CoordinatorCargoTarget `
            -RepoRoot $script:ValidateMatrixTestRepoRoot `
            -ResolvedTarget $target `
            -ExitCode 1 `
            -Started

        $script:CoordinatorCompletionCalls.Count | Should Be 2
        $script:CoordinatorCompletionCalls[0] | Should Match "cargo finish started-job"
        $script:CoordinatorCompletionCalls[1] | Should Match "cargo release started-job"
    }
}

Describe "Get-PrebuildCleanupDecision" {
    It "requires cleanup when free space is at or below the 50 GB threshold" {
        $decision = Get-PrebuildCleanupDecision -FreeBytes 50GB -ThresholdBytes 50GB

        $decision.RequiresCleanup | Should Be $true
    }

    It "skips cleanup when free space is above the threshold" {
        $decision = Get-PrebuildCleanupDecision -FreeBytes 51GB -ThresholdBytes 50GB

        $decision.RequiresCleanup | Should Be $false
    }
}

Describe "Cargo compatibility identity" {
    It "includes platform toolchain architecture workspace and canonical build configuration" {
        $previousToolchain = $env:RUSTUP_TOOLCHAIN
        $previousTarget = $env:CARGO_BUILD_TARGET
        try {
            $env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
            $env:CARGO_BUILD_TARGET = "x86_64-pc-windows-msvc"

            $compatibility = New-CargoCompatibilityJson `
                -ResolvedRepoRoot $script:ValidateMatrixTestRepoRoot `
                -DryRunMode | ConvertFrom-Json

            $compatibility.platform | Should Be "windows"
            $compatibility.toolchain | Should Be "stable-x86_64-pc-windows-msvc"
            $compatibility.target_architecture | Should Be "x86_64-pc-windows-msvc"
            $compatibility.workspace | Should Be "Cargo.toml"
            $compatibility.build_config | Should Match "rustflags"
            $compatibility.build_config | Should Match "cargo_incremental"
        } finally {
            $env:RUSTUP_TOOLCHAIN = $previousToolchain
            $env:CARGO_BUILD_TARGET = $previousTarget
        }
    }

    It "passes the complete document through the coordinator acquire command" {
        $source = Get-Content -Raw -Encoding UTF8 $script:ValidateMatrixScript

        $source | Should Match "--compatibility-json"
        $source | Should Match "New-CargoCompatibilityJson"
        $source | Should Not Match "--reuse-key"
    }

    It "uses an explicit subworkspace manifest as the compatibility workspace" {
        $compatibility = New-CargoCompatibilityJson `
            -ResolvedRepoRoot $script:ValidateMatrixTestRepoRoot `
            -WorkspaceManifest "zircon_plugins/Cargo.toml" `
            -DryRunMode | ConvertFrom-Json

        $compatibility.workspace | Should Be "zircon_plugins/Cargo.toml"
    }

    It "keeps default development release and profiling Cargo profiles in distinct compatibility identities" {
        $development = New-CargoCompatibilityJson `
            -ResolvedRepoRoot $script:ValidateMatrixTestRepoRoot `
            -DryRunMode | ConvertFrom-Json
        $explicitDevelopment = New-CargoCompatibilityJson `
            -ResolvedRepoRoot $script:ValidateMatrixTestRepoRoot `
            -CargoProfile "development" `
            -DryRunMode | ConvertFrom-Json
        $release = New-CargoCompatibilityJson `
            -ResolvedRepoRoot $script:ValidateMatrixTestRepoRoot `
            -CargoProfile "release" `
            -DryRunMode | ConvertFrom-Json
        $profiling = New-CargoCompatibilityJson `
            -ResolvedRepoRoot $script:ValidateMatrixTestRepoRoot `
            -CargoProfile "profiling" `
            -DryRunMode | ConvertFrom-Json

        $development.build_config | Should Match '"cargo_profile":"development"'
        $explicitDevelopment.build_config | Should Be $development.build_config
        $release.build_config | Should Match '"cargo_profile":"release"'
        $profiling.build_config | Should Match '"cargo_profile":"profiling"'
        $release.build_config | Should Not Be $development.build_config
        $profiling.build_config | Should Not Be $development.build_config
        $profiling.build_config | Should Not Be $release.build_config
    }

    It "resolves a nested manifest relative to its Cargo working directory" {
        $workspace = Resolve-WorkspaceManifest `
            -RepoRoot $script:ValidateMatrixTestRepoRoot `
            -RequestedManifestPath "zircon_plugins/Cargo.toml"
        $workspace.RelativePath | Should Be "zircon_plugins/Cargo.toml"
        $workspace.Directory | Should Match "zircon_plugins$"
        $workspace.InvocationManifestPath | Should Be "Cargo.toml"

        $nestedArgs = Get-CargoArgs `
            -Subcommand "build" `
            -ResolvedTargetDir "E:\cargo-targets\pester-manifest" `
            -WorkspaceManifest $workspace.InvocationManifestPath
        ($nestedArgs -join " ") | Should Not Match "--manifest-path"
        ($nestedArgs -join " ") | Should Not Match "zircon_plugins/Cargo.toml"

        $rootArgs = Get-CargoArgs `
            -Subcommand "build" `
            -ResolvedTargetDir "E:\cargo-targets\pester-root" `
            -WorkspaceManifest "Cargo.toml"
        ($rootArgs -join " ") | Should Not Match "--manifest-path"
    }
}

Describe "Cargo profiling workspace contract" {
    It "keeps root and plugin workspaces on the same symbolized profiling profile" {
        $expectedProfile = '(?ms)^\[profile\.profiling\]\s*$.*?^inherits\s*=\s*"release"\s*$.*?^debug\s*=\s*true\s*$.*?^strip\s*=\s*false\s*$'

        foreach ($manifest in @("Cargo.toml", "zircon_plugins/Cargo.toml")) {
            $content = Get-Content -Raw -Encoding UTF8 `
                (Join-Path $script:ValidateMatrixTestRepoRoot $manifest)
            $content | Should Match $expectedProfile
        }
    }
}

Describe "Validator path resolution" {
    It "resolves an aliased repository manifest to its physical workspace directory" {
        $physicalRoot = Join-Path $TestDrive "physical-workspace"
        $aliasRoot = Join-Path $TestDrive "workspace-alias"
        [System.IO.Directory]::CreateDirectory($physicalRoot) | Out-Null
        Set-Content -LiteralPath (Join-Path $physicalRoot "Cargo.toml") -Value "[workspace]" -NoNewline
        New-Item -ItemType Junction -Path $aliasRoot -Target $physicalRoot | Out-Null

        $workspace = Resolve-WorkspaceManifest -RepoRoot $aliasRoot -RequestedManifestPath "Cargo.toml"
        $physical = Resolve-ZirconWindowsPath -Path $physicalRoot

        $workspace.RelativePath | Should Be "Cargo.toml"
        $workspace.Directory | Should Be $physical.DisplayExistingPath
    }

    It "resolves an aliased target directory through its physical ancestor" {
        $physicalRoot = Join-Path $TestDrive "physical-target-root"
        $aliasRoot = Join-Path $TestDrive "target-root-alias"
        [System.IO.Directory]::CreateDirectory($physicalRoot) | Out-Null
        New-Item -ItemType Junction -Path $aliasRoot -Target $physicalRoot | Out-Null
        $requestedTarget = Join-Path $aliasRoot "uncreated-target"

        $target = Resolve-AbsoluteTargetDir `
            -RepoRoot $script:ValidateMatrixTestRepoRoot `
            -CliTargetDir $requestedTarget
        $physical = Resolve-ZirconWindowsPath -Path $requestedTarget

        $target | Should Be $physical.DisplayPath
    }

    It "rejects drive-relative target directories before applying a repository root" {
        { Resolve-AbsoluteTargetDir -RepoRoot $script:ValidateMatrixTestRepoRoot -CliTargetDir "C:ambiguous-target" } |
            Should Throw "Windows paths must be drive-rooted, not drive-relative: 'C:ambiguous-target'."
    }

    It "discovers a repository through an alias as its physical identity" {
        $aliasRoot = Join-Path $TestDrive "repository-alias"
        New-Item -ItemType Junction -Path $aliasRoot -Target $script:ValidateMatrixTestRepoRoot | Out-Null

        $found = Find-RepoRoot (Join-Path $aliasRoot ".codex\\skills\\zircon-dev\\scripts")
        $physical = Resolve-ZirconWindowsPath -Path $script:ValidateMatrixTestRepoRoot

        $found | Should Be $physical.DisplayExistingPath
    }
}

Describe "Ignored test Cargo arguments" {
    It "appends the ignored harness switch after all Cargo arguments" {
        $previousPackage = $script:Package
        $previousLibTests = $script:LibTests
        $previousTestFilter = $script:TestFilter
        $previousIgnoredTests = $script:IgnoredTests
        try {
            $script:Package = "zircon_runtime"
            $script:LibTests = $true
            $script:TestFilter = "export_visual_evidence"
            $script:IgnoredTests = $true

            $arguments = @(Get-CargoArgs `
                -Subcommand "test" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\test" `
                -WorkspaceManifest "Cargo.toml")

            ($arguments -join " ") | Should Be "test -p zircon_runtime --locked --lib export_visual_evidence --target-dir D:\cargo-targets\zircon-engine\pool\test -- --ignored"
        }
        finally {
            $script:Package = $previousPackage
            $script:LibTests = $previousLibTests
            $script:TestFilter = $previousTestFilter
            $script:IgnoredTests = $previousIgnoredTests
        }
    }
}

Describe "Product binary Cargo arguments" {
    It "restricts a package build to the explicitly selected binary" {
        $previousPackage = $script:Package
        $previousBin = Get-Variable -Name Bin -Scope Script -ErrorAction SilentlyContinue
        try {
            $script:Package = "zircon_app"
            $script:Bin = "zircon_runtime"

            $arguments = @(Get-CargoArgs `
                -Subcommand "build" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\product-bin" `
                -WorkspaceManifest "Cargo.toml")

            ($arguments -join " ") | Should Be "build -p zircon_app --bin zircon_runtime --locked --target-dir D:\cargo-targets\zircon-engine\pool\product-bin"
        }
        finally {
            $script:Package = $previousPackage
            if ($null -eq $previousBin) {
                Remove-Variable -Name Bin -Scope Script -ErrorAction SilentlyContinue
            }
            else {
                $script:Bin = $previousBin.Value
            }
        }
    }

    It "maps every Cargo profile exactly once for every compiling command builder" {
        $previousPackage = $script:Package
        try {
            $script:Package = "zircon_runtime"
            $development = @(Get-CargoArgs `
                -Subcommand "build" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\development" `
                -WorkspaceManifest "Cargo.toml")
            $explicitDevelopment = @(Get-CargoArgs `
                -Subcommand "build" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\development" `
                -WorkspaceManifest "Cargo.toml" `
                -CargoProfile "development")
            $developmentTest = @(Get-CargoArgs `
                -Subcommand "test" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\development-test" `
                -WorkspaceManifest "Cargo.toml")
            $explicitDevelopmentTest = @(Get-CargoArgs `
                -Subcommand "test" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\development-test" `
                -WorkspaceManifest "Cargo.toml" `
                -CargoProfile "development")
            $developmentExportContract = @(Get-ExportPlatformContractArgs `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\development-export")
            $explicitDevelopmentExportContract = @(Get-ExportPlatformContractArgs `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\development-export" `
                -CargoProfile "development")
            $developmentProfileContract = @(Get-ProfileFeatureContractArgs `
                -Case ([pscustomobject]@{
                    Package = "zircon_runtime"
                    Features = "target-server"
                    Bin = $null
                }) `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\development-profile")
            $explicitDevelopmentProfileContract = @(Get-ProfileFeatureContractArgs `
                -Case ([pscustomobject]@{
                    Package = "zircon_runtime"
                    Features = "target-server"
                    Bin = $null
                }) `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\development-profile" `
                -CargoProfile "development")
            $releaseBuild = @(Get-CargoArgs `
                -Subcommand "build" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\release-build" `
                -WorkspaceManifest "Cargo.toml" `
                -CargoProfile "release")
            $releaseTest = @(Get-CargoArgs `
                -Subcommand "test" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\release-test" `
                -WorkspaceManifest "Cargo.toml" `
                -CargoProfile "release")
            $releaseExportContract = @(Get-ExportPlatformContractArgs `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\release-export" `
                -CargoProfile "release")
            $releaseProfileContract = @(Get-ProfileFeatureContractArgs `
                -Case ([pscustomobject]@{
                    Package = "zircon_runtime"
                    Features = "target-server"
                    Bin = $null
                }) `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\release-profile" `
                -CargoProfile "release")
            $profilingBuild = @(Get-CargoArgs `
                -Subcommand "build" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\profiling-build" `
                -WorkspaceManifest "Cargo.toml" `
                -CargoProfile "profiling")
            $profilingTest = @(Get-CargoArgs `
                -Subcommand "test" `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\profiling-test" `
                -WorkspaceManifest "Cargo.toml" `
                -CargoProfile "profiling")
            $profilingExportContract = @(Get-ExportPlatformContractArgs `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\profiling-export" `
                -CargoProfile "profiling")
            $profilingProfileContract = @(Get-ProfileFeatureContractArgs `
                -Case ([pscustomobject]@{
                    Package = "zircon_runtime"
                    Features = "target-server"
                    Bin = $null
                }) `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\profiling-profile" `
                -CargoProfile "profiling")
            $cleanup = @(Get-CargoCleanArgs `
                -ResolvedTargetDir "D:\cargo-targets\zircon-engine\pool\release-clean" `
                -WorkspaceManifest "Cargo.toml")

            (@($development | Where-Object { $_ -eq "--release" })).Count | Should Be 0
            (@($development | Where-Object { $_ -eq "--profile" })).Count | Should Be 0
            ($explicitDevelopment -join " ") | Should Be ($development -join " ")
            ($explicitDevelopmentTest -join " ") | Should Be ($developmentTest -join " ")
            ($explicitDevelopmentExportContract -join " ") | Should Be ($developmentExportContract -join " ")
            ($explicitDevelopmentProfileContract -join " ") | Should Be ($developmentProfileContract -join " ")
            (@($releaseBuild | Where-Object { $_ -eq "--release" })).Count | Should Be 1
            (@($releaseTest | Where-Object { $_ -eq "--release" })).Count | Should Be 1
            (@($releaseExportContract | Where-Object { $_ -eq "--release" })).Count | Should Be 1
            (@($releaseProfileContract | Where-Object { $_ -eq "--release" })).Count | Should Be 1
            ($profilingBuild -join " ") | Should Match '--profile profiling'
            ($profilingTest -join " ") | Should Match '--profile profiling'
            ($profilingExportContract -join " ") | Should Match '--profile profiling'
            ($profilingProfileContract -join " ") | Should Match '--profile profiling'
            (@($profilingBuild | Where-Object { $_ -eq "--profile" })).Count | Should Be 1
            (@($profilingTest | Where-Object { $_ -eq "--profile" })).Count | Should Be 1
            (@($profilingExportContract | Where-Object { $_ -eq "--profile" })).Count | Should Be 1
            (@($profilingProfileContract | Where-Object { $_ -eq "--profile" })).Count | Should Be 1
            (@($profilingBuild | Where-Object { $_ -eq "--release" })).Count | Should Be 0
            (@($cleanup | Where-Object { $_ -eq "--release" })).Count | Should Be 0
            (@($cleanup | Where-Object { $_ -eq "--profile" })).Count | Should Be 0
        }
        finally {
            $script:Package = $previousPackage
        }
    }
}

Describe "Published artifact path resolution" {
    It "keeps the final physical path for artifact I/O while checking policy by display path" {
        $targetDirectory = Join-Path $TestDrive "artifact-output-target"
        $junctionDirectory = Join-Path $TestDrive "artifact-output-link"
        [System.IO.Directory]::CreateDirectory($targetDirectory) | Out-Null
        New-Item -ItemType Junction -Path $junctionDirectory -Target $targetDirectory | Out-Null
        $requestedPath = Join-Path $junctionDirectory "published"

        $resolved = Assert-ArtifactOutputDirectory -Path $requestedPath
        $resolution = Resolve-ZirconWindowsPath -Path $requestedPath

        $resolved | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Be (Join-Path $targetDirectory "published")
    }

    It "allows the dedicated MVP product-input root only when explicitly requested" {
        $requestedPath = "D:\ZirconBuilds\mvp-product-inputs-contract-$([guid]::NewGuid().ToString('N'))"

        $resolved = Assert-ArtifactOutputDirectory -Path $requestedPath -MvpProductInputArtifactOutput
        $resolution = Resolve-ZirconWindowsPath -Path $requestedPath

        $resolved | Should Be $resolution.OperationalPath
        $resolution.DisplayPath | Should Match '^D:\\ZirconBuilds\\mvp-product-inputs-'
    }

    It "does not allow the MVP product-input exception outside its physical root" {
        $message = $null
        try {
            Assert-ArtifactOutputDirectory -Path 'D:\ZirconBuilds\unscoped-product-artifacts' -MvpProductInputArtifactOutput
        }
        catch {
            $message = $_.Exception.Message
        }

        $message | Should Match 'MVP product input artifact output must resolve under'
    }
}

Describe "Published artifact hashing" {
    It "uses the managed SHA-256 implementation for artifact bytes" {
        $artifactPath = Join-Path $TestDrive "managed-hash-empty.bin"
        [System.IO.File]::WriteAllBytes($artifactPath, [byte[]]@())

        Get-ManagedFileSha256 -Path $artifactPath | Should Be "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
    }
}

Describe "Release artifact publication" {
    It "publishes omitted-profile artifacts from the development debug directory" {
        $targetDirectory = Join-Path $TestDrive "development-target"
        $debugDirectory = Join-Path $targetDirectory "debug"
        $artifactOutputDirectory = Join-Path $TestDrive "development-published"
        $explicitArtifactOutputDirectory = Join-Path $TestDrive "explicit-development-published"
        [System.IO.Directory]::CreateDirectory($debugDirectory) | Out-Null
        [System.IO.File]::WriteAllBytes(
            (Join-Path $debugDirectory "zircon_runtime.exe"),
            [byte[]](0, 1, 2, 3)
        )

        $published = @(Publish-BuildArtifacts `
            -TargetDirectory $targetDirectory `
            -ArtifactOutputDirectory $artifactOutputDirectory `
            -ArtifactName @("zircon_runtime.exe"))
        $explicitPublished = @(Publish-BuildArtifacts `
            -TargetDirectory $targetDirectory `
            -ArtifactOutputDirectory $explicitArtifactOutputDirectory `
            -ArtifactName @("zircon_runtime.exe") `
            -CargoProfile "development")

        $published.Count | Should Be 1
        $explicitPublished.Count | Should Be 1
        [Convert]::ToBase64String([System.IO.File]::ReadAllBytes($published[0].Path)) |
            Should Be ([Convert]::ToBase64String([byte[]](0, 1, 2, 3)))
        [Convert]::ToBase64String([System.IO.File]::ReadAllBytes($explicitPublished[0].Path)) |
            Should Be ([Convert]::ToBase64String([byte[]](0, 1, 2, 3)))
    }

    It "publishes release artifacts from the release profile directory" {
        $targetDirectory = Join-Path $TestDrive "release-target"
        $releaseDirectory = Join-Path $targetDirectory "release"
        $artifactOutputDirectory = Join-Path $TestDrive "release-published"
        [System.IO.Directory]::CreateDirectory($releaseDirectory) | Out-Null
        [System.IO.File]::WriteAllBytes(
            (Join-Path $releaseDirectory "zircon_runtime.exe"),
            [byte[]](1, 2, 3, 4)
        )

        $published = @(Publish-BuildArtifacts `
            -TargetDirectory $targetDirectory `
            -ArtifactOutputDirectory $artifactOutputDirectory `
            -ArtifactName @("zircon_runtime.exe") `
            -CargoProfile "release")

        $published.Count | Should Be 1
        $published[0].Path | Should Be (
            Resolve-ZirconWindowsPath -Path (Join-Path $artifactOutputDirectory "zircon_runtime.exe")
        ).OperationalPath
        [Convert]::ToBase64String([System.IO.File]::ReadAllBytes($published[0].Path)) |
            Should Be ([Convert]::ToBase64String([byte[]](1, 2, 3, 4)))
    }

    It "publishes profiling artifacts from the profiling profile directory" {
        $targetDirectory = Join-Path $TestDrive "profiling-target"
        $profilingDirectory = Join-Path $targetDirectory "profiling"
        $artifactOutputDirectory = Join-Path $TestDrive "profiling-published"
        [System.IO.Directory]::CreateDirectory($profilingDirectory) | Out-Null
        [System.IO.File]::WriteAllBytes(
            (Join-Path $profilingDirectory "zircon_runtime.exe"),
            [byte[]](4, 3, 2, 1)
        )

        $published = @(Publish-BuildArtifacts `
            -TargetDirectory $targetDirectory `
            -ArtifactOutputDirectory $artifactOutputDirectory `
            -ArtifactName @("zircon_runtime.exe") `
            -CargoProfile "profiling")

        $published.Count | Should Be 1
        [Convert]::ToBase64String([System.IO.File]::ReadAllBytes($published[0].Path)) |
            Should Be ([Convert]::ToBase64String([byte[]](4, 3, 2, 1)))
    }
}

Describe "Cargo profile CLI validation" {
    It "rejects an unknown Cargo profile before it acquires a managed lane" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-CargoProfile",
            "benchmark"
        )

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "CargoProfile"
        $result.Output | Should Match "development,release,profiling"
        $result.Output | Should Not Match "Target dir:"
        $result.Output | Should Not Match "cargo "
    }
}

Describe "Validate matrix CLI dry-run parsing" {
    It "allocates a managed drive-root lane for no-stage sanity checks" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Cargo profile: development"
        $result.Output | Should Match "Dry run: on"
        $result.Output | Should Match "Target dir: $($script:ManagedPoolRegex) \(coordinator managed workspace lane\)"
        $result.Output | Should Match "No stages selected"
        $result.Output | Should Not Match "target\\manual-check"
    }

    It "dry-runs the symbolized profiling profile through the managed lane" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-Package",
            "zircon_runtime",
            "-SkipTest",
            "-CargoProfile",
            "profiling"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Cargo profile: profiling"
        $result.Output | Should Match "cargo build -p zircon_runtime --locked --profile profiling --target-dir $($script:ManagedPoolRegex)"
        $result.Output | Should Not Match "--release"
    }

    It "dry-runs a package through an explicit subworkspace manifest" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-Package",
            "zircon_plugin_ai_editor",
            "-ManifestPath",
            "zircon_plugins/Cargo.toml",
            "-SkipTest"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Workspace manifest: zircon_plugins/Cargo.toml"
        $result.Output | Should Match "Cargo working directory: .*zircon_plugins"
        $result.Output | Should Match "cargo build -p zircon_plugin_ai_editor --locked --target-dir $($script:ManagedPoolRegex)"
        $result.Output | Should Not Match "--manifest-path zircon_plugins/Cargo.toml"
    }

    It "runs only an explicitly filtered ignored test through the managed lane" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-Package",
            "zircon_runtime",
            "-LibTests",
            "-TestFilter",
            "export_render17_pfm1_render_graph_cold_warm_wgpu_png",
            "-IgnoredTests"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "cargo test -p zircon_runtime --locked --lib export_render17_pfm1_render_graph_cold_warm_wgpu_png --target-dir $($script:ManagedPoolRegex) -- --ignored"
    }

    It "rejects ignored-test mode without a focused filter" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-Package",
            "zircon_runtime",
            "-LibTests",
            "-IgnoredTests"
        )

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "-IgnoredTests requires -TestFilter"
    }

    It "rejects an explicit repo-local TargetDir instead of bypassing the service" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-TargetDir",
            "target\custom-dry-run"
        )

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "cargo_target_not_managed|D:\\\\cargo-targets"
    }

    It "rejects an inherited target outside managed lane roots" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir `
            -CargoTargetDir "E:\not-approved\unmanaged" `
            -Arguments @("-DryRun", "-SkipBuild", "-SkipTest")

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "cargo_target_not_managed|D:\\\\cargo-targets"
    }
}

Describe "Export platform contract validation" {
    It "keeps the local contract platform list aligned with CI, including headless" {
        $script:ExportContractPlatforms | Should Be @(
            "windows",
            "linux",
            "macos",
            "android",
            "ios",
            "web_gpu",
            "wasm",
            "headless"
        )
    }

    It "keeps the local validator platform list identical to the GitHub Actions matrix" {
        Get-CiExportPlatformMatrix | Should Be $script:ExportContractPlatforms
    }

    It "keeps export platform selectors unique in CI and the local validator" {
        $localUniquePlatforms = @($script:ExportContractPlatforms | Sort-Object -Unique)
        $ciPlatforms = @(Get-CiExportPlatformMatrix)
        $ciUniquePlatforms = @($ciPlatforms | Sort-Object -Unique)

        $localUniquePlatforms.Count | Should Be $script:ExportContractPlatforms.Count
        $ciUniquePlatforms.Count | Should Be $ciPlatforms.Count
    }

    It "builds the focused runtime export policy test command" {
        $args = Get-ExportPlatformContractArgs -ResolvedTargetDir "target/manual-check"

        ($args -join " ") | Should Match "test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy"
        ($args -contains "--locked") | Should Be $true
        ($args -contains "--target-dir") | Should Be $true
        ($args -contains "target/manual-check") | Should Be $true
    }

    It "keeps every workflow export platform renderable by the local validator" {
        foreach ($platform in Get-CiExportPlatformMatrix) {
            $selectedPlatforms = @(Get-SelectedExportContractPlatforms -Platform $platform)
            $args = Get-ExportPlatformContractArgs -ResolvedTargetDir "target/manual-check"
            $command = $args -join " "

            $selectedPlatforms.Count | Should Be 1
            $selectedPlatforms[0] | Should Be $platform
            $command | Should Match "test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy"
            ($args -contains "--locked") | Should Be $true
            ($args -contains "target/manual-check") | Should Be $true
        }
    }

    It "dry-runs the full export platform matrix through the CLI entry point" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunExportPlatformContract"
        )

        $result.ExitCode | Should Be 0
        foreach ($platform in $script:ExportContractPlatforms) {
            $result.Output | Should Match ("Export platform contract \({0}\)" -f [regex]::Escape($platform))
            $result.Output | Should Match ("ZR_EXPORT_CONTRACT_PLATFORM={0}" -f [regex]::Escape($platform))
        }
        ([regex]::Matches($result.Output, "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir $($script:ManagedPoolRegex)")).Count |
            Should Be $script:ExportContractPlatforms.Count
    }

    It "filters export platform contract cases by platform for low-interference validation" {
        $platforms = @(Get-SelectedExportContractPlatforms -Platform "headless")

        $platforms.Count | Should Be 1
        $platforms[0] | Should Be "headless"
    }

    It "dry-runs only the selected export platform through the CLI entry point" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunExportPlatformContract",
            "-ExportContractPlatform",
            "headless"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Export platform contract \(headless\)"
        $result.Output | Should Match "ZR_EXPORT_CONTRACT_PLATFORM=headless"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir $($script:ManagedPoolRegex)"
        $result.Output | Should Not Match "Export platform contract \(windows\)"
        $result.Output | Should Not Match "Export platform contract \(linux\)"
        $result.Output | Should Not Match "Export platform contract \(macos\)"
        $result.Output | Should Not Match "Export platform contract \(android\)"
        $result.Output | Should Not Match "Export platform contract \(ios\)"
        $result.Output | Should Not Match "Export platform contract \(web_gpu\)"
        $result.Output | Should Not Match "Export platform contract \(wasm\)"
    }

    It "dry-runs export platform commands without requiring cargo discovery" {
        $result = Invoke-ValidateMatrixCliWithoutCargo -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunExportPlatformContract",
            "-ExportContractPlatform",
            "headless"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: $($script:ManagedPoolRegex) \(coordinator managed workspace lane\)"
        $result.Output | Should Match "Dry run selected; skipping cargo discovery and target directory cleanup checks"
        $result.Output | Should Match "Export platform contract \(headless\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir $($script:ManagedPoolRegex)"
    }

    It "dry-runs selected export platform commands without inheriting CARGO_TARGET_DIR" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunExportPlatformContract",
            "-ExportContractPlatform",
            "headless"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: $($script:ManagedPoolRegex) \(coordinator managed workspace lane\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir $($script:ManagedPoolRegex)"
    }

    It "dry-runs selected export platform commands with an explicit TargetDir override" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunExportPlatformContract",
            "-ExportContractPlatform",
            "headless",
            "-TargetDir",
            "E:\cargo-targets\pester-custom-dry-run"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: E:\\cargo-targets\\pester-custom-dry-run \(coordinator validated manual target\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir E:\\cargo-targets\\pester-custom-dry-run"
    }

    It "dry-runs selected export platform commands with verbose cargo output" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunExportPlatformContract",
            "-ExportContractPlatform",
            "headless",
            "-VerboseOutput"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: $($script:ManagedPoolRegex) \(coordinator managed workspace lane\)"
        $result.Output | Should Match "Export platform contract \(headless\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose --target-dir $($script:ManagedPoolRegex)"
    }

    It "dry-runs selected export platform commands without locked mode only when requested" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunExportPlatformContract",
            "-ExportContractPlatform",
            "headless",
            "-NoLocked"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Locked mode: off"
        $result.Output | Should Match "Export platform contract \(headless\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --target-dir $($script:ManagedPoolRegex)"
        $result.Output | Should Not Match "--locked"
    }

    It "throws when a requested export contract platform does not exist" {
        $threw = $false

        try {
            Get-SelectedExportContractPlatforms -Platform "console" | Out-Null
        } catch {
            $threw = $true
            $_.Exception.Message | Should Match "Unknown export contract platform"
        }

        $threw | Should Be $true
    }

    It "rejects an unknown export contract platform through the CLI entry point" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunExportPlatformContract",
            "-ExportContractPlatform",
            "console"
        )

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "Unknown export contract platform 'console'"
        $result.Output | Should Match "Known platforms: windows, linux, macos, android, ios, web_gpu, wasm,"
        $result.Output | Should Match "headless"
    }

    It "rejects an export contract platform selector without the export contract stage" {
        $threw = $false

        try {
            & $script:ValidateMatrixScript -DryRun -SkipBuild -SkipTest -ExportContractPlatform headless | Out-Null
        } catch {
            $threw = $true
            $_.Exception.Message | Should Match "ExportContractPlatform requires -RunExportPlatformContract"
        }

        $threw | Should Be $true
    }

    It "rejects an export contract platform selector without the export contract stage through the CLI entry point" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-ExportContractPlatform",
            "headless"
        )

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "ExportContractPlatform requires -RunExportPlatformContract"
    }

    It "keeps CI wiring on the same focused export policy test and environment variable" {
        $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\ci.yml"
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath

        $workflow | Should Match "ZR_EXPORT_CONTRACT_PLATFORM:\s*\$\{\{\s*matrix\.export-platform\s*\}\}"
        $workflow | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose"
    }

    It "keeps export contract workflow scaffolding aligned with the main CI shape" {
        $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\ci.yml"
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath

        Assert-WorkflowHasContractScaffolding `
            -WorkflowRelativePath ".github\workflows\ci.yml" `
            -WorkflowText $workflow
    }

    It "keeps export contract workflow centered on a matrix-driven focused test job" {
        $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\ci.yml"
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath
        $jobMatch = [regex]::Match(
            $workflow,
            "(?ms)^  export-platform-contract:\s*\r?\n(?<body>.*?)(?=^  [A-Za-z0-9_-]+:\s*$|\z)"
        )

        $jobMatch.Success | Should Be $true
        $job = $jobMatch.Groups["body"].Value
        $job | Should Match "name:\s*Export platform contract \(\$\{\{ matrix\.export-platform \}\}\)"
        $job | Should Match "matrix:\s*\r?\n\s*export-platform:\s*\[windows, linux, macos, android, ios, web_gpu, wasm, headless\]"
        $job | Should Match "Check export policy for \$\{\{ matrix\.export-platform \}\}"
        $job | Should Match "ZR_EXPORT_CONTRACT_PLATFORM:\s*\$\{\{\s*matrix\.export-platform\s*\}\}"
        $job | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose"
        $job | Should Not Match "(?m)run:\s*cargo (?:build|test) --workspace"
        $job | Should Not Match "--manifest-path zircon_plugins/Cargo\.toml --workspace"
    }

    It "keeps export contract Linux dependencies aligned with the main CI runtime dependency set" {
        $mainPackages = Get-WorkflowAptPackages `
            -WorkflowRelativePath ".github\workflows\ci.yml" `
            -StepName "Install Linux system dependencies (winit / wgpu / retained UI)"
        $exportPackages = Get-WorkflowAptPackages `
            -WorkflowRelativePath ".github\workflows\ci.yml" `
            -StepName "Install Linux system dependencies (runtime export contract)"

        $exportPackages | Should Be $mainPackages
    }

    It "keeps runtime and export platform target tokens aligned with the validator matrix" {
        $runtimeTargetPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\platform\target.rs"
        $exportTargetPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\core\framework\project\export_profile.rs"

        Get-RustEnumAsStrTokens -RustPath $runtimeTargetPath -EnumName "PlatformTarget" |
            Should Be $script:ExportContractPlatforms
        Get-RustEnumAsStrTokens -RustPath $exportTargetPath -EnumName "ExportTargetPlatform" |
            Should Be $script:ExportContractPlatforms
    }

    It "keeps runtime export policy test CI platform parsing aligned with the validator matrix" {
        $exportPlanTestPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\tests\plugin_extensions\export_build_plan_platform.rs"

        Get-RustMatchStringArms `
            -RustPath $exportPlanTestPath `
            -FunctionName "export_target_platform_from_ci_name" |
            Should Be $script:ExportContractPlatforms
    }
}

Describe "Profile feature contract validation" {
    It "keeps the local profile feature contract list explicit" {
        $script:ProfileFeatureContractCases | ForEach-Object {
            "{0}|{1}|{2}|{3}" -f $_.Label, $_.Package, $_.Features, $_.Bin
        } | Should Be @(
            "zircon_app target-server|zircon_app|target-server|",
            "zircon_app target-client-platform|zircon_app|target-client,platform-winit,input-gamepad,gamepad-gilrs|zircon_runtime",
            "zircon_app target-editor-host|zircon_app|target-editor-host|zircon_editor",
            "zircon_app target-client shader-pbr-viewer|zircon_app|target-client,platform-winit,input-gamepad,gamepad-gilrs|zircon_shader_pbr_viewer",
            "zircon_runtime target-client|zircon_runtime|target-client|",
            "zircon_runtime target-editor-host|zircon_runtime|target-editor-host|",
            "zircon_runtime target-server|zircon_runtime|target-server|"
        )
    }

    It "keeps the local profile feature list identical to the GitHub Actions matrix" {
        $ciCases = Get-CiProfileFeatureMatrix | ForEach-Object {
            "{0}|{1}|{2}|{3}" -f $_.Label, $_.Package, $_.Features, $_.Bin
        }
        $localCases = $script:ProfileFeatureContractCases | ForEach-Object {
            "{0}|{1}|{2}|{3}" -f $_.Label, $_.Package, $_.Features, $_.Bin
        }

        $ciCases | Should Be $localCases
    }

    It "keeps profile feature labels unique in CI and the local validator" {
        $localLabels = @($script:ProfileFeatureContractCases | ForEach-Object { $_.Label })
        $ciLabels = @(Get-CiProfileFeatureMatrix | ForEach-Object { $_.Label })
        $localUniqueLabels = @($localLabels | Sort-Object -Unique)
        $ciUniqueLabels = @($ciLabels | Sort-Object -Unique)

        $localUniqueLabels.Count | Should Be $localLabels.Count
        $ciUniqueLabels.Count | Should Be $ciLabels.Count
    }

    It "keeps every workflow profile feature case renderable by the local validator" {
        foreach ($case in Get-CiProfileFeatureMatrix) {
            $localCase = @(Get-SelectedProfileFeatureContractCases -Label $case.Label)
            $args = Get-ProfileFeatureContractArgs -Case $localCase[0] -ResolvedTargetDir "target/manual-check"
            $command = $args -join " "

            $binarySelector = if ([string]::IsNullOrWhiteSpace($case.Bin)) {
                ""
            } else {
                " --bin {0}" -f [regex]::Escape($case.Bin)
            }
            $command | Should Match ("check -p {0}{1} --no-default-features --features {2}" -f $case.Package, $binarySelector, [regex]::Escape($case.Features))
            ($args -contains "--locked") | Should Be $true
            ($args -contains "target/manual-check") | Should Be $true
        }
    }

    It "keeps every workflow profile feature backed by its package manifest" {
        foreach ($case in Get-CiProfileFeatureMatrix) {
            $cargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "$($case.Package)\Cargo.toml"
            $case.Features.Split(",") | ForEach-Object {
                Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName $_.Trim() | Out-Null
            }
        }
    }

    It "keeps every workflow profile package backed by its package manifest name" {
        foreach ($case in Get-CiProfileFeatureMatrix) {
            $cargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "$($case.Package)\Cargo.toml"
            Get-CargoPackageName -CargoTomlPath $cargoTomlPath | Should Be $case.Package
        }
    }

    It "keeps every selected workflow binary backed by its package manifest and feature gate" {
        foreach ($case in Get-CiProfileFeatureMatrix | Where-Object { -not [string]::IsNullOrWhiteSpace($_.Bin) }) {
            $cargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "$($case.Package)\Cargo.toml"
            $cargoToml = Get-Content -Raw -Encoding UTF8 $cargoTomlPath
            $featureGate = $case.Features.Split(",")[0].Trim()
            $escapedBin = [regex]::Escape($case.Bin)
            $escapedFeatureGate = [regex]::Escape($featureGate)

            $cargoToml | Should Match (
                "(?ms)\[\[bin\]\]\s+name\s*=\s*`"$escapedBin`".*?required-features\s*=\s*\[`"$escapedFeatureGate`"\]"
            )
        }
    }

    It "builds focused no-default-features cargo check commands" {
        $case = $script:ProfileFeatureContractCases[0]
        $args = Get-ProfileFeatureContractArgs -Case $case -ResolvedTargetDir "target/manual-check"

        ($args -join " ") | Should Match "check -p zircon_app --no-default-features --features target-server"
        ($args -contains "--locked") | Should Be $true
        ($args -contains "--target-dir") | Should Be $true
        ($args -contains "target/manual-check") | Should Be $true
    }

    It "dry-runs the full profile feature matrix through the CLI entry point" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunProfileFeatureContract"
        )

        $result.ExitCode | Should Be 0
        foreach ($case in $script:ProfileFeatureContractCases) {
            $binarySelector = if ($case.PSObject.Properties["Bin"] -and -not [string]::IsNullOrWhiteSpace($case.Bin)) {
                " --bin {0}" -f [regex]::Escape($case.Bin)
            } else {
                ""
            }
            $result.Output | Should Match ("Profile feature contract \({0}\)" -f [regex]::Escape($case.Label))
            $result.Output | Should Match (
                "cargo check -p {0}{1} --no-default-features --features {2} --locked --target-dir {3}" -f
                [regex]::Escape($case.Package),
                $binarySelector,
                [regex]::Escape($case.Features),
                $script:ManagedPoolRegex
            )
        }
        ([regex]::Matches($result.Output, "cargo check -p .*?(?: --bin [^ ]+)? --no-default-features --features .* --locked --target-dir $($script:ManagedPoolRegex)")).Count |
            Should Be $script:ProfileFeatureContractCases.Count
    }

    It "filters profile feature contract cases by label for low-interference validation" {
        $cases = @(Get-SelectedProfileFeatureContractCases -Label "zircon_runtime target-server")

        $cases.Count | Should Be 1
        $cases[0].Package | Should Be "zircon_runtime"
        $cases[0].Features | Should Be "target-server"
    }

    It "dry-runs only the selected profile feature case through the CLI entry point" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunProfileFeatureContract",
            "-ProfileFeatureContractLabel",
            "zircon_runtime target-server"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Profile feature contract \(zircon_runtime target-server\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --target-dir $($script:ManagedPoolRegex)"
        $result.Output | Should Not Match "Profile feature contract \(zircon_app target-server\)"
        $result.Output | Should Not Match "Profile feature contract \(zircon_app target-client-platform\)"
        $result.Output | Should Not Match "Profile feature contract \(zircon_runtime target-client\)"
        $result.Output | Should Not Match "Profile feature contract \(zircon_runtime target-editor-host\)"
    }

    It "dry-runs profile feature commands without requiring cargo discovery" {
        $result = Invoke-ValidateMatrixCliWithoutCargo -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunProfileFeatureContract",
            "-ProfileFeatureContractLabel",
            "zircon_runtime target-server"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: $($script:ManagedPoolRegex) \(coordinator managed workspace lane\)"
        $result.Output | Should Match "Dry run selected; skipping cargo discovery and target directory cleanup checks"
        $result.Output | Should Match "Profile feature contract \(zircon_runtime target-server\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --target-dir $($script:ManagedPoolRegex)"
    }

    It "dry-runs selected profile feature commands without inheriting CARGO_TARGET_DIR" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunProfileFeatureContract",
            "-ProfileFeatureContractLabel",
            "zircon_runtime target-server"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: $($script:ManagedPoolRegex) \(coordinator managed workspace lane\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --target-dir $($script:ManagedPoolRegex)"
    }

    It "dry-runs selected profile feature commands with an explicit TargetDir override" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunProfileFeatureContract",
            "-ProfileFeatureContractLabel",
            "zircon_runtime target-server",
            "-TargetDir",
            "E:\cargo-targets\pester-custom-dry-run"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: E:\\cargo-targets\\pester-custom-dry-run \(coordinator validated manual target\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --target-dir E:\\cargo-targets\\pester-custom-dry-run"
    }

    It "dry-runs selected profile feature commands with verbose cargo output" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunProfileFeatureContract",
            "-ProfileFeatureContractLabel",
            "zircon_runtime target-server",
            "-VerboseOutput"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: $($script:ManagedPoolRegex) \(coordinator managed workspace lane\)"
        $result.Output | Should Match "Profile feature contract \(zircon_runtime target-server\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --verbose --target-dir $($script:ManagedPoolRegex)"
    }

    It "dry-runs selected profile feature commands without locked mode only when requested" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunProfileFeatureContract",
            "-ProfileFeatureContractLabel",
            "zircon_runtime target-server",
            "-NoLocked"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Locked mode: off"
        $result.Output | Should Match "Profile feature contract \(zircon_runtime target-server\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --target-dir $($script:ManagedPoolRegex)"
        $result.Output | Should Not Match "--locked"
    }

    It "throws when a requested profile feature contract label does not exist" {
        $threw = $false

        try {
            Get-SelectedProfileFeatureContractCases -Label "missing profile" | Out-Null
        } catch {
            $threw = $true
            $_.Exception.Message | Should Match "Unknown profile feature contract label"
        }

        $threw | Should Be $true
    }

    It "rejects an unknown profile feature label through the CLI entry point" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-RunProfileFeatureContract",
            "-ProfileFeatureContractLabel",
            "missing profile"
        )

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "Unknown profile feature contract label 'missing profile'"
        $result.Output | Should Match "Known labels: zircon_app target-server, zircon_app"
        $result.Output | Should Match "target-client-platform, zircon_app target-editor-host"
        $result.Output | Should Match "zircon_app target-client shader-pbr-viewer"
        $result.Output | Should Match "zircon_runtime target-client, zircon_runtime target-editor-host, zircon_runtime target-server"
    }

    It "rejects a profile feature selector without the profile feature stage" {
        $threw = $false

        try {
            & $script:ValidateMatrixScript -DryRun -SkipBuild -SkipTest -ProfileFeatureContractLabel "zircon_runtime target-server" | Out-Null
        } catch {
            $threw = $true
            $_.Exception.Message | Should Match "ProfileFeatureContractLabel requires -RunProfileFeatureContract"
        }

        $threw | Should Be $true
    }

    It "rejects a profile feature selector without the profile feature stage through the CLI entry point" {
        $result = Invoke-ValidateMatrixCli -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest",
            "-ProfileFeatureContractLabel",
            "zircon_runtime target-server"
        )

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "ProfileFeatureContractLabel requires -RunProfileFeatureContract"
    }

    It "keeps CI profile feature checks on no-default-features cargo check" {
        $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\profile-feature-contract.yml"
        $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath

        $workflow | Should Match "profile-feature-contract:"
        $workflow | Should Match 'cargo_args=\(check -p "\$\{\{ matrix\.package \}\}"\)'
        $workflow | Should Match "PROFILE_FEATURE_BIN:\s*\$\{\{ matrix\.bin \}\}"
        $workflow | Should Match 'cargo_args\+=\(--bin "\$\{PROFILE_FEATURE_BIN\}"\)'
        $workflow | Should Match 'cargo_args\+=\(--no-default-features --features "\$\{\{ matrix\.features \}\}" --locked --verbose\)'
        $workflow | Should Match 'cargo "\$\{cargo_args\[@\]\}"'
    }

    It "keeps profile contract workflow scaffolding aligned with the main CI shape" {
        $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\profile-feature-contract.yml"
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath

        Assert-WorkflowHasContractScaffolding `
            -WorkflowRelativePath ".github\workflows\profile-feature-contract.yml" `
            -WorkflowText $workflow
    }

    It "keeps profile contract workflow centered on a single matrix-driven job" {
        $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\profile-feature-contract.yml"
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath

        $workflow | Should Match "(?ms)^jobs:\s*\r?\n\s*profile-feature-contract:"
        $jobsBlock = [regex]::Match($workflow, "(?ms)^jobs:[ \t]*\r?\n(?<body>.*)\z").Groups["body"].Value
        ([regex]::Matches($jobsBlock, "(?m)^\s{2}[A-Za-z0-9_-]+:\s*$")).Count | Should Be 1
        $workflow | Should Match "matrix:\s*\r?\n\s*include:"
        $workflow | Should Match "name:\s*Profile feature contract \(\$\{\{ matrix\.label \}\}\)"
        $workflow | Should Match "Check profile features for \$\{\{ matrix\.label \}\}"
        $workflow | Should Match 'cargo_args=\(check -p "\$\{\{ matrix\.package \}\}"\)'
        $workflow | Should Match "PROFILE_FEATURE_BIN:\s*\$\{\{ matrix\.bin \}\}"
        $workflow | Should Match 'cargo_args\+=\(--bin "\$\{PROFILE_FEATURE_BIN\}"\)'
        $workflow | Should Match 'cargo_args\+=\(--no-default-features --features "\$\{\{ matrix\.features \}\}" --locked --verbose\)'
        $workflow | Should Match 'cargo "\$\{cargo_args\[@\]\}"'
        $workflow | Should Not Match "(?m)run:\s*cargo (?:build|test) --workspace"
    }

    It "keeps profile contract Linux dependencies aligned with the main CI runtime dependency set" {
        $mainPackages = Get-WorkflowAptPackages `
            -WorkflowRelativePath ".github\workflows\ci.yml" `
            -StepName "Install Linux system dependencies (winit / wgpu / retained UI)"
        $profilePackages = Get-WorkflowAptPackages `
            -WorkflowRelativePath ".github\workflows\profile-feature-contract.yml" `
            -StepName "Install Linux system dependencies (profile feature contract)"

        $profilePackages | Should Be $mainPackages
    }
}

Describe "Default profile feature topology" {
    It "keeps zircon_app target profiles routed through the expected default and headless feature sets" {
        $cargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_app\Cargo.toml"

        Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName "default" |
            Should Be @("target-client")
        @(Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName "target-client") |
            Should Be @(
                "zircon_runtime/target-client", "ai-contracts", "net-contracts",
                "physics-contracts", "sound-contracts", "animation", "diagnostic-log",
                "dynamic-api", "graphics", "navigation", "script", "text", "ui",
                "default-platform"
            )
        @(Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName "target-editor-host") |
            Should Be @(
                "zircon_runtime/target-editor-host", "ai-contracts", "net-contracts",
                "physics-contracts", "sound-contracts", "animation", "diagnostic-log",
                "dynamic-api", "graphics", "navigation", "script", "text", "ui",
                "dep:zircon_editor", "default-platform"
            )

        $serverFeatures = @(Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName "target-server")
        $serverFeatures | Should Be @("zircon_runtime/target-server", "diagnostic-log", "platform-headless")
        $forbiddenServerFeatures = @(
            "default-platform",
            "platform-window",
            "platform-winit",
            "platform-x11",
            "platform-wayland",
            "input-mouse",
            "input-keyboard",
            "input-touch",
            "input-gamepad",
            "gamepad-gilrs"
        )
        $forbiddenServerFeatures | ForEach-Object {
            ($serverFeatures -contains $_) | Should Be $false
        }
    }

    It "keeps zircon_runtime target profiles routed through default-platform or platform-headless" {
        $cargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\Cargo.toml"

        Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName "default" |
            Should Be @("target-client")
        $expectedInteractiveFeatures = @(
            "core-min", "ai-contracts", "net-contracts", "physics-contracts",
            "sound-contracts", "animation", "diagnostic-log", "dynamic-api",
            "graphics", "navigation", "script", "text", "ui", "default-platform"
        )
        @(Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName "target-client") |
            Should Be $expectedInteractiveFeatures
        @(Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName "target-editor-host") |
            Should Be $expectedInteractiveFeatures

        $serverFeatures = @(Get-CargoFeatureValues -CargoTomlPath $cargoTomlPath -FeatureName "target-server")
        $serverFeatures | Should Be @("core-min", "diagnostic-log", "platform-headless")
        $forbiddenServerFeatures = @(
            "default-platform",
            "platform-window",
            "platform-winit",
            "platform-x11",
            "platform-wayland",
            "input-mouse",
            "input-keyboard",
            "input-touch",
            "input-gamepad",
            "gamepad-gilrs"
        )
        $forbiddenServerFeatures | ForEach-Object {
            ($serverFeatures -contains $_) | Should Be $false
        }
    }

    It "keeps platform-headless feature definitions narrow in app and runtime manifests" {
        $appCargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_app\Cargo.toml"
        $runtimeCargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\Cargo.toml"
        $forbiddenHeadlessFeatures = @(
            "default-platform",
            "platform-window",
            "platform-winit",
            "platform-x11",
            "platform-wayland",
            "input-mouse",
            "input-keyboard",
            "input-touch",
            "input-gamepad",
            "gamepad-gilrs"
        )

        $appHeadlessFeatures = @(
            Get-CargoFeatureValues -CargoTomlPath $appCargoTomlPath -FeatureName "platform-headless"
        )
        $appHeadlessFeatures | Should Be @("zircon_runtime/platform-headless")
        $forbiddenHeadlessFeatures | ForEach-Object {
            ($appHeadlessFeatures -contains $_) | Should Be $false
        }

        $runtimeHeadlessFeatures = @(
            Get-CargoFeatureValues -CargoTomlPath $runtimeCargoTomlPath -FeatureName "platform-headless"
        )
        $runtimeHeadlessFeatures.Count | Should Be 0
    }

    It "keeps the built-in server export profile routed to headless server runtime policy" {
        $defaultProfilePath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\plugin\export_build_plan\default_profile.rs"
        $serverArm = Get-RustMatchArmBody `
            -RustPath $defaultProfilePath `
            -FunctionName "default_profile" `
            -ArmLiteral "server"

        $serverArm | Should Match "RuntimeTargetMode::ServerRuntime"
        $serverArm | Should Match "ExportTargetPlatform::Headless"
        $serverArm | Should Match "RuntimeProfileId::Server"
        $serverArm | Should Not Match "ExportProfile::default"
        $serverArm | Should Not Match "ExportTargetPlatform::Windows"
    }

    It "keeps generated headless server packages on target-server binary entry shape" {
        $cargoManifestTemplatePath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\plugin\export_build_plan\cargo_manifest_template.rs"
        $platformHostFilesPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\plugin\export_build_plan\platform_host_files.rs"
        $mainTemplatePath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\plugin\export_build_plan\main_template.rs"
        $pluginSelectionTemplatePath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\plugin\export_build_plan\plugin_selection_template.rs"
        $cargoManifestTemplate = Get-Content -Raw -Encoding UTF8 $cargoManifestTemplatePath
        $platformHostFiles = Get-Content -Raw -Encoding UTF8 $platformHostFilesPath
        $mainTemplate = Get-Content -Raw -Encoding UTF8 $mainTemplatePath
        $pluginSelectionTemplate = Get-Content -Raw -Encoding UTF8 $pluginSelectionTemplatePath

        $cargoManifestTemplate | Should Match 'features = \[\\"\{target_feature\}\\"\]'
        $cargoManifestTemplate | Should Match 'RuntimeTargetMode::ServerRuntime\s*=>\s*"target-server"'
        $cargoManifestTemplate | Should Match '(?s)ExportPlatformHostKind::Desktop\s*\|\s*crate::plugin::ExportPlatformHostKind::Headless\s*=>\s*\{\}'

        $headlessHostArm = [regex]::Match(
            $platformHostFiles,
            '(?s)ExportPlatformHostKind::Headless\s*=>\s*(?<body>.*?)(?=,\s*crate::plugin::ExportPlatformHostKind::MobileApp)'
        )
        $headlessHostArm.Success | Should Be $true
        $headlessHostArm.Groups["body"].Value | Should Match 'path:\s*"src/main\.rs"'
        $headlessHostArm.Groups["body"].Value | Should Match 'generated headless runtime entry point'
        $headlessHostArm.Groups["body"].Value | Should Match 'main_template\(profile, has_native_dynamic_plugins\)'
        $headlessHostArm.Groups["body"].Value | Should Not Match 'platform/'
        $headlessHostArm.Groups["body"].Value | Should Not Match 'runtime_library_file'

        $pluginSelectionTemplate | Should Match 'RuntimeTargetMode::ServerRuntime\s*=>\s*"EntryProfile::Headless"'
        $mainTemplate | Should Match 'bootstrap_export_runtime'
        $mainTemplate | Should Match 'bootstrap_export_runtime_with_native_plugins_from_export_root'
    }

    It "keeps default-platform window and gamepad capabilities explicit in app and runtime manifests" {
        $appCargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_app\Cargo.toml"
        $runtimeCargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\Cargo.toml"

        Get-CargoFeatureValues -CargoTomlPath $appCargoTomlPath -FeatureName "default-platform" |
            Should Be @(
                "zircon_runtime/default-platform",
                "platform-window",
                "platform-winit",
                "platform-x11",
                "platform-wayland",
                "input-mouse",
                "input-keyboard",
                "input-touch",
                "input-gamepad",
                "gamepad-gilrs"
            )

        Get-CargoFeatureValues -CargoTomlPath $runtimeCargoTomlPath -FeatureName "default-platform" |
            Should Be @(
                "platform-window",
                "platform-winit",
                "platform-x11",
                "platform-wayland",
                "input-mouse",
                "input-keyboard",
                "input-touch",
                "input-gamepad",
                "gamepad-gilrs"
            )
    }

    It "keeps zircon_app build script declaration backed by a local file for CI profile checks" {
        $cargoTomlPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_app\Cargo.toml"
        $buildScript = Get-CargoPackageStringValue -CargoTomlPath $cargoTomlPath -Key "build"

        $buildScript | Should Be "build.rs"
        Test-Path (Join-Path (Split-Path $cargoTomlPath -Parent) $buildScript) | Should Be $true
    }
}

Describe "Platform capability matrix topology" {
    It "keeps capability matrix sources free of panic and placeholder control flow" {
        $matrixRoot = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\platform\capability\matrix"
        $matrixSources = Get-ChildItem -Path $matrixRoot -Filter "*.rs" -Recurse

        foreach ($sourceFile in $matrixSources) {
        $source = Get-Content -Raw -Encoding UTF8 $sourceFile.FullName
            $source | Should Not Match 'panic!\s*\('
            $source | Should Not Match 'todo!\s*\('
            $source | Should Not Match 'unimplemented!\s*\('
        }
    }

    It "keeps server and headless window/input/gamepad capability paths explicit" {
        $windowMatrixPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\platform\capability\matrix\window.rs"
        $inputMatrixPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\platform\capability\matrix\input.rs"
        $gamepadMatrixPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\platform\capability\matrix\gamepad.rs"
        $policyMatrixPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\platform\capability\matrix\policy.rs"
        $windowMatrix = Get-Content -Raw -Encoding UTF8 $windowMatrixPath
        $inputMatrix = Get-Content -Raw -Encoding UTF8 $inputMatrixPath
        $gamepadMatrix = Get-Content -Raw -Encoding UTF8 $gamepadMatrixPath
        $policyMatrix = Get-Content -Raw -Encoding UTF8 $policyMatrixPath

        $serverOrHeadlessGuard = 'target_mode == RuntimeTargetMode::ServerRuntime \|\| target == PlatformTarget::Headless'
        $windowMatrix | Should Match $serverOrHeadlessGuard
        $windowMatrix | Should Match 'CapabilityStatus::Supported\(WindowBackend::Headless\)'
        $windowMatrix | Should Match 'CapabilityStatus::Unavailable\s*\{\s*reason:\s*"headless target has no window event host backend"'
        $windowMatrix | Should Match 'CapabilityStatus::FeatureDisabled\s*\{\s*feature:\s*"platform-headless"'

        $inputMatrix | Should Match $serverOrHeadlessGuard
        $inputMatrix | Should Match 'CapabilityStatus::Supported\(InputBackend::SyntheticOnly\)'
        $inputMatrix | Should Match 'CapabilityStatus::Unavailable\s*\{\s*reason:\s*"headless target has no keyboard event host backend"'
        $inputMatrix | Should Match 'CapabilityStatus::FeatureDisabled\s*\{\s*feature:\s*"input-keyboard"'

        $gamepadMatrix | Should Match $serverOrHeadlessGuard
        $gamepadMatrix | Should Match 'CapabilityStatus::Unavailable\s*\{\s*reason:\s*"headless target has no physical gamepad backend"'
        $gamepadMatrix | Should Match 'CapabilityStatus::FeatureDisabled\s*\{\s*feature:\s*"input-gamepad"'
        $gamepadMatrix | Should Match 'CapabilityStatus::FeatureDisabled\s*\{\s*feature:\s*"gamepad-gilrs"'

        $policyMatrix | Should Match $serverOrHeadlessGuard
        $policyMatrix | Should Match 'EventLoopPolicy::Headless'
    }
}

Describe "M5 contract documentation index" {
    It "keeps the active platform plan pointing at both focused validator contracts" {
        $planPath = Get-ChildItem `
            -LiteralPath (Join-Path $script:ValidateMatrixTestRepoRoot ".codex\plans") `
            -Filter "ZirconEngine Bevy*Platform Window Input Gilrs*.md" |
            Select-Object -First 1 -ExpandProperty FullName
        $plan = Get-Content -Raw -Encoding UTF8 $planPath

        $plan | Should Match "RunExportPlatformContract"
        $plan | Should Match "RunProfileFeatureContract"
        $plan | Should Match "profile-feature contract"
    }

    It "keeps the profile feature documentation linked to the workflow and local validator" {
        $docPath = Join-Path $script:ValidateMatrixTestRepoRoot "docs\zircon_runtime\platform\profile_feature_contract.md"
        $doc = Get-Content -Raw -Encoding UTF8 $docPath

        $doc | Should Match "\.github/workflows/profile-feature-contract\.yml"
        $doc | Should Match "RunProfileFeatureContract"
        $doc | Should Match "Invoke-Pester"
    }

    It "keeps the export platform documentation linked to CI and local validator" {
        $docPath = Join-Path $script:ValidateMatrixTestRepoRoot "docs\zircon_runtime\platform\export_platform_contract.md"
        $doc = Get-Content -Raw -Encoding UTF8 $docPath

        $doc | Should Match "\.github/workflows/ci\.yml"
        $doc | Should Match "RunExportPlatformContract"
        $doc | Should Match "ExportContractPlatform"
        $doc | Should Match "headless"
    }

    It "documents both low-interference validator selectors" {
        $skillPath = Join-Path $script:ValidateMatrixTestRepoRoot ".codex\skills\zircon-dev\validation\SKILL.md"
        $manualPath = Join-Path $script:ValidateMatrixTestRepoRoot ".codex\skills\zircon-dev\validation\manual-commands.md"
        $planPath = Get-ChildItem `
            -LiteralPath (Join-Path $script:ValidateMatrixTestRepoRoot ".codex\plans") `
            -Filter "ZirconEngine Bevy*Platform Window Input Gilrs*.md" |
            Select-Object -First 1 -ExpandProperty FullName
        $exportDocPath = Join-Path $script:ValidateMatrixTestRepoRoot "docs\zircon_runtime\platform\export_platform_contract.md"
        $profileDocPath = Join-Path $script:ValidateMatrixTestRepoRoot "docs\zircon_runtime\platform\profile_feature_contract.md"

        $combinedDocs = @(
            Get-Content -Raw -Encoding UTF8 $skillPath
            Get-Content -Raw -Encoding UTF8 $manualPath
            Get-Content -Raw -Encoding UTF8 $planPath
            Get-Content -Raw -Encoding UTF8 $exportDocPath
            Get-Content -Raw -Encoding UTF8 $profileDocPath
        ) -join "`n"

        $combinedDocs | Should Match "ExportContractPlatform"
        $combinedDocs | Should Match "ProfileFeatureContractLabel"
    }

    It "documents selector stage switch requirements" {
        $skillPath = Join-Path $script:ValidateMatrixTestRepoRoot ".codex\skills\zircon-dev\validation\SKILL.md"
        $manualPath = Join-Path $script:ValidateMatrixTestRepoRoot ".codex\skills\zircon-dev\validation\manual-commands.md"
        $exportDocPath = Join-Path $script:ValidateMatrixTestRepoRoot "docs\zircon_runtime\platform\export_platform_contract.md"
        $profileDocPath = Join-Path $script:ValidateMatrixTestRepoRoot "docs\zircon_runtime\platform\profile_feature_contract.md"

        $combinedDocs = @(
            Get-Content -Raw -Encoding UTF8 $skillPath
            Get-Content -Raw -Encoding UTF8 $manualPath
            Get-Content -Raw -Encoding UTF8 $exportDocPath
            Get-Content -Raw -Encoding UTF8 $profileDocPath
        ) -join "`n"

        $combinedDocs | Should Match 'ExportContractPlatform.*without `-RunExportPlatformContract` is rejected'
        $combinedDocs | Should Match 'ProfileFeatureContractLabel.*without `-RunProfileFeatureContract` is rejected'
        $combinedDocs | Should Match 'silently ignored'
    }

    It "documents dry-run command rendering without cargo discovery" {
        $skillPath = Join-Path $script:ValidateMatrixTestRepoRoot ".codex\skills\zircon-dev\validation\SKILL.md"
        $manualPath = Join-Path $script:ValidateMatrixTestRepoRoot ".codex\skills\zircon-dev\validation\manual-commands.md"
        $exportDocPath = Join-Path $script:ValidateMatrixTestRepoRoot "docs\zircon_runtime\platform\export_platform_contract.md"
        $profileDocPath = Join-Path $script:ValidateMatrixTestRepoRoot "docs\zircon_runtime\platform\profile_feature_contract.md"

        $combinedDocs = @(
            Get-Content -Raw -Encoding UTF8 $skillPath
            Get-Content -Raw -Encoding UTF8 $manualPath
            Get-Content -Raw -Encoding UTF8 $exportDocPath
            Get-Content -Raw -Encoding UTF8 $profileDocPath
        ) -join "`n"

        $combinedDocs | Should Match "DryRun"
        $combinedDocs | Should Match "without requiring Cargo discovery"
        $combinedDocs | Should Match "target-directory cleanup checks"
        $combinedDocs | Should Match "managed.*lane"
        $combinedDocs | Should Match "cargo-targets"
    }
}
