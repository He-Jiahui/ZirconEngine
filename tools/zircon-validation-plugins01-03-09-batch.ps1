[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tasks = @(
    [pscustomobject]@{
        Name = "plugins01-sdk-lib"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_plugin_sdk", "--locked",
            "--release", "--jobs", "1", "--lib", "--", "--nocapture",
            "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "plugins01-sdk-doctest"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_plugin_sdk", "--locked",
            "--release", "--jobs", "1", "--doc"
        )
    },
    [pscustomobject]@{
        Name = "plugins03-native-window-lib"
        Arguments = @(
            "+1.94.1", "test", "-p",
            "zircon_plugin_native_window_hosting_editor", "--features", "editor",
            "--locked", "--release", "--jobs", "1", "--lib", "--",
            "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "plugins09-particle-snapshot"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_plugin_particles_runtime",
            "--locked", "--release", "--jobs", "1",
            "tests::snapshot::particle_snapshot_shared_clone_release_benchmark",
            "--", "--exact", "--ignored", "--nocapture", "--test-threads=1"
        )
    }
)

Push-Location $repoRoot
try {
    foreach ($task in $tasks) {
        Write-Output "PLUGINS01_03_09_BATCH_TASK_START name=$($task.Name)"
        $cargoArguments = $task.Arguments
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "PLUGINS01_03_09_BATCH_TASK_PASS name=$($task.Name)"
    }
}
finally {
    Pop-Location
}

Write-Output "PLUGINS01_03_09_BATCH_PASS task_count=$($tasks.Count)"
