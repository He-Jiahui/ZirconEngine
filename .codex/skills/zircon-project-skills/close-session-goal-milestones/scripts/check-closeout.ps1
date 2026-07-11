[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot,

    [Parameter(Mandatory = $true)]
    [ValidateSet("Milestone", "Goal")]
    [string]$Mode,

    [Parameter(Mandatory = $true)]
    [string]$SessionId,

    [Parameter(Mandatory = $true)]
    [string]$CommitMessage,

    [Parameter(Mandatory = $true)]
    [string]$ManifestPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$errors = [Collections.Generic.List[object]]::new()

function Add-CloseoutError {
    param([string]$Code, [string]$Message, [string[]]$Paths = @())
    $errors.Add([ordered]@{ code = $Code; message = $Message; paths = @($Paths) })
}

function Invoke-GitText {
    param([string[]]$Arguments)
    $output = & git -C $script:resolvedRepo @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Git command failed: git $($Arguments -join ' ')`n$($output -join [Environment]::NewLine)"
    }
    return @($output | ForEach-Object { [string]$_ })
}

function Get-JsonFile {
    param([string]$Path, [string]$Label)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label does not exist: $Path"
    }
    try {
        return Get-Content -Raw -LiteralPath $Path -Encoding UTF8 | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "$Label is not valid JSON: $Path"
    }
}

function Get-PropertyValue {
    param([object]$Value, [string]$Name)
    if ($null -eq $Value) { return $null }
    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property) { return $null }
    return $property.Value
}

function ConvertTo-RepoPath {
    param([string]$RawPath)
    if ([string]::IsNullOrWhiteSpace($RawPath)) {
        return $null
    }
    $candidate = if ([IO.Path]::IsPathRooted($RawPath)) {
        [IO.Path]::GetFullPath($RawPath)
    }
    else {
        [IO.Path]::GetFullPath((Join-Path $script:resolvedRepo $RawPath))
    }
    $rootWithSeparator = $script:resolvedRepo.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if (-not [string]::Equals($candidate, $script:resolvedRepo, [StringComparison]::OrdinalIgnoreCase) -and
        -not $candidate.StartsWith($rootWithSeparator, [StringComparison]::OrdinalIgnoreCase)) {
        return $null
    }
    return $candidate.Substring($script:resolvedRepo.Length).TrimStart('\', '/').Replace('\', '/')
}

function Get-CoordinatorState {
    $evidenceReader = Join-Path $PSScriptRoot "read-closeout-evidence.py"
    $oldPythonPath = $env:PYTHONPATH
    try {
        $env:PYTHONPATH = $script:resolvedRepo
        $evidenceRaw = & python -B $evidenceReader `
            --repo-root $script:resolvedRepo --session-id $SessionId
    }
    finally {
        $env:PYTHONPATH = $oldPythonPath
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator closeout evidence query failed: $($evidenceRaw -join ' ')"
    }
    $evidence = ($evidenceRaw -join "`n") | ConvertFrom-Json
    return [pscustomobject]@{
        branch = [string]$evidence.branch
        service_mode = [string]$evidence.service_mode
        session_id = [string]$evidence.session_id
        session_status = [string]$evidence.session_status
        plan_path = [string]$evidence.plan_path
        owned_dirty_paths = @($evidence.owned_dirty_paths | ForEach-Object { [string]$_ })
        attributed_hashes = $evidence.attributed_hashes
        staged_hashes = $evidence.staged_hashes
        leased_paths = @($evidence.leased_paths | ForEach-Object { [string]$_ })
        open_failure_count = [int]$evidence.open_failure_count
        failure_diagnostics = @($evidence.failure_diagnostics | ForEach-Object { [string]$_ })
        foreign_failure_diagnostics = @($evidence.foreign_failure_diagnostics | ForEach-Object { [string]$_ })
    }
}

