Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpScenarioRegistrySchemaVersion = 1
$script:MvpScenarioRegistryKind = 'zircon.mvp-scenario-registry'
$script:MvpScenarioRegistryMaximumBytes = 131072
$script:MvpScenarioRegistryLowerHexDigits = [char[]]'0123456789abcdef'
$script:MvpScenarioRegistryIdPattern = '^[a-z0-9][a-z0-9._-]{0,127}$'
$script:MvpScenarioExecutionPolicyKind = 'zircon.mvp-scenario-execution-policy'

function Assert-MvpScenarioRegistryExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$ExpectedNames,
        [Parameter(Mandatory)][string]$Label
    )

    if ($null -eq $Value -or $Value -is [Array]) {
        throw "$Label must contain one JSON object."
    }
    $expected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($name in $ExpectedNames) {
        $expected.Add($name) | Out-Null
    }
    $actualNames = @($Value.PSObject.Properties | ForEach-Object { $_.Name })
    foreach ($name in $actualNames) {
        if (-not $expected.Contains($name)) {
            throw "$Label contains unknown property '$name'."
        }
    }
    foreach ($name in $ExpectedNames) {
        if ($null -eq $Value.PSObject.Properties[$name]) {
            throw "$Label is missing property '$name'."
        }
    }
}

function Assert-MvpScenarioRegistryIdentifier {
    param(
        [AllowNull()]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -isnot [string] -or [string]$Value -cnotmatch $script:MvpScenarioRegistryIdPattern) {
        throw "$Label must be a stable lowercase identifier."
    }
    return [string]$Value
}

function Assert-MvpScenarioRegistryJsonInteger {
    param(
        [AllowNull()]$Value,
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][Int64]$Minimum,
        [Parameter(Mandatory)][Int64]$Maximum
    )

    if (($Value -isnot [int] -and $Value -isnot [long]) -or $Value -is [bool]) {
        throw "$Label must be a JSON integer."
    }
    $integer = [Int64]$Value
    if ($integer -lt $Minimum -or $integer -gt $Maximum) {
        throw "$Label must be in range $Minimum..$Maximum."
    }
    return $integer
}

function Assert-MvpScenarioRegistryStringList {
    param(
        [AllowNull()]$Value,
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][ValidateRange(1, 64)][int]$MaximumCount,
        [string[]]$AllowedValues = @()
    )

    if ($Value -isnot [Array]) {
        throw "$Label must be a JSON array."
    }
    $values = @($Value)
    if ($values.Count -eq 0 -or $values.Count -gt $MaximumCount) {
        throw "$Label must contain 1..$MaximumCount values."
    }
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($valueEntry in $values) {
        $value = Assert-MvpScenarioRegistryIdentifier -Value $valueEntry -Label $Label
        if ($AllowedValues.Count -gt 0 -and $value -cnotin $AllowedValues) {
            throw "$Label contains unsupported value '$value'."
        }
        if (-not $seen.Add($value)) {
            throw "$Label contains duplicate value '$value'."
        }
    }
    return ,([string[]]$values)
}

function Get-MvpScenarioRegistryFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::OpenRead($Path)
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $hashBytes = $hasher.ComputeHash($stream)
        $characters = [char[]]::new($hashBytes.Length * 2)
        $index = 0
        foreach ($hashByte in $hashBytes) {
            $characters[$index] = $script:MvpScenarioRegistryLowerHexDigits[$hashByte -shr 4]
            $characters[$index + 1] = $script:MvpScenarioRegistryLowerHexDigits[$hashByte -band 0x0F]
            $index += 2
        }
        return [string]::new($characters)
    }
    finally {
        $hasher.Dispose()
        $stream.Dispose()
    }
}

