Set-StrictMode -Version Latest

$projectSaveEvidenceRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Import-Module (Join-Path $projectSaveEvidenceRepoRoot 'tools\WindowsPathResolver.psm1') -ErrorAction Stop

$mvpProjectSaveDiagnosticTokenByByte = [string[]]::new(256)
for ($byteValue = 0; $byteValue -lt $mvpProjectSaveDiagnosticTokenByByte.Length; $byteValue++) {
    $isUnreserved =
        ($byteValue -ge 0x41 -and $byteValue -le 0x5A) -or
        ($byteValue -ge 0x61 -and $byteValue -le 0x7A) -or
        ($byteValue -ge 0x30 -and $byteValue -le 0x39) -or
        $byteValue -eq 0x2D -or
        $byteValue -eq 0x2E -or
        $byteValue -eq 0x5F -or
        $byteValue -eq 0x7E
    $mvpProjectSaveDiagnosticTokenByByte[$byteValue] = if ($isUnreserved) {
        [string][char]$byteValue
    }
    else {
        '%' + $byteValue.ToString('X2', [Globalization.CultureInfo]::InvariantCulture)
    }
}

function Get-MvpProjectSaveLifecycleDiagnostics {
    param(
        [Parameter(Mandatory)][string]$DiagnosticText
    )

    $marker = 'editor_project_save result='
    $started = [Collections.Generic.List[object]]::new()
    $completed = [Collections.Generic.List[object]]::new()
    $searchIndex = 0
    while ($searchIndex -lt $DiagnosticText.Length) {
        $markerIndex = $DiagnosticText.IndexOf($marker, $searchIndex, [StringComparison]::Ordinal)
        if ($markerIndex -lt 0) {
            break
        }
        $lineStart = $DiagnosticText.LastIndexOf([char]10, $markerIndex)
        if ($lineStart -lt 0) {
            $lineStart = 0
        }
        else {
            $lineStart++
        }
        $nextLineIndex = $DiagnosticText.IndexOf([char]10, $markerIndex)
        if ($nextLineIndex -lt 0) {
            $lineEnd = $DiagnosticText.Length
            $searchIndex = $DiagnosticText.Length
        }
        else {
            $lineEnd = $nextLineIndex
            $searchIndex = $nextLineIndex + 1
        }
        if ($lineEnd -gt $lineStart -and $DiagnosticText[$lineEnd - 1] -eq [char]13) {
            $lineEnd--
        }
        $line = $DiagnosticText.Substring($lineStart, $lineEnd - $lineStart)
        $entry = [pscustomobject]@{
            index = $markerIndex
            text = $line
        }
        if ($line -match 'editor_project_save result=(?:failed|post_persist_sync_failed)(?:\s|$)') {
            throw 'Authoring automation project save diagnostics contain a failed save lifecycle.'
        }
        if ($line -match 'editor_project_save result=started(?:\s|$)') {
            $started.Add($entry)
        }
        if ($line -match 'editor_project_save result=completed(?:\s|$)') {
            $completed.Add($entry)
        }
    }
    return [pscustomobject]@{
        started = $started
        completed = $completed
    }
}

function Get-MvpProjectSaveDiagnosticTokens {
    param(
        [Parameter(Mandatory)][string]$Line
    )

    $values = [Collections.Generic.Dictionary[string, string]]::new([StringComparer]::Ordinal)
    $duplicateCounts = $null
    foreach ($token in $Line.Split(
            [char[]]$null,
            [StringSplitOptions]::RemoveEmptyEntries)) {
        $equalsIndex = $token.IndexOf([char]61)
        if ($equalsIndex -le 0 -or $equalsIndex -ge ($token.Length - 1)) {
            continue
        }
        $name = $token.Substring(0, $equalsIndex)
        if ($values.ContainsKey($name)) {
            if ($null -eq $duplicateCounts) {
                $duplicateCounts = [Collections.Generic.Dictionary[string, int]]::new([StringComparer]::Ordinal)
            }
            [int]$count = 0
            if ($duplicateCounts.TryGetValue($name, [ref]$count)) {
                $duplicateCounts[$name] = $count + 1
            }
            else {
                $duplicateCounts.Add($name, 2)
            }
        }
        else {
            $values.Add($name, $token.Substring($equalsIndex + 1))
        }
    }
    return [pscustomobject]@{
        values = $values
        duplicate_counts = $duplicateCounts
    }
}

