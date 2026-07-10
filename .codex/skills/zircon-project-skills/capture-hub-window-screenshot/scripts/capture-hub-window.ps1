[CmdletBinding()]
param(
    [string]$RepoRoot = "",
    [string]$BinaryPath = "",
    [string]$OutputPath = "",
    [ValidateSet("Isolated", "Current")]
    [string]$ConfigMode = "Isolated",
    [string]$ConfigRoot = "",
    [int]$WaitSeconds = 20,
    [int]$Left = 20,
    [int]$Top = 20,
    [int]$WindowWidth = 0,
    [int]$WindowHeight = 0,
    [int]$ClickX = -1,
    [int]$ClickY = -1,
    [int]$ClickDelayMilliseconds = 900,
    [int]$SecondClickX = -1,
    [int]$SecondClickY = -1,
    [int]$SecondClickDelayMilliseconds = 900,
    [string]$WebViewClickText = "",
    [int]$WebViewClickDelayMilliseconds = 900,
    [string]$RequireWebViewText = "",
    [ValidateSet("", "loading", "running", "warning", "error", "success")]
    [string]$VisualTaskState = "",
    [string]$RequireWindowTitle = "",
    [switch]$LeaveOpen
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "hub-capture-common.ps1")

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\..\..\..\.."))
}

$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
$outDir = Join-Path $RepoRoot "target\hub-visual-check"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

if ([string]::IsNullOrWhiteSpace($BinaryPath)) {
    $BinaryPath = Join-Path $RepoRoot "target\debug\zircon_hub.exe"
}

if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Hub binary not found at '$BinaryPath'. Build it first with: cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1"
}

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $OutputPath = Join-Path $outDir "hub-actual-$stamp.png"
}

$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)
New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName($OutputPath)) | Out-Null

if ($ConfigMode -eq "Isolated") {
    if ([string]::IsNullOrWhiteSpace($ConfigRoot)) {
        $ConfigRoot = Join-Path $outDir "config"
    }

    New-Item -ItemType Directory -Force -Path $ConfigRoot | Out-Null
    $env:LOCALAPPDATA = Join-Path $ConfigRoot "localappdata"
    $env:APPDATA = Join-Path $ConfigRoot "appdata"
    $env:ZIRCON_CONFIG_PATH = Join-Path $ConfigRoot "zircon-editor-config.json"
    New-Item -ItemType Directory -Force -Path $env:LOCALAPPDATA, $env:APPDATA | Out-Null
    if (-not (Test-Path -LiteralPath $env:ZIRCON_CONFIG_PATH)) {
        Set-Content -LiteralPath $env:ZIRCON_CONFIG_PATH -Value "{}" -Encoding UTF8
    }

    if ($WindowWidth -gt 0 -or $WindowHeight -gt 0) {
        if ($WindowWidth -le 0 -or $WindowHeight -le 0) {
            throw "WindowWidth and WindowHeight must be provided together."
        }

        $hubConfigDir = Join-Path $env:LOCALAPPDATA "ZirconHub"
        New-Item -ItemType Directory -Force -Path $hubConfigDir | Out-Null
        $hubConfigPath = Join-Path $hubConfigDir "config.toml"
        $toml = @"
[window]
position_x = $Left
position_y = $Top
width = $WindowWidth
height = $WindowHeight
maximized = false
"@
        Set-Content -LiteralPath $hubConfigPath -Value $toml -Encoding UTF8
    }
}

if (-not ("ZirconHubWindowCapture" -as [type])) {
    Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class ZirconHubWindowCapture {
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
    public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, UIntPtr dwExtraInfo);

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
}

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

function Resolve-CaptureWindowPosition {
    param(
        [int]$RequestedLeft,
        [int]$RequestedTop,
        [int]$WindowWidth,
        [int]$WindowHeight
    )

    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
    $resolvedLeft = [Math]::Max($bounds.Left, [Math]::Min($RequestedLeft, $bounds.Right - $WindowWidth))
    $resolvedTop = [Math]::Max($bounds.Top, [Math]::Min($RequestedTop, $bounds.Bottom - $WindowHeight))

    return [pscustomobject]@{
        Left = $resolvedLeft
        Top  = $resolvedTop
    }
}

