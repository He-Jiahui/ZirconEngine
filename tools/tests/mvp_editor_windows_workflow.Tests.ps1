Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$workflowPath = Join-Path $PSScriptRoot '..\..\.github\workflows\mvp-editor-windows.yml'

if (-not (Test-Path -LiteralPath $workflowPath -PathType Leaf)) {
    throw "MVP Windows workflow is missing: $workflowPath"
}

$workflow = Get-Content -LiteralPath $workflowPath -Raw -Encoding UTF8

function Assert-WorkflowMatch {
    param(
        [Parameter(Mandatory)][string]$Pattern,
        [Parameter(Mandatory)][string]$Message
    )

    if ($workflow -notmatch $Pattern) {
        throw $Message
    }
}

Assert-WorkflowMatch '(?m)^name:\s*MVP Editor Windows\s*$' 'Workflow must have the stable MVP Windows name.'
Assert-WorkflowMatch '(?ms)^jobs:\s*\r?\n\s*mvp-editor-windows:' 'Workflow must define the dedicated MVP Windows job.'
Assert-WorkflowMatch 'runs-on:\s*windows-latest' 'MVP job must run on windows-latest.'
Assert-WorkflowMatch 'actions/checkout@v5' 'MVP job must check out the source.'
Assert-WorkflowMatch 'dtolnay/rust-toolchain@stable' 'MVP job must use the stable Rust toolchain.'
Assert-WorkflowMatch 'Swatinem/rust-cache@v2' 'MVP job must cache Cargo dependencies.'

Assert-WorkflowMatch 'cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked' 'MVP job must build the staged editor profile.'
Assert-WorkflowMatch 'cargo build -p zircon_app --bin zircon_runtime --no-default-features --features target-client --locked' 'MVP job must build the staged runtime profile.'
Assert-WorkflowMatch 'core::project::tests::template_creation::renderable_empty_template_has_the_f2_camera_cube_and_sun_contract\s+--locked\s+--\s+--exact' 'F1 must use the full libtest id before --exact.'
Assert-WorkflowMatch 'core::project::tests::template_creation::template_creation_rebuilds_regenerable_asset_state_from_source_after_deletion\s+--locked\s+--\s+--exact' 'F1 must cover rebuilding deleted derived asset state from source.'
Assert-WorkflowMatch 'core::project::tests::template_creation::template_creation_recovers_a_corrupt_persisted_registry_from_source_metadata\s+--locked\s+--\s+--exact' 'F1 must cover corrupt persisted registry recovery from source metadata.'
Assert-WorkflowMatch 'dynamic_api::session::tests::foundation_render::render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down\s+--locked\s+--\s+--exact' 'F2 must use the full libtest id before --exact.'
Assert-WorkflowMatch 'tests::workbench::project::document_roundtrip::editor_project_document_roundtrips_world_and_workspace\s+--locked\s+--\s+--exact' 'F3 must use the full libtest id before --exact.'
Assert-WorkflowMatch 'f4_project_authoring_survives_full_application_restart' 'MVP job must cover the F4 application authoring restart contract.'
Assert-WorkflowMatch 'cargo test -p zircon_app --test editor_mvp_authoring --no-default-features --features target-editor-host --locked' 'MVP job must run the F4 restart contract through the current App composition integration target.'
Assert-WorkflowMatch "Select-String -LiteralPath test-results/f4-authoring\.log -SimpleMatch 'test result: ok\. 1 passed; 0 failed' -Quiet" 'F4 must reject a zero-test or multi-test exact result.'
Assert-WorkflowMatch 'Stage-MvpProducts\.ps1' 'MVP job must stage source-bound product inputs rather than consume the repository target directory directly.'
Assert-WorkflowMatch 'Invoke-MvpAcceptance\.ps1' 'MVP job must validate staged product evidence through the acceptance driver.'
Assert-WorkflowMatch 'mvp-authoring-automation\.json' 'MVP job must exercise the source-bound normal authoring binding request.'
Assert-WorkflowMatch 'mvp-reopen-automation\.json' 'MVP job must exercise the independent source-bound persisted-state reopen request.'
Assert-WorkflowMatch 'RequireF5Evidence' 'MVP job must require the indivisible F5 creation, authoring, reopen, and visual-evidence contract.'
Assert-WorkflowMatch 'profile-contract-summary\.json' 'MVP job must materialize the profile build summary consumed by F5 acceptance.'
Assert-WorkflowMatch 'workspace-summary\.json' 'MVP job must materialize the focused workspace build/test summary consumed by F5 acceptance.'
Assert-WorkflowMatch "summary_kind\s*=\s*'profile-contract'" 'Profile evidence must declare its canonical summary kind.'
Assert-WorkflowMatch "summary_kind\s*=\s*'workspace'" 'Workspace evidence must declare its canonical summary kind.'
Assert-WorkflowMatch 'source_fingerprint\s*=\s*\$env:GITHUB_SHA' 'Build summaries must bind to the checked-out source fingerprint.'
Assert-WorkflowMatch '-ProfileContractSummaryPath\s+\$profileContractSummaryPath' 'F5 acceptance must receive the explicit profile summary input.'
Assert-WorkflowMatch '-WorkspaceSummaryPath\s+\$workspaceSummaryPath' 'F5 acceptance must receive the explicit workspace summary input.'
$canonicalBuildGates = @(
    [pscustomobject]@{ gate_id = 'zircon-app-target-server'; command = 'cargo check -p zircon_app --no-default-features --features target-server --locked' },
    [pscustomobject]@{ gate_id = 'zircon-app-target-client-platform'; command = 'cargo check -p zircon_app --bin zircon_runtime --no-default-features --features target-client,platform-winit,input-gamepad,gamepad-gilrs --locked' },
    [pscustomobject]@{ gate_id = 'zircon-app-target-editor-host'; command = 'cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked' },
    [pscustomobject]@{ gate_id = 'zircon-app-target-client-shader-pbr-viewer'; command = 'cargo check -p zircon_app --bin zircon_shader_pbr_viewer --no-default-features --features target-client,platform-winit,input-gamepad,gamepad-gilrs --locked' },
    [pscustomobject]@{ gate_id = 'zircon-runtime-target-client'; command = 'cargo check -p zircon_runtime --no-default-features --features target-client --locked' },
    [pscustomobject]@{ gate_id = 'zircon-runtime-target-editor-host'; command = 'cargo check -p zircon_runtime --no-default-features --features target-editor-host --locked' },
    [pscustomobject]@{ gate_id = 'zircon-runtime-target-server'; command = 'cargo check -p zircon_runtime --no-default-features --features target-server --locked' },
    [pscustomobject]@{ gate_id = 'workspace-build'; command = 'cargo build --workspace --locked' },
    [pscustomobject]@{ gate_id = 'workspace-test'; command = 'cargo test --workspace --locked' }
)
foreach ($gate in $canonicalBuildGates) {
    $commandParts = @($gate.command -split ' ')
    $argumentLiteral = (@($commandParts | Select-Object -Skip 1 | ForEach-Object { "'$_'" }) -join ', ')
    $contractLiteral = "'$($gate.gate_id)' = @($argumentLiteral)"
    Assert-WorkflowMatch ([regex]::Escape($contractLiteral)) "MVP job must bind canonical argv to gate '$($gate.gate_id)'."
}
Assert-WorkflowMatch 'gate_id\s*=' 'F5 build summaries must use canonical gate IDs.'
Assert-WorkflowMatch 'started_at_utc\s*=' 'F5 build gates must record absolute process start time.'
Assert-WorkflowMatch 'ended_at_utc\s*=' 'F5 build gates must record absolute process end time.'
Assert-WorkflowMatch 'Get-FileHash.*SHA256' 'F5 build summaries must bind each gate to a hashed log.'
Assert-WorkflowMatch '\$cargoArguments\s*=\s*\[string\[\]\]\$f5CargoGateContracts\[\$GateId\]' 'F5 gate execution must resolve argv directly from its gate ID.'
Assert-WorkflowMatch '&\s+cargo\s+@cargoArguments' 'F5 gate execution must invoke the argv derived from its canonical contract.'
Assert-WorkflowMatch '\$executedCommand\s*=\s*''cargo ''\s*\+\s*\(\$cargoArguments\s+-join\s+'' ''\)' 'F5 gate evidence command must be derived from the exact argv used for execution.'
Assert-WorkflowMatch 'command\s*=\s*\$executedCommand' 'F5 gate evidence must record its exact derived execution command.'
if ($workflow -match 'Invoke-F5CargoGate\s+-GateId[^\r\n]+-(?:Command|Arguments)') {
    throw 'F5 gate call sites must not pass independent declared commands or argv that can drift apart.'
}
Assert-WorkflowMatch "D:\\ZirconBuilds" 'MVP job must use an approved external staging root.'
Assert-WorkflowMatch 'f5-product' 'MVP job must retain the staged F5 manifests, logs, and captures as bounded diagnostics.'