function Read-MvpScenarioRegistry {
    param([Parameter(Mandatory)][string]$Path)

    $resolvedPath = [IO.Path]::GetFullPath($Path)
    if (-not [IO.File]::Exists($resolvedPath)) {
        throw "MVP scenario registry does not exist: $resolvedPath"
    }
    $file = [IO.FileInfo]::new($resolvedPath)
    if (($file.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "MVP scenario registry is a reparse point: $resolvedPath"
    }
    if ($file.Length -eq 0 -or $file.Length -gt $script:MvpScenarioRegistryMaximumBytes) {
        throw "MVP scenario registry must contain 1..$($script:MvpScenarioRegistryMaximumBytes) bytes."
    }
    [byte[]]$bytes = [IO.File]::ReadAllBytes($resolvedPath)
    try {
        $source = [Text.UTF8Encoding]::new($false, $true).GetString($bytes) | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "MVP scenario registry is not strict UTF-8 JSON: $($_.Exception.Message)"
    }
    Assert-MvpScenarioRegistryExactProperties `
        -Value $source `
        -ExpectedNames @('schema_version', 'registry_kind', 'registry_id', 'scenarios') `
        -Label 'MVP scenario registry'
    if (($source.schema_version -isnot [int] -and $source.schema_version -isnot [long]) -or
        [Int64]$source.schema_version -ne $script:MvpScenarioRegistrySchemaVersion) {
        throw "MVP scenario registry has unsupported schema version '$($source.schema_version)'."
    }
    if ([string]$source.registry_kind -cne $script:MvpScenarioRegistryKind) {
        throw "MVP scenario registry has unsupported kind '$($source.registry_kind)'."
    }
    $registryId = Assert-MvpScenarioRegistryIdentifier `
        -Value $source.registry_id `
        -Label 'MVP scenario registry ID'
    if ($source.scenarios -isnot [Array]) {
        throw 'MVP scenario registry scenarios must be a JSON array.'
    }
    $scenarios = @($source.scenarios)
    if ($scenarios.Count -eq 0 -or $scenarios.Count -gt 64) {
        throw 'MVP scenario registry must contain 1..64 scenarios.'
    }
    $scenarioIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($scenario in $scenarios) {
        Assert-MvpScenarioRegistryExactProperties `
            -Value $scenario `
            -ExpectedNames @(
                'scenario_id', 'capability_id', 'owner', 'roles', 'liveness_scenario',
                'automation_request', 'steps', 'progress_event_ids', 'oracle_ids', 'artifact_ids', 'variants',
                'execution_policy'
            ) `
            -Label 'MVP scenario registry entry'
        $scenarioId = Assert-MvpScenarioRegistryIdentifier -Value $scenario.scenario_id -Label 'Scenario ID'
        if (-not $scenarioIds.Add($scenarioId)) {
            throw "MVP scenario registry contains duplicate scenario_id '$scenarioId'."
        }
        $scenario.scenario_id = $scenarioId
        $scenario.capability_id = Assert-MvpScenarioRegistryIdentifier -Value $scenario.capability_id -Label 'Capability ID'
        $scenario.owner = Assert-MvpScenarioRegistryIdentifier -Value $scenario.owner -Label 'Scenario owner'
        $scenario.roles = Assert-MvpScenarioRegistryStringList `
            -Value $scenario.roles `
            -Label 'Scenario roles' `
            -MaximumCount 4 `
            -AllowedValues @('editor', 'runtime')
        $livenessScenario = [string]$scenario.liveness_scenario
        if ($livenessScenario -cnotin @(
                'runtime_first_frame', 'editor_first_frame', 'editor_project_create', 'editor_authoring'
            )) {
            throw "Scenario liveness_scenario '$livenessScenario' is unsupported."
        }
        $scenario.liveness_scenario = $livenessScenario
        if ($null -ne $scenario.automation_request) {
            $automationRequest = [string]$scenario.automation_request
            if ([IO.Path]::GetFileName($automationRequest) -cne $automationRequest -or
                -not $automationRequest.EndsWith('.json', [StringComparison]::Ordinal)) {
                throw "Scenario automation_request '$automationRequest' must be a JSON file name."
            }
        }
        $scenario.steps = Assert-MvpScenarioRegistryStringList -Value $scenario.steps -Label 'Scenario steps' -MaximumCount 32
        $scenario.progress_event_ids = Assert-MvpScenarioRegistryStringList `
            -Value $scenario.progress_event_ids `
            -Label 'Scenario progress event IDs' `
            -MaximumCount 16
        $expectedProgressEventCount = switch ($livenessScenario) {
            'runtime_first_frame' { 3 }
            'editor_first_frame' { 3 }
            'editor_project_create' { 4 }
            'editor_authoring' { 3 }
        }
        if (@($scenario.progress_event_ids).Count -ne $expectedProgressEventCount) {
            throw "Scenario '$scenarioId' progress_event_ids count differs from liveness scenario '$livenessScenario'."
        }
        foreach ($eventId in @($scenario.progress_event_ids)) {
            if ($eventId -notmatch '^mvp\.[a-z0-9.-]+\.v[1-9][0-9]*$') {
                throw "Scenario '$scenarioId' progress event ID '$eventId' is not versioned."
            }
        }
        $scenario.oracle_ids = Assert-MvpScenarioRegistryStringList -Value $scenario.oracle_ids -Label 'Scenario oracle IDs' -MaximumCount 16
        $scenario.artifact_ids = Assert-MvpScenarioRegistryStringList -Value $scenario.artifact_ids -Label 'Scenario artifact IDs' -MaximumCount 16
        $scenario.variants = Assert-MvpScenarioRegistryStringList -Value $scenario.variants -Label 'Scenario variants' -MaximumCount 16
        Assert-MvpScenarioRegistryExactProperties `
            -Value $scenario.execution_policy `
            -ExpectedNames @('device_class', 'attempts', 'progress_inactivity_timeout_seconds', 'step_timeouts') `
            -Label "Scenario '$scenarioId' execution_policy"
        $deviceClass = Assert-MvpScenarioRegistryIdentifier `
            -Value $scenario.execution_policy.device_class `
            -Label "Scenario '$scenarioId' execution_policy.device_class"
        if ($deviceClass -cnotin @($scenario.variants)) {
            throw "Scenario '$scenarioId' execution_policy.device_class '$deviceClass' is not a registered variant."
        }
        $scenario.execution_policy.device_class = $deviceClass
        Assert-MvpScenarioRegistryExactProperties `
            -Value $scenario.execution_policy.attempts `
            -ExpectedNames @('minimum', 'default', 'maximum') `
            -Label "Scenario '$scenarioId' execution_policy.attempts"
        $minimumAttempts = Assert-MvpScenarioRegistryJsonInteger `
            -Value $scenario.execution_policy.attempts.minimum `
            -Label "Scenario '$scenarioId' execution_policy.attempts.minimum" `
            -Minimum 1 `
            -Maximum 4
        $defaultAttempts = Assert-MvpScenarioRegistryJsonInteger `
            -Value $scenario.execution_policy.attempts.default `
            -Label "Scenario '$scenarioId' execution_policy.attempts.default" `
            -Minimum 1 `
            -Maximum 4
        $maximumAttempts = Assert-MvpScenarioRegistryJsonInteger `
            -Value $scenario.execution_policy.attempts.maximum `
            -Label "Scenario '$scenarioId' execution_policy.attempts.maximum" `
            -Minimum 1 `
            -Maximum 4
        if ($minimumAttempts -gt $defaultAttempts -or $defaultAttempts -gt $maximumAttempts) {
            throw "Scenario '$scenarioId' execution_policy attempts must satisfy minimum <= default <= maximum."
        }
        $scenario.execution_policy.attempts.minimum = [int]$minimumAttempts
        $scenario.execution_policy.attempts.default = [int]$defaultAttempts
        $scenario.execution_policy.attempts.maximum = [int]$maximumAttempts
        $progressInactivityTimeoutSeconds = Assert-MvpScenarioRegistryJsonInteger `
            -Value $scenario.execution_policy.progress_inactivity_timeout_seconds `
            -Label "Scenario '$scenarioId' execution_policy.progress_inactivity_timeout_seconds" `
            -Minimum 1 `
            -Maximum 600
        $scenario.execution_policy.progress_inactivity_timeout_seconds = [int]$progressInactivityTimeoutSeconds
        if ($scenario.execution_policy.step_timeouts -isnot [Array]) {
            throw "Scenario '$scenarioId' execution_policy.step_timeouts must be a JSON array."
        }
        $stepTimeouts = @($scenario.execution_policy.step_timeouts)
        if ($stepTimeouts.Count -ne @($scenario.steps).Count) {
            throw "Scenario '$scenarioId' execution_policy.step_timeouts must match scenario steps."
        }
        [Int64]$processTimeoutSeconds = 0
        for ($stepIndex = 0; $stepIndex -lt $stepTimeouts.Count; $stepIndex++) {
            $stepTimeout = $stepTimeouts[$stepIndex]
            Assert-MvpScenarioRegistryExactProperties `
                -Value $stepTimeout `
                -ExpectedNames @('step_id', 'timeout_seconds') `
                -Label "Scenario '$scenarioId' execution_policy.step_timeouts entry"
            $stepId = Assert-MvpScenarioRegistryIdentifier `
                -Value $stepTimeout.step_id `
                -Label "Scenario '$scenarioId' execution_policy step ID"
            if ($stepId -cne [string]$scenario.steps[$stepIndex]) {
                throw "Scenario '$scenarioId' execution_policy.step_timeouts must match scenario steps in order."
            }
            $stepTimeoutSeconds = Assert-MvpScenarioRegistryJsonInteger `
                -Value $stepTimeout.timeout_seconds `
                -Label "Scenario '$scenarioId' execution_policy step '$stepId' timeout_seconds" `
                -Minimum 1 `
                -Maximum 600
            $stepTimeout.step_id = $stepId
            $stepTimeout.timeout_seconds = [int]$stepTimeoutSeconds
            $processTimeoutSeconds += $stepTimeoutSeconds
            if ($processTimeoutSeconds -gt 600) {
                throw "Scenario '$scenarioId' execution_policy step timeout total exceeds 600 seconds."
            }
        }
    }
    return [pscustomobject][ordered]@{
        schema_version = $script:MvpScenarioRegistrySchemaVersion
        registry_kind = $script:MvpScenarioRegistryKind
        registry_id = $registryId
        scenarios = $scenarios
        source_path = $resolvedPath
        bytes = [Int64]$bytes.Length
        sha256 = Get-MvpScenarioRegistryFileSha256 -Path $resolvedPath
    }
}