function Get-MvpProjectSaveDiagnosticField {
    param(
        [Parameter(Mandatory)]$Tokens,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    [string]$value = $null
    [int]$count = if ($Tokens.values.TryGetValue($Name, [ref]$value)) { 1 } else { 0 }
    if ($null -ne $Tokens.duplicate_counts) {
        [int]$duplicateCount = 0
        if ($Tokens.duplicate_counts.TryGetValue($Name, [ref]$duplicateCount)) {
            $count = $duplicateCount
        }
    }
    if ($count -ne 1) {
        throw "$Label must contain exactly one '$Name' field; found $count."
    }
    return $value
}

function ConvertTo-MvpProjectSaveUInt64 {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    [UInt64]$parsed = 0
    if (-not [UInt64]::TryParse($Value, [ref]$parsed)) {
        throw "$Label has non-numeric '$Name' value '$Value'."
    }
    return $parsed
}

function ConvertTo-MvpProjectSaveDiagnosticToken {
    param([Parameter(Mandatory)][string]$Value)

    $builder = [Text.StringBuilder]::new()
    foreach ($byte in [Text.Encoding]::UTF8.GetBytes($Value)) {
        $null = $builder.Append($mvpProjectSaveDiagnosticTokenByByte[$byte])
    }
    return $builder.ToString()
}

function Assert-MvpProjectSaveLifecycleEvidence {
    param(
        [Parameter(Mandatory)][string]$DiagnosticText,
        [Parameter(Mandatory)][string]$SaveOperationId,
        [Parameter(Mandatory)][UInt64]$SaveGeneration,
        [Parameter(Mandatory)][string]$ExpectedProjectPath
    )

    $lifecycleDiagnostics = Get-MvpProjectSaveLifecycleDiagnostics -DiagnosticText $DiagnosticText
    $startedLines = $lifecycleDiagnostics.started
    $completedLines = $lifecycleDiagnostics.completed
    if ($startedLines.Count -ne 1 -or $completedLines.Count -ne 1) {
        throw "Authoring automation project save diagnostics require exactly one started/completed pair; found started=$($startedLines.Count) completed=$($completedLines.Count)."
    }

    if ($completedLines[0].index -le $startedLines[0].index) {
        throw 'Project save completed diagnostic must follow its started diagnostic.'
    }

    $started = [string]$startedLines[0].text
    $completed = [string]$completedLines[0].text
    $startedTokens = Get-MvpProjectSaveDiagnosticTokens -Line $started
    $completedTokens = Get-MvpProjectSaveDiagnosticTokens -Line $completed
    try {
        $expectedProject = Resolve-ZirconWindowsPath -Path $ExpectedProjectPath
    }
    catch {
        throw "Project save evidence has invalid expected project path '$ExpectedProjectPath': $($_.Exception.Message)"
    }
    $pathDiagnosticTokens = @($startedTokens, $completedTokens)
    $pathDiagnosticLabels = [string[]]@(
        'Project save started diagnostic',
        'Project save completed diagnostic'
    )
    for ($diagnosticIndex = 0; $diagnosticIndex -lt $pathDiagnosticTokens.Count; $diagnosticIndex++) {
        $diagnosticTokens = $pathDiagnosticTokens[$diagnosticIndex]
        $diagnosticLabel = $pathDiagnosticLabels[$diagnosticIndex]
        $encodedProject = Get-MvpProjectSaveDiagnosticField `
            -Tokens $diagnosticTokens `
            -Name 'project' `
            -Label $diagnosticLabel
        if ($encodedProject -match '%(?![0-9A-Fa-f]{2})') {
            throw "$diagnosticLabel has malformed percent-encoded project '$encodedProject'."
        }
        $decodedProject = [Uri]::UnescapeDataString($encodedProject)
        $canonicalEncodedProject = ConvertTo-MvpProjectSaveDiagnosticToken -Value $decodedProject
        if ($encodedProject -cne $canonicalEncodedProject) {
            throw "$diagnosticLabel project token '$encodedProject' does not use canonical percent encoding."
        }
        try {
            $actualProject = Resolve-ZirconWindowsPath -Path $decodedProject
        }
        catch {
            throw "$diagnosticLabel has invalid project path '$decodedProject'."
        }
        if (-not $actualProject.OperationalPath.Equals($expectedProject.OperationalPath, [StringComparison]::OrdinalIgnoreCase)) {
            throw "$diagnosticLabel project '$($actualProject.DisplayPath)' differs from staged project '$($expectedProject.DisplayPath)'."
        }
    }
    $preSaveDirty = Get-MvpProjectSaveDiagnosticField `
        -Tokens $startedTokens `
        -Name 'pre_save_dirty' `
        -Label 'Project save started diagnostic'
    if ($preSaveDirty -ne 'true') {
        throw "Project save started diagnostic has pre_save_dirty '$preSaveDirty' instead of 'true'."
    }
    $preSaveDirtyGeneration = ConvertTo-MvpProjectSaveUInt64 `
        -Value (Get-MvpProjectSaveDiagnosticField -Tokens $startedTokens -Name 'pre_save_dirty_generation' -Label 'Project save started diagnostic') `
        -Name 'pre_save_dirty_generation' `
        -Label 'Project save started diagnostic'
    $saveTokenGeneration = ConvertTo-MvpProjectSaveUInt64 `
        -Value (Get-MvpProjectSaveDiagnosticField -Tokens $startedTokens -Name 'save_token_generation' -Label 'Project save started diagnostic') `
        -Name 'save_token_generation' `
        -Label 'Project save started diagnostic'
    if ($preSaveDirtyGeneration -eq 0 -or $saveTokenGeneration -ne $preSaveDirtyGeneration) {
        throw "Project save started diagnostic save_token_generation '$saveTokenGeneration' differs from non-zero pre_save_dirty_generation '$preSaveDirtyGeneration'."
    }

    foreach ($field in @(
        @{ name = 'pre_save_dirty_generation'; expected = $preSaveDirtyGeneration },
        @{ name = 'save_token_generation'; expected = $saveTokenGeneration }
    )) {
        $actual = ConvertTo-MvpProjectSaveUInt64 `
            -Value (Get-MvpProjectSaveDiagnosticField -Tokens $completedTokens -Name $field.name -Label 'Project save completed diagnostic') `
            -Name $field.name `
            -Label 'Project save completed diagnostic'
        if ($actual -ne $field.expected) {
            throw "Project save completed diagnostic '$($field.name)' '$actual' differs from started value '$($field.expected)'."
        }
    }

    $persistedGeneration = ConvertTo-MvpProjectSaveUInt64 `
        -Value (Get-MvpProjectSaveDiagnosticField -Tokens $completedTokens -Name 'persisted_generation' -Label 'Project save completed diagnostic') `
        -Name 'persisted_generation' `
        -Label 'Project save completed diagnostic'
    if ($persistedGeneration -ne $SaveGeneration) {
        throw "Project save completed diagnostic persisted_generation '$persistedGeneration' differs from SaveProject event save_generation '$SaveGeneration'."
    }
    if ($persistedGeneration -ne $saveTokenGeneration) {
        throw "Project save completed diagnostic persisted_generation '$persistedGeneration' differs from unchanged save_token_generation '$saveTokenGeneration'."
    }
    $saveMark = Get-MvpProjectSaveDiagnosticField `
        -Tokens $completedTokens `
        -Name 'save_mark' `
        -Label 'Project save completed diagnostic'
    if ($saveMark -ne 'Marked') {
        throw "Project save completed diagnostic has save_mark '$saveMark' instead of 'Marked'."
    }
    if ($SaveOperationId -ne 'file.project.save') {
        throw 'Project save lifecycle is detached from the normal file.project.save operation.'
    }

    return [pscustomobject][ordered]@{
        pre_save_dirty = $true
        pre_save_dirty_generation = $preSaveDirtyGeneration
        save_token_generation = $saveTokenGeneration
        persisted_generation = $persistedGeneration
        save_mark = $saveMark
    }
}

Export-ModuleMember -Function Assert-MvpProjectSaveLifecycleEvidence