$outputBase = Join-Path ([System.IO.Path]::GetDirectoryName($OutputPath)) ([System.IO.Path]::GetFileNameWithoutExtension($OutputPath))
$stdoutPath = "$outputBase.stdout.log"
$stderrPath = "$outputBase.stderr.log"
$previousVisualTaskState = $env:ZIRCON_HUB_VISUAL_TASK_STATE
$visualTaskStateChanged = -not [string]::IsNullOrWhiteSpace($VisualTaskState)
if ($visualTaskStateChanged) {
    $env:ZIRCON_HUB_VISUAL_TASK_STATE = $VisualTaskState
}
$process = $null
$frontendProcess = $null
$previousWebViewArguments = $null
$debugPort = Get-HubCaptureAvailableTcpPort

try {
    $frontendProcess = Start-HubCaptureFrontendDevServer -RepoRoot $RepoRoot -LogBasePath $outputBase
    $previousWebViewArguments = Set-HubCaptureWebViewDebugEnvironment -Port $debugPort
    $process = Start-Process -FilePath $BinaryPath -WorkingDirectory $RepoRoot -PassThru -WindowStyle Hidden -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath

    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    $selected = $null
    $fallbackSelected = $null

    do {
        Start-Sleep -Milliseconds 500
        $windows = New-Object System.Collections.Generic.List[object]
        $callback = [ZirconHubWindowCapture+EnumWindowsProc]{
            param([IntPtr] $hWnd, [IntPtr] $lParam)

            [uint32]$windowProcessId = 0
            [void][ZirconHubWindowCapture]::GetWindowThreadProcessId($hWnd, [ref]$windowProcessId)

            if ($windowProcessId -eq $process.Id -and [ZirconHubWindowCapture]::IsWindowVisible($hWnd)) {
                $rect = New-Object ZirconHubWindowCapture+RECT

                if ([ZirconHubWindowCapture]::GetWindowRect($hWnd, [ref]$rect)) {
                    $width = $rect.Right - $rect.Left
                    $height = $rect.Bottom - $rect.Top

                    if ($width -gt 100 -and $height -gt 100) {
                        $titleBuilder = New-Object System.Text.StringBuilder 256
                        [void][ZirconHubWindowCapture]::GetWindowText($hWnd, $titleBuilder, $titleBuilder.Capacity)
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

        [void][ZirconHubWindowCapture]::EnumWindows($callback, [IntPtr]::Zero)
        $selected = $windows | Where-Object { $_.Title -eq "Zircon Hub" } | Sort-Object Area -Descending | Select-Object -First 1
        if ($null -eq $selected) {
            $fallbackSelected = $windows | Sort-Object Area -Descending | Select-Object -First 1
        }
    } while ($null -eq $selected -and (Get-Date) -lt $deadline)

    if ($null -eq $selected) {
        $selected = $fallbackSelected
    }

    if ($null -eq $selected) {
        throw "No visible Zircon Hub window found for process $($process.Id)."
    }
    if (-not [string]::IsNullOrWhiteSpace($RequireWindowTitle) -and $selected.Title -ne $RequireWindowTitle) {
        throw "Expected window title '$RequireWindowTitle' for process $($process.Id), but selected '$($selected.Title)'."
    }

    $hwndTopmost = [IntPtr]::new(-1)
    $hwndNotTopmost = [IntPtr]::new(-2)
    $setPositionNoSizeFlags = 0x0040 -bor 0x0001
    $windowPosition = Resolve-CaptureWindowPosition -RequestedLeft $Left -RequestedTop $Top -WindowWidth $selected.Width -WindowHeight $selected.Height
    [void][ZirconHubWindowCapture]::ShowWindow($selected.Handle, 1)
    [void][ZirconHubWindowCapture]::SetWindowPos($selected.Handle, $hwndTopmost, $windowPosition.Left, $windowPosition.Top, 0, 0, $setPositionNoSizeFlags)
    [void][ZirconHubWindowCapture]::SetForegroundWindow($selected.Handle)
    Start-Sleep -Seconds 2

    $captureRect = New-Object ZirconHubWindowCapture+RECT
    [void][ZirconHubWindowCapture]::GetWindowRect($selected.Handle, [ref]$captureRect)
    $captureWidth = $captureRect.Right - $captureRect.Left
    $captureHeight = $captureRect.Bottom - $captureRect.Top
    if ($captureWidth -le 0 -or $captureHeight -le 0) {
        throw "Captured Hub window has invalid bounds before screenshot: ${captureWidth}x${captureHeight}."
    }

    $clicks = @(
        [pscustomobject]@{
            X                 = $ClickX
            Y                 = $ClickY
            DelayMilliseconds = $ClickDelayMilliseconds
        },
        [pscustomobject]@{
            X                 = $SecondClickX
            Y                 = $SecondClickY
            DelayMilliseconds = $SecondClickDelayMilliseconds
        }
    )

    foreach ($click in $clicks) {
        if ($click.X -ge 0 -and $click.Y -ge 0) {
            [void][ZirconHubWindowCapture]::SetCursorPos($captureRect.Left + $click.X, $captureRect.Top + $click.Y)
            Start-Sleep -Milliseconds 120
            [ZirconHubWindowCapture]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
            Start-Sleep -Milliseconds 80
            [ZirconHubWindowCapture]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
            Start-Sleep -Milliseconds $click.DelayMilliseconds
            [void][ZirconHubWindowCapture]::ShowWindow($selected.Handle, 1)
            [void][ZirconHubWindowCapture]::SetWindowPos($selected.Handle, $hwndTopmost, $windowPosition.Left, $windowPosition.Top, 0, 0, $setPositionNoSizeFlags)
            [void][ZirconHubWindowCapture]::SetForegroundWindow($selected.Handle)
            Start-Sleep -Milliseconds 300
            $nextRect = New-Object ZirconHubWindowCapture+RECT
            if ([ZirconHubWindowCapture]::GetWindowRect($selected.Handle, [ref]$nextRect)) {
                $nextWidth = $nextRect.Right - $nextRect.Left
                $nextHeight = $nextRect.Bottom - $nextRect.Top

                if ($nextWidth -gt 0 -and $nextHeight -gt 0) {
                    $captureRect = $nextRect
                    $captureWidth = $nextWidth
                    $captureHeight = $nextHeight
                }
            } elseif ($process.HasExited) {
                throw "Hub process exited after click at $($click.X),$($click.Y)."
            }
        }
    }

    if (-not [string]::IsNullOrWhiteSpace($WebViewClickText)) {
        Invoke-HubCaptureWebViewAction -Port $debugPort -Action "click-text" -Text $WebViewClickText -WaitSeconds $WaitSeconds -DelayMilliseconds $WebViewClickDelayMilliseconds -Title "Zircon Hub"
    }

    if (-not [string]::IsNullOrWhiteSpace($RequireWebViewText)) {
        $textAction = if ($RequireWebViewText.Contains("|||")) { "wait-any-text" } else { "wait-text" }
        Invoke-HubCaptureWebViewAction -Port $debugPort -Action $textAction -Text $RequireWebViewText -WaitSeconds $WaitSeconds -DelayMilliseconds 100 -Title "Zircon Hub"
    }

    if ($captureWidth -le 0 -or $captureHeight -le 0) {
        throw "Captured Hub window has invalid bounds before screenshot: ${captureWidth}x${captureHeight}."
    }

    if ($process.HasExited) {
        throw "Hub process exited before screenshot capture. ExitCode=$($process.ExitCode)."
    }

    Invoke-HubCaptureWebViewScreenshot -Port $debugPort -OutputPath $OutputPath -WaitSeconds $WaitSeconds -Title "Zircon Hub"
    $imageSize = Get-HubCaptureImageSize -Path $OutputPath

    [void][ZirconHubWindowCapture]::SetWindowPos($selected.Handle, $hwndNotTopmost, $captureRect.Left, $captureRect.Top, 0, 0, $setPositionNoSizeFlags)

    [pscustomobject]@{
        Path       = $OutputPath
        Title      = $selected.Title
        Width      = $imageSize.Width
        Height     = $imageSize.Height
        ProcessId  = $process.Id
        ConfigMode = $ConfigMode
    } | Format-List
} finally {
    if ($process -and -not $process.HasExited -and -not $LeaveOpen) {
        $process.CloseMainWindow() | Out-Null
        Start-Sleep -Milliseconds 500

        if (-not $process.HasExited) {
            $process.Kill()
        }
    }

    if ($visualTaskStateChanged) {
        if ($null -eq $previousVisualTaskState) {
            Remove-Item Env:\ZIRCON_HUB_VISUAL_TASK_STATE -ErrorAction SilentlyContinue
        } else {
            $env:ZIRCON_HUB_VISUAL_TASK_STATE = $previousVisualTaskState
        }
    }

    Restore-HubCaptureWebViewDebugEnvironment -PreviousValue $previousWebViewArguments
    Stop-HubCaptureFrontendDevServer -Process $frontendProcess
}