try {
    $script:resolvedRepo = (Resolve-Path -LiteralPath $RepoRoot -ErrorAction Stop).Path
    $manifest = Get-JsonFile -Path $ManifestPath -Label "Closeout manifest"
    $coordinator = Get-CoordinatorState
    $stagedDeleted = @(Invoke-GitText -Arguments @("diff", "--cached", "--name-only", "--diff-filter=D") |
        ForEach-Object { $_.Replace('\', '/') } | Sort-Object -Unique)

    $actualBranch = (Invoke-GitText -Arguments @("branch", "--show-current") | Select-Object -First 1).Trim()
    if ($actualBranch -ne "main" -or [string](Get-PropertyValue $coordinator "branch") -ne "main") {
        Add-CloseoutError "not_on_main" "Closeout is allowed only on main."
    }
    if ([string](Get-PropertyValue $coordinator "service_mode") -ne "read_write") {
        Add-CloseoutError "coordinator_read_only" "Coordinator must be in read_write mode."
    }
    if ([string](Get-PropertyValue $coordinator "session_id") -ne $SessionId -or
        [string](Get-PropertyValue $manifest "session_id") -ne $SessionId) {
        Add-CloseoutError "session_mismatch" "Manifest, coordinator, and requested Session IDs must match."
    }
    $allowedStatuses = @("active", "waiting_validation")
    if ([string](Get-PropertyValue $coordinator "session_status") -notin $allowedStatuses) {
        Add-CloseoutError "session_not_closeable" "Session status is not eligible for closeout."
    }
    if ([string](Get-PropertyValue $manifest "mode") -ne $Mode) {
        Add-CloseoutError "mode_mismatch" "Manifest mode must match the requested closeout mode."
    }
    if ([int](Get-PropertyValue $coordinator "open_failure_count") -gt 0) {
        Add-CloseoutError "open_failure_remaining" "Applicable open Failure handoffs must be resolved before closeout."
    }
    if (@(Get-PropertyValue $coordinator "failure_diagnostics").Count -gt 0) {
        Add-CloseoutError "failure_graph_invalid" "Canonical Failure Markdown has validation diagnostics."
    }
    $milestoneId = [string](Get-PropertyValue $manifest "milestone_id")
    if ($milestoneId -notmatch '^M[0-9]+$') {
        Add-CloseoutError "milestone_id_invalid" "Manifest requires a milestone_id such as M2."
    }
    $planRelative = ConvertTo-RepoPath ([string](Get-PropertyValue $coordinator "plan_path"))
    if ($null -eq $planRelative) {
        Add-CloseoutError "plan_evidence_missing" "Registered Session plan is unavailable."
    }

    $categories = Get-PropertyValue $manifest "categories"
    $requiredCategories = @("code", "docs", "tests", "scripts", "untracked")
    $categoryPaths = [ordered]@{}
    foreach ($category in $requiredCategories) {
        $property = if ($null -eq $categories) { $null } else { $categories.PSObject.Properties[$category] }
        if ($null -eq $property) {
            Add-CloseoutError "manifest_category_missing" "Manifest category is missing: $category"
            $categoryPaths[$category] = @()
            continue
        }
        $categoryPaths[$category] = @($property.Value | ForEach-Object { [string]$_ })
    }

    $manifestPaths = [Collections.Generic.List[string]]::new()
    foreach ($category in $requiredCategories) {
        foreach ($rawPath in @($categoryPaths[$category])) {
            $relative = ConvertTo-RepoPath $rawPath
            if ($null -eq $relative) {
                Add-CloseoutError "manifest_path_outside_repo" "Manifest path escapes the repository." @($rawPath)
                continue
            }
            $absolute = Join-Path $script:resolvedRepo $relative
            if (-not (Test-Path -LiteralPath $absolute -PathType Leaf)) {
                if ($category -eq "untracked" -or $stagedDeleted -notcontains $relative) {
                    Add-CloseoutError "manifest_path_missing" "Manifest path does not exist and is not a staged deletion." @($relative)
                    continue
                }
            }
            if ($category -ne "untracked") {
                $manifestPaths.Add($relative)
            }
        }
    }
    foreach ($rawPath in @($categoryPaths["untracked"])) {
        $relative = ConvertTo-RepoPath $rawPath
        if ($null -ne $relative) { $manifestPaths.Add($relative) }
    }
    $manifestSet = @($manifestPaths | Sort-Object -Unique)

    $planEvidencePaths = @()
    $childPrefix = $null
    if ($null -ne $planRelative) {
        $planEvidencePaths += $planRelative
        $planName = [IO.Path]::GetFileName($planRelative)
        if ($planName -match '^(\d+)-') {
            $planParent = [IO.Path]::GetDirectoryName($planRelative).Replace('\', '/')
            $childPrefix = if ([string]::IsNullOrEmpty($planParent)) { $Matches[1] } else { "$planParent/$($Matches[1])" }
        }
    }
    $stagedPlanEvidence = @(
        if ($null -ne $childPrefix) {
            $manifestSet | Where-Object { $_.StartsWith("$childPrefix/", [StringComparison]::OrdinalIgnoreCase) }
        }
    )
    if ($stagedPlanEvidence.Count -eq 0) {
        Add-CloseoutError "plan_evidence_not_staged" "Closeout requires attributed staged evidence in the registered plan's numbered child directory."
    }
    $planEvidencePaths += $stagedPlanEvidence
    $planTextParts = [Collections.Generic.List[string]]::new()
    foreach ($evidencePath in @($planEvidencePaths | Sort-Object -Unique)) {
        $content = & git -C $script:resolvedRepo show ":$evidencePath" 2>$null
        if ($LASTEXITCODE -ne 0) {
            Add-CloseoutError "plan_evidence_missing" "Plan evidence must exist in the Git index." @($evidencePath)
            continue
        }
        $planTextParts.Add(($content -join "`n"))
    }
    $planText = $planTextParts -join "`n"
    $testingPattern = '(?im)^\|\s*' + [Regex]::Escape($milestoneId) +
        '\s*\|\s*[^|]*(?:-T|testing)[^|]*\|\s*(?:通过|完成|pass(?:ed)?|complete(?:d)?)\s*\|'
    if ([string]::IsNullOrWhiteSpace($planText) -or $planText -notmatch $testingPattern) {
        Add-CloseoutError "testing_stage_incomplete" "The milestone testing stage is not accepted."
    }
    $incompleteStatus = '(?:进行中|验收中|待验收|待完成|未开始|阻塞|失败|pending|blocked|failed|not[-_ ]started|in[-_ ]progress|完成\s*[（(]待)'
    $incompleteTablePattern = '(?im)^\|\s*[^|]+\|\s*[^|]+\|\s*' + $incompleteStatus + '[^|]*\|'
    $milestoneTablePattern = '(?im)^\|\s*' + [Regex]::Escape($milestoneId) + '(?:[.\-_][^|]*)?\s*\|\s*[^|]+\|\s*' + $incompleteStatus + '[^|]*\|'
    $milestoneCheckboxPattern = '(?im)^\s*-\s*\[\s\]\s*(?:\*\*)?' + [Regex]::Escape($milestoneId) + '(?:\b|[.\-_])'
    if ($planText -match $milestoneTablePattern -or $planText -match $milestoneCheckboxPattern) {
        Add-CloseoutError "milestone_incomplete" "Milestone closeout requires every slice in the milestone to be complete."
    }
    if ($Mode -eq "Goal" -and (
        $planText -match '(?m)^\s*-\s*\[\s\]\s+' -or
        $planText -match $incompleteTablePattern
    )) {
        Add-CloseoutError "goal_incomplete" "Goal closeout requires aggregate completion evidence."
    }

    $ownedPaths = @((Get-PropertyValue $coordinator "owned_dirty_paths") | ForEach-Object {
        ConvertTo-RepoPath ([string]$_)
    } | Where-Object { $null -ne $_ } | Sort-Object -Unique)
    $omittedOwned = @($ownedPaths | Where-Object { $manifestSet -notcontains $_ })
    if ($omittedOwned.Count -gt 0) {
        Add-CloseoutError "owned_path_omitted" "Manifest omits paths owned by this Session." $omittedOwned
    }
    $unownedManifest = @($manifestSet | Where-Object { $ownedPaths -notcontains $_ })
    if ($unownedManifest.Count -gt 0) {
        Add-CloseoutError "manifest_path_not_owned" "Manifest contains paths without current-hash Session attribution." $unownedManifest
    }
    $leasedPaths = @((Get-PropertyValue $coordinator "leased_paths") | ForEach-Object {
        ConvertTo-RepoPath ([string]$_)
    } | Where-Object { $null -ne $_ } | Sort-Object -Unique)
    $unleasedManifest = @($manifestSet | Where-Object { $leasedPaths -notcontains $_ })
    if ($unleasedManifest.Count -gt 0) {
        Add-CloseoutError "manifest_path_not_leased" "Every closeout path requires a live lease owned by this Session." $unleasedManifest
    }
    $attributedHashes = Get-PropertyValue $coordinator "attributed_hashes"
    $stagedHashes = Get-PropertyValue $coordinator "staged_hashes"
    $stagedHashMismatch = [Collections.Generic.List[string]]::new()
    foreach ($path in $manifestSet) {
        $attributedProperty = if ($null -eq $attributedHashes) { $null } else { $attributedHashes.PSObject.Properties[$path] }
        $stagedProperty = if ($null -eq $stagedHashes) { $null } else { $stagedHashes.PSObject.Properties[$path] }
        if ($null -eq $attributedProperty -or $null -eq $stagedProperty -or
            -not [object]::Equals($attributedProperty.Value, $stagedProperty.Value)) {
            $stagedHashMismatch.Add($path)
        }
    }
    if ($stagedHashMismatch.Count -gt 0) {
        Add-CloseoutError "staged_content_not_attributed" "Staged content differs from current-hash Session attribution." @($stagedHashMismatch)
    }

    $untrackedActual = @(Invoke-GitText -Arguments @("diff", "--cached", "--name-only", "--diff-filter=A") |
        ForEach-Object { $_.Replace('\', '/') } | Sort-Object -Unique)
    $declaredUntracked = @($categoryPaths["untracked"] | ForEach-Object {
        ConvertTo-RepoPath ([string]$_)
    } | Where-Object { $null -ne $_ } | Sort-Object -Unique)
    $misclassifiedUntracked = @($declaredUntracked | Where-Object { $untrackedActual -notcontains $_ })
    if ($misclassifiedUntracked.Count -gt 0) {
        Add-CloseoutError "untracked_category_mismatch" "Declared untracked paths are not staged additions relative to HEAD." $misclassifiedUntracked
    }
    $missingUntrackedClassification = @($untrackedActual | Where-Object { $manifestSet -contains $_ -and $declaredUntracked -notcontains $_ })
    if ($missingUntrackedClassification.Count -gt 0) {
        Add-CloseoutError "staged_addition_not_untracked" "Every staged addition must appear in the untracked category." $missingUntrackedClassification
    }
    $contentCategories = @("code", "docs", "tests", "scripts")
    $contentPaths = @($contentCategories | ForEach-Object { @($categoryPaths[$_]) } | ForEach-Object {
        ConvertTo-RepoPath ([string]$_)
    } | Where-Object { $null -ne $_ } | Sort-Object -Unique)
    $unclassifiedNew = @($declaredUntracked | Where-Object { $contentPaths -notcontains $_ })
    if ($unclassifiedNew.Count -gt 0) {
        Add-CloseoutError "untracked_content_category_missing" "Every untracked path also requires a content category." $unclassifiedNew
    }

    $stagedPaths = @(Invoke-GitText -Arguments @("diff", "--cached", "--name-only", "--diff-filter=ACMRD") |
        ForEach-Object { $_.Replace('\', '/') } | Sort-Object -Unique)
    $extraStaged = @($stagedPaths | Where-Object { $manifestSet -notcontains $_ })
    $missingStaged = @($manifestSet | Where-Object { $stagedPaths -notcontains $_ })
    if ($extraStaged.Count -gt 0 -or $missingStaged.Count -gt 0) {
        Add-CloseoutError "staged_scope_mismatch" "Staged paths must exactly equal the manifest scope." @($extraStaged + $missingStaged)
    }
    if ($stagedPaths.Count -eq 0) {
        Add-CloseoutError "empty_commit_scope" "Closeout cannot create an empty commit."
    }

    $subject = ($CommitMessage -split "`r?`n", 2)[0].Trim()
    $expectedModule = if ($null -eq $planRelative) {
        $null
    }
    else {
        Split-Path -Leaf (Split-Path -Parent $planRelative)
    }
    $moduleMatch = [Regex]::Match($subject, '^【([^【】\r\n/\\]+)】(.+)$')
    $semanticSubject = $subject
    if (-not $moduleMatch.Success -or
        $null -eq $expectedModule -or
        -not [string]::Equals($moduleMatch.Groups[1].Value, $expectedModule, [StringComparison]::OrdinalIgnoreCase)) {
        Add-CloseoutError "invalid_commit_module" "Prefix the subject with the registered plan folder, for example 【runtime】."
    }
    elseif ($moduleMatch.Success) {
        $semanticSubject = $moduleMatch.Groups[2].Value
    }
    $conventional = '^(feat|fix|docs|test|refactor|perf|build|ci|chore|revert)(\([^)]+\))?!?: .+'
    if ($semanticSubject -notmatch $conventional -or
        $CommitMessage -match '(?i)\[zircon-session:' -or
        $CommitMessage -match '(?i)\bcheckpoint\b') {
        Add-CloseoutError "invalid_commit_message" "Use a Conventional Commit after the module prefix, without Session tags or checkpoint wording."
    }

    $diffLines = Invoke-GitText -Arguments @("diff", "--cached", "--unified=0", "--no-color")
    $addedText = @($diffLines | Where-Object { $_.StartsWith("+") -and -not $_.StartsWith("+++") }) -join "`n"
    $webhookHost = @("qyapi", "weixin", "qq", "com") -join '\.'
    $webhookPattern = $webhookHost + '/cgi-bin/webhook/send\?key='
    $capabilityName = @("ZIRCON", "COORDINATOR", "MAINTENANCE", "TOKEN") -join "_"
    $secretValue = '(?:"[^"\r\n]+"|''[^''\r\n]+''|[^\s,;}]+)'
    $capabilityPattern = '(?i)["'']?' + [Regex]::Escape($capabilityName) + '["'']?\s*[:=]\s*' + $secretValue
    $credentialPattern = '(?i)["'']?(?:api[_-]?key|access[_-]?token|client[_-]?secret|password)["'']?\s*[:=]\s*' + $secretValue
    $wecomKeyPattern = '(?i)["'']?(?:wecom|wechat)[_-]?(?:webhook[_-]?)?key["'']?\s*[:=]\s*' + $secretValue
    if ($addedText -match $webhookPattern -or
        $addedText -match $capabilityPattern -or
        $addedText -match $credentialPattern -or
        $addedText -match $wecomKeyPattern) {
        Add-CloseoutError "sensitive_staged_content" "Staged added lines contain webhook, maintenance capability, or credential material."
    }

    $result = [ordered]@{
        status = if ($errors.Count -eq 0) { "ok" } else { "error" }
        mode = $Mode
        session_id = $SessionId
        staged_paths = $stagedPaths
        categories = $categoryPaths
        keep_session_active = $Mode -eq "Milestone"
        complete_goal = $Mode -eq "Goal" -and $errors.Count -eq 0
        foreign_failure_diagnostics = @(Get-PropertyValue $coordinator "foreign_failure_diagnostics")
        errors = @($errors)
    }
    $result | ConvertTo-Json -Depth 8
    if ($errors.Count -gt 0) { exit 1 }
    exit 0
}
catch {
    [ordered]@{
        status = "error"
        mode = $Mode
        session_id = $SessionId
        staged_paths = @()
        categories = [ordered]@{}
        keep_session_active = $Mode -eq "Milestone"
        complete_goal = $false
        errors = @([ordered]@{
            code = "checker_failure"
            message = "$($_.Exception.Message) (line $($_.InvocationInfo.ScriptLineNumber))"
            paths = @()
        })
    } | ConvertTo-Json -Depth 8
    exit 1
}
