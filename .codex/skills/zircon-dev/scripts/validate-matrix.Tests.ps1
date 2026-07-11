$script:ValidateMatrixScript = Join-Path $PSScriptRoot "validate-matrix.ps1"
$script:ValidateMatrixTestRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")).Path
$script:OriginalValidateMatrixTestMode = $env:VALIDATE_MATRIX_TEST_MODE
$script:OriginalCargoTargetDir = $env:CARGO_TARGET_DIR

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

function Get-CiProfileFeatureMatrix {
    $workflowPath = Join-Path $script:ValidateMatrixTestRepoRoot ".github\workflows\profile-feature-contract.yml"
    $workflow = Get-Content -Raw -Encoding UTF8 $workflowPath
    $caseMatches = [regex]::Matches(
        $workflow,
        "(?ms)^\s*-\s+label:\s*(?<label>[^\r\n]+)\s+package:\s*(?<package>[^\r\n]+)\s+features:\s*(?<features>[^\r\n]+)"
    )

    if ($caseMatches.Count -eq 0) {
        throw "Could not find profile-feature matrix cases in $workflowPath"
    }

    return $caseMatches | ForEach-Object {
        [pscustomobject]@{
            Label    = $_.Groups["label"].Value.Trim()
            Package  = $_.Groups["package"].Value.Trim()
            Features = $_.Groups["features"].Value.Trim()
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

    It "releases a leased job when Cargo discovery fails before start" {
        $client = Join-Path $script:ValidateMatrixTestRepoRoot "tools\zircon-session.ps1"
        $beforeRaw = & $client -Command cargo -RepoRoot $script:ValidateMatrixTestRepoRoot -Json list
        $beforeIds = @((($beforeRaw -join "`n") | ConvertFrom-Json).jobs | ForEach-Object job_id)
        $result = Invoke-ValidateMatrixCliWithoutCargo -Arguments @("-SkipTest")

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "cargo"
        $raw = & $client -Command cargo -RepoRoot $script:ValidateMatrixTestRepoRoot -Json list
        $jobs = (($raw -join "`n") | ConvertFrom-Json).jobs
        $ownerId = Resolve-OwnerId -RepoRoot $script:ValidateMatrixTestRepoRoot
        $created = @($jobs | Where-Object {
            $beforeIds -notcontains $_.job_id -and $_.session_id -eq $ownerId
        })
        $created.Count | Should Be 1
        $created[0].status | Should Be "released"
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

Describe "Validate matrix CLI dry-run parsing" {
    It "allocates a managed drive-root lane for no-stage sanity checks" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir -Arguments @(
            "-DryRun",
            "-SkipBuild",
            "-SkipTest"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Dry run: on"
        $result.Output | Should Match "Target dir: [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+ \(coordinator managed workspace lane\)"
        $result.Output | Should Match "No stages selected"
        $result.Output | Should Not Match "target\\manual-check"
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
        $result.Output | Should Match "cargo_target_not_managed|managed targets/zircon-engine/lanes"
    }

    It "rejects an inherited target outside managed lane roots" {
        $result = Invoke-ValidateMatrixCliWithCargoTargetDir `
            -CargoTargetDir "E:\cargo-targets\unmanaged" `
            -Arguments @("-DryRun", "-SkipBuild", "-SkipTest")

        $result.ExitCode | Should Not Be 0
        $result.Output | Should Match "cargo_target_not_managed|managed targets/zircon-engine/lanes"
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
        ([regex]::Matches($result.Output, "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+")).Count |
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
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
        $result.Output | Should Match "Target dir: [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+ \(coordinator managed workspace lane\)"
        $result.Output | Should Match "Dry run selected; skipping cargo discovery and target directory cleanup checks"
        $result.Output | Should Match "Export platform contract \(headless\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
        $result.Output | Should Match "Target dir: [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+ \(coordinator managed workspace lane\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
            "E:\targets\zircon-engine\lanes\pester-custom-dry-run"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: E:\\targets\\zircon-engine\\lanes\\pester-custom-dry-run \(coordinator validated manual target\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --target-dir E:\\targets\\zircon-engine\\lanes\\pester-custom-dry-run"
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
        $result.Output | Should Match "Target dir: [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+ \(coordinator managed workspace lane\)"
        $result.Output | Should Match "Export platform contract \(headless\)"
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
        $result.Output | Should Match "cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
        $exportTargetPath = Join-Path $script:ValidateMatrixTestRepoRoot "zircon_runtime\src\plugin\export_profile.rs"

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
            "{0}|{1}|{2}" -f $_.Label, $_.Package, $_.Features
        } | Should Be @(
            "zircon_app target-server|zircon_app|target-server",
            "zircon_app target-client-platform|zircon_app|target-client,platform-winit,input-gamepad,gamepad-gilrs",
            "zircon_runtime target-client|zircon_runtime|target-client",
            "zircon_runtime target-editor-host|zircon_runtime|target-editor-host",
            "zircon_runtime target-server|zircon_runtime|target-server"
        )
    }

    It "keeps the local profile feature list identical to the GitHub Actions matrix" {
        $ciCases = Get-CiProfileFeatureMatrix | ForEach-Object {
            "{0}|{1}|{2}" -f $_.Label, $_.Package, $_.Features
        }
        $localCases = $script:ProfileFeatureContractCases | ForEach-Object {
            "{0}|{1}|{2}" -f $_.Label, $_.Package, $_.Features
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

            $command | Should Match ("check -p {0} --no-default-features --features {1}" -f $case.Package, [regex]::Escape($case.Features))
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
            $result.Output | Should Match ("Profile feature contract \({0}\)" -f [regex]::Escape($case.Label))
            $result.Output | Should Match (
                "cargo check -p {0} --no-default-features --features {1} --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+" -f
                [regex]::Escape($case.Package),
                [regex]::Escape($case.Features)
            )
        }
        ([regex]::Matches($result.Output, "cargo check -p .* --no-default-features --features .* --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+")).Count |
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
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
        $result.Output | Should Match "Target dir: [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+ \(coordinator managed workspace lane\)"
        $result.Output | Should Match "Dry run selected; skipping cargo discovery and target directory cleanup checks"
        $result.Output | Should Match "Profile feature contract \(zircon_runtime target-server\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
        $result.Output | Should Match "Target dir: [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+ \(coordinator managed workspace lane\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
            "E:\targets\zircon-engine\lanes\pester-custom-dry-run"
        )

        $result.ExitCode | Should Be 0
        $result.Output | Should Match "Target dir: E:\\targets\\zircon-engine\\lanes\\pester-custom-dry-run \(coordinator validated manual target\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --target-dir E:\\targets\\zircon-engine\\lanes\\pester-custom-dry-run"
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
        $result.Output | Should Match "Target dir: [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+ \(coordinator managed workspace lane\)"
        $result.Output | Should Match "Profile feature contract \(zircon_runtime target-server\)"
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --locked --verbose --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
        $result.Output | Should Match "cargo check -p zircon_runtime --no-default-features --features target-server --target-dir [D-F]:\\targets\\zircon-engine\\lanes\\workspace-[0-9a-f]+"
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
        $result.Output | Should Match "target-client-platform, zircon_runtime target-client, zircon_runtime target-editor-host, zircon_runtime"
        $result.Output | Should Match "target-server"
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
        $workflow | Should Match "cargo check -p .*\{\{ matrix\.package \}\} --no-default-features --features .*\{\{ matrix\.features \}\} --locked --verbose"
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
        $workflow | Should Match "cargo check -p \$\{\{ matrix\.package \}\} --no-default-features --features \$\{\{ matrix\.features \}\} --locked --verbose"
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
        $combinedDocs | Should Match "targets\\zircon-engine\\lanes|targets/zircon-engine/lanes"
    }
}
