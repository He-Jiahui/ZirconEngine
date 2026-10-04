[CmdletBinding()]
param(
    [string]$RepoRoot = "",
    [string]$BinaryPath = "",
    [string]$OutputDir = "",
    [int]$WaitSeconds = 25,
    [int]$Left = 20,
    [int]$Top = 20,
    [int]$WindowWidth = 1568,
    [int]$WindowHeight = 1003,
    [string[]]$OnlyStates = @()
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\..\..\..\.."))
}

$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)

if ([string]::IsNullOrWhiteSpace($BinaryPath)) {
    $BinaryPath = Join-Path $RepoRoot "target\debug\zircon_hub.exe"
}
$BinaryPath = [System.IO.Path]::GetFullPath($BinaryPath)

if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Hub binary not found at '$BinaryPath'. Build it before capturing the visual state matrix."
}

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $RepoRoot "target\hub-visual-check\tauri-visual-state-matrix"
}
$OutputDir = [System.IO.Path]::GetFullPath($OutputDir)
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$captureScript = Join-Path $PSScriptRoot "capture-hub-window.ps1"
if (-not (Test-Path -LiteralPath $captureScript)) {
    throw "Capture helper not found at '$captureScript'."
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

function New-VisualProject {
    param(
        [string]$ProjectRoot,
        [string]$Name
    )

    New-Item -ItemType Directory -Force -Path $ProjectRoot, (Join-Path $ProjectRoot "assets"), (Join-Path $ProjectRoot "scenes") | Out-Null
    $projectGuid = [guid]::NewGuid().ToString()
    $manifest = @"
name = $(ConvertTo-TomlString $Name)
format_version = 3
project_guid = $(ConvertTo-TomlString $projectGuid)
default_scene = "res://scenes/main.scene.toml"
asset_roots = ["assets"]
library_version = 1
"@
    Set-Content -LiteralPath (Join-Path $ProjectRoot "zircon-project.toml") -Value $manifest -Encoding UTF8
    return $projectGuid
}

function ConvertTo-RecentProjectToml {
    param(
        [string]$Name,
        [string]$ProjectPath,
        [string]$ProjectGuid,
        [long]$LastOpenedUnixMs
    )

    return @"

[[recent_projects]]
path = $(ConvertTo-TomlString $ProjectPath)
last_opened_unix_ms = $LastOpenedUnixMs
[recent_projects.summary]
name = $(ConvertTo-TomlString $Name)
default_scene = "res://scenes/main.scene.toml"
format_version = 3
project_guid = $(ConvertTo-TomlString $ProjectGuid)
"@
}

function Initialize-VisualStateConfig {
    param(
        [string]$ConfigRoot,
        [string]$Page,
        [string]$ProjectSubpage,
        [string]$ProjectViewMode,
        [bool]$IncludeProject
    )

    $localAppData = Join-Path $ConfigRoot "localappdata"
    $appData = Join-Path $ConfigRoot "appdata"
    $userProfile = Join-Path $ConfigRoot "userprofile"
    $hubConfigDir = Join-Path $localAppData "ZirconHub"
    $projectRoot = Join-Path $ConfigRoot "C\ZirconProjects"
    $buildOutput = Join-Path $ConfigRoot "build-output"
    $deviceRoot = Join-Path $ConfigRoot "device"
    $engineRoot = Join-Path $ConfigRoot "engines"
    $engineId = "zircon-1.8.2"
    $engineSourceDir = Join-Path $engineRoot $engineId
    $engineOutputDir = Join-Path $buildOutput $engineId
    $editorConfigPath = Join-Path $ConfigRoot "zircon-editor-config.json"

    New-Item -ItemType Directory -Force -Path $localAppData, $appData, $userProfile, $hubConfigDir, $projectRoot, $buildOutput, $deviceRoot, $engineSourceDir, $engineOutputDir | Out-Null
    Set-Content -LiteralPath $editorConfigPath -Value "{}" -Encoding UTF8

    $nowMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $recentText = ""
    $metadataText = ""
    $selectedProjectText = ""

    if ($IncludeProject) {
        $projectPath = Join-Path $projectRoot "Elysium"
        $projectGuid = New-VisualProject -ProjectRoot $projectPath -Name "Elysium Chronicles"
        $projectPath = [System.IO.Path]::GetFullPath($projectPath)
        $metadataKey = ConvertTo-ProjectMetadataKey $projectPath
        $recentText = ConvertTo-RecentProjectToml -Name "Elysium Chronicles" -ProjectPath $projectPath -ProjectGuid $projectGuid -LastOpenedUnixMs $nowMs
        $metadataText = @"

[project_metadata.$(ConvertTo-TomlString $metadataKey)]
pinned = true
engine_id = $(ConvertTo-TomlString $engineId)
last_selected_template = "renderable-empty"
"@
        $selectedProjectText = "selected_project_path = $(ConvertTo-TomlString $projectPath)"
    }

    $hubConfigPath = Join-Path $hubConfigDir "config.toml"
    $toml = @"
active_engine_id = $(ConvertTo-TomlString $engineId)

[settings]
python_path = "python"
cargo_path = "cargo"
rustup_path = "rustup"
default_project_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($projectRoot)))
default_source_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($engineSourceDir)))
default_build_output_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($engineOutputDir)))
default_device_install_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($deviceRoot)))
language = "English"
build_profile = "Debug"
jobs = 1

