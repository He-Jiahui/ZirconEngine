$ErrorActionPreference = 'Stop'

$environmentPolicyModule = Join-Path $PSScriptRoot 'MvpProcessEnvironmentPolicy.psm1'
Import-Module $environmentPolicyModule -ErrorAction Stop

$script:MvpStageHostEnvironmentNames = @(
    'PATH',
    'PATHEXT',
    'SystemRoot',
    'TEMP',
    'TMP',
    'WINDIR'
)
$script:MvpStageCommonDeclaredEnvironmentNames = @(
    'ZIRCON_ASSET_ROOT',
    'ZIRCON_LOG_FILTER',
    'ZIRCON_LOG_ROOT',
    'ZIRCON_RUNTIME_LIBRARY'
)

function Get-MvpStageProcessEnvironmentPolicy {
    param(
        [Parameter(Mandatory)]
        [ValidateSet('runtime_first_frame', 'editor_first_frame', 'editor_project_create', 'editor_authoring')]
        [string]$Scenario
    )

    $policy = switch ($Scenario) {
        'runtime_first_frame' {
            [pscustomobject]@{
                id = 'mvp.runtime-first-frame.v1'
                declared_names = $script:MvpStageCommonDeclaredEnvironmentNames + @(
                    'ZIRCON_RUNTIME_CAPTURE_FRAME_PNG',
                    'ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME',
                    'ZIRCON_RUNTIME_MVP_INPUT_PROBE'
                )
            }
        }
        'editor_first_frame' {
            [pscustomobject]@{
                id = 'mvp.editor-first-frame.v1'
                declared_names = $script:MvpStageCommonDeclaredEnvironmentNames + @(
                    'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG',
                    'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME'
                )
            }
        }
        'editor_project_create' {
            [pscustomobject]@{
                id = 'mvp.editor-project-create.v1'
                declared_names = $script:MvpStageCommonDeclaredEnvironmentNames + @(
                    'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG',
                    'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME'
                )
            }
        }
        'editor_authoring' {
            [pscustomobject]@{
                id = 'mvp.editor-authoring.v1'
                declared_names = $script:MvpStageCommonDeclaredEnvironmentNames
            }
        }
    }
    return New-MvpProcessEnvironmentPolicy `
        -PolicyId $policy.id `
        -InheritedNames $script:MvpStageHostEnvironmentNames `
        -DeclaredNames $policy.declared_names
}

Export-ModuleMember -Function 'Get-MvpStageProcessEnvironmentPolicy'
