$script:CleanupScript = Join-Path $PSScriptRoot "cleanup-stale-targets.ps1"
. $script:CleanupScript

function New-TestCleanupRoot {
    param([string]$Name = "cargo-targets")

    $root = Join-Path $TestDrive (Join-Path ([guid]::NewGuid().ToString("N")) $Name)
    New-Item -ItemType Directory -Path $root -Force | Out-Null
    return (Get-Item -LiteralPath $root)
}

function Set-StaleDirectory {
    param(
        [System.IO.DirectoryInfo]$Root,
        [string]$Name,
        [datetime]$Timestamp = [datetime]::UtcNow.AddHours(-4)
    )

    $directory = New-Item -ItemType Directory -Path (Join-Path $Root.FullName $Name) -Force
    $directory.LastWriteTimeUtc = $Timestamp
    return (Get-Item -LiteralPath $directory.FullName)
}

Describe "cleanup-stale-targets unmanaged discovery" {
    It "returns only stale unmanaged direct child directories" {
        $root = New-TestCleanupRoot
        $stale = Set-StaleDirectory -Root $root -Name "stale"
        $fresh = Set-StaleDirectory -Root $root -Name "fresh" -Timestamp ([datetime]::UtcNow)
        New-Item -ItemType File -Path (Join-Path $root.FullName "file.bin") | Out-Null
        $managed = [System.Collections.Generic.HashSet[string]]::new(
            [System.StringComparer]::OrdinalIgnoreCase
        )

        $candidates = @(Get-UnmanagedCleanupCandidates `
            -Roots @($root.FullName) `
            -ManagedPathKeys $managed `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)))

        $candidates.Count | Should Be 1
        $candidates[0].Path | Should Be $stale.FullName
        (Test-Path -LiteralPath $fresh.FullName) | Should Be $true
    }

    It "excludes coordinator candidates and denials" {
        $root = New-TestCleanupRoot
        $candidate = Set-StaleDirectory -Root $root -Name "managed-candidate"
        $denied = Set-StaleDirectory -Root $root -Name "managed-denied"
        $unmanaged = Set-StaleDirectory -Root $root -Name "unmanaged"
        $managed = [System.Collections.Generic.HashSet[string]]::new(
            [System.StringComparer]::OrdinalIgnoreCase
        )
        $managed.Add((ConvertTo-CleanupPathKey $candidate.FullName)) | Out-Null
        $managed.Add((ConvertTo-CleanupPathKey $denied.FullName)) | Out-Null

        $candidates = @(Get-UnmanagedCleanupCandidates `
            -Roots @($root.FullName) `
            -ManagedPathKeys $managed `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)))

        $candidates.Count | Should Be 1
        $candidates[0].Path | Should Be $unmanaged.FullName
    }

    It "excludes a direct-child ancestor of a nested managed pool" {
        $root = New-TestCleanupRoot
        $managedAncestor = Set-StaleDirectory -Root $root -Name "zircon-engine"
        $managedPool = New-Item -ItemType Directory -Path (
            Join-Path $managedAncestor.FullName "pool\compatibility-key"
        ) -Force
        $unmanaged = Set-StaleDirectory -Root $root -Name "unmanaged"
        $managed = [System.Collections.Generic.HashSet[string]]::new(
            [System.StringComparer]::OrdinalIgnoreCase
        )
        $managed.Add((ConvertTo-CleanupPathKey $managedPool.FullName)) | Out-Null

        $candidates = @(Get-UnmanagedCleanupCandidates `
            -Roots @($root.FullName) `
            -ManagedPathKeys $managed `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)))

        $candidates.Count | Should Be 1
        $candidates[0].Path | Should Be $unmanaged.FullName
    }

    It "ignores missing roots and never selects nested paths independently" {
        $root = New-TestCleanupRoot
        $parent = Set-StaleDirectory -Root $root -Name "parent"
        $nested = New-Item -ItemType Directory -Path (Join-Path $parent.FullName "nested")
        $nested.LastWriteTimeUtc = [datetime]::UtcNow.AddHours(-6)
        $parent.LastWriteTimeUtc = [datetime]::UtcNow.AddHours(-4)
        $managed = [System.Collections.Generic.HashSet[string]]::new()

        $candidates = @(Get-UnmanagedCleanupCandidates `
            -Roots @((Join-Path $TestDrive "missing"), $root.FullName) `
            -ManagedPathKeys $managed `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)))

        $candidates.Count | Should Be 1
        $candidates[0].Path | Should Be $parent.FullName
    }

    It "rejects reparse-point children" {
        $root = New-TestCleanupRoot
        $outside = New-Item -ItemType Directory -Path (Join-Path $TestDrive "outside")
        $link = Join-Path $root.FullName "junction"
        $created = $false
        if ($env:OS -eq "Windows_NT") {
            & cmd.exe /d /c "mklink /J `"$link`" `"$($outside.FullName)`"" | Out-Null
            $created = $LASTEXITCODE -eq 0
        }
        if (-not $created) {
            try {
                New-Item -ItemType SymbolicLink -Path $link -Target $outside.FullName | Out-Null
                $created = $true
            } catch {
                Set-TestInconclusive "Directory reparse points are unavailable: $_"
            }
        }
        (Get-Item -LiteralPath $link).LastWriteTimeUtc = [datetime]::UtcNow.AddHours(-4)

        $candidates = @(Get-UnmanagedCleanupCandidates `
            -Roots @($root.FullName) `
            -ManagedPathKeys ([System.Collections.Generic.HashSet[string]]::new()) `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)))

        $candidates.Count | Should Be 0
    }
}

Describe "cleanup-stale-targets apply revalidation" {
    It "keeps a candidate that became fresh after planning" {
        $root = New-TestCleanupRoot
        $candidate = Set-StaleDirectory -Root $root -Name "became-fresh"
        $candidate.LastWriteTimeUtc = [datetime]::UtcNow

        $result = Remove-UnmanagedCleanupCandidate `
            -Root $root.FullName `
            -Path $candidate.FullName `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)) `
            -ManagedPathKeys ([System.Collections.Generic.HashSet[string]]::new()) `
            -Confirm:$false

        $result.Status | Should Be "retained"
        (Test-Path -LiteralPath $candidate.FullName) | Should Be $true
    }

    It "never accepts a configured root as a deletion candidate" {
        $root = New-TestCleanupRoot

        $result = Remove-UnmanagedCleanupCandidate `
            -Root $root.FullName `
            -Path $root.FullName `
            -CutoffUtc ([datetime]::UtcNow.AddHours(1)) `
            -ManagedPathKeys ([System.Collections.Generic.HashSet[string]]::new()) `
            -Confirm:$false

        $result.Status | Should Be "retained"
        (Test-Path -LiteralPath $root.FullName) | Should Be $true
    }

    It "removes a reviewed stale unmanaged direct child" {
        $root = New-TestCleanupRoot
        $candidate = Set-StaleDirectory -Root $root -Name "delete-me"

        $result = Remove-UnmanagedCleanupCandidate `
            -Root $root.FullName `
            -Path $candidate.FullName `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)) `
            -ManagedPathKeys ([System.Collections.Generic.HashSet[string]]::new()) `
            -Confirm:$false

        $result.Status | Should Be "deleted"
        (Test-Path -LiteralPath $candidate.FullName) | Should Be $false
    }

    It "honors WhatIf for unmanaged deletion" {
        $root = New-TestCleanupRoot
        $candidate = Set-StaleDirectory -Root $root -Name "what-if"

        $result = Remove-UnmanagedCleanupCandidate `
            -Root $root.FullName `
            -Path $candidate.FullName `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)) `
            -ManagedPathKeys ([System.Collections.Generic.HashSet[string]]::new()) `
            -WhatIf

        $result.Status | Should Be "retained"
        (Test-Path -LiteralPath $candidate.FullName) | Should Be $true
    }

    It "keeps a candidate when a managed nested pool appears after planning" {
        $root = New-TestCleanupRoot
        $candidate = Set-StaleDirectory -Root $root -Name "zircon-engine"
        $managedPool = New-Item -ItemType Directory -Path (
            Join-Path $candidate.FullName "pool\new-key"
        ) -Force
        $managed = [System.Collections.Generic.HashSet[string]]::new(
            [System.StringComparer]::OrdinalIgnoreCase
        )
        $managed.Add((ConvertTo-CleanupPathKey $managedPool.FullName)) | Out-Null

        $result = Remove-UnmanagedCleanupCandidate `
            -Root $root.FullName `
            -Path $candidate.FullName `
            -CutoffUtc ([datetime]::UtcNow.AddHours(-2)) `
            -ManagedPathKeys $managed `
            -Confirm:$false

        $result.Status | Should Be "retained"
        $result.Reason | Should Be "managed_path_overlap"
        (Test-Path -LiteralPath $candidate.FullName) | Should Be $true
    }
}
