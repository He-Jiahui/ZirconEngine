[CmdletBinding()]
param(
    [string]$RepoRoot = "",
    [string]$BinaryPath = "",
    [string]$OutputDir = "",
    [ValidateSet("Isolated", "Current")]
    [string]$ConfigMode = "Isolated",
    [string]$ConfigRoot = "",
    [int]$WaitSeconds = 25,
    [int]$Left = 20,
    [int]$Top = 20,
    [int]$WindowWidth = 1600,
    [int]$WindowHeight = 1024,
    [int]$PinnedProjectCount = 0,
    [int]$NewProjectClickX = 1316,
    [int]$NewProjectClickY = 124,
    [int]$BackClickX = 274,
    [int]$BackClickY = 160,
    [int]$BrowserClickX = 1515,
    [int]$BrowserClickY = 206,
    [int]$DetailClickX = 1534,
    [int]$DetailClickY = 355,
    [int]$DeleteClickX = 0,
    [int]$DeleteClickY = 0,
    [int]$DeleteScrollNotches = -1,
    [int]$BrowserFilterMenuClickX = 0,
    [int]$BrowserFilterMenuClickY = 0,
    [int]$BrowserSortMenuClickX = 0,
    [int]$BrowserSortMenuClickY = 0,
    [string]$LogPath = "",
    [switch]$CapturePendingDelete,
    [switch]$CaptureBrowserMenus,
    [switch]$LeaveOpen
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "hub-capture-common.ps1")

function Resolve-ProjectPageClickDefaults {
    param(
        [int]$WindowWidth,
        [int]$WindowHeight
    )

    if ($WindowWidth -le 1350) {
        $newProjectX = [int][Math]::Round($WindowWidth * 0.838)
    } else {
        $newProjectX = [int][Math]::Round($WindowWidth * 0.915)
    }
    if ($WindowWidth -le 1100) {
        $newProjectY = [int][Math]::Max(96, [Math]::Min(104, [Math]::Round($WindowHeight * 0.138)))
    } else {
        $newProjectY = [int][Math]::Max(120, [Math]::Min(128, [Math]::Round($WindowHeight * 0.138)))
    }

    if ($WindowWidth -le 1100) {
        $browserX = $WindowWidth - 58
        $browserY = [int][Math]::Round([Math]::Max(228, [Math]::Min(232, $WindowHeight * 0.32)))
    } elseif ($WindowWidth -le 1350) {
        $browserX = [int][Math]::Round($WindowWidth * 0.281)
        $browserY = [int][Math]::Round([Math]::Max(755, [Math]::Min(765, $WindowHeight * 0.845)))
    } else {
        $browserX = [int][Math]::Round($WindowWidth * 0.607)
        $browserY = [int][Math]::Round([Math]::Max(535, [Math]::Min(565, $WindowHeight * 0.552)))
    }

    if ($WindowWidth -le 1100) {
        $detailY = [int][Math]::Round([Math]::Max(384, [Math]::Min(404, $WindowHeight * 0.538)))
        $detailX = $WindowWidth - 74
        $deleteX = [int][Math]::Round($WindowWidth * 0.54)
        $deleteY = [int][Math]::Round([Math]::Max(304, [Math]::Min(340, $WindowHeight * 0.445)))
        $deleteScrollNotches = 24
    } elseif ($WindowWidth -le 1350) {
        $detailY = [int][Math]::Round([Math]::Max(344, [Math]::Min(352, $WindowHeight * 0.386)))
        $detailX = [int][Math]::Round($WindowWidth * 0.714)
        $deleteX = [int][Math]::Round($WindowWidth * 0.805)
        $deleteY = [int][Math]::Round($WindowHeight * 0.507)
        $deleteScrollNotches = 0
    } else {
        $detailY = [int][Math]::Round([Math]::Max(398, [Math]::Min(412, $WindowHeight * 0.40)))
        $detailX = [int][Math]::Round($WindowWidth * 0.714)
        $deleteX = [int][Math]::Round($WindowWidth * 0.805)
        $deleteY = [int][Math]::Round($WindowHeight * 0.49)
        $deleteScrollNotches = 0
    }

    if ($WindowWidth -le 1100) {
        $browserFilterMenuX = [int][Math]::Round($WindowWidth * 0.416)
        $browserSortMenuX = [int][Math]::Round($WindowWidth * 0.789)
        $browserMenuY = [int][Math]::Round([Math]::Max(288, [Math]::Min(300, $WindowHeight * 0.38)))
    } else {
        $browserFilterMenuX = [int][Math]::Round($WindowWidth * 0.67)
        $browserSortMenuX = [int][Math]::Round($WindowWidth * 0.879)
        $browserMenuY = [int][Math]::Round([Math]::Max(262, [Math]::Min(274, $WindowHeight * 0.267)))
    }

    return [pscustomobject]@{
        NewProjectX = $newProjectX
        NewProjectY = $newProjectY
        BrowserX = $browserX
        BrowserY = $browserY
        DetailX = [int][Math]::Max(1, $detailX)
        DetailY = $detailY
        DeleteX = [int][Math]::Max(1, $deleteX)
        DeleteY = $deleteY
        DeleteScrollNotches = $deleteScrollNotches
        BrowserFilterMenuX = [int][Math]::Max(1, $browserFilterMenuX)
        BrowserFilterMenuY = $browserMenuY
        BrowserSortMenuX = [int][Math]::Max(1, $browserSortMenuX)
        BrowserSortMenuY = $browserMenuY
    }
}

