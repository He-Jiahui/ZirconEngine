$script:WprScript = Join-Path $PSScriptRoot "..\ui-profile-wpr.ps1"
$script:CaptureScript = Join-Path $PSScriptRoot "..\ui-profile-capture.ps1"
if (Test-Path -LiteralPath $script:WprScript) {
    . $script:WprScript
}

function New-ZirconUiWprTestRoot {
    $root = "E:\zircon-profiles\pester-ui-profile-wpr-$([guid]::NewGuid().ToString('N'))"
    New-Item -ItemType Directory -Force -Path $root | Out-Null
    return $root
}

Describe "UI profile WPR capture" {
    It "exposes the strict WPR capture commands" {
        Get-Command Start-ZirconUiProfileWprCapture -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty
        Get-Command Stop-ZirconUiProfileWprCapture -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty
        Get-Command Register-ZirconUiProfileWprProductProcess -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty
        Get-Command Complete-ZirconUiProfileWprProductProcess -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty
    }

    It "rejects a recorder session on the system drive" {
        {
            Start-ZirconUiProfileWprCapture `
                -ProfileDir "C:\zircon-profiles\forbidden-wpr-session"
        } | Should Throw "D:, E:, or F:"
    }

    It "starts file-mode CPU sampling with a session-owned recording directory" {
        $root = New-ZirconUiWprTestRoot
        try {
            Mock Resolve-ZirconUiProfileNativeTool {
                return "D:\tools\wpr.exe"
            }
            Mock Invoke-ZirconUiProfileNativeTool {
                return 0
            }

            $capture = Start-ZirconUiProfileWprCapture -ProfileDir $root

            $capture.profile_dir | Should Be ([IO.Path]::GetFullPath($root))
            $capture.temporary_directory | Should Be (Join-Path $root "wpr-recording")
            $capture.trace_path | Should Be (Join-Path $root "system.etl")
            $capture.product_process | Should BeNullOrEmpty
            Test-Path -LiteralPath $capture.temporary_directory -PathType Container |
                Should Be $true
            Assert-MockCalled Invoke-ZirconUiProfileNativeTool -Times 1 -Exactly -ParameterFilter {
                $FilePath -eq "D:\tools\wpr.exe" -and
                $Arguments.Count -eq 5 -and
                $Arguments[0] -eq "-start" -and
                $Arguments[1] -eq "CPU" -and
                $Arguments[2] -eq "-filemode" -and
                $Arguments[3] -eq "-recordtempto" -and
                $Arguments[4] -eq (Join-Path $root "wpr-recording")
            }
        }
        finally {
            if (Test-Path -LiteralPath $root) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }

    It "stops WPR and writes source-bound ETL and sampled-profile evidence" {
        $root = New-ZirconUiWprTestRoot
        try {
            Mock Resolve-ZirconUiProfileNativeTool {
                param([string]$Name)
                if ($Name -eq "wpr.exe") {
                    return "D:\tools\wpr.exe"
                }
                if ($Name -eq "xperf.exe") {
                    return "D:\tools\xperf.exe"
                }
                throw "unexpected tool: $Name"
            }
            Mock Invoke-ZirconUiProfileNativeTool {
                param([string]$FilePath, [string[]]$Arguments)
                if ($FilePath.EndsWith("wpr.exe")) {
                    [IO.File]::WriteAllBytes($Arguments[1], [byte[]](1, 2, 3, 4, 5))
                }
                elseif ($FilePath.EndsWith("xperf.exe")) {
                    $outputIndex = [Array]::IndexOf($Arguments, "-o")
                    "zircon_editor.exe!ui::surface::rebuild 42" |
                        Set-Content -LiteralPath $Arguments[$outputIndex + 1] -Encoding UTF8
                }
                return 0
            }

            $capture = [pscustomobject]@{
                schema_version = 1
                profile_dir = $root
                temporary_directory = Join-Path $root "wpr-recording"
                trace_path = Join-Path $root "system.etl"
                wpr_path = "D:\tools\wpr.exe"
                started_utc = [datetime]::UtcNow.ToString("o")
            }
            New-Item -ItemType Directory -Force -Path $capture.temporary_directory | Out-Null

            $receiptPath = Stop-ZirconUiProfileWprCapture -Capture $capture
            $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json

            Test-Path -LiteralPath $receiptPath -PathType Leaf | Should Be $true
            $receipt.schema_version | Should Be 2
            $receipt.evidence_kind | Should Be "windows_sampled_cpu_profile"
            $receipt.is_product_timing | Should Be $false
            $receipt.attribution.scope | Should Be "system"
            $receipt.attribution.product_process_filter_applied | Should Be $false
            $receipt.attribution.product_acceptance_requirement |
                Should Match "zircon_editor.exe"
            $receipt.trace.sha256 | Should Not BeNullOrEmpty
            $receipt.trace.byte_length | Should Be 5
            $receipt.sampled_profile.sha256 | Should Not BeNullOrEmpty
            $receipt.sampled_profile.byte_length | Should BeGreaterThan 0
            $receipt.storage.system_drive_used | Should Be $false
            $receipt.analysis.command | Should Match "profile -detail"
            Assert-MockCalled Invoke-ZirconUiProfileNativeTool -Times 1 -Exactly -ParameterFilter {
                $FilePath -eq "D:\tools\xperf.exe" -and
                $Arguments -contains "-symbols" -and
                $Arguments -contains "profile" -and
                $Arguments -contains "-detail"
            }
        }
        finally {
            if (Test-Path -LiteralPath $root) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }

    It "exports a PID and lifetime filtered Editor sampled-stack product" {
        $root = New-ZirconUiWprTestRoot
        try {
            $editorPath = Join-Path $root "zircon_editor.exe"
            [IO.File]::WriteAllBytes($editorPath, [byte[]](90, 73, 82, 67, 79, 78))
            Mock Resolve-ZirconUiProfileNativeTool { return "D:\tools\xperf.exe" }
            Mock Invoke-ZirconUiProfileNativeTool {
                param([string]$FilePath, [string[]]$Arguments)
                if ($FilePath.EndsWith("wpr.exe")) {
                    [IO.File]::WriteAllBytes($Arguments[1], [byte[]](1, 2, 3, 4, 5))
                }
                else {
                    $outputIndex = [Array]::IndexOf($Arguments, "-o")
                    ($Arguments -join " ") |
                        Set-Content -LiteralPath $Arguments[$outputIndex + 1] -Encoding UTF8
                }
                return 0
            }

            $capture = [pscustomobject]@{
                schema_version = 2
                profile_dir = $root
                temporary_directory = Join-Path $root "wpr-recording"
                trace_path = Join-Path $root "system.etl"
                wpr_path = "D:\tools\wpr.exe"
                started_utc = "2026-08-31T00:00:00.0000000Z"
                product_process = $null
            }
            $process = [pscustomobject]@{
                Id = 4242
                ProcessName = "zircon_editor"
                StartTime = [datetime]::Parse("2026-08-31T00:00:10.0000000Z")
                ExitTime = [datetime]::Parse("2026-08-31T00:00:20.0000000Z")
                HasExited = $true
            }
            New-Item -ItemType Directory -Force -Path $capture.temporary_directory | Out-Null

            Register-ZirconUiProfileWprProductProcess `
                -Capture $capture `
                -Process $process `
                -ExecutablePath $editorPath
            Complete-ZirconUiProfileWprProductProcess -Capture $capture -Process $process
            $receiptPath = Stop-ZirconUiProfileWprCapture -Capture $capture
            $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json

            $receipt.schema_version | Should Be 2
            $receipt.is_product_timing | Should Be $true
            $receipt.attribution.scope | Should Be "product_process"
            $receipt.attribution.product_process_filter_applied | Should Be $true
            $receipt.attribution.process_lifetime_range_applied | Should Be $true
            $receipt.attribution.process.process_id | Should Be 4242
            $receipt.attribution.process.executable.sha256 | Should Not BeNullOrEmpty
            $receipt.product_sampled_stacks.sha256 | Should Not BeNullOrEmpty
            $receipt.product_sampled_stacks.byte_length | Should BeGreaterThan 0
            Assert-MockCalled Invoke-ZirconUiProfileNativeTool -Times 1 -Exactly -ParameterFilter {
                $FilePath -eq "D:\tools\xperf.exe" -and
                $Arguments -contains "stack" -and
                $Arguments -contains "-pid" -and
                $Arguments -contains "4242" -and
                $Arguments -contains "-range" -and
                $Arguments -contains "10000000" -and
                $Arguments -contains "20000000"
            }
        }
        finally {
            if (Test-Path -LiteralPath $root) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }

    It "fails closed when a registered product process has no completed lifetime" {
        $root = New-ZirconUiWprTestRoot
        try {
            Mock Resolve-ZirconUiProfileNativeTool { return "D:\tools\xperf.exe" }
            Mock Invoke-ZirconUiProfileNativeTool {
                param([string]$FilePath, [string[]]$Arguments)
                if ($FilePath.EndsWith("wpr.exe")) {
                    [IO.File]::WriteAllBytes($Arguments[1], [byte[]](1, 2, 3, 4, 5))
                }
                else {
                    $outputIndex = [Array]::IndexOf($Arguments, "-o")
                    "system profile" |
                        Set-Content -LiteralPath $Arguments[$outputIndex + 1] -Encoding UTF8
                }
                return 0
            }

            $capture = [pscustomobject]@{
                schema_version = 2
                profile_dir = $root
                temporary_directory = Join-Path $root "wpr-recording"
                trace_path = Join-Path $root "system.etl"
                wpr_path = "D:\tools\wpr.exe"
                started_utc = "2026-08-31T00:00:00.0000000Z"
                product_process = [pscustomobject]@{
                    process_id = 4242
                    process_name = "zircon_editor"
                    started_utc = "2026-08-31T00:00:10.0000000Z"
                    completed_utc = $null
                    executable = $null
                }
            }
            New-Item -ItemType Directory -Force -Path $capture.temporary_directory | Out-Null

            { Stop-ZirconUiProfileWprCapture -Capture $capture } |
                Should Throw "complete product process lifetime"
        }
        finally {
            if (Test-Path -LiteralPath $root) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }

    It "fails closed when WPR reports success without a nonempty ETL" {
        $root = New-ZirconUiWprTestRoot
        try {
            Mock Invoke-ZirconUiProfileNativeTool { return 0 }
            $capture = [pscustomobject]@{
                schema_version = 1
                profile_dir = $root
                temporary_directory = Join-Path $root "wpr-recording"
                trace_path = Join-Path $root "system.etl"
                wpr_path = "D:\tools\wpr.exe"
                started_utc = [datetime]::UtcNow.ToString("o")
            }
            New-Item -ItemType Directory -Force -Path $capture.temporary_directory | Out-Null

            { Stop-ZirconUiProfileWprCapture -Capture $capture } |
                Should Throw "nonempty WPR ETL"
        }
        finally {
            if (Test-Path -LiteralPath $root) {
                Remove-Item -LiteralPath $root -Recurse -Force
            }
        }
    }

    It "routes the product capture through the strict WPR helper" {
        $captureSource = Get-Content -LiteralPath $script:CaptureScript -Raw
        $wprSource = Get-Content -LiteralPath $script:WprScript -Raw

        $captureSource | Should Match 'ui-profile-wpr\.ps1'
        $captureSource | Should Match 'Start-ZirconUiProfileWprCapture'
        $captureSource | Should Match 'Stop-ZirconUiProfileWprCapture'
        $captureSource | Should Match 'Register-ZirconUiProfileWprProductProcess'
        $captureSource | Should Match 'Complete-ZirconUiProfileWprProductProcess'
        $captureSource | Should Match '-WprCapture\s+\$wprCapture'
        $captureSource | Should Not Match '(?m)^\s*wpr\.exe\s+-stop'
        $wprSource | Should Match '"-recordtempto"'
        $wprSource | Should Match '"profile"'
        $wprSource | Should Match '"-detail"'
    }
}
