$ErrorActionPreference = "Stop"

function Get-HubCaptureAvailableTcpPort {
    $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, 0)
    $listener.Start()
    try {
        return [int]$listener.LocalEndpoint.Port
    } finally {
        $listener.Stop()
    }
}

function Test-HubCaptureFrontendDevServer {
    try {
        $response = Invoke-WebRequest -Uri "http://127.0.0.1:1420" -UseBasicParsing -TimeoutSec 3
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

function Start-HubCaptureFrontendDevServer {
    param(
        [string]$RepoRoot,
        [string]$LogBasePath
    )

    if (Test-HubCaptureFrontendDevServer) {
        return $null
    }

    $hubRoot = Join-Path $RepoRoot "zircon_hub"
    $viteBin = Join-Path $hubRoot "node_modules\vite\bin\vite.js"
    if (-not (Test-Path -LiteralPath $viteBin)) {
        throw "Vite dev server entry was not found at '$viteBin'. Run npm install in zircon_hub first."
    }

    $stdoutPath = "$LogBasePath.vite.stdout.log"
    $stderrPath = "$LogBasePath.vite.stderr.log"
    Remove-Item -LiteralPath $stdoutPath, $stderrPath -Force -ErrorAction SilentlyContinue

    $process = Start-Process `
        -FilePath "node.exe" `
        -ArgumentList @($viteBin, "--host", "127.0.0.1", "--port", "1420") `
        -WorkingDirectory $hubRoot `
        -PassThru `
        -WindowStyle Hidden `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath

    $deadline = (Get-Date).AddSeconds(20)
    do {
        if ($process.HasExited) {
            throw "Vite dev server exited before Hub capture could start. See '$stderrPath'."
        }
        if (Test-HubCaptureFrontendDevServer) {
            return $process
        }
        Start-Sleep -Milliseconds 250
    } while ((Get-Date) -lt $deadline)

    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
    throw "Timed out waiting for Vite dev server at http://127.0.0.1:1420. See '$stderrPath'."
}

function Stop-HubCaptureFrontendDevServer {
    param([System.Diagnostics.Process]$Process)

    if ($Process -and -not $Process.HasExited) {
        Stop-Process -Id $Process.Id -Force
    }
}

function Invoke-HubCaptureWebViewScreenshot {
    param(
        [int]$Port,
        [string]$OutputPath,
        [int]$WaitSeconds = 20,
        [string]$Title = "Zircon Hub"
    )

    $captureScript = Join-Path $PSScriptRoot "capture-webview-screenshot.mjs"
    if (-not (Test-Path -LiteralPath $captureScript)) {
        throw "WebView capture helper not found at '$captureScript'."
    }

    $nodeOutput = & node.exe $captureScript --port $Port --output $OutputPath --wait-ms ([Math]::Max(1, $WaitSeconds) * 1000) --title $Title 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "WebView screenshot capture failed for '$OutputPath': $($nodeOutput -join "`n")"
    }
    if ($nodeOutput) {
        $nodeOutput | ForEach-Object { Write-Host $_ }
    }
}

function Invoke-HubCaptureWebViewAction {
    param(
        [int]$Port,
        [ValidateSet("scroll-text", "click-text", "wait-text", "wait-any-text")]
        [string]$Action,
        [string]$Text,
        [int]$WaitSeconds = 20,
        [int]$DelayMilliseconds = 900,
        [string]$Title = "Zircon Hub"
    )

    $actionScript = Join-Path $PSScriptRoot "invoke-webview-action.mjs"
    if (-not (Test-Path -LiteralPath $actionScript)) {
        throw "WebView action helper not found at '$actionScript'."
    }

    $nodeOutput = & node.exe $actionScript --port $Port --action $Action --text $Text --wait-ms ([Math]::Max(1, $WaitSeconds) * 1000) --title $Title 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "WebView action '$Action' failed for '$Text': $($nodeOutput -join "`n")"
    }
    if ($nodeOutput) {
        $nodeOutput | ForEach-Object { Write-Host $_ }
    }

    Start-Sleep -Milliseconds $DelayMilliseconds
}

function Get-HubCaptureImageSize {
    param([string]$Path)

    $bitmap = [System.Drawing.Bitmap]::FromFile($Path)
    try {
        return [pscustomobject]@{
            Width = $bitmap.Width
            Height = $bitmap.Height
        }
    } finally {
        $bitmap.Dispose()
    }
}

function Set-HubCaptureWebViewDebugEnvironment {
    param([int]$Port)

    $previous = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
    $debugArg = "--remote-debugging-port=$Port"
    if ([string]::IsNullOrWhiteSpace($previous)) {
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $debugArg
    } else {
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "$previous $debugArg"
    }

    return $previous
}

function Restore-HubCaptureWebViewDebugEnvironment {
    param([AllowNull()][string]$PreviousValue)

    if ($null -eq $PreviousValue) {
        Remove-Item Env:\WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
    } else {
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $PreviousValue
    }
}
