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
Assert-WorkflowMatch 'renderable_empty_template_has_the_f2_camera_cube_and_sun_contract' 'MVP job must cover the F1 RenderableEmpty template contract.'
Assert-WorkflowMatch 'render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down' 'MVP job must cover the F2 persisted render/input contract.'
Assert-WorkflowMatch 'editor_project_document_roundtrips_world_and_workspace' 'MVP job must cover the F3 persisted project roundtrip.'
Assert-WorkflowMatch 'f4_project_authoring_survives_full_application_restart' 'MVP job must cover the F4 application authoring restart contract.'
Assert-WorkflowMatch 'cargo test -p zircon_app --test editor_mvp_authoring --no-default-features --features target-editor-host --locked' 'MVP job must run the F4 restart contract through the current App composition integration target.'
Assert-WorkflowMatch 'Stage-MvpProducts\.ps1' 'MVP job must stage source-bound product inputs rather than consume the repository target directory directly.'
Assert-WorkflowMatch 'Invoke-MvpAcceptance\.ps1' 'MVP job must validate staged product evidence through the acceptance driver.'
Assert-WorkflowMatch 'mvp-authoring-automation\.json' 'MVP job must exercise the source-bound normal authoring binding request.'
Assert-WorkflowMatch 'mvp-reopen-automation\.json' 'MVP job must exercise the independent source-bound persisted-state reopen request.'
Assert-WorkflowMatch 'RequireF5Evidence' 'MVP job must require the indivisible F5 creation, authoring, reopen, and visual-evidence contract.'
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
if ($workflow -match '(?m)^\s*path:\s*(?:target|\*\*/target)') {
    throw 'MVP Windows workflow must not upload the Cargo target tree.'
}

Write-Host 'MVP Windows workflow contract passed'
