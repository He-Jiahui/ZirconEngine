[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$filter = "runtime92_owned_descriptors_recovery_batch_"
$arraySource = Join-Path $repoRoot "tools/runtime92_owned_array_layer_descriptors_model.rs"
$cubeSource = Join-Path $repoRoot "tools/runtime92_owned_cubemap_face_descriptors_model.rs"
$arrayBinary = Join-Path $repoRoot "target/runtime92-owned-array-layer-descriptors-model.exe"
$cubeBinary = Join-Path $repoRoot "target/runtime92-owned-cubemap-face-descriptors-model.exe"

Push-Location $repoRoot
try {
    cargo +1.94.1 test -p zircon_runtime --lib --locked --release --jobs 1 $filter -- `
        --nocapture --test-threads=1
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    python -m unittest `
        tools.tests.test_runtime92_owned_array_layer_descriptors_performance_contract `
        tools.tests.test_runtime92_owned_cubemap_face_descriptors_performance_contract -v
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $arrayBinary) | Out-Null
    & rustc +1.94.1 --edition=2021 -C opt-level=3 $arraySource -o $arrayBinary
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    & rustc +1.94.1 --edition=2021 -C opt-level=3 $cubeSource -o $cubeBinary
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    & $arrayBinary
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    & $cubeBinary
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & rustfmt +1.94.1 --edition 2021 --check `
        zircon_runtime/src/asset/assets/texture/array_asset.rs `
        zircon_runtime/src/asset/assets/texture/cube_asset.rs `
        tools/runtime92_owned_array_layer_descriptors_model.rs `
        tools/runtime92_owned_cubemap_face_descriptors_model.rs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    git diff --check -- `
        docs/plans/optimize/zircon_runtime/92/2026-08-28-owned-array-layer-descriptors.md `
        docs/plans/optimize/zircon_runtime/92/2026-08-28-owned-cubemap-face-descriptors.md `
        tools/runtime92_owned_array_layer_descriptors_model.rs `
        tools/runtime92_owned_cubemap_face_descriptors_model.rs `
        tools/zircon-validation-runtime92-owned-descriptors-recovery-batch.ps1 `
        zircon_runtime/src/asset/assets/texture/array_asset.rs `
        zircon_runtime/src/asset/assets/texture/cube_asset.rs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME92_OWNED_DESCRIPTORS_RECOVERY_BATCH_PASS rust_tests=2 python_contracts=8 release_models=2"
