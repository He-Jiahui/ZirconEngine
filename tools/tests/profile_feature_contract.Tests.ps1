Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$workflowPath = Join-Path $PSScriptRoot '..\..\.github\workflows\profile-feature-contract.yml'
$validatorPath = Join-Path $PSScriptRoot '..\..\.codex\skills\zircon-dev\scripts\validate-matrix.ps1'

if (-not (Test-Path -LiteralPath $workflowPath -PathType Leaf)) {
    throw "Profile feature contract workflow is missing: $workflowPath"
}
if (-not (Test-Path -LiteralPath $validatorPath -PathType Leaf)) {
    throw "Profile feature contract validator is missing: $validatorPath"
}

$workflow = Get-Content -LiteralPath $workflowPath -Raw -Encoding UTF8
$validator = Get-Content -LiteralPath $validatorPath -Raw -Encoding UTF8

function Assert-WorkflowMatch {
    param(
        [Parameter(Mandatory)][string]$Pattern,
        [Parameter(Mandatory)][string]$Message
    )

    if ($workflow -notmatch $Pattern) {
        throw $Message
    }
}

Assert-WorkflowMatch '(?ms)- label: zircon_app target-client-platform\s+package: zircon_app\s+features: target-client,platform-winit,input-gamepad,gamepad-gilrs\s+bin: zircon_runtime' 'The target-client profile must check the zircon_runtime startup binary.'
Assert-WorkflowMatch '(?ms)- label: zircon_app target-editor-host\s+package: zircon_app\s+features: target-editor-host\s+bin: zircon_editor' 'The target-editor-host profile must check the zircon_editor startup binary.'
Assert-WorkflowMatch '(?ms)- label: zircon_app target-client shader-pbr-viewer\s+package: zircon_app\s+features: target-client,platform-winit,input-gamepad,gamepad-gilrs\s+bin: zircon_shader_pbr_viewer' 'The target-client shader profile must check the zircon_shader_pbr_viewer binary.'
Assert-WorkflowMatch '(?ms)- label: zircon_app target-server\s+package: zircon_app\s+features: target-server\s+bin: ""' 'The target-server profile must remain a library-surface check, not a desktop product binary.'
Assert-WorkflowMatch '(?ms)- label: zircon_runtime target-client\s+package: zircon_runtime\s+features: target-client\s+bin: ""' 'The runtime target-client profile must remain in the complete profile contract.'
Assert-WorkflowMatch '(?ms)- label: zircon_runtime target-editor-host\s+package: zircon_runtime\s+features: target-editor-host\s+bin: ""' 'The runtime target-editor-host profile must remain in the complete profile contract.'
Assert-WorkflowMatch '(?ms)- label: zircon_runtime target-server\s+package: zircon_runtime\s+features: target-server\s+bin: ""' 'The runtime target-server profile must remain in the complete profile contract.'
Assert-WorkflowMatch 'if \[\[ -n "\$\{PROFILE_FEATURE_BIN\}" \]\]; then\s+cargo_args\+=\(--bin "\$\{PROFILE_FEATURE_BIN\}"\)' 'A profile matrix binary must be forwarded to Cargo.'
Assert-WorkflowMatch 'cargo_args\+=\(--no-default-features --features "\$\{\{ matrix\.features \}\}" --locked --verbose\)' 'Each profile contract check must disable implicit default features.'

if ($validator -notmatch '(?ms)Label = "zircon_app target-client-platform"\s+Package = "zircon_app"\s+Features = "target-client,platform-winit,input-gamepad,gamepad-gilrs"\s+Bin = "zircon_runtime"') {
    throw 'The local target-client profile must check the zircon_runtime startup binary.'
}
if ($validator -notmatch '(?ms)Label = "zircon_app target-editor-host"\s+Package = "zircon_app"\s+Features = "target-editor-host"\s+Bin = "zircon_editor"') {
    throw 'The local target-editor-host profile must check the zircon_editor startup binary.'
}
if ($validator -notmatch '(?ms)Label = "zircon_app target-client shader-pbr-viewer"\s+Package = "zircon_app"\s+Features = "target-client,platform-winit,input-gamepad,gamepad-gilrs"\s+Bin = "zircon_shader_pbr_viewer"') {
    throw 'The local target-client shader profile must check the zircon_shader_pbr_viewer binary.'
}
if ($validator -notmatch '(?ms)Label = "zircon_app target-server"\s+Package = "zircon_app"\s+Features = "target-server"\s+Bin = \$null') {
    throw 'The local target-server profile must remain a library-surface check, not a desktop product binary.'
}
if ($validator -notmatch '(?ms)Label = "zircon_runtime target-client"\s+Package = "zircon_runtime"\s+Features = "target-client"\s+Bin = \$null') {
    throw 'The local runtime target-client profile must remain in the complete profile contract.'
}
if ($validator -notmatch '(?ms)Label = "zircon_runtime target-editor-host"\s+Package = "zircon_runtime"\s+Features = "target-editor-host"\s+Bin = \$null') {
    throw 'The local runtime target-editor-host profile must remain in the complete profile contract.'
}
if ($validator -notmatch '(?ms)Label = "zircon_runtime target-server"\s+Package = "zircon_runtime"\s+Features = "target-server"\s+Bin = \$null') {
    throw 'The local runtime target-server profile must remain in the complete profile contract.'
}

Write-Host 'Profile feature contract source guard passed'