$recentText
$metadataText

[[engines]]
id = $(ConvertTo-TomlString $engineId)
display_name = "Zircon Engine 1.8.2"
source_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($engineSourceDir)))
output_dir = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($engineOutputDir)))
last_build_unix_ms = 0
build_history = []

[window]
position_x = $Left
position_y = $Top
width = $WindowWidth
height = $WindowHeight
maximized = false

[runtime]
selected_page = $(ConvertTo-TomlString $Page)
project_subpage = $(ConvertTo-TomlString $ProjectSubpage)
project_filter = "all"
project_sort = "last-modified"
project_view_mode = $(ConvertTo-TomlString $ProjectViewMode)
search_query = ""
selected_template_id = "renderable-empty"
new_project_location = $(ConvertTo-TomlString ([System.IO.Path]::GetFullPath($projectRoot)))
new_project_engine_id = $(ConvertTo-TomlString $engineId)
$selectedProjectText
"@
    Set-Content -LiteralPath $hubConfigPath -Value $toml -Encoding UTF8

    return [pscustomobject]@{
        ConfigRoot = $ConfigRoot
        LocalAppData = $localAppData
        AppData = $appData
        UserProfile = $userProfile
        EditorConfigPath = $editorConfigPath
    }
}

function Invoke-VisualStateCapture {
    param(
        [string]$Name,
        [string]$Page,
        [string]$ProjectSubpage = "dashboard",
        [string]$ProjectViewMode = "grid",
        [bool]$IncludeProject = $true,
        [string]$VisualTaskState = "",
        [int]$ClickX = -1,
        [int]$ClickY = -1,
        [string]$WebViewClickText = "",
        [string]$RequireWebViewText = ""
    )

    if ([string]::IsNullOrWhiteSpace($RequireWebViewText)) {
        throw "Visual state capture '$Name' must require state-specific WebView text before capture."
    }

    $configRoot = Join-Path $OutputDir "config-$Name"
    $config = Initialize-VisualStateConfig `
        -ConfigRoot $configRoot `
        -Page $Page `
        -ProjectSubpage $ProjectSubpage `
        -ProjectViewMode $ProjectViewMode `
        -IncludeProject $IncludeProject

    $previousLocalAppData = $env:LOCALAPPDATA
    $previousAppData = $env:APPDATA
    $previousUserProfile = $env:USERPROFILE
    $previousEditorConfig = $env:ZIRCON_CONFIG_PATH
    try {
        $env:LOCALAPPDATA = $config.LocalAppData
        $env:APPDATA = $config.AppData
        $env:USERPROFILE = $config.UserProfile
        $env:ZIRCON_CONFIG_PATH = $config.EditorConfigPath

        $outputPath = Join-Path $OutputDir "hub-state-$Name.png"
        $requireAccent = $Name -ne "loading"
        $captureSucceeded = $false
        $lastCaptureError = $null
        for ($attempt = 0; $attempt -lt 3; $attempt += 1) {
            Remove-Item -LiteralPath $outputPath -Force -ErrorAction SilentlyContinue
            try {
                & $captureScript `
                    -RepoRoot $RepoRoot `
                    -BinaryPath $BinaryPath `
                    -OutputPath $outputPath `
                    -ConfigMode Current `
                    -WaitSeconds $WaitSeconds `
                    -Left $Left `
                    -Top $Top `
                    -ClickX $ClickX `
                    -ClickY $ClickY `
                    -ClickDelayMilliseconds 900 `
                    -WebViewClickText $WebViewClickText `
                    -WebViewClickDelayMilliseconds 900 `
                    -RequireWebViewText $RequireWebViewText `
                    -VisualTaskState $VisualTaskState `
                    -RequireWindowTitle "Zircon Hub" | Out-Host

                if (-not (Test-HubScreenshotMostlyWhite -Path $outputPath) -and (-not $requireAccent -or -not (Test-HubScreenshotMissingAccent -Path $outputPath))) {
                    $captureSucceeded = $true
                    break
                }

                $lastCaptureError = "Screenshot for '$Name' failed visual sanity checks."
            } catch {
                $lastCaptureError = $_
            }

            if ($attempt -lt 2) {
                Start-Sleep -Seconds 8
            }
        }

        if (-not $captureSucceeded -and $null -ne $lastCaptureError) {
            throw $lastCaptureError
        }

        if (-not (Test-Path -LiteralPath $outputPath)) {
            throw "Expected screenshot was not written: $outputPath"
        }
        if (Test-HubScreenshotMostlyWhite -Path $outputPath) {
            throw "Screenshot for '$Name' is mostly white and cannot be trusted: $outputPath"
        }
        if ($requireAccent -and (Test-HubScreenshotMissingAccent -Path $outputPath)) {
            throw "Screenshot for '$Name' does not contain enough Hub accent pixels and cannot be trusted: $outputPath"
        }

        return [pscustomobject]@{
            Name = $Name
            Path = [System.IO.Path]::GetFullPath($outputPath)
        }
    } finally {
        # WebView2 child processes can outlive the Tauri parent briefly. Give the
        # next isolated state a clean process/profile handoff before relaunching.
        Start-Sleep -Seconds 4

        if ($null -eq $previousLocalAppData) {
            Remove-Item Env:\LOCALAPPDATA -ErrorAction SilentlyContinue
        } else {
            $env:LOCALAPPDATA = $previousLocalAppData
        }

        if ($null -eq $previousAppData) {
            Remove-Item Env:\APPDATA -ErrorAction SilentlyContinue
        } else {
            $env:APPDATA = $previousAppData
        }

        if ($null -eq $previousUserProfile) {
            Remove-Item Env:\USERPROFILE -ErrorAction SilentlyContinue
        } else {
            $env:USERPROFILE = $previousUserProfile
        }

        if ($null -eq $previousEditorConfig) {
            Remove-Item Env:\ZIRCON_CONFIG_PATH -ErrorAction SilentlyContinue
        } else {
            $env:ZIRCON_CONFIG_PATH = $previousEditorConfig
        }
    }
}