function ConvertTo-TomlString {
    param([string]$Value)

    $escaped = $Value.Replace('\', '\\').Replace('"', '\"')
    return '"' + $escaped + '"'
}

function ConvertTo-ProjectMetadataKey {
    param([string]$Path)

    $key = $Path.Replace('\', '/')
    while ($key.EndsWith('/') -and $key.Length -gt 1) {
        $key = $key.Substring(0, $key.Length - 1)
    }
    return $key.ToLowerInvariant()
}

function New-ProjectCover {
    param(
        [string]$ProjectPath,
        [string]$BackColor,
        [string]$AccentColor
    )

    $zirconDir = Join-Path $ProjectPath ".zircon"
    New-Item -ItemType Directory -Force -Path $zirconDir | Out-Null
    $coverPath = Join-Path $zirconDir "cover.svg"
    $svg = @"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 420 180">
  <rect width="420" height="180" fill="$BackColor"/>
  <path d="M0 148 L76 74 L135 132 L215 46 L311 144 L420 98 L420 180 L0 180 Z" fill="$AccentColor" opacity="0.82"/>
  <circle cx="330" cy="42" r="26" fill="#f4e7b5" opacity="0.72"/>
</svg>
"@
    Set-Content -LiteralPath $coverPath -Value $svg -Encoding UTF8
}

function Initialize-IsolatedProjectsConfig {
    param(
        [string]$RepoRoot,
        [string]$ConfigRoot,
        [int]$WindowWidth,
        [int]$WindowHeight,
        [int]$PinnedProjectCount
    )

    $localAppData = Join-Path $ConfigRoot "localappdata"
    $appData = Join-Path $ConfigRoot "appdata"
    $hubConfigDir = Join-Path $localAppData "ZirconHub"
    $projectRoot = Join-Path $ConfigRoot "C\ZirconProjects"
    $buildOutput = Join-Path $ConfigRoot "build-output"
    $deviceRoot = Join-Path $ConfigRoot "device"
    $engineRoot = Join-Path $ConfigRoot "engines"
    $engineId = "zircon-1.8.2"
    $activeEngineSourceDir = Join-Path $engineRoot $engineId
    $activeEngineOutputDir = Join-Path $buildOutput $engineId

    New-Item -ItemType Directory -Force -Path $localAppData, $appData, $hubConfigDir, $projectRoot, $buildOutput, $deviceRoot, $engineRoot | Out-Null

    $nowMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $projects = @(
        @{ Name = "Elysium Chronicles"; Path = Join-Path $projectRoot "Elysium"; DisplayPath = "C:\ZirconProjects\Elysium"; Back = "#9fb9b3"; Accent = "#244b43"; Stamp = $nowMs - (2 * 60 * 60 * 1000); EngineId = "zircon-1.8.2" },
        @{ Name = "Stellar Outpost"; Path = Join-Path $projectRoot "StellarOutpost"; DisplayPath = "C:\ZirconProjects\StellarOutpost"; Back = "#092337"; Accent = "#1ab4d2"; Stamp = $nowMs - (24 * 60 * 60 * 1000); EngineId = "zircon-1.8.2" },
        @{ Name = "Sands of Time"; Path = Join-Path $projectRoot "SandsOfTime"; DisplayPath = "C:\ZirconProjects\SandsOfTime"; Back = "#8b744d"; Accent = "#513019"; Stamp = $nowMs - (3 * 24 * 60 * 60 * 1000); EngineId = "zircon-1.8.1" },
        @{ Name = "Whispering Woods"; Path = Join-Path $projectRoot "WhisperingWoods"; DisplayPath = "C:\ZirconProjects\WhisperingWoods"; Back = "#476452"; Accent = "#1b3028"; Stamp = $nowMs - (7 * 24 * 60 * 60 * 1000); EngineId = "zircon-1.8.0" },
        @{ Name = "Neon Streets"; Path = Join-Path $projectRoot "NeonStreets"; DisplayPath = "C:\ZirconProjects\NeonStreets"; Back = "#0d1026"; Accent = "#2d1f58"; Stamp = $nowMs - (14 * 24 * 60 * 60 * 1000); EngineId = "zircon-1.7.9" }
    )

    foreach ($project in $projects) {
        New-Item -ItemType Directory -Force -Path $project.Path | Out-Null
        New-ProjectCover -ProjectPath $project.Path -BackColor $project.Back -AccentColor $project.Accent
        $manifest = @"
[project]
name = $(ConvertTo-TomlString $project.Name)
template = "renderable-empty"
"@
        Set-Content -LiteralPath (Join-Path $project.Path "zircon-project.toml") -Value $manifest -Encoding UTF8
    }

    $recentText = ($projects | ForEach-Object {
        $projectPath = [System.IO.Path]::GetFullPath($_.Path)
        @"

[[recent_projects]]
display_name = $(ConvertTo-TomlString $_.Name)
path = $(ConvertTo-TomlString $projectPath)
last_opened_unix_ms = $($_.Stamp)
"@
    }) -join "`n"

    $metadataText = ($projects | ForEach-Object -Begin { $index = 0 } -Process {
        $projectPath = [System.IO.Path]::GetFullPath($_.Path)
        $key = ConvertTo-ProjectMetadataKey $projectPath
        $pinned = if ($index -lt $PinnedProjectCount) { "true" } else { "false" }
        $index += 1
        @"

[project_metadata.$(ConvertTo-TomlString $key)]
pinned = $pinned
engine_id = $(ConvertTo-TomlString $_.EngineId)
last_selected_template = "renderable-empty"
"@
    }) -join "`n"

    $engineText = @(
        @{ Id = "zircon-1.8.2"; Name = "Zircon Engine 1.8.2" },
        @{ Id = "zircon-1.8.1"; Name = "Zircon Engine 1.8.1" },
        @{ Id = "zircon-1.8.0"; Name = "Zircon Engine 1.8.0" },
        @{ Id = "zircon-1.7.9"; Name = "Zircon Engine 1.7.9" }
    ) | ForEach-Object {
        $engineSourceDir = Join-Path $engineRoot $_.Id
        $engineOutputDir = Join-Path $buildOutput $_.Id
        New-Item -ItemType Directory -Force -Path $engineSourceDir, $engineOutputDir | Out-Null
        @"

[[engines]]
id = $(ConvertTo-TomlString $_.Id)
display_name = $(ConvertTo-TomlString $_.Name)
source_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($engineSourceDir)))
output_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($engineOutputDir)))
last_build_unix_ms = 0
build_history = []
"@
    }
    $engineText = $engineText -join "`n"

    $hubConfigPath = Join-Path $hubConfigDir "config.toml"
    $editorConfigPath = Join-Path $ConfigRoot "zircon-editor-config.json"
    $toml = @"
