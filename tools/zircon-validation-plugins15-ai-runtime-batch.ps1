[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$benchmarks = @(
    "manager::state::optimization_tests::immutable_compiled_tree_generation_release_benchmark_evidence",
    "perception::stimuli::ordered_snapshot_tests::ordered_perception_stimuli_release_benchmark_evidence",
    "perception::scan::sampling_tests::single_pass_perception_sampling_release_benchmark_evidence",
    "manager::snapshot::optimization_tests::targeted_debug_snapshot_release_benchmark_evidence"
)

Push-Location $repoRoot
try {
    foreach ($benchmark in $benchmarks) {
        Write-Output "PLUGINS15_BATCH_BENCHMARK_START name=$benchmark"
        $cargoArguments = @(
            "+1.94.1",
            "test",
            "-p",
            "zircon_plugin_ai_runtime",
            "--locked",
            "--release",
            "--jobs",
            "1",
            $benchmark,
            "--",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1"
        )
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "PLUGINS15_BATCH_BENCHMARK_PASS name=$benchmark"
    }
}
finally {
    Pop-Location
}

Write-Output "PLUGINS15_BATCH_PASS benchmark_count=$($benchmarks.Count)"
