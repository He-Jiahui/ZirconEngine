Set-StrictMode -Version Latest

function Get-MvpProjectSaveDiagnosticField {
    param(
        [Parameter(Mandatory)][string]$Line,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Label
    )

    $pattern = '(?:^|\s)' + [regex]::Escape($Name) + '=([^\s]+)'
    $matches = [regex]::Matches($Line, $pattern)
    if ($matches.Count -ne 1) {
        throw "$Label must contain exactly one '$Name' field; found $($matches.Count)."
    }
    return $matches[0].Groups[1].Value
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
        $isUnreserved =
            ($byte -ge 0x41 -and $byte -le 0x5A) -or
            ($byte -ge 0x61 -and $byte -le 0x7A) -or
            ($byte -ge 0x30 -and $byte -le 0x39) -or
            $byte -eq 0x2D -or
            $byte -eq 0x2E -or
            $byte -eq 0x5F -or
            $byte -eq 0x7E
        if ($isUnreserved) {
            $null = $builder.Append([char]$byte)
        }
        else {
            $null = $builder.Append('%')
            $null = $builder.Append(
                $byte.ToString('X2', [Globalization.CultureInfo]::InvariantCulture)
            )
        }
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

    $diagnosticLines = @($DiagnosticText -split '\r?\n')
    $projectSaveLines = @(
        for ($index = 0; $index -lt $diagnosticLines.Count; $index++) {
            if ($diagnosticLines[$index] -match 'editor_project_save result=') {
                [pscustomobject]@{
                    index = $index
                    text = $diagnosticLines[$index]
                }
            }
        }
    )
    $failureLines = @(
        $projectSaveLines | Where-Object {
            $_.text -match 'editor_project_save result=(?:failed|post_persist_sync_failed)(?:\s|$)'
        }
    )
    if ($failureLines.Count -ne 0) {
        throw 'Authoring automation project save diagnostics contain a failed save lifecycle.'
    }
    $startedLines = @(
        $projectSaveLines | Where-Object {
            $_.text -match 'editor_project_save result=started(?:\s|$)'
        }
    )
    $completedLines = @(
        $projectSaveLines | Where-Object {
            $_.text -match 'editor_project_save result=completed(?:\s|$)'
        }
    )
    if ($startedLines.Count -ne 1 -or $completedLines.Count -ne 1) {
        throw "Authoring automation project save diagnostics require exactly one started/completed pair; found started=$($startedLines.Count) completed=$($completedLines.Count)."
    }

    if ($completedLines[0].index -le $startedLines[0].index) {
        throw 'Project save completed diagnostic must follow its started diagnostic.'
    }

    $started = [string]$startedLines[0].text
    $completed = [string]$completedLines[0].text
    $expectedProject = [IO.Path]::GetFullPath($ExpectedProjectPath)
    foreach ($diagnostic in @(
        @{ line = $started; label = 'Project save started diagnostic' },
        @{ line = $completed; label = 'Project save completed diagnostic' }
    )) {
        $encodedProject = Get-MvpProjectSaveDiagnosticField `
            -Line $diagnostic.line `
            -Name 'project' `
            -Label $diagnostic.label
        if ($encodedProject -match '%(?![0-9A-Fa-f]{2})') {
            throw "$($diagnostic.label) has malformed percent-encoded project '$encodedProject'."
        }
        $decodedProject = [Uri]::UnescapeDataString($encodedProject)
        $canonicalEncodedProject = ConvertTo-MvpProjectSaveDiagnosticToken -Value $decodedProject
        if ($encodedProject -cne $canonicalEncodedProject) {
            throw "$($diagnostic.label) project token '$encodedProject' does not use canonical percent encoding."
        }
        try {
            $actualProject = [IO.Path]::GetFullPath($decodedProject)
        }
        catch {
            throw "$($diagnostic.label) has invalid project path '$decodedProject'."
        }
        if (-not $actualProject.Equals($expectedProject, [StringComparison]::OrdinalIgnoreCase)) {
            throw "$($diagnostic.label) project '$decodedProject' differs from staged project '$expectedProject'."
        }
    }
    $preSaveDirty = Get-MvpProjectSaveDiagnosticField `
        -Line $started `
        -Name 'pre_save_dirty' `
        -Label 'Project save started diagnostic'
    if ($preSaveDirty -ne 'true') {
        throw "Project save started diagnostic has pre_save_dirty '$preSaveDirty' instead of 'true'."
    }
    $preSaveDirtyGeneration = ConvertTo-MvpProjectSaveUInt64 `
        -Value (Get-MvpProjectSaveDiagnosticField -Line $started -Name 'pre_save_dirty_generation' -Label 'Project save started diagnostic') `
        -Name 'pre_save_dirty_generation' `
        -Label 'Project save started diagnostic'
    $saveTokenGeneration = ConvertTo-MvpProjectSaveUInt64 `
        -Value (Get-MvpProjectSaveDiagnosticField -Line $started -Name 'save_token_generation' -Label 'Project save started diagnostic') `
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
            -Value (Get-MvpProjectSaveDiagnosticField -Line $completed -Name $field.name -Label 'Project save completed diagnostic') `
            -Name $field.name `
            -Label 'Project save completed diagnostic'
        if ($actual -ne $field.expected) {
            throw "Project save completed diagnostic '$($field.name)' '$actual' differs from started value '$($field.expected)'."
        }
    }

    $persistedGeneration = ConvertTo-MvpProjectSaveUInt64 `
        -Value (Get-MvpProjectSaveDiagnosticField -Line $completed -Name 'persisted_generation' -Label 'Project save completed diagnostic') `
        -Name 'persisted_generation' `
        -Label 'Project save completed diagnostic'
    if ($persistedGeneration -ne $SaveGeneration) {
        throw "Project save completed diagnostic persisted_generation '$persistedGeneration' differs from SaveProject event save_generation '$SaveGeneration'."
    }
    if ($persistedGeneration -ne $saveTokenGeneration) {
        throw "Project save completed diagnostic persisted_generation '$persistedGeneration' differs from unchanged save_token_generation '$saveTokenGeneration'."
    }
    $saveMark = Get-MvpProjectSaveDiagnosticField `
        -Line $completed `
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