active_engine_id = $(ConvertTo-TomlString $engineId)

[settings]
python_path = "python"
cargo_path = "cargo"
rustup_path = "rustup"
default_project_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($projectRoot)))
default_source_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($activeEngineSourceDir)))
default_build_output_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($activeEngineOutputDir)))
default_device_install_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($deviceRoot)))
language = "English"
build_profile = "Debug"
jobs = 1

$recentText
$metadataText
$engineText

[window]
position_x = $Left
position_y = $Top
width = $WindowWidth
height = $WindowHeight
maximized = false
"@

    Set-Content -LiteralPath $hubConfigPath -Value $toml -Encoding UTF8
    Set-Content -LiteralPath $editorConfigPath -Value "{}" -Encoding UTF8

    return [pscustomobject]@{
        LocalAppData = $localAppData
        AppData = $appData
        EditorConfigPath = $editorConfigPath
        HubConfigPath = $hubConfigPath
    }
}

function Set-HubCaptureRuntimeState {
    param(
        [string]$HubConfigPath,
        [string]$ProjectSubpage = "dashboard",
        [string]$ProjectViewMode = "grid",
        [string]$SelectedProjectPath = ""
    )

    if (-not (Test-Path -LiteralPath $HubConfigPath)) {
        return
    }

    $text = Get-Content -Raw -LiteralPath $HubConfigPath
    if ($text -notmatch "(?m)^\[runtime\]\s*$") {
        $text = $text.TrimEnd() + "`n`n[runtime]`n"
    }

    function Set-TomlRuntimeKey {
        param(
            [string]$Source,
            [string]$Key,
            [string]$Value
        )

        $pattern = "(?m)^$([regex]::Escape($Key))\s*=.*$"
        if ($Source -match $pattern) {
            return [regex]::Replace($Source, $pattern, "$Key = $Value")
        }

        return [regex]::Replace($Source, "(?m)^(\[runtime\]\s*\r?\n)", "`$1$Key = $Value`n")
    }

    $text = Set-TomlRuntimeKey -Source $text -Key "selected_page" -Value (ConvertTo-TomlString "projects")
    $text = Set-TomlRuntimeKey -Source $text -Key "project_subpage" -Value (ConvertTo-TomlString $ProjectSubpage)
    $text = Set-TomlRuntimeKey -Source $text -Key "project_view_mode" -Value (ConvertTo-TomlString $ProjectViewMode)
    $text = Set-TomlRuntimeKey -Source $text -Key "search_query" -Value (ConvertTo-TomlString "")
    $text = Set-TomlRuntimeKey -Source $text -Key "selected_project_path" -Value (ConvertTo-TomlString $SelectedProjectPath)

    Set-Content -LiteralPath $HubConfigPath -Value $text -Encoding UTF8
}

function Add-CaptureTypes {
    if ("ZirconHubProjectPageCapture" -as [type]) {
        return
    }

    Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class ZirconHubProjectPageCapture {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);

    [DllImport("user32.dll")]
    public static extern bool SetCursorPos(int X, int Y);

    [DllImport("user32.dll")]
    public static extern void mouse_event(uint dwFlags, uint dx, uint dy, int dwData, UIntPtr dwExtraInfo);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }
}
"@

    Add-Type -AssemblyName System.Drawing
    Add-Type -AssemblyName System.Windows.Forms
}

function Resolve-CaptureWindowPosition {
    param(
        [object]$Window,
        [int]$Left,
        [int]$Top
    )

    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
    $resolvedLeft = [Math]::Max($bounds.Left, [Math]::Min($Left, $bounds.Right - $Window.Width))
    $resolvedTop = [Math]::Max($bounds.Top, [Math]::Min($Top, $bounds.Bottom - $Window.Height))

    return [pscustomobject]@{
        Left = $resolvedLeft
        Top = $resolvedTop
    }
}

function Get-HubWindowCandidates {
    param([int]$ProcessId)

    $windows = New-Object System.Collections.Generic.List[object]
    $callback = [ZirconHubProjectPageCapture+EnumWindowsProc]{
        param([IntPtr] $hWnd, [IntPtr] $lParam)

        [uint32]$windowProcessId = 0
        [void][ZirconHubProjectPageCapture]::GetWindowThreadProcessId($hWnd, [ref]$windowProcessId)

        if ($windowProcessId -eq $ProcessId -and [ZirconHubProjectPageCapture]::IsWindowVisible($hWnd)) {
            $rect = New-Object ZirconHubProjectPageCapture+RECT
            if ([ZirconHubProjectPageCapture]::GetWindowRect($hWnd, [ref]$rect)) {
                $width = $rect.Right - $rect.Left
                $height = $rect.Bottom - $rect.Top
                if ($width -gt 100 -and $height -gt 100) {
                    $titleBuilder = New-Object System.Text.StringBuilder 256
                    [void][ZirconHubProjectPageCapture]::GetWindowText($hWnd, $titleBuilder, $titleBuilder.Capacity)
                    $windows.Add([pscustomobject]@{
                        Handle = $hWnd
                        Title  = $titleBuilder.ToString()
                        Left   = $rect.Left
                        Top    = $rect.Top
                        Width  = $width
                        Height = $height
                        Area   = $width * $height
                    }) | Out-Null
                }
            }
        }

        return $true
    }

    [void][ZirconHubProjectPageCapture]::EnumWindows($callback, [IntPtr]::Zero)
    return $windows
}

