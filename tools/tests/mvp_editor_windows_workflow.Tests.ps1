Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$workflowPath = Join-Path $PSScriptRoot '..\..\.github\workflows\mvp-editor-windows.yml'
$requiredScriptRunnerPath = Join-Path $PSScriptRoot 'mvp-required-script-contracts.Tests.ps1'
$buildGateRegistryModulePath = Join-Path $PSScriptRoot '..\mvp\MvpBuildGateRegistry.psm1'

if (-not (Test-Path -LiteralPath $workflowPath -PathType Leaf)) {
    throw "MVP Windows workflow is missing: $workflowPath"
}
if (-not (Test-Path -LiteralPath $requiredScriptRunnerPath -PathType Leaf)) {
    throw "MVP required script runner is missing: $requiredScriptRunnerPath"
}

$workflow = Get-Content -LiteralPath $workflowPath -Raw -Encoding UTF8
$requiredScriptRunner = Get-Content -LiteralPath $requiredScriptRunnerPath -Raw -Encoding UTF8
Import-Module $buildGateRegistryModulePath -Force -ErrorAction Stop

function Assert-WorkflowMatch {
    param(
        [Parameter(Mandatory)][string]$Pattern,
        [Parameter(Mandatory)][string]$Message
    )

    if ($workflow -notmatch $Pattern) {
        throw $Message
    }
}

function Assert-RequiredScriptRunnerMatch {
    param(
        [Parameter(Mandatory)][string]$Pattern,
        [Parameter(Mandatory)][string]$Message
    )

    if ($requiredScriptRunner -notmatch $Pattern) {
        throw $Message
    }
}

Assert-WorkflowMatch '(?m)^name:\s*MVP Editor Windows\s*$' 'Workflow must have the stable MVP Windows name.'
Assert-WorkflowMatch '(?ms)^jobs:\s*\r?\n\s*mvp-editor-windows:' 'Workflow must define the dedicated MVP Windows job.'
Assert-WorkflowMatch 'runs-on:\s*windows-latest' 'MVP job must run on windows-latest.'
Assert-WorkflowMatch 'actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09' 'MVP job must pin checkout to the reviewed upstream commit.'
Assert-WorkflowMatch 'dtolnay/rust-toolchain@75be91dd2711b583df57c31d0873b4145c89f1d9' 'MVP job must pin the Rust 1.94.1 action branch to a reviewed commit.'
Assert-WorkflowMatch 'Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6' 'MVP job must pin the Cargo cache action to the dereferenced upstream commit.'
Assert-WorkflowMatch 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02' 'MVP job must pin artifact upload to the reviewed upstream commit.'
Assert-WorkflowMatch 'MVP_RUST_TOOLCHAIN:\s*1\.94\.1' 'MVP job must declare the repository-authoritative Rust toolchain.'
Assert-WorkflowMatch 'runner_image:\s*\$env:ImageOS/\$env:ImageVersion' 'MVP environment receipt must record the concrete hosted runner image.'
Assert-WorkflowMatch 'MVP_EVIDENCE_ROOT:\s*D:\\ZirconBuilds\\mvp-ci-results-\$\{\{\s*github\.run_id\s*\}\}-\$\{\{\s*github\.run_attempt\s*\}\}' 'MVP diagnostics must use a source-bound approved physical artifact root.'

