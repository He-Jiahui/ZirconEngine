[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tasks = @(
    [pscustomobject]@{
        Name = "plugins06-neural-only"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_app", "--lib",
            "--no-default-features", "--features", "first-party-neural-editor-plugin",
            "--locked", "--release", "--jobs", "1",
            "entry::first_party_editor_plugins::tests::app_composition_projects_selected_neural_editor_provider",
            "--", "--exact", "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "plugins06-navigation-only"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_app", "--lib",
            "--no-default-features", "--features",
            "first-party-navigation-editor-plugin", "--locked", "--release",
            "--jobs", "1",
            "entry::first_party_editor_plugins::tests::app_composition_projects_selected_navigation_editor_provider",
            "--", "--exact", "--nocapture", "--test-threads=1"
        )
    }
)

Push-Location $repoRoot
try {
    foreach ($task in $tasks) {
        Write-Output "PLUGINS06_BATCH_TASK_START name=$($task.Name)"
        $cargoArguments = $task.Arguments
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "PLUGINS06_BATCH_TASK_PASS name=$($task.Name)"
    }
}
finally {
    Pop-Location
}

Write-Output "PLUGINS06_BATCH_PASS task_count=$($tasks.Count)"