function Test-HubScreenshotMostlyWhite {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return $false
    }

    $bitmap = [System.Drawing.Bitmap]::FromFile($Path)
    try {
        $stepX = [Math]::Max(1, [int][Math]::Floor($bitmap.Width / 32))
        $stepY = [Math]::Max(1, [int][Math]::Floor($bitmap.Height / 32))
        $whiteLike = 0
        $samples = 0

        for ($x = 0; $x -lt $bitmap.Width; $x += $stepX) {
            for ($y = 0; $y -lt $bitmap.Height; $y += $stepY) {
                $pixel = $bitmap.GetPixel($x, $y)
                $samples += 1
                if ($pixel.R -gt 245 -and $pixel.G -gt 245 -and $pixel.B -gt 245) {
                    $whiteLike += 1
                }
            }
        }

        return $samples -gt 0 -and (($whiteLike / $samples) -gt 0.92)
    } finally {
        $bitmap.Dispose()
    }
}

function Test-HubScreenshotMissingAccent {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return $false
    }

    $bitmap = [System.Drawing.Bitmap]::FromFile($Path)
    try {
        $stepX = [Math]::Max(1, [int][Math]::Floor($bitmap.Width / 96))
        $stepY = [Math]::Max(1, [int][Math]::Floor($bitmap.Height / 96))
        $accentLike = 0
        $samples = 0

        for ($x = 0; $x -lt $bitmap.Width; $x += $stepX) {
            for ($y = 0; $y -lt $bitmap.Height; $y += $stepY) {
                $pixel = $bitmap.GetPixel($x, $y)
                $samples += 1
                $tealLike = $pixel.G -gt 95 -and $pixel.B -gt 80 -and $pixel.R -lt 100
                $redLike = $pixel.R -gt 120 -and $pixel.G -lt 100 -and $pixel.B -lt 100
                if ($tealLike -or $redLike) {
                    $accentLike += 1
                }
            }
        }

        return $samples -gt 0 -and (($accentLike / $samples) -lt 0.002)
    } finally {
        $bitmap.Dispose()
    }
}

