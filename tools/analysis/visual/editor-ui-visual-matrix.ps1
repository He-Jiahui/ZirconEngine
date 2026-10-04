# Capture each profile on a display with its real Windows DPI. The caller keeps
# GetDpiForWindow and Winit scale/extent validation authoritative.
function Get-ZirconEditorVisualCaptureCases {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet('100', '150')]
        [string]$DpiProfile
    )

    $cases = if ($DpiProfile -eq '100') {
        @(
            @{ CaseId = '1280x800'; LogicalWidth = 1280; LogicalHeight = 800 },
            @{ CaseId = '900x620'; LogicalWidth = 900; LogicalHeight = 620 },
            @{ CaseId = '640x520'; LogicalWidth = 640; LogicalHeight = 520 }
        )
    }
    else {
        @(@{ CaseId = '1280x800-dpi150'; LogicalWidth = 1280; LogicalHeight = 800 })
    }
    $scale = if ($DpiProfile -eq '100') { 1.0 } else { 1.5 }
    $dpi = if ($DpiProfile -eq '100') { 96 } else { 144 }
    foreach ($case in $cases) {
        $case.Width = [int][Math]::Round($case.LogicalWidth * $scale)
        $case.Height = [int][Math]::Round($case.LogicalHeight * $scale)
        $case.ExpectedWindowDpi = $dpi
        $case.ExpectedWinitScaleFactor = $scale
        $case
    }
}