function Wait-HubWindow {
    param(
        [int]$ProcessId,
        [int]$WaitSeconds
    )

    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    $fallbackSelected = $null
    do {
        Start-Sleep -Milliseconds 500
        $candidates = Get-HubWindowCandidates -ProcessId $ProcessId
        $selected = $candidates | Where-Object { $_.Title -eq "Zircon Hub" } | Sort-Object Area -Descending | Select-Object -First 1
        if ($null -eq $selected) {
            $fallbackSelected = $candidates | Sort-Object Area -Descending | Select-Object -First 1
        }
        if ($null -ne $selected) {
            return $selected
        }
    } while ((Get-Date) -lt $deadline)

    if ($null -ne $fallbackSelected) {
        throw "No titled Zircon Hub window found for process $ProcessId. Largest fallback window was '$($fallbackSelected.Title)' at $($fallbackSelected.Width)x$($fallbackSelected.Height)."
    }

    throw "No visible Zircon Hub window found for process $ProcessId."
}

function Get-HubWindowInfo {
    param([IntPtr]$Handle)

    $rect = New-Object ZirconHubProjectPageCapture+RECT
    if (-not [ZirconHubProjectPageCapture]::GetWindowRect($Handle, [ref]$rect)) {
        throw "Could not read Zircon Hub window bounds for handle $Handle."
    }
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -le 0 -or $height -le 0) {
        throw "Zircon Hub window bounds are invalid: left=$($rect.Left), top=$($rect.Top), right=$($rect.Right), bottom=$($rect.Bottom)."
    }
    $titleBuilder = New-Object System.Text.StringBuilder 256
    [void][ZirconHubProjectPageCapture]::GetWindowText($Handle, $titleBuilder, $titleBuilder.Capacity)
    return [pscustomobject]@{
        Handle = $Handle
        Title = $titleBuilder.ToString()
        Left = $rect.Left
        Top = $rect.Top
        Width = $width
        Height = $height
    }
}

function Assert-HubProcessAlive {
    param(
        [object]$Process,
        [string]$Stage
    )

    if ($Process -and $Process.HasExited) {
        throw "Zircon Hub exited while capturing $Stage. ExitCode=$($Process.ExitCode)."
    }
}

function Bring-HubWindowToFront {
    param([object]$Window)

    $hwndTopmost = [IntPtr]::new(-1)
    [void][ZirconHubProjectPageCapture]::ShowWindow($Window.Handle, 1)
    [void][ZirconHubProjectPageCapture]::SetWindowPos($Window.Handle, $hwndTopmost, $Window.Left, $Window.Top, $Window.Width, $Window.Height, 0x0040)
    [void][ZirconHubProjectPageCapture]::SetForegroundWindow($Window.Handle)
    Start-Sleep -Milliseconds 180
}

function Save-HubCapture {
    param(
        [object]$Window,
        [string]$Path
    )

    $Window = Get-HubWindowInfo -Handle $Window.Handle
    if ($Window.Title -ne "Zircon Hub") {
        throw "Refusing to capture '$Path' from window titled '$($Window.Title)'; expected 'Zircon Hub'."
    }
    if ($script:MinimumCaptureWidth -gt 0 -and $Window.Width -lt $script:MinimumCaptureWidth) {
        throw "Refusing to capture '$Path' because window width $($Window.Width) is below minimum $script:MinimumCaptureWidth."
    }
    if ($script:MinimumCaptureHeight -gt 0 -and $Window.Height -lt $script:MinimumCaptureHeight) {
        throw "Refusing to capture '$Path' because window height $($Window.Height) is below minimum $script:MinimumCaptureHeight."
    }
    if ($Window.Width -le 0 -or $Window.Height -le 0) {
        throw "Cannot capture '$Path' because window size is $($Window.Width)x$($Window.Height)."
    }
    Bring-HubWindowToFront -Window $Window
    New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName($Path)) | Out-Null
    Invoke-HubCaptureWebViewScreenshot -Port $script:WebViewDebugPort -OutputPath $Path -WaitSeconds $script:WebViewWaitSeconds -Title "Zircon Hub"
}

function Get-CaptureDifferenceRatio {
    param(
        [string]$BeforePath,
        [string]$AfterPath,
        [int]$Stride = 8,
        [int]$Tolerance = 32
    )

    $before = New-Object System.Drawing.Bitmap($BeforePath)
    $after = New-Object System.Drawing.Bitmap($AfterPath)

    try {
        $width = [Math]::Min($before.Width, $after.Width)
        $height = [Math]::Min($before.Height, $after.Height)
        $total = 0
        $changed = 0

        for ($y = 0; $y -lt $height; $y += $Stride) {
            for ($x = 0; $x -lt $width; $x += $Stride) {
                $beforePixel = $before.GetPixel($x, $y)
                $afterPixel = $after.GetPixel($x, $y)
                $delta =
                    [Math]::Abs([int]$beforePixel.R - [int]$afterPixel.R) +
                    [Math]::Abs([int]$beforePixel.G - [int]$afterPixel.G) +
                    [Math]::Abs([int]$beforePixel.B - [int]$afterPixel.B)
                if ($delta -gt $Tolerance) {
                    $changed += 1
                }
                $total += 1
            }
        }

        if ($total -eq 0) {
            return 0.0
        }
        return [double]$changed / [double]$total
    } finally {
        $before.Dispose()
        $after.Dispose()
    }
}

