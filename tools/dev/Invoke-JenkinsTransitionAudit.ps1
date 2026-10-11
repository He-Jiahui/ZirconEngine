[CmdletBinding()]
param(
    [string]$Repository = 'E:\Git\ZirconEngine',
    [Parameter(Mandatory)][string]$Python,
    [switch]$Deep,
    [switch]$Notify
)
$ErrorActionPreference = 'Stop'
$env:PYTHONDONTWRITEBYTECODE = '1'
$auditArguments = @('-B', (Join-Path $Repository 'tools\dev\jenkins_transition_monitor.py'), '--repo', $Repository)
if ($Deep) { $auditArguments += '--deep' }
& $Python @auditArguments
$auditExit = $LASTEXITCODE
if ($auditExit -eq 0 -and $Notify) {
    $latestPath = 'E:\cargo-targets\zircon-local\reports\jenkins-transition-monitor\latest.json'
    $latest = Get-Content -LiteralPath $latestPath -Raw | ConvertFrom-Json
    if ($latest.attention.notify) {
        # Local desktop notice only. No webhook, email, chat message, or credential.
        Add-Type -AssemblyName System.Windows.Forms
        Add-Type -AssemblyName System.Drawing
        $notice = New-Object System.Windows.Forms.NotifyIcon
        try {
            $notice.Icon = [System.Drawing.SystemIcons]::Warning
            $notice.Visible = $true
            $notice.BalloonTipTitle = 'ZirconEngine audit changed'
            $notice.BalloonTipText = "Jenkins healthy: $($latest.jenkins.healthy); findings: $($latest.findings.Count). Report: $latestPath"
            $notice.ShowBalloonTip(8000)
            Start-Sleep -Seconds 8
        } finally {
            $notice.Dispose()
        }
    }
}
exit $auditExit