if ($workflow -match 'cargo build -p zircon_app --bin zircon_(?:editor|runtime)') {
    throw 'MVP workflow must not rebuild mutable-checkout product binaries before the immutable BuildSet product build.'
}
Assert-WorkflowMatch 'core::project::tests::template_creation::renderable_empty_template_has_the_f2_camera_cube_and_sun_contract\s+--locked\s+--\s+--exact' 'F1 must use the full libtest id before --exact.'
Assert-WorkflowMatch 'core::project::tests::template_creation::template_creation_rebuilds_regenerable_asset_state_from_source_after_deletion\s+--locked\s+--\s+--exact' 'F1 must cover rebuilding deleted derived asset state from source.'
Assert-WorkflowMatch 'core::project::tests::template_creation::template_creation_recovers_a_corrupt_persisted_registry_from_source_metadata\s+--locked\s+--\s+--exact' 'F1 must cover corrupt persisted registry recovery from source metadata.'
Assert-WorkflowMatch 'dynamic_api::session::tests::foundation_render::render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down\s+--locked\s+--\s+--exact' 'F2 must use the full libtest id before --exact.'
Assert-WorkflowMatch 'tests::workbench::project::document_roundtrip::editor_project_document_roundtrips_world_and_workspace\s+--locked\s+--\s+--exact' 'F3 must use the full libtest id before --exact.'
Assert-WorkflowMatch 'f4_project_authoring_survives_full_application_restart' 'MVP job must cover the F4 application authoring restart contract.'
Assert-WorkflowMatch 'cargo test -p zircon_app --test editor_mvp_authoring --no-default-features --features target-editor-host --locked' 'MVP job must run the F4 restart contract through the current App composition integration target.'
Assert-WorkflowMatch '\$env:MVP_EVIDENCE_ROOT\\f4-authoring\.log' 'F4 must reject a zero-test or multi-test exact result.'
Assert-WorkflowMatch 'Stage-MvpProducts\.ps1' 'MVP job must stage source-bound product inputs rather than consume the repository target directory directly.'
Assert-WorkflowMatch 'Invoke-MvpAcceptance\.ps1' 'MVP job must validate staged product evidence through the acceptance driver.'
Assert-WorkflowMatch 'mvp-authoring-automation\.json' 'MVP job must exercise the source-bound normal authoring binding request.'
Assert-WorkflowMatch 'mvp-reopen-automation\.json' 'MVP job must exercise the independent source-bound persisted-state reopen request.'
Assert-WorkflowMatch 'RequireF5Evidence' 'MVP job must require the indivisible F5 creation, authoring, reopen, and visual-evidence contract.'
Assert-WorkflowMatch 'profile-contract-summary\.json' 'MVP job must materialize the profile build summary consumed by F5 acceptance.'
Assert-WorkflowMatch 'workspace-summary\.json' 'MVP job must materialize the focused workspace build/test summary consumed by F5 acceptance.'
Assert-WorkflowMatch "summary_kind\s*=\s*'profile-contract'" 'Profile evidence must declare its canonical summary kind.'
Assert-WorkflowMatch "summary_kind\s*=\s*'workspace'" 'Workspace evidence must declare its canonical summary kind.'
Assert-WorkflowMatch 'Import-Module \.\\tools\\mvp\\MvpProductInputManifest\.psm1' 'Build summaries and staging must resolve the canonical ProductInputManifest.'
Assert-WorkflowMatch '\$productInputs\s*=\s*Resolve-MvpProductInputManifest -Path \$productInputManifest' 'Build summaries and staging must verify the ProductInputManifest before consuming source identity.'
Assert-WorkflowMatch '\$null -eq \$productInputs\.build_set' 'Build summaries and staging must reject a ProductInputManifest without a BuildSet receipt.'
Assert-WorkflowMatch '\$sourceFingerprint\s*=\s*\[string\]\$productInputs\.build_set\.build_set_id' 'Build summaries must bind to the verified BuildSetId.'
Assert-WorkflowMatch 'source_fingerprint\s*=\s*\$sourceFingerprint' 'Build summaries must retain the canonical source fingerprint.'
Assert-WorkflowMatch 'Build-MvpProductInputs\.ps1 -ArtifactOutputDirectory \$binaryInputRoot' 'F5 staging must receive managed, source-bound product inputs.'
if (([regex]::Matches($workflow, 'Build-MvpProductInputs\.ps1 -ArtifactOutputDirectory \$binaryInputRoot')).Count -ne 1) {
    throw 'MVP workflow must build ProductInputs exactly once and reuse the resulting BuildSet across F5 gates and staging.'
}
Assert-WorkflowMatch '-ProductInputManifest\s+\$productInputManifest' 'F5 staging must not bypass product input provenance with raw artifact paths.'
Assert-WorkflowMatch '\$binaryInputRoot = Join-Path ''D:\\ZirconBuilds'' "mvp-product-inputs-\$runIdentity"' 'F5 product inputs must use the managed physical artifact root.'
Assert-WorkflowMatch '\$evidenceRoot = Join-Path ''D:\\ZirconBuilds'' "mvp-f5-evidence-\$runIdentity"' 'F5 evidence must use the managed physical artifact root.'
Assert-WorkflowMatch 'Run focused MVP control-plane contracts' 'MVP job must run the focused control-plane contract batch.'
Assert-WorkflowMatch 'MVP_POWERSHELL_VERSION:\s*7\.4\.19' 'Control-plane runtime must pin the PowerShell LTS patch version.'
Assert-WorkflowMatch 'MVP_POWERSHELL_SHA256:\s*CD62AD6D8174CC6FB85B335A0058444BC934FE27C39FA97FE342134286D28AF9' 'Pinned PowerShell archive must be verified against the official release digest.'
Assert-WorkflowMatch 'MVP_PESTER_VERSION:\s*4\.10\.1' 'Control-plane runtime must pin the Pester version.'
Assert-WorkflowMatch 'PowerShell-\$env:MVP_POWERSHELL_VERSION-win-x64\.zip' 'Control-plane runtime must download the versioned official PowerShell archive.'
Assert-WorkflowMatch 'Get-FileHash[^\r\n]+-Algorithm SHA256' 'Control-plane runtime must hash the downloaded PowerShell archive before extraction.'
Assert-WorkflowMatch 'Save-Module\s+-Name Pester\s+-RequiredVersion \$env:MVP_PESTER_VERSION' 'Control-plane runtime must materialize the exact Pester package version.'
Assert-WorkflowMatch 'Import-Module[^\r\n]+Pester\.psd1[^\r\n]+-Force[^\r\n]+-ErrorAction Stop' 'Pinned test process must import Pester from the managed module root.'
Assert-WorkflowMatch '& \$env:MVP_PINNED_PWSH[^\r\n]+-File \$controlPlaneRunner' 'Control-plane batch must execute inside the pinned PowerShell runtime.'
Assert-WorkflowMatch 'Invoke-Pester -Script \$controlPlaneSuites -PassThru -OutputFormat NUnitXml -OutputFile \$controlPlaneResultPath' 'Control-plane batch must publish a structured Pester result artifact.'
Assert-WorkflowMatch 'mvp-acceptance-staging-projection\.Tests\.ps1' 'Control-plane batch must cover staging projection integrity.'
Assert-WorkflowMatch 'mvp-acceptance-staging-tree-manifest\.Tests\.ps1' 'Control-plane batch must cover staging tree-manifest integrity.'
Assert-WorkflowMatch 'mvp-acceptance-snapshot-admission\.Tests\.ps1' 'Control-plane batch must cover snapshot resource admission and deadlines.'
Assert-WorkflowMatch 'mvp-staging-tree-manifest-device-path\.Tests\.ps1' 'Control-plane batch must cover device-path manifest guards.'
Assert-WorkflowMatch 'mvp-build-set\.Tests\.ps1' 'Control-plane batch must cover immutable BuildSet publication.'
Assert-WorkflowMatch 'mvp-build-summary-evidence\.Tests\.ps1' 'Control-plane batch must cover build-summary evidence.'
Assert-WorkflowMatch 'mvp-artifact-storage-policy\.Tests\.ps1' 'Control-plane batch must cover versioned artifact-root admission.'
Assert-WorkflowMatch 'mvp-project-copy-policy\.Tests\.ps1' 'Control-plane batch must cover registry-declared project source and derived-state pruning.'
Assert-WorkflowMatch 'mvp-product-inputs\.Tests\.ps1' 'Control-plane batch must cover product-input provenance.'
Assert-WorkflowMatch 'render-extract-profiling-inputs\.Tests\.ps1' 'Control-plane batch must cover RenderExtract profiling BuildSet publication.'
Assert-WorkflowMatch 'render-extract-scale-project\.Tests\.ps1' 'Control-plane batch must cover RenderExtract scale-project BuildSet binding.'
Assert-WorkflowMatch 'render-extract-baseline-capture\.Tests\.ps1' 'Control-plane batch must cover RenderExtract capture admission and evidence.'
Assert-WorkflowMatch 'render-extract-machine-evidence\.Tests\.ps1' 'Control-plane batch must cover RenderExtract machine evidence.'
Assert-WorkflowMatch 'render-extract-baseline-report\.Tests\.ps1' 'Control-plane batch must cover RenderExtract report validation and publication.'
Assert-WorkflowMatch 'resource-management-baseline-plan\.Tests\.ps1' 'Control-plane batch must cover resource baseline plan validation.'
Assert-WorkflowMatch 'resource-management-scale-project\.Tests\.ps1' 'Control-plane batch must cover scale-project generation and change sets.'
Assert-WorkflowMatch 'resource-management-baseline-report\.Tests\.ps1' 'Control-plane batch must cover fail-closed resource observation reporting.'
Assert-WorkflowMatch 'resource-management-comparison\.Tests\.ps1' 'Control-plane batch must cover fail-closed resource cohort comparison.'
Assert-WorkflowMatch 'render-extract-process-lifecycle\.Tests\.ps1' 'Control-plane batch must cover RenderExtract process Job lifecycle.'
Assert-WorkflowMatch 'staged-process-supervisor\.Tests\.ps1' 'Control-plane batch must cover supervised process receipt integrity.'
Assert-WorkflowMatch 'mvp-process-environment-policy\.Tests\.ps1' 'Control-plane batch must cover the versioned child environment policy.'
Assert-WorkflowMatch 'mvp-process-output-capture\.Tests\.ps1' 'Control-plane batch must cover the shared stdout/stderr tail budget.'
Assert-WorkflowMatch 'mvp-process-lifecycle-journal\.Tests\.ps1' 'Control-plane batch must cover bounded streaming journal resume and tail reads.'
Assert-WorkflowMatch 'mvp-process-liveness-probe\.Tests\.ps1' 'Control-plane batch must cover registry-declared typed liveness.'
Assert-WorkflowMatch 'mvp-staging-cancellation-request\.Tests\.ps1' 'Control-plane batch must cover run-bound external cancellation.'
Assert-WorkflowMatch 'mvp-staging-terminal-receipt\.Tests\.ps1' 'Control-plane batch must cover context-bound terminal receipts.'
Assert-WorkflowMatch 'mvp-automation-scenario-spec\.Tests\.ps1' 'Control-plane batch must cover versioned automation scenarios.'
Assert-WorkflowMatch 'mvp-scenario-registry\.Tests\.ps1' 'Control-plane batch must cover the scenario registry.'
Assert-WorkflowMatch 'mvp-process-qualification-context\.Tests\.ps1' 'Control-plane batch must cover process qualification context linkage.'
Assert-WorkflowMatch 'mvp-stage-job-authority\.Tests\.ps1' 'Control-plane batch must cover Windows Job process-tree authority.'
Assert-WorkflowMatch 'mvp-run-artifact-budget\.Tests\.ps1' 'Control-plane batch must cover shared run artifact quotas.'
Assert-WorkflowMatch 'staged-process-log-summary\.Tests\.ps1' 'Control-plane batch must cover bounded diagnostic summaries.'
Assert-WorkflowMatch 'Invoke-Pester -Script \$controlPlaneSuites -PassThru' 'Control-plane batch must report its aggregate Pester result.'
Assert-WorkflowMatch '\$controlPlaneResult\.FailedCount -ne 0 -or \$controlPlaneResult\.TotalCount -ne 361' 'Control-plane batch must reject failed or incomplete Pester execution.'
Assert-WorkflowMatch "Invoke-Pester -Script '\.\\tools\\tests\\mvp-required-script-contracts\.Tests\.ps1' -PassThru" 'Control-plane batch must register required script-level gates as named Pester cases.'
Assert-WorkflowMatch 'Join-Path \$env:MVP_EVIDENCE_ROOT ''mvp-required-script-contracts-nunit\.xml''' 'Required script contracts must publish one dedicated NUnit result.'
Assert-WorkflowMatch '\$scriptContractResult\.FailedCount -ne 0 -or \$scriptContractResult\.TotalCount -ne 3' 'Required script contracts must reject failed or missing Pester registration.'
foreach ($scriptName in @('mvp-staging', 'mvp-acceptance', 'mvp_editor_windows_workflow')) {
    Assert-RequiredScriptRunnerMatch ([regex]::Escape("$scriptName.Tests.ps1")) "Required script runner must register '$scriptName.Tests.ps1'."
}
Assert-RequiredScriptRunnerMatch 'RedirectStandardOutput = \$true' 'Required script runner must stream stdout into a per-case artifact.'
Assert-RequiredScriptRunnerMatch 'RedirectStandardError = \$true' 'Required script runner must stream stderr into a per-case artifact.'
Assert-RequiredScriptRunnerMatch 'WaitForExit\(\$TimeoutMinutes \* 60 \* 1000\)' 'Required script runner must enforce each case timeout.'
Assert-RequiredScriptRunnerMatch '\$process\.Kill\(\$true\)' 'Required script runner must terminate the timed-out child process tree.'
if ($workflow -match '& \$env:MVP_PINNED_PWSH -NoProfile -NonInteractive -File \.\\tools\\tests\\(?:mvp-staging|mvp-acceptance|mvp_editor_windows_workflow)\.Tests\.ps1') {
    throw 'Control-plane batch must not execute required script-level gates outside Pester registration.'
}
Assert-WorkflowMatch '\$env:MVP_EVIDENCE_ROOT' 'MVP diagnostics must resolve from the declared physical artifact root.'
if ($workflow -match '\$env:RUNNER_TEMP') {
    throw 'MVP workflow must not route diagnostics, product inputs, or evidence through the runner temporary directory.'
}
if ($workflow -match 'test-results') {
    throw 'MVP workflow must not route diagnostics or evidence through a runner-relative test-results directory.'
}
Assert-WorkflowMatch '-ProfileContractSummaryPath\s+\$profileContractSummaryPath' 'F5 acceptance must receive the explicit profile summary input.'
Assert-WorkflowMatch '-WorkspaceSummaryPath\s+\$workspaceSummaryPath' 'F5 acceptance must receive the explicit workspace summary input.'
Assert-WorkflowMatch 'Import-Module \.\\tools\\mvp\\MvpBuildGateRegistry\.psm1' 'F5 gate execution must load the shared versioned gate registry.'
Assert-WorkflowMatch '\$gateRegistrySnapshot\s*=\s*Get-MvpBuildGateRegistrySnapshot' 'F5 gate execution must freeze one registry snapshot before resolving gate groups.'
Assert-WorkflowMatch 'Get-MvpBuildGateContract -SummaryKind ''profile-contract'' -RegistrySnapshot \$gateRegistrySnapshot' 'F5 profile gates must resolve from the frozen registry snapshot.'
Assert-WorkflowMatch 'Get-MvpBuildGateContract -SummaryKind ''workspace'' -RegistrySnapshot \$gateRegistrySnapshot' 'F5 workspace gates must resolve from the frozen registry snapshot.'
$registryReceiptBindings = [regex]::Matches($workflow, 'gate_registry\s*=\s*\$gateRegistrySnapshot\.receipt').Count
if ($registryReceiptBindings -ne 2) {
    throw "Both F5 build summaries must bind the frozen gate registry receipt; found $registryReceiptBindings bindings."
}
Assert-WorkflowMatch '(?s)\$profileContractSummary\s*=\s*\[ordered\]@\{.*?schema_version\s*=\s*2.*?gate_registry\s*=\s*\$gateRegistrySnapshot\.receipt' 'Profile build summary must use receipt-bound schema v2.'
Assert-WorkflowMatch '(?s)\$workspaceSummary\s*=\s*\[ordered\]@\{.*?schema_version\s*=\s*2.*?gate_registry\s*=\s*\$gateRegistrySnapshot\.receipt' 'Workspace build summary must use receipt-bound schema v2.'
$canonicalBuildGates = @(
    @(Get-MvpBuildGateContract -SummaryKind 'profile-contract') +
    @(Get-MvpBuildGateContract -SummaryKind 'workspace')
)
if ($canonicalBuildGates.Count -ne 9) {
    throw "MVP build gate registry must expose exactly 9 current gates; found $($canonicalBuildGates.Count)."
}
Assert-WorkflowMatch 'gate_id\s*=' 'F5 build summaries must use canonical gate IDs.'
Assert-WorkflowMatch 'started_at_utc\s*=' 'F5 build gates must record absolute process start time.'
Assert-WorkflowMatch 'ended_at_utc\s*=' 'F5 build gates must record absolute process end time.'
Assert-WorkflowMatch 'Get-FileHash.*SHA256' 'F5 build summaries must bind each gate to a hashed log.'
Assert-WorkflowMatch '\$cargoArguments\s*=\s*\[string\[\]\]\$contract\.cargo_arguments' 'F5 gate execution must resolve argv directly from its registered contract.'
Assert-WorkflowMatch '&\s+cargo\s+@cargoArguments' 'F5 gate execution must invoke the argv derived from its canonical contract.'
Assert-WorkflowMatch '\$executedCommand\s*=\s*\[string\]\$contract\.command' 'F5 gate evidence command must come from the same registered contract as its argv.'
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
Assert-WorkflowMatch 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02' 'MVP job must upload diagnostic evidence through the pinned action.'
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
    '\$env:MVP_EVIDENCE_ROOT\\f[1234]-[^\r\n]+ -SimpleMatch ''test result: ok\. 1 passed; 0 failed'' -Quiet'
).Count
if ($exactOneTestAssertions -ne 6) {
    throw 'The three F1, F2, F3, and F4 exact gates must each reject a zero-test or multi-test result.'
}

if ($workflow -match '(?m)^\s*path:\s*(?:target|\*\*/target)') {
    throw 'MVP Windows workflow must not upload the Cargo target tree.'
}
Assert-WorkflowMatch 'path:\s*\$\{\{\s*env\.MVP_EVIDENCE_ROOT\s*\}\}' 'MVP workflow must upload the approved physical evidence root.'

Write-Host 'MVP Windows workflow contract passed'