function Assert-CaptureChanged {
    param(
        [string]$BeforePath,
        [string]$AfterPath,
        [string]$Stage,
        [double]$MinimumDifference = 0.02
    )

    $change = Test-CaptureChanged -BeforePath $BeforePath -AfterPath $AfterPath -MinimumDifference $MinimumDifference
    if (-not $change.Changed) {
        $ratio = $change.Ratio
        throw "Capture for $Stage did not change enough from the previous page. DifferenceRatio=$([Math]::Round($ratio, 4)), expected at least $MinimumDifference. Check click coordinates or Hub navigation before trusting '$AfterPath'."
    }
}

function Test-CaptureChanged {
    param(
        [string]$BeforePath,
        [string]$AfterPath,
        [double]$MinimumDifference = 0.02
    )

    $ratio = Get-CaptureDifferenceRatio -BeforePath $BeforePath -AfterPath $AfterPath
    return [pscustomobject]@{
        Changed = $ratio -ge $MinimumDifference
        Ratio = $ratio
    }
}

function Invoke-HubClick {
    param(
        [object]$Window,
        [int]$X,
        [int]$Y,
        [int]$DelayMilliseconds = 900,
        [int]$Repeat = 1
    )

    [void][ZirconHubProjectPageCapture]::ShowWindow($Window.Handle, 1)
    [void][ZirconHubProjectPageCapture]::SetForegroundWindow($Window.Handle)
    Start-Sleep -Milliseconds 160

    foreach ($index in 1..$Repeat) {
        [void][ZirconHubProjectPageCapture]::SetCursorPos($Window.Left + $X, $Window.Top + $Y)
        Start-Sleep -Milliseconds 120
        [ZirconHubProjectPageCapture]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 80
        [ZirconHubProjectPageCapture]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
        if ($index -lt $Repeat) {
            Start-Sleep -Milliseconds 320
        }
    }

    Start-Sleep -Milliseconds $DelayMilliseconds
}