function Test-VisualStateSelected {
    param([string]$Name)

    return $OnlyStates.Count -eq 0 -or $OnlyStates -contains $Name
}

$captures = @()
if (Test-VisualStateSelected "editor") { $captures += Invoke-VisualStateCapture -Name "editor" -Page "editor" -RequireWebViewText "Launch Target" }
if (Test-VisualStateSelected "assets") { $captures += Invoke-VisualStateCapture -Name "assets" -Page "assets" -RequireWebViewText "Assets Catalog" }
if (Test-VisualStateSelected "builds") { $captures += Invoke-VisualStateCapture -Name "builds" -Page "builds" -RequireWebViewText "Build Workflow" }
if (Test-VisualStateSelected "plugins") { $captures += Invoke-VisualStateCapture -Name "plugins" -Page "plugins" -RequireWebViewText "Plugins Catalog" }
if (Test-VisualStateSelected "cloud") { $captures += Invoke-VisualStateCapture -Name "cloud" -Page "cloud" -RequireWebViewText "Package Outputs" }
if (Test-VisualStateSelected "team") { $captures += Invoke-VisualStateCapture -Name "team" -Page "team" -RequireWebViewText "Team Members" }
if (Test-VisualStateSelected "learn") { $captures += Invoke-VisualStateCapture -Name "learn" -Page "learn" -RequireWebViewText "Learn Catalog" }
if (Test-VisualStateSelected "settings") { $captures += Invoke-VisualStateCapture -Name "settings" -Page "settings" -RequireWebViewText "Build Defaults" }
if (Test-VisualStateSelected "source-engine-popup") { $captures += Invoke-VisualStateCapture -Name "source-engine-popup" -Page "projects" -WebViewClickText "Zircon Engine 1.8.2" -RequireWebViewText "Manage engines" }
if (Test-VisualStateSelected "user-menu") { $captures += Invoke-VisualStateCapture -Name "user-menu" -Page "projects" -WebViewClickText "My Account" -RequireWebViewText "Preferences" }
if (Test-VisualStateSelected "project-browser-empty") { $captures += Invoke-VisualStateCapture -Name "project-browser-empty" -Page "projects" -ProjectSubpage "project-browser" -ProjectViewMode "list" -IncludeProject $false -RequireWebViewText "No projects found" }
if (Test-VisualStateSelected "loading") { $captures += Invoke-VisualStateCapture -Name "loading" -Page "builds" -VisualTaskState "loading" -RequireWebViewText "Loading Hub state" }
if (Test-VisualStateSelected "error") { $captures += Invoke-VisualStateCapture -Name "error" -Page "builds" -VisualTaskState "error" -RequireWebViewText "Visual verification error state" }

$captures | Format-Table -AutoSize