function Resolve-MvpScenarioExecutionPolicy {
    param(
        [Parameter(Mandatory)]$ScenarioRegistration,
        [Parameter(Mandatory)][ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')][string]$ScenarioVariant,
        [Nullable[int]]$RequestedAttemptCount,
        [Nullable[int]]$RequestedTimeoutSeconds,
        [Nullable[int]]$RequestedProgressInactivityTimeoutSeconds
    )

    $scenarioId = [string]$ScenarioRegistration.scenario_id
    if ($ScenarioVariant -cne [string]$ScenarioRegistration.execution_policy.device_class -or
        $ScenarioVariant -cnotin @($ScenarioRegistration.variants)) {
        throw "Scenario '$scenarioId' does not register execution policy device class '$ScenarioVariant'."
    }
    $attempts = $ScenarioRegistration.execution_policy.attempts
    $attemptCount = [int]$attempts.default
    if ($PSBoundParameters.ContainsKey('RequestedAttemptCount') -and $null -ne $RequestedAttemptCount) {
        $attemptCount = [int]$RequestedAttemptCount
        if ($attemptCount -lt [int]$attempts.minimum -or $attemptCount -gt [int]$attempts.maximum) {
            throw "Scenario '$scenarioId' requested attempt count $attemptCount is outside policy range $($attempts.minimum)..$($attempts.maximum)."
        }
    }
    [int]$policyProcessTimeoutSeconds = 0
    foreach ($stepTimeout in @($ScenarioRegistration.execution_policy.step_timeouts)) {
        $policyProcessTimeoutSeconds += [int]$stepTimeout.timeout_seconds
    }
    $processTimeoutSeconds = $policyProcessTimeoutSeconds
    if ($PSBoundParameters.ContainsKey('RequestedTimeoutSeconds') -and $null -ne $RequestedTimeoutSeconds) {
        $requestedTimeout = [int]$RequestedTimeoutSeconds
        if ($requestedTimeout -lt 1 -or $requestedTimeout -gt $policyProcessTimeoutSeconds) {
            throw "Scenario '$scenarioId' requested timeout $requestedTimeout must be within 1..$policyProcessTimeoutSeconds seconds."
        }
        $processTimeoutSeconds = $requestedTimeout
    }
    $policyProgressTimeoutSeconds = [int]$ScenarioRegistration.execution_policy.progress_inactivity_timeout_seconds
    $progressTimeoutSeconds = $policyProgressTimeoutSeconds
    if ($PSBoundParameters.ContainsKey('RequestedProgressInactivityTimeoutSeconds') -and
        $null -ne $RequestedProgressInactivityTimeoutSeconds) {
        $requestedProgressTimeout = [int]$RequestedProgressInactivityTimeoutSeconds
        if ($requestedProgressTimeout -lt 1 -or $requestedProgressTimeout -gt $policyProgressTimeoutSeconds) {
            throw "Scenario '$scenarioId' requested progress inactivity timeout $requestedProgressTimeout must be within 1..$policyProgressTimeoutSeconds seconds."
        }
        $progressTimeoutSeconds = $requestedProgressTimeout
    }
    return [pscustomobject][ordered]@{
        schema_version = 1
        policy_kind = $script:MvpScenarioExecutionPolicyKind
        scenario_id = $scenarioId
        scenario_variant = $ScenarioVariant
        device_class = [string]$ScenarioRegistration.execution_policy.device_class
        attempt_count = $attemptCount
        attempt_minimum = [int]$attempts.minimum
        attempt_default = [int]$attempts.default
        attempt_maximum = [int]$attempts.maximum
        process_timeout_seconds = $processTimeoutSeconds
        policy_process_timeout_seconds = $policyProcessTimeoutSeconds
        progress_inactivity_timeout_seconds = $progressTimeoutSeconds
        policy_progress_inactivity_timeout_seconds = $policyProgressTimeoutSeconds
        step_timeouts = @($ScenarioRegistration.execution_policy.step_timeouts)
    }
}

function Get-MvpScenarioRegistration {
    param(
        [Parameter(Mandatory)]$Registry,
        [Parameter(Mandatory)][ValidatePattern('^[a-z0-9][a-z0-9._-]{0,127}$')][string]$ScenarioId
    )

    $matches = @($Registry.scenarios | Where-Object { [string]$_.scenario_id -ceq $ScenarioId })
    if ($matches.Count -ne 1) {
        throw "MVP scenario '$ScenarioId' is not registered."
    }
    return $matches[0]
}

function Get-MvpScenarioRegistryReceipt {
    param([Parameter(Mandatory)]$Registry)

    return [pscustomobject][ordered]@{
        schema_version = [int]$Registry.schema_version
        registry_kind = [string]$Registry.registry_kind
        registry_id = [string]$Registry.registry_id
        scenario_count = @($Registry.scenarios).Count
        bytes = [Int64]$Registry.bytes
        sha256 = [string]$Registry.sha256
    }
}

Export-ModuleMember -Function @(
    'Read-MvpScenarioRegistry',
    'Get-MvpScenarioRegistration',
    'Get-MvpScenarioRegistryReceipt',
    'Resolve-MvpScenarioExecutionPolicy'
)