function Invoke-HubMouseWheel {
    param(
        [object]$Window,
        [int]$X,
        [int]$Y,
        [int]$Notches,
        [int]$DelayMilliseconds = 700
    )

    if ($Notches -eq 0) {
        return
    }

    [void][ZirconHubProjectPageCapture]::ShowWindow($Window.Handle, 1)
    [void][ZirconHubProjectPageCapture]::SetForegroundWindow($Window.Handle)
    Start-Sleep -Milliseconds 160
    [void][ZirconHubProjectPageCapture]::SetCursorPos($Window.Left + $X, $Window.Top + $Y)
    Start-Sleep -Milliseconds 120

    $steps = [Math]::Abs($Notches)
    $delta = if ($Notches -gt 0) { -120 } else { 120 }
    foreach ($index in 1..$steps) {
        [ZirconHubProjectPageCapture]::mouse_event(0x0800, 0, 0, $delta, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 45
    }

    Start-Sleep -Milliseconds $DelayMilliseconds
}

function Invoke-HubProjectDeleteAction {
    param(
        [ValidateSet("scroll-text", "click-text")]
        [string]$Action,
        [int]$DelayMilliseconds
    )

    $errors = @()
    foreach ($label in @("删除项目", "Delete Project")) {
        try {
            Invoke-HubCaptureWebViewAction -Port $script:WebViewDebugPort -Action $Action -Text $label -WaitSeconds $script:WebViewWaitSeconds -DelayMilliseconds $DelayMilliseconds
            return $label
        } catch {
            $errors += $_.Exception.Message
        }
    }

    throw "Could not invoke project delete action '$Action' with localized labels. $($errors -join "`n")"
}

function Start-HubCaptureSession {
    param(
        [string]$BinaryPath,
        [string]$RepoRoot,
        [string]$StdoutPath,
        [string]$StderrPath,
        [int]$WaitSeconds,
        [int]$Left,
        [int]$Top,
        [int]$WindowWidth,
        [int]$WindowHeight,
        [string]$Stage
    )

    $sessionProcess = Start-Process -FilePath $BinaryPath -WorkingDirectory $RepoRoot -PassThru -WindowStyle Hidden -RedirectStandardOutput $StdoutPath -RedirectStandardError $StderrPath
    $sessionWindow = Wait-HubWindow -ProcessId $sessionProcess.Id -WaitSeconds $WaitSeconds
    $windowPosition = Resolve-CaptureWindowPosition -Window $sessionWindow -Left $Left -Top $Top
    $hwndTopmost = [IntPtr]::new(-1)
    $setPositionNoSizeFlags = 0x0040 -bor 0x0001
    [void][ZirconHubProjectPageCapture]::ShowWindow($sessionWindow.Handle, 1)
    [void][ZirconHubProjectPageCapture]::SetWindowPos($sessionWindow.Handle, $hwndTopmost, $windowPosition.Left, $windowPosition.Top, 0, 0, $setPositionNoSizeFlags)
    [void][ZirconHubProjectPageCapture]::SetForegroundWindow($sessionWindow.Handle)
    Start-Sleep -Seconds 2
    Assert-HubProcessAlive -Process $sessionProcess -Stage $Stage

    return [pscustomobject]@{
        Process = $sessionProcess
        Window = Get-HubWindowInfo -Handle $sessionWindow.Handle
    }
}

function Stop-HubCaptureSession {
    param([System.Diagnostics.Process]$Process)

    if (-not $Process -or $Process.HasExited) {
        return
    }

    $Process.CloseMainWindow() | Out-Null
    Start-Sleep -Milliseconds 500
    if (-not $Process.HasExited) {
        $Process.Kill()
    }
}

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\..\..\..\.."))
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)

if ([string]::IsNullOrWhiteSpace($BinaryPath)) {
    $BinaryPath = Join-Path $RepoRoot "target\debug\zircon_hub.exe"
}
$BinaryPath = [System.IO.Path]::GetFullPath($BinaryPath)

if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Hub binary not found at '$BinaryPath'. Build it first with: cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1"
}

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $RepoRoot "target\hub-visual-check"
}
$OutputDir = [System.IO.Path]::GetFullPath($OutputDir)
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

if ([string]::IsNullOrWhiteSpace($LogPath)) {
    $LogPath = Join-Path $OutputDir "hub-project-pages-capture.log"
}
$LogPath = [System.IO.Path]::GetFullPath($LogPath)
$stdoutPath = [System.IO.Path]::ChangeExtension($LogPath, ".stdout.log")
$stderrPath = [System.IO.Path]::ChangeExtension($LogPath, ".stderr.log")
Remove-Item -LiteralPath $stdoutPath, $stderrPath -Force -ErrorAction SilentlyContinue

if ([string]::IsNullOrWhiteSpace($ConfigRoot)) {
    $ConfigRoot = Join-Path $OutputDir "project-pages-config"
}
$ConfigRoot = [System.IO.Path]::GetFullPath($ConfigRoot)
$configRootForDelete = $ConfigRoot.TrimEnd('\')
$outputRootForDelete = $OutputDir.TrimEnd('\')

Add-CaptureTypes

$script:MinimumCaptureWidth = [int][Math]::Floor($WindowWidth * 0.90)
$script:MinimumCaptureHeight = [int][Math]::Floor($WindowHeight * 0.90)

$oldLocalAppData = $env:LOCALAPPDATA
$oldAppData = $env:APPDATA
$oldZirconConfigPath = $env:ZIRCON_CONFIG_PATH
$frontendProcess = $null
$oldWebViewArguments = $null
$process = $null
$config = $null

try {
    $frontendProcess = Start-HubCaptureFrontendDevServer -RepoRoot $RepoRoot -LogBasePath ([System.IO.Path]::ChangeExtension($LogPath, $null))
    $script:WebViewDebugPort = Get-HubCaptureAvailableTcpPort
    $script:WebViewWaitSeconds = $WaitSeconds
    $oldWebViewArguments = Set-HubCaptureWebViewDebugEnvironment -Port $script:WebViewDebugPort

    if ($ConfigMode -eq "Isolated") {
        if (-not $configRootForDelete.StartsWith($outputRootForDelete + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to clear ConfigRoot outside OutputDir: $ConfigRoot"
        }
        if (Test-Path -LiteralPath $ConfigRoot) {
            Remove-Item -LiteralPath $ConfigRoot -Recurse -Force
        }
        $config = Initialize-IsolatedProjectsConfig -RepoRoot $RepoRoot -ConfigRoot $ConfigRoot -WindowWidth $WindowWidth -WindowHeight $WindowHeight -PinnedProjectCount $PinnedProjectCount
        $env:LOCALAPPDATA = $config.LocalAppData
        $env:APPDATA = $config.AppData
        $env:ZIRCON_CONFIG_PATH = $config.EditorConfigPath
        Set-HubCaptureRuntimeState -HubConfigPath $config.HubConfigPath -ProjectSubpage "dashboard" -ProjectViewMode "grid"
    }

    $hwndNotTopmost = [IntPtr]::new(-2)
    $dashboard = Join-Path $OutputDir "hub-projects-dashboard.png"
    $newProject = Join-Path $OutputDir "hub-projects-new-project.png"
    $browser = Join-Path $OutputDir "hub-projects-browser.png"
    $browserFilterMenu = Join-Path $OutputDir "hub-projects-browser-filter-menu.png"
    $browserSortMenu = Join-Path $OutputDir "hub-projects-browser-sort-menu.png"
    $detail = Join-Path $OutputDir "hub-projects-detail.png"
    $detailDeleteReady = Join-Path $OutputDir "hub-projects-detail-delete-ready.png"
    $detailPendingDelete = Join-Path $OutputDir "hub-projects-detail-delete-confirm.png"
    $clickDefaults = Resolve-ProjectPageClickDefaults -WindowWidth $WindowWidth -WindowHeight $WindowHeight

    if (-not $PSBoundParameters.ContainsKey("NewProjectClickX")) {
        $NewProjectClickX = $clickDefaults.NewProjectX
    }
    if (-not $PSBoundParameters.ContainsKey("NewProjectClickY")) {
        $NewProjectClickY = $clickDefaults.NewProjectY
    }
    if (-not $PSBoundParameters.ContainsKey("BrowserClickX")) {
        $BrowserClickX = $clickDefaults.BrowserX
    }
    if (-not $PSBoundParameters.ContainsKey("BrowserClickY")) {
        $BrowserClickY = $clickDefaults.BrowserY
    }
    if (-not $PSBoundParameters.ContainsKey("DetailClickX")) {
        $DetailClickX = $clickDefaults.DetailX
    }
    if (-not $PSBoundParameters.ContainsKey("DetailClickY")) {
        $DetailClickY = $clickDefaults.DetailY
    }
    if (-not $PSBoundParameters.ContainsKey("DeleteClickX") -or $DeleteClickX -le 0) {
        $DeleteClickX = $clickDefaults.DeleteX
    }
    if (-not $PSBoundParameters.ContainsKey("DeleteClickY") -or $DeleteClickY -le 0) {
        $DeleteClickY = $clickDefaults.DeleteY
    }
    if (-not $PSBoundParameters.ContainsKey("DeleteScrollNotches") -or $DeleteScrollNotches -lt 0) {
        $DeleteScrollNotches = $clickDefaults.DeleteScrollNotches
    }
    if (-not $PSBoundParameters.ContainsKey("BrowserFilterMenuClickX") -or $BrowserFilterMenuClickX -le 0) {
        $BrowserFilterMenuClickX = $clickDefaults.BrowserFilterMenuX
    }
    if (-not $PSBoundParameters.ContainsKey("BrowserFilterMenuClickY") -or $BrowserFilterMenuClickY -le 0) {
        $BrowserFilterMenuClickY = $clickDefaults.BrowserFilterMenuY
    }
    if (-not $PSBoundParameters.ContainsKey("BrowserSortMenuClickX") -or $BrowserSortMenuClickX -le 0) {
        $BrowserSortMenuClickX = $clickDefaults.BrowserSortMenuX
    }
    if (-not $PSBoundParameters.ContainsKey("BrowserSortMenuClickY") -or $BrowserSortMenuClickY -le 0) {
        $BrowserSortMenuClickY = $clickDefaults.BrowserSortMenuY
    }

    $clicksPath = Join-Path $OutputDir "hub-project-pages-clicks.json"
    [pscustomobject]@{
        WindowWidth = $WindowWidth
        WindowHeight = $WindowHeight
        NewProjectClickX = $NewProjectClickX
        NewProjectClickY = $NewProjectClickY
        BrowserClickX = $BrowserClickX
        BrowserClickY = $BrowserClickY
        DetailClickX = $DetailClickX
        DetailClickY = $DetailClickY
        DeleteClickX = $DeleteClickX
        DeleteClickY = $DeleteClickY
        DeleteScrollNotches = $DeleteScrollNotches
        BrowserFilterMenuClickX = $BrowserFilterMenuClickX
        BrowserFilterMenuClickY = $BrowserFilterMenuClickY
        BrowserSortMenuClickX = $BrowserSortMenuClickX
        BrowserSortMenuClickY = $BrowserSortMenuClickY
        CapturePendingDelete = [bool]$CapturePendingDelete
        CaptureBrowserMenus = [bool]$CaptureBrowserMenus
    } | ConvertTo-Json | Set-Content -LiteralPath $clicksPath -Encoding UTF8

    $session = Start-HubCaptureSession -BinaryPath $BinaryPath -RepoRoot $RepoRoot -StdoutPath $stdoutPath -StderrPath $stderrPath -WaitSeconds $WaitSeconds -Left $Left -Top $Top -WindowWidth $WindowWidth -WindowHeight $WindowHeight -Stage "dashboard"
    $process = $session.Process
    $window = $session.Window
    [void][ZirconHubProjectPageCapture]::SetCursorPos($window.Left + 8, $window.Top + 8)
    Start-Sleep -Milliseconds 120
    Save-HubCapture -Window $window -Path $dashboard

    $focusX = [int][Math]::Max(240, [Math]::Round($WindowWidth * 0.25))
    $focusY = [int][Math]::Max(145, [Math]::Min(170, [Math]::Round($WindowHeight * 0.18)))
    Invoke-HubClick -Window $window -X $focusX -Y $focusY -DelayMilliseconds 250
    Invoke-HubClick -Window $window -X $NewProjectClickX -Y $NewProjectClickY -DelayMilliseconds 1200
    Assert-HubProcessAlive -Process $process -Stage "new-project"
    $window = Get-HubWindowInfo -Handle $window.Handle
    Save-HubCapture -Window $window -Path $newProject
    Assert-CaptureChanged -BeforePath $dashboard -AfterPath $newProject -Stage "new-project"

    [void][ZirconHubProjectPageCapture]::SetWindowPos($window.Handle, $hwndNotTopmost, $window.Left, $window.Top, $window.Width, $window.Height, 0x0040)
    Stop-HubCaptureSession -Process $process
    $process = $null
    if ($ConfigMode -eq "Isolated" -and $config) {
        Set-HubCaptureRuntimeState -HubConfigPath $config.HubConfigPath -ProjectSubpage "dashboard" -ProjectViewMode "grid"
    }

    $session = Start-HubCaptureSession -BinaryPath $BinaryPath -RepoRoot $RepoRoot -StdoutPath $stdoutPath -StderrPath $stderrPath -WaitSeconds $WaitSeconds -Left $Left -Top $Top -WindowWidth $WindowWidth -WindowHeight $WindowHeight -Stage "project-browser-launch"
    $process = $session.Process
    $window = $session.Window
    Invoke-HubClick -Window $window -X $focusX -Y $focusY -DelayMilliseconds 250
    Invoke-HubClick -Window $window -X $BrowserClickX -Y $BrowserClickY -DelayMilliseconds 1200
    Assert-HubProcessAlive -Process $process -Stage "project-browser"
    $window = Get-HubWindowInfo -Handle $window.Handle
    Save-HubCapture -Window $window -Path $browser
    Assert-CaptureChanged -BeforePath $dashboard -AfterPath $browser -Stage "project-browser"

    Invoke-HubCaptureWebViewAction -Port $script:WebViewDebugPort -Action "click-text" -Text "Elysium Chronicles" -WaitSeconds $script:WebViewWaitSeconds -DelayMilliseconds 1200
    Assert-HubProcessAlive -Process $process -Stage "project-detail"
    $window = Get-HubWindowInfo -Handle $window.Handle
    Save-HubCapture -Window $window -Path $detail
    $detailMinimumDifference = 0.10
    $detailChange = Test-CaptureChanged -BeforePath $browser -AfterPath $detail -MinimumDifference $detailMinimumDifference
    if (-not $detailChange.Changed) {
        Invoke-HubCaptureWebViewAction -Port $script:WebViewDebugPort -Action "click-text" -Text "Elysium Chronicles" -WaitSeconds $script:WebViewWaitSeconds -DelayMilliseconds 1200
        Assert-HubProcessAlive -Process $process -Stage "project-detail-retry"
        $window = Get-HubWindowInfo -Handle $window.Handle
        Save-HubCapture -Window $window -Path $detail
    }
    Assert-CaptureChanged -BeforePath $browser -AfterPath $detail -Stage "project-detail" -MinimumDifference $detailMinimumDifference

    if ($CapturePendingDelete) {
        [void](Invoke-HubProjectDeleteAction -Action "scroll-text" -DelayMilliseconds 700)
        Assert-HubProcessAlive -Process $process -Stage "project-detail-delete-scroll"
        $window = Get-HubWindowInfo -Handle $window.Handle
        Save-HubCapture -Window $window -Path $detailDeleteReady
        Assert-CaptureChanged -BeforePath $detail -AfterPath $detailDeleteReady -Stage "project-detail-delete-scroll" -MinimumDifference 0.01

        [void](Invoke-HubProjectDeleteAction -Action "click-text" -DelayMilliseconds 1200)
        Assert-HubProcessAlive -Process $process -Stage "project-detail-delete-confirm"
        $window = Get-HubWindowInfo -Handle $window.Handle
        Save-HubCapture -Window $window -Path $detailPendingDelete
        Assert-CaptureChanged -BeforePath $detailDeleteReady -AfterPath $detailPendingDelete -Stage "project-detail-delete-confirm" -MinimumDifference 0.02
    }

    [void][ZirconHubProjectPageCapture]::SetWindowPos($window.Handle, $hwndNotTopmost, $window.Left, $window.Top, $window.Width, $window.Height, 0x0040)

    if ($CaptureBrowserMenus) {
        Stop-HubCaptureSession -Process $process
        $process = $null

        # Dark reference menus are intentionally compact; keep the gate nonzero
        # so stale captures fail without rejecting a visible opened menu.
        $menuMinimumDifference = 0.0008
        $menuTargets = @(
            [pscustomobject]@{ Stage = "project-browser-filter-menu"; ClickX = $BrowserFilterMenuClickX; ClickY = $BrowserFilterMenuClickY; Path = $browserFilterMenu },
            [pscustomobject]@{ Stage = "project-browser-sort-menu"; ClickX = $BrowserSortMenuClickX; ClickY = $BrowserSortMenuClickY; Path = $browserSortMenu }
        )

        foreach ($target in $menuTargets) {
            if ($ConfigMode -eq "Isolated" -and $config) {
                Set-HubCaptureRuntimeState -HubConfigPath $config.HubConfigPath -ProjectSubpage "project-browser" -ProjectViewMode "list"
            }
            $session = Start-HubCaptureSession -BinaryPath $BinaryPath -RepoRoot $RepoRoot -StdoutPath $stdoutPath -StderrPath $stderrPath -WaitSeconds $WaitSeconds -Left $Left -Top $Top -WindowWidth $WindowWidth -WindowHeight $WindowHeight -Stage $target.Stage
            $process = $session.Process
            $window = $session.Window
            Invoke-HubClick -Window $window -X $focusX -Y $focusY -DelayMilliseconds 250
            Assert-HubProcessAlive -Process $process -Stage "$($target.Stage)-browser"
            $window = Get-HubWindowInfo -Handle $window.Handle
            Invoke-HubClick -Window $window -X $target.ClickX -Y $target.ClickY -DelayMilliseconds 700
            Assert-HubProcessAlive -Process $process -Stage $target.Stage
            $window = Get-HubWindowInfo -Handle $window.Handle
            Save-HubCapture -Window $window -Path $target.Path
            Assert-CaptureChanged -BeforePath $browser -AfterPath $target.Path -Stage $target.Stage -MinimumDifference $menuMinimumDifference
            [void][ZirconHubProjectPageCapture]::SetWindowPos($window.Handle, $hwndNotTopmost, $window.Left, $window.Top, $window.Width, $window.Height, 0x0040)

            if (-not $LeaveOpen -or $target.Stage -ne $menuTargets[-1].Stage) {
                Stop-HubCaptureSession -Process $process
                $process = $null
            }
        }
    }

    $pages = @(
        [pscustomobject]@{ Page = "dashboard"; Path = $dashboard },
        [pscustomobject]@{ Page = "new-project"; Path = $newProject },
        [pscustomobject]@{ Page = "project-browser"; Path = $browser },
        [pscustomobject]@{ Page = "project-detail"; Path = $detail }
    )
    if ($CapturePendingDelete) {
        $pages += [pscustomobject]@{ Page = "project-detail-delete-confirm"; Path = $detailPendingDelete }
    }
    if ($CaptureBrowserMenus) {
        $pages += [pscustomobject]@{ Page = "project-browser-filter-menu"; Path = $browserFilterMenu }
        $pages += [pscustomobject]@{ Page = "project-browser-sort-menu"; Path = $browserSortMenu }
    }
    $pages | Format-Table -AutoSize
} finally {
    $env:LOCALAPPDATA = $oldLocalAppData
    $env:APPDATA = $oldAppData
    $env:ZIRCON_CONFIG_PATH = $oldZirconConfigPath
    Restore-HubCaptureWebViewDebugEnvironment -PreviousValue $oldWebViewArguments
    Stop-HubCaptureFrontendDevServer -Process $frontendProcess

    if ($process -and -not $process.HasExited -and -not $LeaveOpen) {
        $process.CloseMainWindow() | Out-Null
        Start-Sleep -Milliseconds 500
        if (-not $process.HasExited) {
            $process.Kill()
        }
    }

    foreach ($path in @($stdoutPath, $stderrPath)) {
        if ((Test-Path -LiteralPath $path) -and (Get-Item -LiteralPath $path).Length -gt 0) {
            Write-Host "==== $path ===="
            Get-Content -LiteralPath $path -Tail 80
        }
    }
}