if ($workflow -match 'Tee-Object -FilePath test-results/f5-(?:stage|acceptance)\.json') {
    throw 'MVP Windows workflow must not upload raw stage or acceptance control output containing absolute machine paths.'
}

Assert-WorkflowMatch 'WGPU adapter unavailable' 'MVP job must state a visible unavailable-adapter policy instead of accepting blank captures.'
Assert-WorkflowMatch 'actions/upload-artifact@v4' 'MVP job must upload diagnostic evidence.'
Assert-WorkflowMatch 'if:\s*always\(\)' 'MVP job must upload evidence after failures as well.'
Assert-WorkflowMatch 'retention-days:\s*7' 'MVP job must bound evidence retention.'

$cargoExitSnapshots = [regex]::Matches($workflow, '\$cargoExitCode\s*=\s*\$LASTEXITCODE').Count
if ($cargoExitSnapshots -lt 8) {
    throw 'Every MVP build/test command piped through Tee-Object must preserve Cargo''s exit code explicitly.'
}

if ($workflow -match 'continue-on-error:\s*true') {
    throw 'MVP Windows workflow must not silently accept a failed build, test, or capture.'
}

$exactOneTestAssertions = [regex]::Matches(
    $workflow,
    "Select-String -LiteralPath test-results/f[1234]-[^\r\n]+ -SimpleMatch 'test result: ok\. 1 passed; 0 failed' -Quiet"
).Count
if ($exactOneTestAssertions -ne 6) {
    throw 'The three F1, F2, F3, and F4 exact gates must each reject a zero-test or multi-test result.'
}

if ($workflow -match '(?m)^\s*path:\s*(?:target|\*\*/target)') {
    throw 'MVP Windows workflow must not upload the Cargo target tree.'
}

Write-Host 'MVP Windows workflow contract passed'
